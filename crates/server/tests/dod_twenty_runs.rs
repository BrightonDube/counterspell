//! Definition of done: twenty concurrent runs end to end (run-lifecycle T36, requirements R20 to R22).
//!
//! Twenty users submit at the same instant. Three judge workers run them, five clients drop their stream and
//! resume, and five other users plus an anonymous client try to read and cancel every run. The test then checks
//! the queue, the event logs, the access rules and the canary, and writes `target/run-lifecycle/dod-report.json`.
//!
//! Run with `DATABASE_URL=<url> cargo test -p counterspell-server --test dod_twenty_runs -- --ignored pg_ --nocapture`.

mod support;

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    str::FromStr,
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use counterspell_common::ids::{RunId, UserId};
use counterspell_judge::{
    config::JudgeConfig,
    testkit::{Script, ScriptedExecutor},
    worker::Worker,
};
use counterspell_server::{
    build_router,
    config::ServerConfig,
    ratelimit::{Limiter, MemoryLimiter},
    runs::wake::RunWaker,
    testkit::{ensure_user, BearerTestResolver},
    AppState,
};
use serde_json::{json, Value};
use sqlx::{postgres::PgPoolOptions, PgPool};
use support::{config_for, publish_two_sum, report, serve, sse_client::SseResponse, Fixture};
use tokio::{
    sync::{watch, Barrier},
    task::JoinHandle,
};
use uuid::Uuid;

const RUNS: usize = 20;
const OTHER_USERS: usize = 5;
const RESUMERS: usize = 5;
const WORKERS: usize = 3;
const TESTS_PER_RUN: u32 = 3;
const TEST_DELAY: Duration = Duration::from_millis(40);
/// A marker that stands for hidden data. The scripted judges never hold it, so the scan proves only that no
/// public surface carries it. The canary test (T34) covers a judge that does hold hidden data.
const CANARY: &str = "canary-dod-hidden-7f3a";

/// What one submitting client measured and saw.
struct Observed {
    run: RunId,
    enqueue_ms: f64,
    first_event_ms: f64,
    completion_ms: f64,
    /// Wall-clock time the terminal event arrived, in ms since the Unix epoch.
    done_received_ms: f64,
    /// Sequence numbers in the order the client received them.
    seen: Vec<i64>,
}

/// One submission: the run, the moment the request was sent, and how long the `202` took.
#[derive(Clone, Copy)]
struct Submitted {
    user: usize,
    run: RunId,
    sent: Instant,
    enqueue_ms: f64,
}

/// Database timings for one run, in milliseconds: queue wait, execution, and when the `done` event was written.
#[derive(Clone, Copy)]
struct DbTimings {
    queue: f64,
    execution: f64,
    done_created: f64,
}

fn owner(number: usize) -> UserId {
    UserId::from_uuid(Uuid::from_u128(number as u128))
}

fn bearer(number: usize) -> String {
    format!("Bearer test-user-{number}")
}

fn elapsed_ms(since: Instant) -> f64 {
    since.elapsed().as_secs_f64() * 1000.0
}

fn epoch_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0.0, |d| d.as_secs_f64() * 1000.0)
}

/// Settings for the fleet: rate limits high enough for twenty users, and the defaults for everything else.
fn server_config(static_dir: &Path) -> ServerConfig {
    let mut config = config_for(static_dir);
    config.limits.run_rate_user = 100;
    config.limits.run_rate_ip = 1000;
    config.limits.run_active_cap = 10;
    config
}

fn app_state(pool: &PgPool, static_dir: &Path) -> AppState {
    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    AppState::new(pool.clone(), server_config(static_dir), limiter)
        .with_user_resolver(BearerTestResolver)
        .with_run_waker(RunWaker::start(pool.clone()))
}

/// Starts one judge with its own pool, as a separate process would have.
async fn start_judge(pool: &PgPool, name: &str, stop: watch::Receiver<bool>) -> JoinHandle<()> {
    let judge_pool = PgPoolOptions::new()
        .max_connections(4)
        .connect_with((*pool.connect_options()).clone())
        .await
        .expect("judge pool");
    let config =
        JudgeConfig::from_lookup(|key: &str| (key == "JUDGE_CONCURRENCY").then(|| "2".to_owned()))
            .expect("judge config");
    let executor = ScriptedExecutor::new(Script::Pass {
        tests: TESTS_PER_RUN,
        delay: TEST_DELAY,
    });
    let worker = Arc::new(Worker::new(judge_pool, config, executor, name.to_owned()));
    tokio::spawn(worker.run(stop))
}

