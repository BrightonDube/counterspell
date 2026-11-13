//! Tests for `POST /api/runs/{id}/cancel` (run-lifecycle T27, design Path 8 and requirement R17).
//!
//! The server only sets `jobs.cancel_requested_at`. The judge makes the state change. The queued-cancel test
//! runs the real judge claim path, so it checks the whole effect of a cancel.
//!
//! Run with `DATABASE_URL=<url> cargo test -p counterspell-server --test cancel -- --ignored pg_`.

mod support;

use std::{sync::Arc, time::Duration};

use counterspell_common::{
    ids::{RunId, UserId},
    status::RunMode,
};
use counterspell_judge::{
    config::JudgeConfig,
    testkit::{Script, ScriptedExecutor},
    worker::Worker,
};
use counterspell_server::{
    build_router,
    config::RunLimits,
    ratelimit::{Limiter, MemoryLimiter},
    runs::enqueue::{enqueue, NewRun},
    testkit::BearerTestResolver,
    AppState,
};
use serde_json::Value;
use sqlx::PgPool;
use support::{config_for, lazy_pool, post, serve, Fixture, Reply};
use uuid::Uuid;

fn state_with(pool: PgPool, static_dir: &std::path::Path) -> AppState {
    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    AppState::new(pool, config_for(static_dir), limiter).with_user_resolver(BearerTestResolver)
}

fn limits() -> RunLimits {
    RunLimits {
        run_rate_user: 30,
        run_rate_ip: 100,
        run_active_cap: 10,
        queue_depth_max: 200,
        max_source_bytes: 65_536,
        request_body_limit_bytes: 131_072,
    }
}

fn owner(number: u128) -> UserId {
    UserId::from_uuid(Uuid::from_u128(number))
}

fn auth(number: u128) -> String {
    format!("Bearer test-user-{number}")
}

async fn queued_run(pool: &PgPool, user: UserId) -> RunId {
    let run = NewRun::new(
        RunId::new(),
        user,
        "two-sum",
        "python",
        RunMode::Submit,
        "print(1)\n",
        None,
    );
    enqueue(pool, &run, &limits()).await.expect("enqueue");
    run.run_id
}

/// Sends `POST /api/runs/{id}/cancel` with a JSON content type and an empty body.
async fn cancel(address: std::net::SocketAddr, run: RunId, user: u128) -> Reply {
    let token = auth(user);
    post(
        address,
        &format!("/api/runs/{}/cancel", run.as_uuid()),
        &[
            ("Authorization", token.as_str()),
            ("Content-Type", "application/json"),
        ],
        "{}",
    )
    .await
}

fn json(reply: &Reply) -> Value {
    serde_json::from_str(&reply.body).expect("reply body is JSON")
}

async fn cancel_flag(pool: &PgPool, run: RunId) -> Option<String> {
    sqlx::query_scalar("SELECT cancel_requested_at::text FROM jobs WHERE submission_id = $1")
        .bind(run.as_uuid())
        .fetch_one(pool)
        .await
        .expect("read cancel flag")
}

async fn status_of(pool: &PgPool, run: RunId) -> String {
    sqlx::query_scalar("SELECT status FROM submissions WHERE id = $1")
        .bind(run.as_uuid())
        .fetch_one(pool)
        .await
        .expect("read status")
}

fn judge_config() -> JudgeConfig {
    JudgeConfig::from_lookup(|_name: &str| None::<String>).expect("judge defaults load")
}

