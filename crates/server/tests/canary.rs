//! Canary privacy test (run-lifecycle T34, requirement R15 and property P12).
//!
//! Hidden data must never reach a public surface. Each run here carries unique marker strings in its hidden
//! data. A hostile executor puts them into its failure text. The test then scans every public surface:
//! `run_events`, `test_results`, `GET /api/runs/{id}` and the raw SSE bytes.
//!
//! Run with `DATABASE_URL=<url> cargo test -p counterspell-server --test canary -- --ignored pg_`.

mod support;

use std::{sync::Arc, time::Duration};

use counterspell_common::{
    events::{PublicEvent, PublicTest},
    ids::{RunId, UserId},
    status::{RunMode, TestVerdict, Verdict},
};
use counterspell_judge::{
    config::JudgeConfig,
    executor::{ExecError, RunContext, RunExecutor, RunInput, RunOutcome},
    queue::finish::{FinalResult, TestRecord},
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
use serde_json::Value;
use sqlx::PgPool;
use support::{config_for, get, publish_two_sum, serve, sse_client::SseResponse, Fixture};
use uuid::Uuid;

/// Marker strings that stand in for hidden data. They are unique per run, so a match can only come from
/// that run's hidden data.
#[derive(Clone)]
struct Markers {
    input: String,
    expected: String,
    output: String,
}

impl Markers {
    fn fresh() -> Self {
        let id = Uuid::new_v4().simple();
        Self {
            input: format!("canary-input-{id}"),
            expected: format!("canary-expected-{id}"),
            output: format!("canary-output-{id}"),
        }
    }

    fn all(&self) -> [&str; 3] {
        [&self.input, &self.expected, &self.output]
    }
}

/// Plays a run the way a judge would, while trying to leak the hidden markers into public output.
///
/// The first attempt fails with a message that contains the markers. That message is internal: it goes to
/// the job's `last_error`, never to a public event. The second attempt writes only typed public events and
/// returns hidden test records, which carry no text fields at all.
struct HostileExecutor {
    markers: Markers,
}

impl RunExecutor for HostileExecutor {
    async fn execute(&self, run: RunInput, ctx: RunContext) -> Result<RunOutcome, ExecError> {
        if run.attempt == 1 {
            return Err(ExecError::Infrastructure(format!(
                "sandbox failed on {} expecting {}",
                self.markers.input, self.markers.expected
            )));
        }
        ctx.emit(&PublicEvent::Compiling).await?;
        let mut tests = Vec::new();
        for ordinal in 1..=3 {
            let verdict = TestVerdict::Pass;
            ctx.emit(&PublicEvent::Test(PublicTest::Hidden {
                ordinal,
                verdict,
                time_ms: 2,
            }))
            .await?;
            tests.push(TestRecord::Hidden {
                ordinal,
                verdict,
                time_ms: 2,
            });
        }
        Ok(RunOutcome::Completed(FinalResult {
            verdict: Verdict::Accepted,
            passed: 3,
            total: 3,
            score: 100,
            runtime_ms: 2,
            compile_output: Some("compiled without errors".to_owned()),
            measured: None,
            tests,
        }))
    }
}

fn limits() -> RunLimits {
    RunLimits {
        run_rate_user: 30,
        run_rate_ip: 100,
        run_active_cap: 10,
        queue_depth_max: 200,
        max_source_bytes: Some(65_536),
        request_body_limit_bytes: 131_072,
    }
}

fn judge_config() -> JudgeConfig {
    JudgeConfig::from_lookup(|_name: &str| None::<String>).expect("judge defaults load")
}

/// Saves a queued run in `mode` for `user`, runs it through a failed attempt and a retry, and returns it.
async fn run_with_retry(pool: &PgPool, user: UserId, mode: RunMode, markers: Markers) -> RunId {
    ensure_user(pool, user).await.expect("user fixture");
    publish_two_sum(pool).await;
    let run = NewRun::new(
        RunId::new(),
        user,
        "two-sum",
        "python",
        mode,
        "print(1)\n",
        None,
    );
    enqueue(pool, &run, &limits()).await.expect("enqueue");

    let worker = Worker::new(
        pool.clone(),
        judge_config(),
        HostileExecutor {
            markers: markers.clone(),
        },
        "canary-judge".to_owned(),
    );
    assert!(
        worker.process_next().await.expect("first claim"),
        "first attempt claimed"
    );
    // The retry waits for a backoff. Move it to now so the same test can run the second attempt.
    sqlx::query("UPDATE jobs SET run_after = now() WHERE submission_id = $1")
        .bind(run.run_id.as_uuid())
        .execute(pool)
        .await
        .expect("end the backoff");
    assert!(
        worker.process_next().await.expect("retry claim"),
        "retry claimed"
    );
    run.run_id
}

/// Every public text a learner or another user could read for `run`, as one string per surface.
async fn public_surfaces(
    pool: &PgPool,
    address: std::net::SocketAddr,
    run: RunId,
    token: &str,
) -> Vec<(&'static str, String)> {
    let events: String = sqlx::query_scalar(
        "SELECT COALESCE(string_agg(type || ' ' || payload::text, E'\\n' ORDER BY seq), '') \
         FROM run_events WHERE submission_id = $1",
    )
    .bind(run.as_uuid())
    .fetch_one(pool)
    .await
    .expect("run events");
    let results: String = sqlx::query_scalar(
        "SELECT COALESCE(string_agg(row_to_json(t)::text, E'\\n'), '') \
         FROM test_results t WHERE submission_id = $1",
    )
    .bind(run.as_uuid())
    .fetch_one(pool)
    .await
    .expect("test results");

    let path = format!("/api/runs/{}", run.as_uuid());
    let reply = get(address, &path, &[("Authorization", token)]).await;
    assert_eq!(reply.status, 200, "the owner reads the run");

    // The dashboard lists the user's recent runs, so it is a public surface too.
    let dashboard = get(address, "/api/dashboard", &[("Authorization", token)]).await;
    assert_eq!(dashboard.status, 200, "the owner reads the dashboard");

    let mut stream = SseResponse::open(
        address,
        &format!("/api/runs/{}/events", run.as_uuid()),
        &[("Authorization", token)],
    )
    .await;
    while stream
        .next_frame_within(Duration::from_secs(3))
        .await
        .is_some()
    {}

    vec![
        ("run_events", events),
        ("test_results", results),
        ("GET /api/runs/{id}", reply.body),
        ("GET /api/dashboard", dashboard.body),
        ("raw SSE bytes", stream.transcript().to_owned()),
    ]
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_canary_not_in_any_public_surface(pool: PgPool) {
    let fixture = Fixture::new();
    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    let state = AppState::new(pool.clone(), config_for(&fixture.dir), limiter)
        .with_user_resolver(BearerTestResolver);
    let address = serve(build_router(state)).await;
    let token = "Bearer test-user-1";

    for mode in [RunMode::Run, RunMode::Submit] {
        let markers = Markers::fresh();
        let run = run_with_retry(
            &pool,
            UserId::from_uuid(Uuid::from_u128(1)),
            mode,
            markers.clone(),
        )
        .await;

        let surfaces = public_surfaces(&pool, address, run, token).await;
        for (surface, text) in &surfaces {
            // Vacuity guard: an empty surface would pass the scan without proving anything.
            assert!(!text.is_empty(), "{surface} must hold the public result");
            for marker in markers.all() {
                assert!(!text.contains(marker), "marker found in {surface}");
            }
        }
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_hidden_test_payload_keys_are_exact(pool: PgPool) {
    let markers = Markers::fresh();
    let run = run_with_retry(
        &pool,
        UserId::from_uuid(Uuid::from_u128(1)),
        RunMode::Submit,
        markers,
    )
    .await;

    let payloads: Vec<Value> = sqlx::query_scalar(
        "SELECT payload FROM run_events WHERE submission_id = $1 AND type = 'test' ORDER BY seq",
    )
    .bind(run.as_uuid())
    .fetch_all(&pool)
    .await
    .expect("test payloads");

    assert_eq!(payloads.len(), 3, "one event per hidden test");
    for payload in payloads {
        let mut keys: Vec<&str> = payload
            .as_object()
            .expect("payload is an object")
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();
        assert_eq!(keys, vec!["ordinal", "sample", "time_ms", "verdict"]);
        assert_eq!(payload["sample"], Value::Bool(false));
    }
}
