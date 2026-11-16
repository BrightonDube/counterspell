//! Restart and recovery tests (run-lifecycle T35, requirements R9, R13.9, R22.4 and R22.7).
//!
//! The judge and the server hold no run state in memory. These tests stop a server or a judge in the middle
//! of a run, start a new one on the same database, and check that the run still finishes once, with the
//! full event log.
//!
//! Run with `DATABASE_URL=<url> cargo test -p counterspell-server --test restart -- --ignored pg_`.

mod support;

use std::{sync::Arc, time::Duration};

use counterspell_common::{
    ids::{RunId, UserId},
    status::RunMode,
};
use counterspell_judge::{
    config::JudgeConfig,
    queue::reap::reap_once,
    testkit::{age_created, age_lease, Script, ScriptedExecutor},
    worker::Worker,
};
use counterspell_server::{
    build_router,
    config::RunLimits,
    ratelimit::{Limiter, MemoryLimiter},
    runs::enqueue::{enqueue, NewRun},
    testkit::{ensure_user, BearerTestResolver},
    AppState,
};
use sqlx::PgPool;
use support::{
    config_for, publish_two_sum,
    run_writes::{finish_run, write_events, write_events_from},
    sse_client::{Frame, SseResponse},
    Fixture,
};
use tokio::task::JoinHandle;
use uuid::Uuid;

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

/// The database default for `jobs.max_attempts` (design section 4).
const MAX_ATTEMPTS: u32 = 3;

fn judge_config() -> JudgeConfig {
    JudgeConfig::from_lookup(|_name: &str| None::<String>).expect("judge defaults load")
}

fn owner() -> UserId {
    UserId::from_uuid(Uuid::from_u128(1))
}

fn token() -> &'static str {
    "Bearer test-user-1"
}

fn events_path(run: RunId) -> String {
    format!("/api/runs/{}/events", run.as_uuid())
}

/// Saves a queued run for the test owner and returns its id.
async fn queued_run(pool: &PgPool) -> RunId {
    ensure_user(pool, owner()).await.expect("user fixture");
    publish_two_sum(pool).await;
    let run = NewRun::new(
        RunId::new(),
        owner(),
        "two-sum",
        "python",
        RunMode::Submit,
        "print(1)\n",
        None,
    );
    enqueue(pool, &run, &limits()).await.expect("enqueue");
    run.run_id
}

/// Starts the real router on a random local port. Returns the address and the task that serves it.
async fn serve_with_handle(
    pool: PgPool,
    static_dir: &std::path::Path,
) -> (std::net::SocketAddr, JoinHandle<()>) {
    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    let state =
        AppState::new(pool, config_for(static_dir), limiter).with_user_resolver(BearerTestResolver);
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let address = listener.local_addr().expect("address");
    let router = build_router(state);
    let task = tokio::spawn(async move {
        let _ = axum::serve(
            listener,
            router.into_make_service_with_connect_info::<std::net::SocketAddr>(),
        )
        .await;
    });
    (address, task)
}