#[tokio::test]
async fn cancel_requires_json_before_touching_the_database() {
    let fixture = Fixture::new();
    let state = AppState::new(
        lazy_pool(),
        config_for(&fixture.dir),
        Limiter::Memory(Arc::new(MemoryLimiter::default())),
    )
    .with_user_resolver(BearerTestResolver);
    let address = serve(build_router(state)).await;
    let token = auth(1);

    let reply = post(
        address,
        "/api/runs/00000000-0000-0000-0000-000000000001/cancel",
        &[
            ("Authorization", token.as_str()),
            ("Content-Type", "text/plain"),
        ],
        "{}",
    )
    .await;

    assert_eq!(reply.status, 415);
    assert_eq!(json(&reply)["code"], "unsupported_media_type");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_cancel_queued_is_finished_as_cancelled_without_the_executor(pool: PgPool) {
    let fixture = Fixture::new();
    let run = queued_run(&pool, owner(1)).await;
    let address = serve(build_router(state_with(pool.clone(), &fixture.dir))).await;

    let reply = cancel(address, run, 1).await;
    assert_eq!(reply.status, 202);
    assert_eq!(json(&reply)["status"], "queued");

    // A scripted executor that would pass every test. If the judge ran it, the run would complete.
    let worker = Worker::new(
        pool.clone(),
        judge_config(),
        ScriptedExecutor::new(Script::Pass {
            tests: 2,
            delay: Duration::ZERO,
        }),
        "worker-a".to_owned(),
    );
    assert!(worker
        .process_next()
        .await
        .expect("claim the cancelled job"));

    let (status, reason, verdict): (String, Option<String>, Option<String>) =
        sqlx::query_as("SELECT status, failure_reason, verdict FROM submissions WHERE id = $1")
            .bind(run.as_uuid())
            .fetch_one(&pool)
            .await
            .expect("read result");
    assert_eq!(status, "failed");
    assert_eq!(reason.as_deref(), Some("cancelled"));
    assert_eq!(verdict, None);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_cancel_twice_keeps_the_first_timestamp(pool: PgPool) {
    let fixture = Fixture::new();
    let run = queued_run(&pool, owner(1)).await;
    let address = serve(build_router(state_with(pool.clone(), &fixture.dir))).await;

    let first = cancel(address, run, 1).await;
    let stamped = cancel_flag(&pool, run).await;
    let second = cancel(address, run, 1).await;

    assert_eq!(first.status, 202);
    assert_eq!(second.status, 202);
    assert!(stamped.is_some());
    assert_eq!(cancel_flag(&pool, run).await, stamped);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_cancel_after_done_changes_nothing(pool: PgPool) {
    let fixture = Fixture::new();
    let run = queued_run(&pool, owner(1)).await;
    let id = run.as_uuid();
    for status in ["claimed", "compiling", "running"] {
        sqlx::query("UPDATE submissions SET status = $2 WHERE id = $1")
            .bind(id)
            .bind(status)
            .execute(&pool)
            .await
            .expect("move status");
    }
    sqlx::query(
        "UPDATE submissions SET status = 'completed', verdict = 'accepted', passed = 2, total = 2, \
         score = 100, started_at = now(), finished_at = now(), last_event_seq = 1 WHERE id = $1",
    )
    .bind(id)
    .execute(&pool)
    .await
    .expect("complete run");
    sqlx::query("UPDATE jobs SET state = 'succeeded', lease_owner = NULL, lease_expires_at = NULL WHERE submission_id = $1")
        .bind(id)
        .execute(&pool)
        .await
        .expect("succeed job");
    let address = serve(build_router(state_with(pool.clone(), &fixture.dir))).await;

    let reply = cancel(address, run, 1).await;

    assert_eq!(reply.status, 200);
    assert_eq!(json(&reply)["status"], "completed");
    assert_eq!(cancel_flag(&pool, run).await, None);
    assert_eq!(status_of(&pool, run).await, "completed");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_cancel_foreign_and_missing_are_404_and_set_no_flag(pool: PgPool) {
    let fixture = Fixture::new();
    let run = queued_run(&pool, owner(1)).await;
    let address = serve(build_router(state_with(pool.clone(), &fixture.dir))).await;

    let foreign = cancel(address, run, 2).await;
    let missing = cancel(address, RunId::new(), 1).await;

    assert_eq!(foreign.status, 404);
    assert_eq!(missing.status, 404);
    assert_eq!(foreign.body, missing.body);
    assert_eq!(cancel_flag(&pool, run).await, None);
}

/// A judge config whose heartbeat renews every second, so a cancel is seen within about a second.
fn fast_renew_config() -> JudgeConfig {
    JudgeConfig::from_lookup(|name: &str| match name {
        "RENEW_SECS" => Some("1".to_owned()),
        "LEASE_SECS" => Some("5".to_owned()),
        _ => None,
    })
    .expect("fast renew config loads")
}

/// Waits until the judge has claimed the run, so a cancel arrives while it is running.
async fn wait_until_claimed(pool: &PgPool, run: RunId) {
    for _ in 0..200 {
        let state: String = sqlx::query_scalar("SELECT state FROM jobs WHERE submission_id = $1")
            .bind(run.as_uuid())
            .fetch_one(pool)
            .await
            .expect("read job state");
        if state == "claimed" {
            return;
        }
        tokio::time::sleep(Duration::from_millis(25)).await;
    }
    panic!("the judge never claimed the run");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_cancel_running_stops_the_executor_and_finishes_as_cancelled(pool: PgPool) {
    let fixture = Fixture::new();
    let run = queued_run(&pool, owner(1)).await;
    let address = serve(build_router(state_with(pool.clone(), &fixture.dir))).await;

    // A program that never finishes on its own. Only the cancel can end it.
    let worker = Arc::new(Worker::new(
        pool.clone(),
        fast_renew_config(),
        ScriptedExecutor::new(Script::Hang),
        "worker-a".to_owned(),
    ));
    let judge = tokio::spawn({
        let worker = Arc::clone(&worker);
        async move { worker.process_next().await }
    });
    wait_until_claimed(&pool, run).await;

    let reply = cancel(address, run, 1).await;
    assert_eq!(reply.status, 202);

    let claimed = tokio::time::timeout(Duration::from_secs(20), judge)
        .await
        .expect("the judge stops the run within the deadline")
        .expect("judge task joins")
        .expect("claim does not fail");
    assert!(claimed);

    let (status, reason, verdict): (String, Option<String>, Option<String>) =
        sqlx::query_as("SELECT status, failure_reason, verdict FROM submissions WHERE id = $1")
            .bind(run.as_uuid())
            .fetch_one(&pool)
            .await
            .expect("read result");
    assert_eq!(status, "failed");
    assert_eq!(reason.as_deref(), Some("cancelled"));
    assert_eq!(verdict, None);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_cancel_never_yields_both_done_and_error(pool: PgPool) {
    let fixture = Fixture::new();
    let run = queued_run(&pool, owner(1)).await;
    let address = serve(build_router(state_with(pool.clone(), &fixture.dir))).await;

    // A program that is slow enough for the cancel to land while it runs, so the race is real.
    let worker = Arc::new(Worker::new(
        pool.clone(),
        fast_renew_config(),
        ScriptedExecutor::new(Script::Pass {
            tests: 40,
            delay: Duration::from_millis(50),
        }),
        "worker-a".to_owned(),
    ));
    let judge = tokio::spawn({
        let worker = Arc::clone(&worker);
        async move { worker.process_next().await }
    });
    wait_until_claimed(&pool, run).await;
    let reply = cancel(address, run, 1).await;
    assert_eq!(reply.status, 202);

    tokio::time::timeout(Duration::from_secs(30), judge)
        .await
        .expect("the run ends within the deadline")
        .expect("judge task joins")
        .expect("claim does not fail");

    // Whichever side won the race, exactly one terminal event exists, and it is never both.
    let terminal: Vec<String> = sqlx::query_scalar(
        "SELECT type FROM run_events WHERE submission_id = $1 AND type IN ('done', 'error') ORDER BY seq",
    )
    .bind(run.as_uuid())
    .fetch_all(&pool)
    .await
    .expect("read terminal events");
    assert_eq!(
        terminal.len(),
        1,
        "expected one terminal event, got {terminal:?}"
    );
}