/// Submits one run as `user` with its own idempotency key.
async fn submit(address: std::net::SocketAddr, user: usize) -> Submitted {
    let body =
        json!({"problem": "two-sum", "lang": "python", "code": "print(1)\n", "mode": "submit"})
            .to_string();
    let key = Uuid::new_v4().to_string();
    let token = bearer(user);
    let sent = Instant::now();
    let reply = support::post(
        address,
        "/api/runs",
        &[
            ("Authorization", token.as_str()),
            ("Content-Type", "application/json"),
            ("Idempotency-Key", key.as_str()),
        ],
        &body,
    )
    .await;
    let enqueue_ms = elapsed_ms(sent);
    assert_eq!(reply.status, 202, "submit for user {user}: {}", reply.body);
    let parsed: Value = serde_json::from_str(&reply.body).expect("JSON reply");
    let run = RunId::from_str(parsed["run_id"].as_str().expect("run_id")).expect("valid run id");
    Submitted {
        user,
        run,
        sent,
        enqueue_ms,
    }
}

/// Reads one run's stream to its terminal event. A resumer drops after two events and reconnects with
/// `Last-Event-ID`. Returns the sequence numbers seen, the time to the first event and the arrival time of `done`.
async fn stream_to_done(
    address: std::net::SocketAddr,
    run: RunId,
    user: usize,
    resume: bool,
) -> (Vec<i64>, f64, f64) {
    let path = format!("/api/runs/{}/events", run.as_uuid());
    let token = bearer(user);
    let opened = Instant::now();
    let mut first = SseResponse::open(address, &path, &[("Authorization", token.as_str())]).await;
    let mut seen: Vec<i64> = Vec::new();
    let mut first_event_ms = None;

    while !(resume && seen.len() == 2) {
        let Some(frame) = first.next_frame_within(Duration::from_secs(60)).await else {
            break;
        };
        if frame.kind().is_none() {
            continue;
        }
        first_event_ms.get_or_insert_with(|| elapsed_ms(opened));
        seen.push(frame.seq().expect("numbered event"));
        if frame.kind() == Some("done") {
            return (seen, first_event_ms.unwrap_or(0.0), epoch_ms());
        }
    }
    if !resume {
        return (seen, first_event_ms.unwrap_or(0.0), epoch_ms());
    }

    // The connection drops here. The client reconnects with the last sequence number it kept.
    drop(first);
    let resume_from = seen.last().copied().unwrap_or(0).to_string();
    let mut second = SseResponse::open(
        address,
        &path,
        &[
            ("Authorization", token.as_str()),
            ("Last-Event-ID", resume_from.as_str()),
        ],
    )
    .await;
    while let Some(frame) = second.next_frame_within(Duration::from_secs(60)).await {
        if frame.kind().is_none() {
            continue;
        }
        let done = frame.kind() == Some("done");
        seen.push(frame.seq().expect("numbered event"));
        if done {
            break;
        }
    }
    (seen, first_event_ms.unwrap_or(0.0), epoch_ms())
}

/// Other users and an anonymous client try every read and cancel endpoint on every run. Returns each status.
async fn probe_everything(
    address: std::net::SocketAddr,
    runs: &[RunId],
) -> Vec<(&'static str, u16)> {
    let mut outcomes = Vec::new();
    for run in runs {
        let path = format!("/api/runs/{}", run.as_uuid());
        let events = format!("{path}/events");
        let cancel = format!("{path}/cancel");
        for user in 0..OTHER_USERS {
            let token = bearer(RUNS + 1 + user);
            let headers = [("Authorization", token.as_str())];
            let stream = SseResponse::open(address, &events, &headers).await;
            outcomes.push(("events", stream.status.as_u16()));
            outcomes.push(("get", support::get(address, &path, &headers).await.status));
            let cancelled = support::post(
                address,
                &cancel,
                &[
                    ("Authorization", token.as_str()),
                    ("Content-Type", "application/json"),
                ],
                "{}",
            )
            .await;
            outcomes.push(("cancel", cancelled.status));
        }
        let anonymous = SseResponse::open(address, &events, &[]).await;
        outcomes.push(("events", anonymous.status.as_u16()));
        outcomes.push(("get", support::get(address, &path, &[]).await.status));
    }
    outcomes
}

