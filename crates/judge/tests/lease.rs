//! Database tests for lease renewal and release (run-lifecycle T13, design Path 3).
//!
//! A worker keeps its lease by renewing it. Ownership, not the clock, decides
//! whether a renewal counts. A released run goes back to the queue with one more
//! attempt, or ends as `internal_error` when its attempts run out.
//!
//! Run with `cargo test -p counterspell-judge --test lease -- --ignored pg_`.

use counterspell_common::events::PublicEvent;
use counterspell_judge::config::JudgeConfig;
use counterspell_judge::queue::claim::claim_next;
use counterspell_judge::queue::emit::emit;
use counterspell_judge::queue::lease::{release_for_retry, renew, RetryOutcome};
use counterspell_judge::queue::QueueError;
use counterspell_judge::testkit::{age_lease, assert_invariants, force_claim, seed_run};
use sqlx::PgPool;
use std::time::Duration;

/// Default settings, read through a lookup that sets nothing.
fn config() -> JudgeConfig {
    JudgeConfig::from_lookup(|_| None).expect("defaults are valid")
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_lease_renewal_extends_and_reports_cancel(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let token = force_claim(&pool, run, "worker-a").await.unwrap();
    age_lease(&pool, run).await.unwrap();

    let renewal = renew(&pool, &token, Duration::from_secs(60)).await.unwrap();
    assert!(!renewal.cancel_requested);
    let extended: bool =
        sqlx::query_scalar("SELECT lease_expires_at > now() FROM jobs WHERE submission_id = $1")
            .bind(run.as_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert!(extended, "renewal moves the lease into the future");

    sqlx::query("UPDATE jobs SET cancel_requested_at = now() WHERE submission_id = $1")
        .bind(run.as_uuid())
        .execute(&pool)
        .await
        .unwrap();
    let renewal = renew(&pool, &token, Duration::from_secs(60)).await.unwrap();
    assert!(
        renewal.cancel_requested,
        "the renewal reports a cancel request"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_renew_after_expiry_but_before_reap_succeeds(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let token = force_claim(&pool, run, "worker-a").await.unwrap();
    // The lease time has passed, but nothing has reclaimed the job yet.
    age_lease(&pool, run).await.unwrap();

    let result = renew(&pool, &token, Duration::from_secs(60)).await;
    assert!(result.is_ok(), "ownership still holds: {result:?}");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_renew_after_reap_fails(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let token = force_claim(&pool, run, "worker-a").await.unwrap();
    // A reaper has taken the job back: queued, no owner, no lease.
    sqlx::query(
        "UPDATE jobs SET state = 'queued', lease_owner = NULL, lease_expires_at = NULL \
         WHERE submission_id = $1",
    )
    .bind(run.as_uuid())
    .execute(&pool)
    .await
    .unwrap();

    let result = renew(&pool, &token, Duration::from_secs(60)).await;
    assert!(matches!(result, Err(QueueError::LeaseLost)), "{result:?}");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_lost_lease_stops_writes(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let old = force_claim(&pool, run, "worker-a").await.unwrap();
    // Another worker now holds the same job, so every write from `old` must fail.
    sqlx::query("UPDATE jobs SET lease_owner = 'worker-b' WHERE submission_id = $1")
        .bind(run.as_uuid())
        .execute(&pool)
        .await
        .unwrap();

    assert!(matches!(
        renew(&pool, &old, Duration::from_secs(60)).await,
        Err(QueueError::LeaseLost)
    ));
    assert!(matches!(
        emit(&pool, &old, &PublicEvent::Compiling).await,
        Err(QueueError::LeaseLost)
    ));
    assert!(matches!(
        release_for_retry(&pool, &old, "stale", &config()).await,
        Err(QueueError::LeaseLost)
    ));
    assert_invariants(&pool).await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_reclaim_by_same_worker_gets_new_fence(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let first = force_claim(&pool, run, "worker-a").await.unwrap();
    assert_eq!(
        release_for_retry(&pool, &first, "transient", &config())
            .await
            .unwrap(),
        RetryOutcome::Requeued
    );
    // Skip the backoff so the same worker can claim again straight away.
    sqlx::query("UPDATE jobs SET run_after = now() - interval '1 second' WHERE submission_id = $1")
        .bind(run.as_uuid())
        .execute(&pool)
        .await
        .unwrap();

    let second = claim_next(&pool, "worker-a", Duration::from_secs(30))
        .await
        .unwrap()
        .expect("the requeued run is ready")
        .token;
    assert_eq!(second.attempts(), 1);

    assert!(matches!(
        renew(&pool, &first, Duration::from_secs(30)).await,
        Err(QueueError::LeaseLost)
    ));
    assert!(renew(&pool, &second, Duration::from_secs(30)).await.is_ok());
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_release_requeues_with_backoff(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let token = force_claim(&pool, run, "worker-a").await.unwrap();

    let outcome = release_for_retry(&pool, &token, "worker lost its sandbox", &config())
        .await
        .unwrap();
    assert_eq!(outcome, RetryOutcome::Requeued);

    let (state, attempts, owner, delayed, last_error, status): (
        String,
        i16,
        Option<String>,
        bool,
        Option<String>,
        String,
    ) = sqlx::query_as(
        "SELECT j.state, j.attempts, j.lease_owner, j.run_after > now(), j.last_error, s.status \
         FROM jobs j JOIN submissions s ON s.id = j.submission_id WHERE j.submission_id = $1",
    )
    .bind(run.as_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(state, "queued");
    assert_eq!(attempts, 1);
    assert_eq!(owner, None);
    assert!(delayed, "the retry waits for its backoff");
    assert_eq!(last_error.as_deref(), Some("worker lost its sandbox"));
    assert_eq!(status, "queued");
    assert_invariants(&pool).await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_release_exhausts_to_internal_error(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let token = force_claim(&pool, run, "worker-a").await.unwrap();
    // This was the only allowed attempt, so releasing it must end the run.
    sqlx::query("UPDATE jobs SET max_attempts = 1 WHERE submission_id = $1")
        .bind(run.as_uuid())
        .execute(&pool)
        .await
        .unwrap();

    let outcome = release_for_retry(&pool, &token, "sandbox image missing", &config())
        .await
        .unwrap();
    assert_eq!(outcome, RetryOutcome::Exhausted);

    let (status, reason, finished, job_state): (String, Option<String>, bool, String) =
        sqlx::query_as(
            "SELECT s.status, s.failure_reason, s.finished_at IS NOT NULL, j.state \
             FROM submissions s JOIN jobs j ON j.submission_id = s.id WHERE s.id = $1",
        )
        .bind(run.as_uuid())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!(status, "internal_error");
    assert_eq!(reason.as_deref(), Some("retries_exhausted"));
    assert!(finished);
    assert_eq!(job_state, "dead");

    let error_payload: serde_json::Value = sqlx::query_scalar(
        "SELECT payload FROM run_events WHERE submission_id = $1 AND type = 'error'",
    )
    .bind(run.as_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(error_payload["code"], "retries_exhausted");
    assert_invariants(&pool).await.unwrap();
}
