//! Database tests for the worker loop (run-lifecycle T17, design Path 7).
//!
//! The worker claims a run, runs it through an executor, and turns the outcome into
//! exactly one write. A panic or an infrastructure error is never reported as a
//! verdict. A cancel reaches a running executor through the heartbeat.
//!
//! Tests use `process_next`, which claims one run and runs it to its end, so no test
//! waits for a poll. The one test that depends on the heartbeat interval is named
//! `..._real_time`.
//!
//! Run with `cargo test -p counterspell-judge --test worker -- --ignored pg_`.

use counterspell_common::status::FailureReason;
use counterspell_judge::config::JudgeConfig;
use counterspell_judge::executor::{ExecError, RunContext, RunExecutor, RunInput, RunOutcome};
use counterspell_judge::testkit::{assert_invariants, seed_run, Script, ScriptedExecutor};
use counterspell_judge::worker::Worker;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::{mpsc, watch};
use uuid::Uuid;

/// Default settings.
fn config() -> JudgeConfig {
    JudgeConfig::from_lookup(|_| None).expect("defaults are valid")
}

/// Settings with a one-second heartbeat, so a cancel is seen quickly in real-time tests.
fn fast_heartbeat() -> JudgeConfig {
    JudgeConfig::from_lookup(|name| match name {
        "RENEW_SECS" => Some("1".to_owned()),
        "LEASE_SECS" => Some("5".to_owned()),
        _ => None,
    })
    .expect("the fast settings are valid")
}

/// The run status, job state and attempt count.
async fn state(pool: &PgPool, run: Uuid) -> (String, String, i16) {
    sqlx::query_as(
        "SELECT s.status, j.state, j.attempts FROM submissions s \
         JOIN jobs j ON j.submission_id = s.id WHERE s.id = $1",
    )
    .bind(run)
    .fetch_one(pool)
    .await
    .unwrap()
}

/// An executor that announces it has started, then waits for the stop signal.
struct Announcing {
    started: mpsc::UnboundedSender<()>,
}

