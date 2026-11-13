//! Database tests for claiming a run (run-lifecycle T11, design Path 2).
//!
//! A claim picks one queued job with `FOR UPDATE SKIP LOCKED`, leases it, and moves
//! the run to `claimed`. Cancelled jobs go first. A retried job starts with a
//! `restarted` event.
//!
//! Run with `cargo test -p counterspell-judge --test claim -- --ignored pg_`.

use counterspell_common::ids::RunId;
use counterspell_common::status::RunMode;
use counterspell_judge::queue::claim::claim_next;
use counterspell_judge::testkit::{assert_invariants, seed_run};
use sqlx::PgPool;
use std::collections::HashSet;
use std::time::Duration;
use tokio::task::JoinSet;
use uuid::Uuid;

/// The lease length used by every claim in these tests.
const LEASE: Duration = Duration::from_secs(30);

/// Claims with a fixed worker name and returns the run id, or `None` when nothing is eligible.
async fn claim_as(pool: &PgPool, worker: &str) -> Option<RunId> {
    claim_next(pool, worker, LEASE)
        .await
        .unwrap()
        .map(|claimed| claimed.token.run_id())
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_claim_exclusive_under_contention(pool: PgPool) {
    let mut seeded = Vec::new();
    for _ in 0..50 {
        seeded.push(seed_run(&pool).await.unwrap());
    }

    // Eight workers claim until nothing is left. Each job must be won by exactly one.
    let mut workers = JoinSet::new();
    for worker in 0..8 {
        let pool = pool.clone();
        workers.spawn(async move {
            let mut won = Vec::new();
            while let Some(run) = claim_as(&pool, &format!("worker-{worker}")).await {
                won.push(run);
            }
            won
        });
    }
    let mut claimed = Vec::new();
    while let Some(result) = workers.join_next().await {
        claimed.extend(result.unwrap());
    }

    let unique: HashSet<Uuid> = claimed.iter().map(|run| run.as_uuid()).collect();
    assert_eq!(claimed.len(), 50, "every job is claimed");
    assert_eq!(unique.len(), 50, "no job is claimed twice");
    let expected: HashSet<Uuid> = seeded.iter().map(|run| run.as_uuid()).collect();
    assert_eq!(unique, expected, "no job is lost");
    assert_invariants(&pool).await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_claim_order_and_run_after(pool: PgPool) {
    let first = seed_run(&pool).await.unwrap();
    let delayed = seed_run(&pool).await.unwrap();
    let third = seed_run(&pool).await.unwrap();

    // The middle job is not ready for an hour, so it must be skipped.
    sqlx::query("UPDATE jobs SET run_after = now() + interval '1 hour' WHERE submission_id = $1")
        .bind(delayed.as_uuid())
        .execute(&pool)
        .await
        .unwrap();

    assert_eq!(claim_as(&pool, "w").await, Some(first));
    assert_eq!(claim_as(&pool, "w").await, Some(third));
    assert_eq!(claim_as(&pool, "w").await, None);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_claim_prefers_cancelled(pool: PgPool) {
    let older = seed_run(&pool).await.unwrap();
    let cancelled = seed_run(&pool).await.unwrap();

    sqlx::query("UPDATE jobs SET cancel_requested_at = now() WHERE submission_id = $1")
        .bind(cancelled.as_uuid())
        .execute(&pool)
        .await
        .unwrap();

    // The cancelled job is claimed first, and the claim says so, so the judge can finish it without running code.
    let first = claim_next(&pool, "w", LEASE)
        .await
        .unwrap()
        .expect("a job is queued");
    assert_eq!(first.token.run_id(), cancelled);
    assert!(first.cancel_requested);
    let second = claim_next(&pool, "w", LEASE)
        .await
        .unwrap()
        .expect("a job is queued");
    assert_eq!(second.token.run_id(), older);
    assert!(!second.cancel_requested);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_duplicate_claim_attempts(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();

    assert_eq!(claim_as(&pool, "w1").await, Some(run));
    // A claimed job is never handed out again, even to the same worker.
    assert_eq!(claim_as(&pool, "w1").await, None);
    assert_eq!(claim_as(&pool, "w2").await, None);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_claim_sets_submission_state(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();

    let claimed = claim_next(&pool, "worker-a", LEASE)
        .await
        .unwrap()
        .expect("one job is queued");

    assert_eq!(claimed.token.run_id(), run);
    assert_eq!(claimed.token.lease_owner(), "worker-a");
    assert_eq!(claimed.token.attempts(), 0);
    assert_eq!(claimed.problem_slug, "two-sum");
    assert_eq!(claimed.lang, "python");
    assert_eq!(claimed.mode, RunMode::Submit);
    assert_eq!(claimed.code, "print(1)");

    let (status, started, job_state, owner, lease_is_future): (
        String,
        bool,
        String,
        Option<String>,
        bool,
    ) = sqlx::query_as(
        "SELECT s.status, s.started_at IS NOT NULL, j.state, j.lease_owner, \
                j.lease_expires_at > now() \
         FROM submissions s JOIN jobs j ON j.submission_id = s.id WHERE s.id = $1",
    )
    .bind(run.as_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();

    assert_eq!(status, "claimed");
    assert!(started, "the first claim sets started_at");
    assert_eq!(job_state, "claimed");
    assert_eq!(owner.as_deref(), Some("worker-a"));
    assert!(lease_is_future, "the lease runs from the database clock");
    assert_invariants(&pool).await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_claim_after_retry_appends_restarted(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    // A job that already failed one attempt goes back to the queue with attempts = 1.
    sqlx::query("UPDATE jobs SET attempts = 1 WHERE submission_id = $1")
        .bind(run.as_uuid())
        .execute(&pool)
        .await
        .unwrap();

    let claimed = claim_next(&pool, "worker-b", LEASE)
        .await
        .unwrap()
        .expect("the retried job is queued");
    assert_eq!(claimed.token.attempts(), 1);

    let (seq, attempt, event_type, payload): (i32, i16, String, serde_json::Value) =
        sqlx::query_as(
            "SELECT seq, attempt, type, payload FROM run_events WHERE submission_id = $1",
        )
        .bind(run.as_uuid())
        .fetch_one(&pool)
        .await
        .unwrap();
    assert_eq!((seq, attempt, event_type.as_str()), (1, 2, "restarted"));
    assert_eq!(payload, serde_json::json!({ "attempt": 2 }));
    assert_invariants(&pool).await.unwrap();
}
