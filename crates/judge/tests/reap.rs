//! Database tests for the reaper (run-lifecycle T15, design Path 6).
//!
//! The reaper takes back jobs whose lease expired or whose claim is too old, ends
//! cancelled orphans, and expires queued jobs that waited past their time to live.
//! Every change happens under the job's row lock, after the condition is checked again.
//!
//! Run with `cargo test -p counterspell-judge --test reap -- --ignored pg_`.

use counterspell_common::ids::RunId;
use counterspell_judge::config::JudgeConfig;
use counterspell_judge::queue::reap::{reap_once, run_reaper, ReapReport};
use counterspell_judge::testkit::{
    age_created, age_lease, assert_invariants, force_claim, seed_run,
};
use sqlx::PgPool;

/// Default settings, read through a lookup that sets nothing.
fn config() -> JudgeConfig {
    JudgeConfig::from_lookup(|_| None).expect("defaults are valid")
}

/// The job state, its attempt count and the run status, as text.
async fn job_and_run(pool: &PgPool, run: RunId) -> (String, i16, String) {
    sqlx::query_as(
        "SELECT j.state, j.attempts, s.status FROM jobs j \
         JOIN submissions s ON s.id = j.submission_id WHERE s.id = $1",
    )
    .bind(run.as_uuid())
    .fetch_one(pool)
    .await
    .unwrap()
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_reaper_requeues_expired_lease(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    force_claim(&pool, run, "worker-a").await.unwrap();
    age_lease(&pool, run).await.unwrap();

    let report = reap_once(&pool, &config()).await.unwrap();

    assert_eq!(
        report,
        ReapReport {
            requeued: 1,
            ..ReapReport::default()
        }
    );
    assert_eq!(
        job_and_run(&pool, run).await,
        ("queued".into(), 1, "queued".into())
    );
    assert_invariants(&pool).await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_reaper_marks_dead_after_max_attempts(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    force_claim(&pool, run, "worker-a").await.unwrap();
    // One attempt has already been used and this is the last allowed one.
    sqlx::query("UPDATE jobs SET attempts = 1, max_attempts = 2 WHERE submission_id = $1")
        .bind(run.as_uuid())
        .execute(&pool)
        .await
        .unwrap();
    age_lease(&pool, run).await.unwrap();

    let report = reap_once(&pool, &config()).await.unwrap();

    assert_eq!(
        report,
        ReapReport {
            exhausted: 1,
            ..ReapReport::default()
        }
    );
    assert_eq!(
        job_and_run(&pool, run).await,
        ("dead".into(), 2, "internal_error".into())
    );
    let reason: Option<String> =
        sqlx::query_scalar("SELECT failure_reason FROM submissions WHERE id = $1")
            .bind(run.as_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(reason.as_deref(), Some("retries_exhausted"));
    assert_invariants(&pool).await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_reaper_two_reapers_do_not_double_count(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    force_claim(&pool, run, "worker-a").await.unwrap();
    age_lease(&pool, run).await.unwrap();

    // Two reapers run at once. Only one may take the job, so attempts rises by exactly one.
    let first = {
        let pool = pool.clone();
        tokio::spawn(async move { reap_once(&pool, &config()).await })
    };
    let second = {
        let pool = pool.clone();
        tokio::spawn(async move { reap_once(&pool, &config()).await })
    };
    let total_requeued =
        first.await.unwrap().unwrap().requeued + second.await.unwrap().unwrap().requeued;

    assert_eq!(total_requeued, 1);
    assert_eq!(
        job_and_run(&pool, run).await.1,
        1,
        "attempts rises by exactly one"
    );
    assert_invariants(&pool).await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_max_claim_age_reclaims_stuck_run(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    force_claim(&pool, run, "worker-a").await.unwrap();
    // The lease is still in the future, but the claim is older than the maximum claim age.
    sqlx::query("UPDATE jobs SET claimed_at = now() - interval '1 day' WHERE submission_id = $1")
        .bind(run.as_uuid())
        .execute(&pool)
        .await
        .unwrap();

    let report = reap_once(&pool, &config()).await.unwrap();

    assert_eq!(report.requeued, 1);
    assert_eq!(job_and_run(&pool, run).await.0, "queued");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_expiry_of_stale_queue(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    // Waited an hour, longer than the ten-minute queue time to live.
    age_created(&pool, run, 3600).await.unwrap();

    let report = reap_once(&pool, &config()).await.unwrap();

    assert_eq!(
        report,
        ReapReport {
            expired: 1,
            ..ReapReport::default()
        }
    );
    assert_eq!(
        job_and_run(&pool, run).await,
        ("failed".into(), 0, "expired".into())
    );
    let (reason, code): (Option<String>, String) = sqlx::query_as(
        "SELECT s.failure_reason, e.payload->>'code' FROM submissions s \
         JOIN run_events e ON e.submission_id = s.id AND e.type = 'error' WHERE s.id = $1",
    )
    .bind(run.as_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(reason.as_deref(), Some("expired"));
    assert_eq!(code, "expired");
    let started: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM run_events WHERE submission_id = $1 AND type <> 'error'",
    )
    .bind(run.as_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(started, 0, "an expired run is never executed");
    assert_invariants(&pool).await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_reaper_finishes_cancelled_orphan(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    force_claim(&pool, run, "worker-a").await.unwrap();
    sqlx::query("UPDATE jobs SET cancel_requested_at = now() WHERE submission_id = $1")
        .bind(run.as_uuid())
        .execute(&pool)
        .await
        .unwrap();
    age_lease(&pool, run).await.unwrap();

    let report = reap_once(&pool, &config()).await.unwrap();

    assert_eq!(
        report,
        ReapReport {
            cancelled: 1,
            ..ReapReport::default()
        }
    );
    // A cancelled run is ended, not retried, so no attempt is spent.
    assert_eq!(
        job_and_run(&pool, run).await,
        ("failed".into(), 0, "failed".into())
    );
    let reason: Option<String> =
        sqlx::query_scalar("SELECT failure_reason FROM submissions WHERE id = $1")
            .bind(run.as_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(reason.as_deref(), Some("cancelled"));
    assert_invariants(&pool).await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_run_reaper_stops_on_shutdown(pool: PgPool) {
    let (stop, signal) = tokio::sync::watch::channel(false);
    let reaper = {
        let pool = pool.clone();
        tokio::spawn(async move { run_reaper(&pool, &config(), signal).await })
    };
    // Setting the flag before the reaper runs still stops it, so the test needs no sleep.
    stop.send(true).unwrap();
    reaper.await.expect("the reaper returns after shutdown");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_reaper_does_not_touch_live_leases(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    force_claim(&pool, run, "worker-a").await.unwrap();

    let report = reap_once(&pool, &config()).await.unwrap();

    assert_eq!(report, ReapReport::default());
    assert_eq!(
        job_and_run(&pool, run).await,
        ("claimed".into(), 0, "claimed".into())
    );
}