/// Starts a judge that claims `run` and then waits in the executor. Returns once the run is `claimed`.
/// The caller aborts the task to model a judge that dies mid-run.
async fn hang_until_claimed(pool: &PgPool, run: RunId) -> JoinHandle<()> {
    let worker = Worker::new(
        pool.clone(),
        judge_config(),
        ScriptedExecutor::new(Script::Hang),
        format!("hanging-judge-{}", Uuid::new_v4()),
    );
    let task = tokio::spawn(async move {
        let _ = worker.process_next().await;
    });
    for _ in 0..100 {
        let status: String = sqlx::query_scalar("SELECT status FROM submissions WHERE id = $1")
            .bind(run.as_uuid())
            .fetch_one(pool)
            .await
            .expect("status");
        if status == "claimed" {
            return task;
        }
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
    panic!("the hanging judge never claimed the run");
}

/// Lets the next claim take the run now, instead of after the retry backoff.
async fn end_backoff(pool: &PgPool, run: RunId) {
    sqlx::query("UPDATE jobs SET run_after = now() WHERE submission_id = $1")
        .bind(run.as_uuid())
        .execute(pool)
        .await
        .expect("end the backoff");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_restart_server_mid_stream_resumes_without_loss(pool: PgPool) {
    let fixture = Fixture::new();
    let run = queued_run(&pool).await;
    // Two events exist before the first stream opens. The rest arrive while the server is down or live.
    write_events(&pool, run, 2).await;

    let (first_address, first_server) = serve_with_handle(pool.clone(), &fixture.dir).await;
    let mut before = SseResponse::open(
        first_address,
        &events_path(run),
        &[("Authorization", token())],
    )
    .await;
    let mut seen: Vec<i64> = Vec::new();
    for _ in 0..2 {
        let frame = before
            .next_frame_within(Duration::from_secs(2))
            .await
            .expect("an event before the restart");
        seen.push(frame.seq().expect("numbered"));
    }

    // The server stops while the stream is open and the run is still going.
    first_server.abort();
    drop(before);
    // The judge keeps writing while the server is down.
    write_events_from(&pool, run, 3, 2).await;

    let (second_address, _second_server) = serve_with_handle(pool.clone(), &fixture.dir).await;
    let resume_from = seen.last().copied().unwrap_or(0).to_string();
    let mut after = SseResponse::open(
        second_address,
        &events_path(run),
        &[
            ("Authorization", token()),
            ("Last-Event-ID", resume_from.as_str()),
        ],
    )
    .await;
    // The run finishes while the resumed stream is open, so the last event arrives live.
    finish_run(&pool, run, 5).await;
    while let Some(frame) = after.next_frame_within(Duration::from_secs(3)).await {
        if frame.kind().is_none() {
            continue;
        }
        let done = frame.kind() == Some("done");
        seen.push(frame.seq().expect("numbered"));
        if done {
            break;
        }
    }

    assert!(
        !after.transport_failed(),
        "the resumed stream ended normally"
    );
    assert_eq!(
        seen,
        vec![1, 2, 3, 4, 5],
        "the full log arrives once, in order"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_restart_judge_fleet_mid_run_recovers_with_restarted(pool: PgPool) {
    let fixture = Fixture::new();
    let run = queued_run(&pool).await;

    // The first judge dies while it holds the lease.
    let dying = hang_until_claimed(&pool, run).await;
    dying.abort();
    age_lease(&pool, run).await.expect("the lease expires");
    let report = reap_once(&pool, &judge_config()).await.expect("reap");
    assert_eq!(
        report.requeued, 1,
        "the reaper returns the run to the queue"
    );
    end_backoff(&pool, run).await;

    // A new judge finishes it.
    let healthy = Worker::new(
        pool.clone(),
        judge_config(),
        ScriptedExecutor::new(Script::Pass {
            tests: 2,
            delay: Duration::ZERO,
        }),
        "new-judge".to_owned(),
    );
    assert!(healthy.process_next().await.expect("retry claim"));

    let (address, _server) = serve_with_handle(pool.clone(), &fixture.dir).await;
    let mut stream =
        SseResponse::open(address, &events_path(run), &[("Authorization", token())]).await;
    let mut events: Vec<Frame> = Vec::new();
    while let Some(frame) = stream.next_frame_within(Duration::from_secs(3)).await {
        if frame.kind().is_none() {
            continue;
        }
        let done = frame.kind() == Some("done");
        events.push(frame);
        if done {
            break;
        }
    }

    let attempt_two: Vec<&Frame> = events
        .iter()
        .filter(|frame| frame.envelope()["attempt"] == 2)
        .collect();
    assert_eq!(
        attempt_two[0].kind(),
        Some("restarted"),
        "restarted opens attempt 2"
    );
    let dones = events
        .iter()
        .filter(|frame| frame.kind() == Some("done"))
        .count();
    assert_eq!(dones, 1, "exactly one terminal result");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_restart_with_no_judge_expires_stale_queue(pool: PgPool) {
    let fixture = Fixture::new();
    let run = queued_run(&pool).await;
    // Waiting for a judge longer than the queue time to live (600 s by default).
    age_created(&pool, run, 700).await.expect("age the job");

    let report = reap_once(&pool, &judge_config()).await.expect("reap");
    assert_eq!(report.expired, 1, "the stale job expires");

    let (address, _server) = serve_with_handle(pool.clone(), &fixture.dir).await;
    let reply = support::get(
        address,
        &format!("/api/runs/{}", run.as_uuid()),
        &[("Authorization", token())],
    )
    .await;
    let body: serde_json::Value = serde_json::from_str(&reply.body).expect("JSON");
    assert_eq!(body["status"], "expired");
    assert_eq!(body["terminal"], true);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_retries_exhausted_end_to_end(pool: PgPool) {
    let fixture = Fixture::new();
    let run = queued_run(&pool).await;

    // Every attempt hangs and its judge dies, so the reaper eventually runs out of attempts.
    for _ in 0..MAX_ATTEMPTS {
        end_backoff(&pool, run).await;
        let dying = hang_until_claimed(&pool, run).await;
        dying.abort();
        age_lease(&pool, run).await.expect("the lease expires");
        reap_once(&pool, &judge_config()).await.expect("reap");
    }

    let (address, _server) = serve_with_handle(pool.clone(), &fixture.dir).await;
    let reply = support::get(
        address,
        &format!("/api/runs/{}", run.as_uuid()),
        &[("Authorization", token())],
    )
    .await;
    let body: serde_json::Value = serde_json::from_str(&reply.body).expect("JSON");
    assert_eq!(body["status"], "internal_error");
    assert_eq!(body["terminal"], true);

    let mut stream =
        SseResponse::open(address, &events_path(run), &[("Authorization", token())]).await;
    let mut last: Option<Frame> = None;
    while let Some(frame) = stream.next_frame_within(Duration::from_secs(3)).await {
        if frame.kind().is_some() {
            last = Some(frame);
        }
    }
    let last = last.expect("the stream ends with the error event");
    assert_eq!(
        last.kind(),
        Some("error"),
        "the stream ends with an error event"
    );
    assert_eq!(last.envelope()["payload"]["code"], "retries_exhausted");
}
