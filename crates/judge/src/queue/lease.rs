//! Lease renewal (design Path 3) and release for retry.
//!
//! Renewal checks ownership, not the clock: a renewal after the lease time has
//! passed still counts until a reaper takes the job away. Release sends an attempt
//! back to the queue with a backoff, or ends the run as `internal_error` when no
//! attempts are left. The reaper uses the same body, `requeue_or_exhaust_locked`.

use super::emit::append_event_locked;
use super::fence::lock_claim;
use super::{ClaimToken, QueueError};
use crate::config::JudgeConfig;
use counterspell_common::events::{ErrorPayload, PublicEvent};
use counterspell_common::ids::RunId;
use counterspell_common::status::FailureReason;
use sqlx::{PgPool, Postgres, Transaction};
use std::time::Duration;

/// The most characters of an internal error message stored in `jobs.last_error`.
pub(crate) const LAST_ERROR_MAX_CHARS: usize = 500;

/// What a renewal reported back to the worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Renewal {
    /// True when the server has asked for this run to be cancelled.
    pub cancel_requested: bool,
}

/// What a release or a reaper pass did with a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RetryOutcome {
    /// The job is queued again after a backoff, with one more attempt used.
    Requeued,
    /// No attempts were left, so the run ended as `internal_error`.
    Exhausted,
}

/// The job fields a requeue or exhaust decision needs. The caller has locked the job.
#[derive(Debug, Clone, Copy)]
pub(crate) struct JobSnapshot {
    /// The job row.
    pub job_id: i64,
    /// The run the job belongs to.
    pub run: RunId,
    /// Attempts used so far.
    pub attempts: i16,
    /// The attempt limit for this job.
    pub max_attempts: i16,
}

/// Extends the lease for `token` by `lease`, if this worker still owns the claim.
///
/// The update matches the job's state, owner and attempt count, so an old token
/// can never extend a newer claim.
///
/// # Errors
///
/// Returns `LeaseLost` when no claimed row matches, or a database error.
pub async fn renew(
    pool: &PgPool,
    token: &ClaimToken,
    lease: Duration,
) -> Result<Renewal, QueueError> {
    let cancel_requested: Option<bool> = sqlx::query_scalar(
        "UPDATE jobs SET lease_expires_at = now() + make_interval(secs => $4), updated_at = now() \
         WHERE id = $1 AND state = 'claimed' AND lease_owner = $2 AND attempts = $3 \
         RETURNING cancel_requested_at IS NOT NULL",
    )
    .bind(token.job_id())
    .bind(token.lease_owner())
    .bind(token.attempts())
    .bind(lease.as_secs_f64())
    .fetch_optional(pool)
    .await?;
    cancel_requested
        .map(|cancel_requested| Renewal { cancel_requested })
        .ok_or(QueueError::LeaseLost)
}

/// Gives the attempt back after an infrastructure failure, under the fence.
///
/// # Errors
///
/// Returns `LeaseLost` when the claim no longer matches (nothing changes), or a database error.
pub async fn release_for_retry(
    pool: &PgPool,
    token: &ClaimToken,
    cause: &str,
    cfg: &JudgeConfig,
) -> Result<RetryOutcome, QueueError> {
    let mut tx = pool.begin().await?;
    lock_claim(&mut tx, token).await?;
    let max_attempts: i16 = sqlx::query_scalar("SELECT max_attempts FROM jobs WHERE id = $1")
        .bind(token.job_id())
        .fetch_one(&mut *tx)
        .await?;
    let job = JobSnapshot {
        job_id: token.job_id(),
        run: token.run_id(),
        attempts: token.attempts(),
        max_attempts,
    };
    let outcome = requeue_or_exhaust_locked(&mut tx, &job, cause, cfg).await?;
    tx.commit().await?;
    Ok(outcome)
}

/// Requeues a locked job with a backoff, or ends its run as `internal_error`.
///
/// The caller must already hold the job row lock (through the fence or as the
/// reaper). The cause is stored only in `jobs.last_error`, never in a public event.
///
/// # Errors
///
/// Returns a database error when a statement fails.
pub(crate) async fn requeue_or_exhaust_locked(
    tx: &mut Transaction<'_, Postgres>,
    job: &JobSnapshot,
    cause: &str,
    cfg: &JudgeConfig,
) -> Result<RetryOutcome, QueueError> {
    let last_error = sanitize_last_error(cause);
    let next_attempts = job.attempts.saturating_add(1);

    if next_attempts < job.max_attempts {
        let delay = backoff(cfg, job.attempts);
        sqlx::query(
            "UPDATE jobs SET state = 'queued', attempts = $2, lease_owner = NULL, \
             lease_expires_at = NULL, claimed_at = NULL, \
             run_after = now() + make_interval(secs => $3), last_error = $4, updated_at = now() \
             WHERE id = $1",
        )
        .bind(job.job_id)
        .bind(next_attempts)
        .bind(delay.as_secs_f64())
        .bind(&last_error)
        .execute(&mut **tx)
        .await?;
        sqlx::query("UPDATE submissions SET status = 'queued' WHERE id = $1")
            .bind(job.run.as_uuid())
            .execute(&mut **tx)
            .await?;
        Ok(RetryOutcome::Requeued)
    } else {
        sqlx::query(
            "UPDATE jobs SET state = 'dead', attempts = $2, lease_owner = NULL, \
             lease_expires_at = NULL, last_error = $3, updated_at = now() \
             WHERE id = $1",
        )
        .bind(job.job_id)
        .bind(next_attempts)
        .bind(&last_error)
        .execute(&mut **tx)
        .await?;
        let ended = PublicEvent::Error(ErrorPayload::for_reason(FailureReason::RetriesExhausted));
        append_event_locked(tx, job.run, next_attempts, &ended).await?;
        Ok(RetryOutcome::Exhausted)
    }
}

/// Makes an internal cause safe to store: control characters become spaces, and the
/// text is cut to [`LAST_ERROR_MAX_CHARS`] characters.
pub(crate) fn sanitize_last_error(cause: &str) -> String {
    cause
        .chars()
        .map(|c| if c.is_control() { ' ' } else { c })
        .take(LAST_ERROR_MAX_CHARS)
        .collect()
}

/// The wait before the next attempt: base times two to the power of the attempts
/// used so far, never longer than the cap.
pub(crate) fn backoff(cfg: &JudgeConfig, attempts: i16) -> Duration {
    let exponent = u32::try_from(attempts.max(0)).unwrap_or(0).min(16);
    cfg.retry_backoff_base
        .saturating_mul(2u32.pow(exponent))
        .min(cfg.retry_backoff_cap)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> JudgeConfig {
        JudgeConfig::from_lookup(|_| None).expect("defaults are valid")
    }

    #[test]
    fn backoff_doubles_then_stops_at_the_cap() {
        let config = cfg();
        assert_eq!(backoff(&config, 0), Duration::from_secs(2));
        assert_eq!(backoff(&config, 1), Duration::from_secs(4));
        assert_eq!(backoff(&config, 2), Duration::from_secs(8));
        assert_eq!(backoff(&config, 10), Duration::from_secs(30));
    }

    #[test]
    fn sanitize_removes_control_characters_and_truncates() {
        assert_eq!(sanitize_last_error("line\none\ttwo"), "line one two");
        let long = "a".repeat(LAST_ERROR_MAX_CHARS + 25);
        assert_eq!(
            sanitize_last_error(&long).chars().count(),
            LAST_ERROR_MAX_CHARS
        );
    }
}
