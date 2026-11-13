//! Tests for `GET /api/runs/{id}` (run-lifecycle T26, design Path 10 and requirements R16).
//!
//! Run with `DATABASE_URL=<url> cargo test -p counterspell-server --test get_run -- --ignored pg_`.

mod support;

use std::sync::Arc;

use counterspell_common::{
    ids::{RunId, UserId},
    status::RunMode,
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
use support::{config_for, get, serve, Fixture, Reply};
use uuid::Uuid;

fn state_with(pool: PgPool, static_dir: &std::path::Path) -> AppState {
    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    AppState::new(pool, config_for(static_dir), limiter).with_user_resolver(BearerTestResolver)
}

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

/// The owner the test identity `test-user-<n>` maps to.
fn owner(number: u128) -> UserId {
    UserId::from_uuid(Uuid::from_u128(number))
}

fn auth(number: u128) -> String {
    format!("Bearer test-user-{number}")
}

/// Saves a queued run for `user`, the same way `POST /api/runs` does, and returns its id.
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

/// Moves a queued run to `completed` with one sample and one hidden test, through the allowed states.
async fn complete_by_hand(pool: &PgPool, run: RunId) {
    let id = run.as_uuid();
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
         last_event_seq = 3 WHERE id = $1",
    )
    .bind(id)
    .execute(pool)
    .await
    .expect("complete run");
    sqlx::query("UPDATE jobs SET state = 'succeeded', lease_owner = NULL, lease_expires_at = NULL WHERE submission_id = $1")
        .bind(id)
        .execute(pool)
        .await
        .expect("succeed job");
    sqlx::query(
        "INSERT INTO test_results (submission_id, ordinal, is_sample, verdict, time_ms, public_detail) \
         VALUES ($1, 1, true, 'pass', 3, '{\"input\":\"[1]\",\"expected\":\"1\",\"got\":\"1\"}'), \
                ($1, 2, false, 'wrong_answer', 5, NULL)",
    )
    .bind(id)
    .execute(pool)
    .await
    .expect("insert results");
}

fn json(reply: &Reply) -> Value {
    serde_json::from_str(&reply.body).expect("reply body is JSON")
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_get_run_requires_session(pool: PgPool) {
    let fixture = Fixture::new();
    let run = queued_run(&pool, owner(1)).await;
    let address = serve(build_router(state_with(pool.clone(), &fixture.dir))).await;

    let reply = get(address, &format!("/api/runs/{}", run.as_uuid()), &[]).await;

    assert_eq!(reply.status, 401);
    assert_eq!(json(&reply)["code"], "unauthorized");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_get_run_foreign_and_missing_are_identical_404(pool: PgPool) {
    let fixture = Fixture::new();
    let run = queued_run(&pool, owner(1)).await;
    let address = serve(build_router(state_with(pool.clone(), &fixture.dir))).await;

    let foreign = get(
        address,
        &format!("/api/runs/{}", run.as_uuid()),
        &[("Authorization", auth(2).as_str())],
    )
    .await;
    let missing = get(
        address,
        &format!("/api/runs/{}", RunId::new()),
        &[("Authorization", auth(1).as_str())],
    )
    .await;
    let malformed = get(
        address,
        "/api/runs/not-a-run-id",
        &[("Authorization", auth(1).as_str())],
    )
    .await;

    for reply in [&foreign, &missing, &malformed] {
        assert_eq!(reply.status, 404);
        assert_eq!(json(reply)["code"], "not_found");
    }
    assert_eq!(foreign.body, missing.body);
    assert_eq!(foreign.body, malformed.body);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_get_run_nonterminal_shape(pool: PgPool) {
    let fixture = Fixture::new();
    let run = queued_run(&pool, owner(1)).await;
    let address = serve(build_router(state_with(pool.clone(), &fixture.dir))).await;

    let reply = get(
        address,
        &format!("/api/runs/{}", run.as_uuid()),
        &[("Authorization", auth(1).as_str())],
    )
    .await;

    assert_eq!(reply.status, 200);
    let body = json(&reply);
    assert_eq!(body["run_id"], run.to_string());
    assert_eq!(body["problem"], "two-sum");
    assert_eq!(body["status"], "queued");
    assert_eq!(body["terminal"], false);
    assert_eq!(body["verdict"], Value::Null);
    assert_eq!(body["attempt"], 1);
    assert_eq!(body["tests"], serde_json::json!([]));
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_get_run_terminal_reload_is_identical_and_hides_hidden_detail(pool: PgPool) {
    let fixture = Fixture::new();
    let run = queued_run(&pool, owner(1)).await;
    complete_by_hand(&pool, run).await;
    let address = serve(build_router(state_with(pool.clone(), &fixture.dir))).await;
    let path = format!("/api/runs/{}", run.as_uuid());

    let first = get(address, &path, &[("Authorization", auth(1).as_str())]).await;
    let second = get(address, &path, &[("Authorization", auth(1).as_str())]).await;

    assert_eq!(first.status, 200);
    assert_eq!(
        first.body, second.body,
        "two reads must match byte for byte"
    );
    let body = json(&first);
    assert_eq!(body["status"], "completed");
    assert_eq!(body["terminal"], true);
    assert_eq!(body["verdict"], "wrong_answer");
    assert_eq!(body["passed"], 1);
    assert_eq!(body["total"], 2);
    assert_eq!(body["last_seq"], 3);
    assert!(body["finished_at"].is_string());

    let tests = body["tests"].as_array().expect("tests array");
    assert_eq!(tests.len(), 2);
    assert_eq!(tests[0]["sample"], true);
    assert_eq!(tests[0]["detail"]["input"], "[1]");
    assert_eq!(tests[0]["detail"]["expected"], "1");
    assert_eq!(tests[0]["detail"]["got"], "1");
    assert_eq!(tests[1]["sample"], false);
    assert!(
        tests[1].get("detail").is_none(),
        "a hidden test must carry no detail key"
    );
}