/// Every public text stored for a run, for the canary scan.
async fn public_text(pool: &PgPool, run: RunId) -> String {
    let events: String = sqlx::query_scalar(
        "SELECT COALESCE(string_agg(type || ' ' || payload::text, E'\\n' ORDER BY seq), '') \
         FROM run_events WHERE submission_id = $1",
    )
    .bind(run.as_uuid())
    .fetch_one(pool)
    .await
    .expect("events");
    let results: String = sqlx::query_scalar(
        "SELECT COALESCE(string_agg(row_to_json(t)::text, E'\\n'), '') FROM test_results t WHERE submission_id = $1",
    )
    .bind(run.as_uuid())
    .fetch_one(pool)
    .await
    .expect("results");
    format!("{events}\n{results}")
}

/// Database timings keyed by run, so each metric is joined to the right run.
async fn database_timings(pool: &PgPool) -> HashMap<Uuid, DbTimings> {
    let rows: Vec<(Uuid, f64, f64, f64)> = sqlx::query_as(
        "SELECT s.id,
                (EXTRACT(EPOCH FROM (s.started_at - s.created_at)) * 1000.0)::float8,
                (EXTRACT(EPOCH FROM (s.finished_at - s.started_at)) * 1000.0)::float8,
                (EXTRACT(EPOCH FROM e.created_at) * 1000.0)::float8
           FROM submissions s
           JOIN run_events e ON e.submission_id = s.id AND e.type = 'done'",
    )
    .fetch_all(pool)
    .await
    .expect("timings");
    rows.into_iter()
        .map(|(id, queue, execution, done_created)| {
            (
                id,
                DbTimings {
                    queue,
                    execution,
                    done_created,
                },
            )
        })
        .collect()
}

/// The number of runs as the `i64` that SQL counts use.
fn runs_i64() -> i64 {
    i64::try_from(RUNS).expect("the run count fits in i64")
}

async fn count(pool: &PgPool, sql: &str) -> i64 {
    sqlx::query_scalar(sql)
        .fetch_one(pool)
        .await
        .expect("count")
}

