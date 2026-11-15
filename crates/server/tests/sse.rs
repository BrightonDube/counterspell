//! Tests for the run event stream (run-lifecycle T29 and T31, design Path 9, requirements R12 and R13).
//!
//! Run with `DATABASE_URL=<url> cargo test -p counterspell-server --test sse -- --ignored pg_`.
//! The tests open real HTTP connections to a local server and read the SSE bytes as a browser would.

mod support;

use std::{
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

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
    config::{RunLimits, ServerConfig},
    ratelimit::{Limiter, MemoryLimiter},
    runs::{
        enqueue::{enqueue, NewRun},
        sse::replay_events_from,
    },
    testkit::{ensure_user, BearerTestResolver},
    AppState,
};
use sqlx::PgPool;
use support::{
    publish_two_sum, serve,
    sse_client::{Frame, SseResponse},
    Fixture,
};
use uuid::Uuid;

fn limits() -> RunLimits {
    RunLimits {
        run_rate_user: 30,
        run_rate_ip: 100,
        run_active_cap: 3,
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

fn events_path(run: RunId) -> String {
    format!("/api/runs/{}/events", run.as_uuid())
}

/// Settings from the defaults plus `overrides`. The database URLs are never used by these tests.
fn config_with(static_dir: &Path, overrides: &[(&str, &str)]) -> ServerConfig {
    let dir = static_dir.to_string_lossy().into_owned();
    let pairs: Vec<(String, String)> = overrides
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect();
    ServerConfig::from_lookup(move |name| {
        if let Some((_, value)) = pairs.iter().find(|(key, _)| key.as_str() == name) {
            return Some(value.clone());
        }
        match name {
            "DATABASE_URL" => Some("postgres://unused@127.0.0.1:1/unused".to_owned()),
            "VALKEY_URL" => Some("redis://127.0.0.1:1".to_owned()),
            "STATIC_DIR" => Some(dir.clone()),
            _ => None,
        }
    })
    .expect("config loads")
}

fn state_with(pool: PgPool, static_dir: &Path, overrides: &[(&str, &str)]) -> AppState {
    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    AppState::new(pool, config_with(static_dir, overrides), limiter)
        .with_user_resolver(BearerTestResolver)
}

fn judge_config() -> JudgeConfig {
    JudgeConfig::from_lookup(|_name: &str| None::<String>).expect("judge defaults load")
}

/// Saves a queued run for `user` and returns its id.
async fn queued_run(pool: &PgPool, user: UserId) -> RunId {
    ensure_user(pool, user).await.expect("user fixture");
    publish_two_sum(pool).await;
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

/// Adds `count` non-terminal events, numbered from 1, the way the judge writes them.
async fn add_events(pool: &PgPool, run: RunId, count: i32) {
    sqlx::query(
        "INSERT INTO run_events (submission_id, seq, attempt, type, payload) \
         SELECT $1, g, 1, 'test', '{}'::jsonb FROM generate_series(1, $2) AS g",
    )
    .bind(run.as_uuid())
    .bind(count)
    .execute(pool)
    .await
    .expect("insert events");
}

/// Writes the terminal `done` event at `last_seq` and moves the run to `completed`, as the judge does.
async fn finish_run(pool: &PgPool, run: RunId, last_seq: i32) {
    let id = run.as_uuid();
    sqlx::query(
        "INSERT INTO run_events (submission_id, seq, attempt, type, payload) \
         VALUES ($1, $2, 1, 'done', '{\"passed\":1,\"total\":2}'::jsonb)",
    )
    .bind(id)
    .bind(last_seq)
    .execute(pool)
    .await
    .expect("done event");
    for status in ["claimed", "compiling", "running"] {
        sqlx::query("UPDATE submissions SET status = $2 WHERE id = $1")
            .bind(id)
            .bind(status)
            .execute(pool)
            .await
            .expect("move status");
    }
    sqlx::query(
        "UPDATE submissions SET status = 'completed', verdict = 'wrong_answer', passed = 1, total = 2, \
         score = 50, runtime_ms = 41, memory_kb = 9210, started_at = now(), finished_at = now(), \
         last_event_seq = $2 WHERE id = $1",
    )
    .bind(id)
    .bind(last_seq)
    .execute(pool)
    .await
    .expect("complete run");
}

/// Reads frames until the terminal `done` event, and returns the event frames in order.
async fn read_until_done(stream: &mut SseResponse) -> Vec<Frame> {
    let mut events = Vec::new();
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
    events
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_replay_is_owner_scoped(pool: PgPool) {
    let run = queued_run(&pool, owner(1)).await;
    add_events(&pool, run, 3).await;

    let as_owner = replay_events_from(&pool, owner(1), run, 0)
        .await
        .expect("owner replay");
    let as_stranger = replay_events_from(&pool, owner(2), run, 0)
        .await
        .expect("stranger replay");

    assert_eq!(
        as_owner.iter().map(|row| row.seq).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert!(
        as_stranger.is_empty(),
        "another user must see no events of this run"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_replay_pages_in_order_from_the_cursor(pool: PgPool) {
    let run = queued_run(&pool, owner(1)).await;
    add_events(&pool, run, 450).await;

    let first = replay_events_from(&pool, owner(1), run, 0)
        .await
        .expect("first page");
    let second = replay_events_from(&pool, owner(1), run, 200)
        .await
        .expect("second page");
    let last = replay_events_from(&pool, owner(1), run, 450)
        .await
        .expect("empty page");

    assert_eq!(first.len(), 200, "a page holds at most 200 events");
    assert_eq!(first.first().map(|row| row.seq), Some(1));
    assert_eq!(first.last().map(|row| row.seq), Some(200));
    assert_eq!(second.first().map(|row| row.seq), Some(201));
    assert_eq!(second.last().map(|row| row.seq), Some(400));
    assert!(last.is_empty(), "nothing after the last event");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_sse_requires_session_and_ownership(pool: PgPool) {
    let fixture = Fixture::new();
    let run = queued_run(&pool, owner(1)).await;
    let address = serve(build_router(state_with(pool.clone(), &fixture.dir, &[]))).await;
    let token_two = auth(2);
    let token_one = auth(1);

    let anonymous = SseResponse::open(address, &events_path(run), &[]).await;
    assert_eq!(anonymous.status, 401);

    let foreign = SseResponse::open(
        address,
        &events_path(run),
        &[("Authorization", token_two.as_str())],
    )
    .await;
    let foreign_status = foreign.status;
    let foreign_body = foreign.read_body().await;

    let missing = SseResponse::open(
        address,
        &events_path(RunId::new()),
        &[("Authorization", token_two.as_str())],
    )
    .await;
    let missing_status = missing.status;
    let missing_body = missing.read_body().await;

    let malformed = SseResponse::open(
        address,
        "/api/runs/not-a-run/events",
        &[("Authorization", token_one.as_str())],
    )
    .await;
    let malformed_status = malformed.status;
    let malformed_body = malformed.read_body().await;

    assert_eq!(foreign_status, 404);
    assert_eq!(missing_status, 404);
    assert_eq!(malformed_status, 404);
    assert_eq!(
        foreign_body, missing_body,
        "foreign and missing runs look the same"
    );
    assert_eq!(
        malformed_body, missing_body,
        "malformed ids look the same too"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_sse_replay_after_drop_at_every_position(pool: PgPool) {
    let fixture = Fixture::new();
    let run = queued_run(&pool, owner(1)).await;
    add_events(&pool, run, 4).await;
    finish_run(&pool, run, 5).await;
    let address = serve(build_router(state_with(pool.clone(), &fixture.dir, &[]))).await;
    let token = auth(1);

    for dropped_after in 0..4 {
        let mut first = SseResponse::open(
            address,
            &events_path(run),
            &[("Authorization", token.as_str())],
        )
        .await;
        let mut seen: Vec<i64> = Vec::new();
        for _ in 0..dropped_after {
            let frame = first
                .next_frame_within(Duration::from_secs(2))
                .await
                .expect("a replayed event");
            seen.push(frame.seq().expect("numbered event"));
        }
        // The connection drops here, as a flaky network would drop it.
        drop(first);

        let resume_from = seen.last().copied().unwrap_or(0).to_string();
        let mut second = SseResponse::open(
            address,
            &events_path(run),
            &[
                ("Authorization", token.as_str()),
                ("Last-Event-ID", resume_from.as_str()),
            ],
        )
        .await;
        for frame in read_until_done(&mut second).await {
            seen.push(frame.seq().expect("numbered event"));
        }

        assert_eq!(
            seen,
            vec![1, 2, 3, 4, 5],
            "dropped after {dropped_after} events"
        );
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_sse_poll_only_without_notify(pool: PgPool) {
    // The default waker never sends, so only the 250 ms poll can deliver this event.
    let fixture = Fixture::new();
    let run = queued_run(&pool, owner(1)).await;
    let address = serve(build_router(state_with(pool.clone(), &fixture.dir, &[]))).await;
    let token = auth(1);

    let mut stream = SseResponse::open(
        address,
        &events_path(run),
        &[("Authorization", token.as_str())],
    )
    .await;
    add_events(&pool, run, 1).await;

    let frame = stream
        .next_frame_within(Duration::from_secs(1))
        .await
        .expect("the poll delivers the event within a second");
    assert_eq!(frame.seq(), Some(1));
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_sse_last_event_id_forms(pool: PgPool) {
    let fixture = Fixture::new();
    let run = queued_run(&pool, owner(1)).await;
    add_events(&pool, run, 4).await;
    finish_run(&pool, run, 5).await;
    let address = serve(build_router(state_with(pool.clone(), &fixture.dir, &[]))).await;
    let token = auth(1);
    let auth_header = ("Authorization", token.as_str());
    let path = events_path(run);

    let mut by_header =
        SseResponse::open(address, &path, &[auth_header, ("Last-Event-ID", "2")]).await;
    let by_header_first = first_event_seq(&mut by_header).await;
    assert_eq!(by_header_first, Some(3));

    let mut by_query =
        SseResponse::open(address, &format!("{path}?last_event_id=2"), &[auth_header]).await;
    assert_eq!(first_event_seq(&mut by_query).await, Some(3));

    let mut header_wins = SseResponse::open(
        address,
        &format!("{path}?last_event_id=3"),
        &[auth_header, ("Last-Event-ID", "1")],
    )
    .await;
    assert_eq!(first_event_seq(&mut header_wins).await, Some(2));

    for bad in ["abc", "-1"] {
        let rejected =
            SseResponse::open(address, &path, &[auth_header, ("Last-Event-ID", bad)]).await;
        assert_eq!(rejected.status, 400, "{bad}");
        let body: serde_json::Value =
            serde_json::from_str(&rejected.read_body().await).expect("JSON error");
        assert_eq!(body["code"], "invalid_last_event_id", "{bad}");
    }

    let caught_up = SseResponse::open(address, &path, &[auth_header, ("Last-Event-ID", "9")]).await;
    assert_eq!(
        caught_up.status, 204,
        "a finished run past its last event has nothing to send"
    );
}

/// The sequence number of the first event a stream sends, or `None` when it sends nothing.
async fn first_event_seq(stream: &mut SseResponse) -> Option<i64> {
    loop {
        let frame = stream.next_frame_within(Duration::from_secs(2)).await?;
        if frame.kind().is_some() {
            return frame.seq();
        }
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_sse_heartbeat_headers_and_close(pool: PgPool) {
    let fixture = Fixture::new();
    let run = queued_run(&pool, owner(1)).await;
    let overrides = [("SSE_HEARTBEAT_SECS", "1")];
    let address = serve(build_router(state_with(
        pool.clone(),
        &fixture.dir,
        &overrides,
    )))
    .await;
    let token = auth(1);

    let mut stream = SseResponse::open(
        address,
        &events_path(run),
        &[("Authorization", token.as_str())],
    )
    .await;
    assert_eq!(stream.status, 200);
    assert_eq!(stream.headers["content-type"], "text/event-stream");
    assert_eq!(stream.headers["cache-control"], "no-store");
    assert_eq!(stream.headers["x-accel-buffering"], "no");
    assert_eq!(stream.headers["x-content-type-options"], "nosniff");

    let heartbeat = stream
        .next_frame_within(Duration::from_secs(3))
        .await
        .expect("a heartbeat while the run is idle");
    assert!(matches!(heartbeat, Frame::Comment(_)), "{heartbeat:?}");

    add_events(&pool, run, 4).await;
    finish_run(&pool, run, 5).await;
    let events = read_until_done(&mut stream).await;
    assert_eq!(
        events.iter().filter_map(Frame::seq).collect::<Vec<_>>(),
        vec![1, 2, 3, 4, 5]
    );
    assert!(
        stream
            .next_frame_within(Duration::from_secs(2))
            .await
            .is_none(),
        "the stream closes after the terminal event"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_sse_stream_limit_per_user(pool: PgPool) {
    let fixture = Fixture::new();
    let run = queued_run(&pool, owner(1)).await;
    let other = queued_run(&pool, owner(2)).await;
    let overrides = [
        ("SSE_MAX_STREAMS_PER_USER", "2"),
        ("SSE_HEARTBEAT_SECS", "1"),
    ];
    let address = serve(build_router(state_with(
        pool.clone(),
        &fixture.dir,
        &overrides,
    )))
    .await;
    let token_one = auth(1);
    let token_two = auth(2);

    let first = SseResponse::open(
        address,
        &events_path(run),
        &[("Authorization", token_one.as_str())],
    )
    .await;
    let second = SseResponse::open(
        address,
        &events_path(run),
        &[("Authorization", token_one.as_str())],
    )
    .await;
    assert_eq!(first.status, 200);
    assert_eq!(second.status, 200);

    let refused = SseResponse::open(
        address,
        &events_path(run),
        &[("Authorization", token_one.as_str())],
    )
    .await;
    assert_eq!(refused.status, 429);
    assert!(refused.headers.contains_key("retry-after"));
    let body: serde_json::Value = serde_json::from_str(&refused.read_body().await).expect("JSON");
    assert_eq!(body["code"], "too_many_streams");

    let other_user = SseResponse::open(
        address,
        &events_path(other),
        &[("Authorization", token_two.as_str())],
    )
    .await;
    assert_eq!(
        other_user.status, 200,
        "one user's streams do not use another user's slots"
    );

    // Dropping a stream frees its slot once the server notices the closed socket (at the next heartbeat).
    drop(first);
    let deadline = Instant::now() + Duration::from_secs(6);
    loop {
        let retry = SseResponse::open(
            address,
            &events_path(run),
            &[("Authorization", token_one.as_str())],
        )
        .await;
        if retry.status == 200 {
            break;
        }
        assert!(
            Instant::now() < deadline,
            "the slot was not freed after the drop"
        );
        tokio::time::sleep(Duration::from_millis(200)).await;
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_sse_max_age_closes_and_resumes(pool: PgPool) {
    let fixture = Fixture::new();
    let run = queued_run(&pool, owner(1)).await;
    add_events(&pool, run, 1).await;
    let overrides = [("SSE_MAX_AGE_SECS", "1")];
    let address = serve(build_router(state_with(
        pool.clone(),
        &fixture.dir,
        &overrides,
    )))
    .await;
    let token = auth(1);

    let mut stream = SseResponse::open(
        address,
        &events_path(run),
        &[("Authorization", token.as_str())],
    )
    .await;
    assert_eq!(first_event_seq(&mut stream).await, Some(1));

    // The one-second max age closes the stream while the run is still open.
    while stream
        .next_frame_within(Duration::from_secs(3))
        .await
        .is_some()
    {}

    finish_run(&pool, run, 2).await;
    let mut resumed = SseResponse::open(
        address,
        &events_path(run),
        &[("Authorization", token.as_str()), ("Last-Event-ID", "1")],
    )
    .await;
    let seqs: Vec<i64> = read_until_done(&mut resumed)
        .await
        .iter()
        .filter_map(Frame::seq)
        .collect();
    assert_eq!(
        seqs,
        vec![2],
        "the resumed stream sends only the events after the cursor"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_sse_restarted_event_visible_after_retry(pool: PgPool) {
    let fixture = Fixture::new();
    let run = queued_run(&pool, owner(1)).await;

    // The first judge crashes after one event. Its lease is released for a retry.
    let crashing = Worker::new(
        pool.clone(),
        judge_config(),
        ScriptedExecutor::new(Script::CrashAfter(1)),
        "crashing-judge".to_owned(),
    );
    assert!(crashing.process_next().await.expect("first claim"));
    // The retry waits for a backoff. Move it to now so the second judge can claim it.
    sqlx::query("UPDATE jobs SET run_after = now() WHERE submission_id = $1")
        .bind(run.as_uuid())
        .execute(&pool)
        .await
        .expect("end the backoff");

    let healthy = Worker::new(
        pool.clone(),
        judge_config(),
        ScriptedExecutor::new(Script::Pass {
            tests: 1,
            delay: Duration::ZERO,
        }),
        "healthy-judge".to_owned(),
    );
    assert!(healthy.process_next().await.expect("retry claim"));

    let address = serve(build_router(state_with(pool.clone(), &fixture.dir, &[]))).await;
    let token = auth(1);
    let mut stream = SseResponse::open(
        address,
        &events_path(run),
        &[("Authorization", token.as_str())],
    )
    .await;
    let events = read_until_done(&mut stream).await;

    let kinds: Vec<&str> = events.iter().filter_map(Frame::kind).collect();
    assert_eq!(
        kinds,
        vec!["compiling", "restarted", "compiling", "test", "done"]
    );
    let attempts: Vec<i64> = events
        .iter()
        .map(|frame| frame.envelope()["attempt"].as_i64().expect("attempt"))
        .collect();
    assert_eq!(attempts, vec![1, 2, 2, 2, 2], "restarted opens attempt 2");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_sse_sends_nothing_after_the_terminal_event(pool: PgPool) {
    // A row written after `done` is not a normal state, but the stream must still stop at `done`.
    let fixture = Fixture::new();
    let run = queued_run(&pool, owner(1)).await;
    add_events(&pool, run, 1).await;
    finish_run(&pool, run, 2).await;
    sqlx::query(
        "INSERT INTO run_events (submission_id, seq, attempt, type, payload) \
         VALUES ($1, 3, 1, 'test', '{}'::jsonb)",
    )
    .bind(run.as_uuid())
    .execute(&pool)
    .await
    .expect("row after done");
    let address = serve(build_router(state_with(pool.clone(), &fixture.dir, &[]))).await;
    let token = auth(1);

    let mut stream = SseResponse::open(
        address,
        &events_path(run),
        &[("Authorization", token.as_str())],
    )
    .await;
    let seqs: Vec<i64> = read_until_done(&mut stream)
        .await
        .iter()
        .filter_map(Frame::seq)
        .collect();

    assert_eq!(seqs, vec![1, 2], "nothing may follow the terminal event");
    assert!(
        stream
            .next_frame_within(Duration::from_secs(2))
            .await
            .is_none(),
        "the stream closes at the terminal event"
    );
}