impl RunExecutor for Announcing {
    async fn execute(&self, _run: RunInput, mut ctx: RunContext) -> Result<RunOutcome, ExecError> {
        // The receiver is held by the test, so a send only fails if the test has already ended.
        let _ = self.started.send(());
        ctx.stopped().await;
        Err(ExecError::Stopped)
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_worker_runs_scripted_job_to_completion(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let worker = Worker::new(
        pool.clone(),
        config(),
        ScriptedExecutor::new(Script::Pass {
            tests: 2,
            delay: std::time::Duration::ZERO,
        }),
        "judge-a".to_owned(),
    );

    assert!(worker.process_next().await.unwrap());

    assert_eq!(
        state(&pool, run.as_uuid()).await,
        ("completed".into(), "succeeded".into(), 0)
    );
    assert_invariants(&pool).await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_executor_infrastructure_error_retries(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let worker = Worker::new(
        pool.clone(),
        config(),
        ScriptedExecutor::new(Script::Infra("sandbox image missing".to_owned())),
        "judge-a".to_owned(),
    );

    assert!(worker.process_next().await.unwrap());

    // An infrastructure failure uses one attempt and is never reported as a wrong answer.
    assert_eq!(
        state(&pool, run.as_uuid()).await,
        ("queued".into(), "queued".into(), 1)
    );
    let last_error: Option<String> =
        sqlx::query_scalar("SELECT last_error FROM jobs WHERE submission_id = $1")
            .bind(run.as_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(last_error.as_deref(), Some("sandbox image missing"));
    assert_invariants(&pool).await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_executor_panic_is_contained(pool: PgPool) {
    let panicking = seed_run(&pool).await.unwrap();
    let healthy = seed_run(&pool).await.unwrap();

    // The first worker's program panics. The panic must not escape to the caller.
    let crashing = Worker::new(
        pool.clone(),
        config(),
        ScriptedExecutor::new(Script::Panic),
        "judge-a".to_owned(),
    );
    assert!(crashing.process_next().await.unwrap());
    assert_eq!(state(&pool, panicking.as_uuid()).await.0, "queued");
    assert_eq!(state(&pool, panicking.as_uuid()).await.2, 1);

    // The worker keeps going: the next ready run completes normally.
    let steady = Worker::new(
        pool.clone(),
        config(),
        ScriptedExecutor::new(Script::Pass {
            tests: 1,
            delay: std::time::Duration::ZERO,
        }),
        "judge-b".to_owned(),
    );
    assert!(steady.process_next().await.unwrap());
    assert_eq!(state(&pool, healthy.as_uuid()).await.0, "completed");
    assert_invariants(&pool).await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_cancel_queued_never_invokes_executor(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    sqlx::query("UPDATE jobs SET cancel_requested_at = now() WHERE submission_id = $1")
        .bind(run.as_uuid())
        .execute(&pool)
        .await
        .unwrap();
    // If the executor ran, this script would requeue the run. The cancel must stop that.
    let worker = Worker::new(
        pool.clone(),
        config(),
        ScriptedExecutor::new(Script::Infra("the executor must not run".to_owned())),
        "judge-a".to_owned(),
    );

    assert!(worker.process_next().await.unwrap());

    let (status, job_state, attempts) = state(&pool, run.as_uuid()).await;
    assert_eq!(
        (status.as_str(), job_state.as_str(), attempts),
        ("failed", "failed", 0)
    );
    let reason: Option<String> =
        sqlx::query_scalar("SELECT failure_reason FROM submissions WHERE id = $1")
            .bind(run.as_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(reason.as_deref(), Some(FailureReason::Cancelled.as_str()));
    assert_invariants(&pool).await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_cancel_running_stops_executor_real_time(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let (started_tx, mut started_rx) = mpsc::unbounded_channel();
    let worker = Worker::new(
        pool.clone(),
        fast_heartbeat(),
        Announcing {
            started: started_tx,
        },
        "judge-a".to_owned(),
    );
    let running = tokio::spawn(async move { worker.process_next().await });

    // The executor is running and holds the claim. Now the user cancels.
    started_rx.recv().await.expect("the executor started");
    sqlx::query("UPDATE jobs SET cancel_requested_at = now() WHERE submission_id = $1")
        .bind(run.as_uuid())
        .execute(&pool)
        .await
        .unwrap();

    // The next heartbeat sees the cancel, stops the executor, and the run ends as cancelled.
    assert!(running.await.unwrap().unwrap());
    let (status, job_state, attempts) = state(&pool, run.as_uuid()).await;
    assert_eq!(
        (status.as_str(), job_state.as_str(), attempts),
        ("failed", "failed", 0)
    );
    assert_invariants(&pool).await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_worker_concurrency_limit_respected(pool: PgPool) {
    for _ in 0..3 {
        seed_run(&pool).await.unwrap();
    }
    let settings = JudgeConfig::from_lookup(|name| match name {
        "JUDGE_CONCURRENCY" => Some("2".to_owned()),
        "SHUTDOWN_GRACE_SECS" => Some("1".to_owned()),
        "CLAIM_POLL_MS" => Some("50".to_owned()),
        _ => None,
    })
    .expect("settings are valid");
    let (started_tx, mut started_rx) = mpsc::unbounded_channel();
    let (stop, signal) = watch::channel(false);
    let worker = Arc::new(Worker::new(
        pool.clone(),
        settings,
        Announcing {
            started: started_tx,
        },
        "judge-a".to_owned(),
    ));
    let looping = tokio::spawn(Arc::clone(&worker).run(signal));

    // Two runs start. Every executor waits, so the two slots stay full.
    started_rx.recv().await.expect("first run started");
    started_rx.recv().await.expect("second run started");
    for _ in 0..100 {
        tokio::task::yield_now().await;
    }
    assert!(
        started_rx.try_recv().is_err(),
        "a third run must wait for a free slot"
    );

    stop.send(true).unwrap();
    looping.await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_graceful_shutdown_releases_job_real_time(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let settings = JudgeConfig::from_lookup(|name| match name {
        "SHUTDOWN_GRACE_SECS" => Some("1".to_owned()),
        "CLAIM_POLL_MS" => Some("50".to_owned()),
        _ => None,
    })
    .expect("settings are valid");
    let (started_tx, mut started_rx) = mpsc::unbounded_channel();
    let (stop, signal) = watch::channel(false);
    let worker = Arc::new(Worker::new(
        pool.clone(),
        settings,
        Announcing {
            started: started_tx,
        },
        "judge-a".to_owned(),
    ));
    let looping = tokio::spawn(Arc::clone(&worker).run(signal));

    started_rx.recv().await.expect("the run started");
    // Shutdown waits for the one-second grace period, then stops the executor and releases the run.
    stop.send(true).unwrap();
    looping.await.unwrap();

    assert_eq!(
        state(&pool, run.as_uuid()).await,
        ("queued".into(), "queued".into(), 1)
    );
    assert_invariants(&pool).await.unwrap();
}

/// An executor that never returns and never looks at its stop signal.
struct Stuck;

impl RunExecutor for Stuck {
    async fn execute(&self, _run: RunInput, _ctx: RunContext) -> Result<RunOutcome, ExecError> {
        std::future::pending().await
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_worker_hard_deadline_releases_stuck_executor_real_time(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    // The smallest allowed claim age is ten seconds, and the worker gives up five seconds before it.
    let settings = JudgeConfig::from_lookup(|name| match name {
        "MAX_CLAIM_AGE_SECS" => Some("10".to_owned()),
        _ => None,
    })
    .expect("settings are valid");
    let worker = Worker::new(pool.clone(), settings, Stuck, "judge-a".to_owned());

    assert!(worker.process_next().await.unwrap());

    assert_eq!(
        state(&pool, run.as_uuid()).await,
        ("queued".into(), "queued".into(), 1)
    );
    assert_invariants(&pool).await.unwrap();
}