fn report_path() -> PathBuf {
    PathBuf::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../target/run-lifecycle/dod-report.json"
    ))
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_dod_twenty_concurrent_runs(pool: PgPool) {
    let fixture = Fixture::new();
    for user in 1..=RUNS {
        ensure_user(&pool, owner(user)).await.expect("user");
    }
    for user in 0..OTHER_USERS {
        ensure_user(&pool, owner(RUNS + 1 + user))
            .await
            .expect("other user");
    }
    publish_two_sum(&pool).await;

    let address = serve(build_router(app_state(&pool, &fixture.dir))).await;
    let (stop_tx, stop_rx) = watch::channel(false);
    let mut judges = Vec::new();
    for index in 0..WORKERS {
        judges.push(start_judge(&pool, &format!("dod-judge-{index}"), stop_rx.clone()).await);
    }

    // Phase 1: all twenty submissions fire at the same instant.
    let barrier = Arc::new(Barrier::new(RUNS));
    let mut submits = tokio::task::JoinSet::new();
    for user in 1..=RUNS {
        let barrier = barrier.clone();
        submits.spawn(async move {
            barrier.wait().await;
            submit(address, user).await
        });
    }
    let mut submitted = Vec::new();
    while let Some(result) = submits.join_next().await {
        submitted.push(result.expect("submit task"));
    }
    submitted.sort_by_key(|s| s.user);
    let runs: Vec<RunId> = submitted.iter().map(|s| s.run).collect();

    // Phase 2: streams (five resume) and the probes run at the same time.
    let probes = {
        let runs = runs.clone();
        tokio::spawn(async move { probe_everything(address, &runs).await })
    };
    let mut streams = tokio::task::JoinSet::new();
    for (index, sub) in submitted.iter().copied().enumerate() {
        streams.spawn(async move {
            let (seen, first_event_ms, done_received_ms) =
                stream_to_done(address, sub.run, sub.user, index < RESUMERS).await;
            Observed {
                run: sub.run,
                enqueue_ms: sub.enqueue_ms,
                first_event_ms,
                completion_ms: elapsed_ms(sub.sent),
                done_received_ms,
                seen,
            }
        });
    }
    let mut observed = Vec::new();
    while let Some(result) = streams.join_next().await {
        observed.push(result.expect("stream task"));
    }
    let probe_outcomes = probes.await.expect("probe task");

    stop_tx.send(true).expect("stop judges");
    for judge in judges {
        judge.await.expect("judge stopped");
    }

    // No lost jobs.
    assert_eq!(
        count(&pool, "SELECT count(*) FROM submissions").await,
        runs_i64(),
        "no lost submissions"
    );
    assert_eq!(
        count(&pool, "SELECT count(*) FROM jobs WHERE state = 'succeeded'").await,
        runs_i64()
    );
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM jobs WHERE state IN ('queued', 'claimed')"
        )
        .await,
        0
    );
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM submissions WHERE status = 'completed'"
        )
        .await,
        runs_i64()
    );
    // Nothing was cancelled: the probes must not have changed any run.
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM jobs WHERE cancel_requested_at IS NOT NULL"
        )
        .await,
        0,
        "no probe cancelled a run"
    );
    // No duplicate terminal results.
    assert_eq!(
        count(&pool, "SELECT count(*) FROM run_events WHERE type = 'done'").await,
        runs_i64()
    );
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM run_events WHERE type = 'error'"
        )
        .await,
        0
    );
    assert_eq!(
        count(&pool, "SELECT count(*) FROM test_results").await,
        runs_i64() * i64::from(TESTS_PER_RUN),
        "one result row per test"
    );
    // Sequence numbers are gap-free: the last seq equals the number of events.
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM submissions s \
             WHERE s.last_event_seq <> (SELECT count(*) FROM run_events e WHERE e.submission_id = s.id)"
        )
        .await,
        0,
        "event sequence numbers are gap-free"
    );

    // Unauthorized probes got only 401 or 404.
    assert!(!probe_outcomes.is_empty(), "the probes ran");
    for (surface, status) in &probe_outcomes {
        assert!(
            matches!(status, 401 | 404),
            "{surface} answered {status} to a probe"
        );
    }

    // Each client's stream is exactly the stored log, with no gaps and no duplicates.
    for stream in &observed {
        let last = i64::try_from(stream.seen.len()).expect("sequence count fits in i64");
        let expected: Vec<i64> = (1..=last).collect();
        assert_eq!(
            stream.seen, expected,
            "run {} stream is gap-free and unduplicated",
            stream.run
        );
    }

    // No canary on any public surface, and the scan had real text to check.
    for run in &runs {
        let text = public_text(&pool, *run).await;
        assert!(
            !text.is_empty(),
            "the public text for run {run} is not empty"
        );
        assert!(!text.contains(CANARY), "canary leaked for run {run}");
    }

    // Latency: the enqueue p95 target, and every run finished within a minute.
    let timings = database_timings(&pool).await;
    let mut enqueue = Vec::new();
    let mut first_event = Vec::new();
    let mut completion = Vec::new();
    let mut queue_wait = Vec::new();
    let mut execution = Vec::new();
    let mut delivery = Vec::new();
    for stream in &observed {
        let db = timings
            .get(&stream.run.as_uuid())
            .expect("timings for the run");
        enqueue.push(stream.enqueue_ms);
        first_event.push(stream.first_event_ms);
        completion.push(stream.completion_ms);
        queue_wait.push(db.queue);
        execution.push(db.execution);
        delivery.push((stream.done_received_ms - db.done_created).max(0.0));
    }
    let enqueue_p95 = report::stats(&enqueue).expect("enqueue samples").p95;
    assert!(
        enqueue_p95 <= 300.0,
        "enqueue p95 {enqueue_p95:.1} ms exceeds 300 ms"
    );
    let slowest = completion.iter().copied().fold(0.0_f64, f64::max);
    assert!(slowest <= 60_000.0, "a run took {slowest:.0} ms");

    let rows = [
        ("enqueue_ms", report::stats(&enqueue)),
        ("first_event_ms", report::stats(&first_event)),
        ("completion_ms", report::stats(&completion)),
        ("queue_wait_ms", report::stats(&queue_wait)),
        ("execution_ms", report::stats(&execution)),
        ("delivery_ms", report::stats(&delivery)),
    ];
    let path = report_path();
    std::fs::create_dir_all(path.parent().expect("report directory"))
        .expect("create report directory");
    let body = json!({
        "runs": RUNS,
        "judges": WORKERS,
        "judge_concurrency": 2,
        "metrics": report::to_json(&rows),
    });
    std::fs::write(&path, serde_json::to_string_pretty(&body).expect("JSON"))
        .expect("write report");
    println!("{}", report::markdown_table(&rows));
    println!("report written to {}", path.display());
}
