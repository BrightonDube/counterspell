//! The reaper (design Path 6). It takes back work that no live worker holds, and
//! expires queued work that waited too long.
//!
//! Several judge processes may run it at once. Each candidate is found without a
//! lock, then locked and checked again inside its own transaction. A job that another
//! reaper or a live worker has already changed fails the check and is skipped, so
//! attempts rise by exactly one.

use super::emit::append_event_locked;
use super::lease::{requeue_or_exhaust_locked, JobSnapshot, RetryOutcome};
use super::QueueError;
use crate::config::JudgeConfig;
use counterspell_common::events::{ErrorPayload, PublicEvent};
use counterspell_common::ids::RunId;
use counterspell_common::status::FailureReason;
use sqlx::PgPool;
use tokio::sync::watch;
use uuid::Uuid;

/// The most jobs one pass changes, so a long backlog cannot hold locks for long.
const BATCH: i64 = 20;

/// What one reaper pass changed.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct ReapReport {
    /// Jobs whose expired lease sent them back to the queue.
    pub requeued: u32,
    /// Jobs whose attempts ran out, so the run ended as `internal_error`.
    pub exhausted: u32,
    /// Cancelled runs that no worker was finishing, now ended as `failed`.
    pub cancelled: u32,
    /// Queued jobs that waited past the queue time to live, now `expired`.
    pub expired: u32,
}

/// Runs one pass over expired claims, cancelled orphans and stale queued jobs.
///
/// # Errors
///
/// Returns `QueueError::Database` when a candidate query fails. Changes already
/// committed for earlier jobs stay in place.
pub async fn reap_once(pool: &PgPool, cfg: &JudgeConfig) -> Result<ReapReport, QueueError> {
    let mut report = ReapReport::default();
    let max_claim_age = cfg.max_claim_age.as_secs_f64();

    let claimed: Vec<i64> = sqlx::query_scalar(
        "SELECT id FROM jobs \
         WHERE state = 'claimed' \
           AND (lease_expires_at < now() OR claimed_at < now() - make_interval(secs => $1)) \
         ORDER BY lease_expires_at \
         LIMIT $2",
    )
    .bind(max_claim_age)
    .bind(BATCH)
    .fetch_all(pool)
    .await?;
    for job_id in claimed {
        reap_claim(pool, cfg, job_id, &mut report).await?;
    }

    let queued: Vec<i64> = sqlx::query_scalar(
        "SELECT id FROM jobs \
         WHERE state = 'queued' AND created_at < now() - make_interval(secs => $1) \
         ORDER BY created_at \
         LIMIT $2",
    )
    .bind(cfg.queue_ttl.as_secs_f64())
    .bind(BATCH)
    .fetch_all(pool)
    .await?;
    for job_id in queued {
        expire_queued(pool, cfg, job_id, &mut report).await?;
    }
    Ok(report)
}

/// Locks one stuck claim, checks it is still stuck, and ends or requeues it.
async fn reap_claim(
    pool: &PgPool,
    cfg: &JudgeConfig,
    job_id: i64,
    report: &mut ReapReport,
) -> Result<(), QueueError> {
    let mut tx = pool.begin().await?;
    let max_claim_age = cfg.max_claim_age.as_secs_f64();
    let locked: Option<(Uuid, i16, i16, bool)> = sqlx::query_as(
        "SELECT submission_id, attempts, max_attempts, cancel_requested_at IS NOT NULL \
         FROM jobs \
         WHERE id = $1 AND state = 'claimed' \
           AND (lease_expires_at < now() OR claimed_at < now() - make_interval(secs => $2)) \
         FOR UPDATE SKIP LOCKED",
    )
    .bind(job_id)
    .bind(max_claim_age)
    .fetch_optional(&mut *tx)
    .await?;
    let Some((run_uuid, attempts, max_attempts, cancel_requested)) = locked else {
        return Ok(());
    };
    let run = RunId::from_uuid(run_uuid);

    if cancel_requested {
        // The user cancelled and nothing is finishing the run, so end it. Retrying would be wrong.
        sqlx::query(
            "UPDATE jobs SET state = 'failed', lease_owner = NULL, lease_expires_at = NULL, \
             last_error = 'cancelled', updated_at = now() WHERE id = $1",
        )
        .bind(job_id)
        .execute(&mut *tx)
        .await?;
        let ended = PublicEvent::Error(ErrorPayload::for_reason(FailureReason::Cancelled));
        append_event_locked(&mut tx, run, attempts.saturating_add(1), &ended).await?;
        report.cancelled += 1;
    } else {
        let job = JobSnapshot {
            job_id,
            run,
            attempts,
            max_attempts,
        };
        match requeue_or_exhaust_locked(&mut tx, &job, "lease expired", cfg).await? {
            RetryOutcome::Requeued => report.requeued += 1,
            RetryOutcome::Exhausted => report.exhausted += 1,
        }
    }
    tx.commit().await?;
    Ok(())
}

/// Locks one stale queued job, checks it is still stale, and expires it without running it.
async fn expire_queued(
    pool: &PgPool,
    cfg: &JudgeConfig,
    job_id: i64,
    report: &mut ReapReport,
) -> Result<(), QueueError> {
    let mut tx = pool.begin().await?;
    let locked: Option<(Uuid, i16)> = sqlx::query_as(
        "SELECT submission_id, attempts FROM jobs \
         WHERE id = $1 AND state = 'queued' AND created_at < now() - make_interval(secs => $2) \
         FOR UPDATE SKIP LOCKED",
    )
    .bind(job_id)
    .bind(cfg.queue_ttl.as_secs_f64())
    .fetch_optional(&mut *tx)
    .await?;
    let Some((run_uuid, attempts)) = locked else {
        return Ok(());
    };

    sqlx::query(
        "UPDATE jobs SET state = 'failed', last_error = 'expired', updated_at = now() WHERE id = $1",
    )
    .bind(job_id)
    .execute(&mut *tx)
    .await?;
    let ended = PublicEvent::Error(ErrorPayload::for_reason(FailureReason::Expired));
    append_event_locked(
        &mut tx,
        RunId::from_uuid(run_uuid),
        attempts.saturating_add(1),
        &ended,
    )
    .await?;
    tx.commit().await?;
    report.expired += 1;
    Ok(())
}

/// Runs [`reap_once`] at start-up and then every `reaper_interval`, until `shutdown` is set.
///
/// A failed pass is logged and the next pass still runs. The log names the error only,
/// never any run content.
pub async fn run_reaper(pool: &PgPool, cfg: &JudgeConfig, mut shutdown: watch::Receiver<bool>) {
    let mut ticker = tokio::time::interval(cfg.reaper_interval);
    loop {
        tokio::select! {
            _ = ticker.tick() => {
                if let Err(error) = reap_once(pool, cfg).await {
                    tracing::warn!(%error, "reaper pass failed");
                }
            }
            changed = shutdown.changed() => {
                if changed.is_err() || *shutdown.borrow() {
                    break;
                }
            }
        }
    }
}
