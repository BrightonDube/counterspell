//! Claiming the next queued run (design Path 2).
//!
//! One transaction picks one ready job with `FOR UPDATE SKIP LOCKED`, leases it to
//! this worker, and moves the run to `claimed`. Cancelled jobs are taken first. A
//! job that has already been attempted gets a `restarted` event before the worker
//! starts. Every time value comes from the database clock.
//!
//! Lock order is the job row first, then the submission row, the same order as
//! every other judge write.

use super::emit::append_event_locked;
use super::{ClaimToken, QueueError};
use counterspell_common::events::PublicEvent;
use counterspell_common::ids::RunId;
use sqlx::PgPool;
use std::time::Duration;
use uuid::Uuid;

/// A run this worker has just claimed, with the fields it needs to execute it.
///
/// The struct does not derive `Debug`, so the source code cannot end up in a log by accident.
#[derive(Clone)]
pub struct ClaimedRun {
    /// The fence that every later write for this run must present.
    pub token: ClaimToken,
    /// The problem the run is for.
    pub problem_slug: String,
    /// The language the source is written in.
    pub lang: String,
    /// The source text the user submitted.
    pub code: String,
}

/// Claims the next ready run for `worker` and leases it for `lease`.
///
/// Returns `None` when no job is ready. A job that is already claimed is never
/// returned again. If the job has been attempted before, a `restarted` event is
/// stored in the same transaction before the claim is returned.
///
/// # Errors
///
/// Returns `QueueError::Database` when a statement fails. Nothing is written in that case.
pub async fn claim_next(
    pool: &PgPool,
    worker: &str,
    lease: Duration,
) -> Result<Option<ClaimedRun>, QueueError> {
    let mut tx = pool.begin().await?;

    let claimed_job: Option<(i64, Uuid, i16)> = sqlx::query_as(
        "WITH next AS ( \
             SELECT id FROM jobs \
              WHERE state = 'queued' AND run_after <= now() \
              ORDER BY cancel_requested_at NULLS LAST, run_after, id \
              FOR UPDATE SKIP LOCKED \
              LIMIT 1 \
         ) \
         UPDATE jobs j \
            SET state = 'claimed', lease_owner = $1, \
                lease_expires_at = now() + make_interval(secs => $2), \
                claimed_at = now(), updated_at = now() \
           FROM next \
          WHERE j.id = next.id \
         RETURNING j.id, j.submission_id, j.attempts",
    )
    .bind(worker)
    .bind(lease.as_secs_f64())
    .fetch_optional(&mut *tx)
    .await?;

    let Some((job_id, run_uuid, attempts)) = claimed_job else {
        // Nothing was claimed, so there is nothing to commit. Dropping `tx` rolls back.
        return Ok(None);
    };
    let run = RunId::from_uuid(run_uuid);

    sqlx::query(
        "UPDATE submissions SET status = 'claimed', started_at = COALESCE(started_at, now()) \
         WHERE id = $1 AND status = 'queued'",
    )
    .bind(run_uuid)
    .execute(&mut *tx)
    .await?;

    let (problem_slug, lang, code): (String, String, String) =
        sqlx::query_as("SELECT problem_slug, lang, code FROM submissions WHERE id = $1")
            .bind(run_uuid)
            .fetch_one(&mut *tx)
            .await?;

    if attempts > 0 {
        // The stored attempt count is zero-based, so this claim is attempt `attempts + 1`.
        // The count is bounded by the table's CHECK, so the clamp never changes a real value.
        let attempt = attempts.saturating_add(1);
        let restarted = PublicEvent::Restarted {
            attempt: u16::try_from(attempt).unwrap_or(u16::MAX),
        };
        append_event_locked(&mut tx, run, attempt, &restarted).await?;
    }

    tx.commit().await?;
    Ok(Some(ClaimedRun {
        token: ClaimToken::new(job_id, run, worker.to_owned(), attempts),
        problem_slug,
        lang,
        code,
    }))
}
