//! Tests for `POST /api/runs/ghost` (ide-workspace T02, design "Custom ghost run contract").
//!
//! The validation tests use a database pool that never connects, like `post_runs.rs`: a request
//! the checks refuse produces its exact error code, which also proves nothing was looked up or
//! queued. The `pg_` tests need a throwaway Postgres 17:
//! `DATABASE_URL=<url> cargo test -p counterspell-server --test ghost_runs -- --ignored pg_`.

mod support;

use std::{path::Path, sync::Arc};

use counterspell_common::ids::UserId;
use counterspell_server::{
    build_router,
    config::ServerConfig,
    ratelimit::{Limiter, MemoryLimiter},
    testkit::{ensure_user, BearerTestResolver, InMemoryProblems},
    AppState,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use support::{lazy_pool, post, serve, Fixture, Reply};
use uuid::Uuid;

const USER: u32 = 1;
const OTHER: u32 = 11;

async fn seed_users(pool: &PgPool) {
    for number in [USER, OTHER] {
        ensure_user(pool, UserId::from_uuid(Uuid::from_u128(u128::from(number))))
            .await
            .expect("user fixture");
    }
    support::publish_two_sum(pool).await;
}

fn config_with(static_dir: &Path, overrides: &[(&str, &str)]) -> ServerConfig {
    let dir = static_dir.to_string_lossy().into_owned();
    let overrides: Vec<(String, String)> = overrides
        .iter()
        .map(|(name, value)| ((*name).to_owned(), (*value).to_owned()))
        .collect();
    ServerConfig::from_lookup(move |name| {
        if let Some((_, value)) = overrides.iter().find(|(key, _)| key.as_str() == name) {
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

fn app_state(pool: PgPool, config: ServerConfig) -> AppState {
    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    AppState::new(pool, config, limiter)
        .with_user_resolver(BearerTestResolver)
        .with_problems(InMemoryProblems::new(&["two-sum"]))
}

fn ghost_body(problem: &str, lang: &str, code: &str, submission: &str, input: &Value) -> String {
    json!({
        "problem": problem,
        "lang": lang,
        "code": code,
        "ghost": { "submission_id": submission, "input": input },
    })
    .to_string()
}

async fn submit(address: std::net::SocketAddr, user: u32, key: Option<&str>, body: &str) -> Reply {
    let auth = format!("Bearer test-user-{user}");
    let mut headers = vec![
        ("Authorization", auth.as_str()),
        ("Content-Type", "application/json"),
    ];
    if let Some(key) = key {
        headers.push(("Idempotency-Key", key));
    }
    post(address, "/api/runs/ghost", &headers, body).await
}

fn json_body(reply: &Reply) -> Value {
    serde_json::from_str(&reply.body).expect("reply body is JSON")
}

fn error_code(reply: &Reply) -> String {
    json_body(reply)["code"]
        .as_str()
        .expect("error has a code")
        .to_owned()
}

/// The source submission id every validation-only test can use. These tests never reach the
/// ownership lookup, so the id does not need a row.
const SOURCE: &str = "00000000-0000-0000-0000-0000000000aa";

async fn count_submissions(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM submissions")
        .fetch_one(pool)
        .await
        .expect("count submissions")
}

async fn count_jobs(pool: &PgPool) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM jobs")
        .fetch_one(pool)
        .await
        .expect("count jobs")
}

fn code() -> &'static str {
    "print(1)\n"
}

/// The validation cases: expected code, content type and body. Kept out of the test so
/// both stay under the function-length limit.
fn rejected_requests() -> Vec<(&'static str, &'static str, String)> {
    let json = "application/json";
    vec![
        (
            "source_too_large",
            json,
            ghost_body(
                "two-sum",
                "python",
                &"#".repeat(65_537),
                SOURCE,
                &json!([1, 2]),
            ),
        ),
        (
            "invalid_source",
            json,
            ghost_body(
                "two-sum",
                "python",
                "print(1)\u{0}\n",
                SOURCE,
                &json!([1, 2]),
            ),
        ),
        (
            "source_does_not_parse",
            json,
            ghost_body("two-sum", "python", "def (:\n", SOURCE, &json!([1, 2])),
        ),
        (
            "invalid_request",
            json,
            ghost_body("two-sum", "ruby", code(), SOURCE, &json!([1, 2])),
        ),
        (
            "invalid_request",
            json,
            // `mode` is not part of the ghost contract.
            json!({
                "problem": "two-sum", "lang": "python", "code": code(),
                "mode": "run",
                "ghost": {"submission_id": SOURCE, "input": [1, 2]},
            })
            .to_string(),
        ),
        (
            "invalid_request",
            json,
            json!({
                "problem": "two-sum", "lang": "python", "code": code(),
                "ghost": {"submission_id": SOURCE, "input": [1, 2]}, "extra": true,
            })
            .to_string(),
        ),
        (
            "invalid_request",
            json,
            // missing `input`
            ghost_body_missing(
                "two-sum",
                "python",
                code(),
                &json!({"submission_id": SOURCE}),
            ),
        ),
        (
            "invalid_request",
            json,
            // missing `ghost`
            json!({"problem": "two-sum", "lang": "python", "code": code()}).to_string(),
        ),
        (
            "invalid_request",
            json,
            ghost_body("two-sum", "python", code(), "not-a-uuid", &json!([1, 2])),
        ),
        (
            "ghost_input_too_large",
            json,
            ghost_body(
                "two-sum",
                "python",
                code(),
                SOURCE,
                &json!("x".repeat(65_537)),
            ),
        ),
        (
            "unsupported_media_type",
            "text/plain",
            ghost_body("two-sum", "python", code(), SOURCE, &json!([1, 2])),
        ),
        (
            "unknown_problem",
            json,
            ghost_body("no-such-problem", "python", code(), SOURCE, &json!([1, 2])),
        ),
    ]
}

#[tokio::test]
async fn validation_rejects_without_side_effects() {
    let fixture = Fixture::new();
    let state = app_state(lazy_pool(), config_with(&fixture.dir, &[]));
    let address = serve(build_router(state)).await;

    for (expected, content_type, body) in rejected_requests() {
        let auth = format!("Bearer test-user-{USER}");
        let reply = post(
            address,
            "/api/runs/ghost",
            &[
                ("Authorization", auth.as_str()),
                ("Content-Type", content_type),
            ],
            &body,
        )
        .await;
        assert_eq!(
            error_code(&reply),
            expected,
            "body {body} should fail with {expected}, got: {}",
            reply.body
        );
    }
}

fn ghost_body_missing(problem: &str, lang: &str, code: &str, ghost: &Value) -> String {
    json!({"problem": problem, "lang": lang, "code": code, "ghost": ghost}).to_string()
}

#[tokio::test]
async fn ghost_run_rejects_a_problem_language_pair_without_starter() {
    // LC05: the ghost path applies the same problem-language contract as normal runs.
    let fixture = Fixture::new();
    let state = app_state(lazy_pool(), config_with(&fixture.dir, &[]))
        .with_problems(InMemoryProblems::new(&["two-sum"]).with_languages(&["python"]));
    let address = serve(build_router(state)).await;

    let reply = submit(
        address,
        1,
        None,
        &ghost_body("two-sum", "java", "class S {}\n", SOURCE, &json!([1, 2])),
    )
    .await;

    assert_eq!(reply.status, 400);
    assert_eq!(error_code(&reply), "unsupported_language");
}

#[tokio::test]
async fn ghost_run_requires_a_signed_in_user() {
    let fixture = Fixture::new();
    let state = app_state(lazy_pool(), config_with(&fixture.dir, &[]));
    let address = serve(build_router(state)).await;
    let reply = post(
        address,
        "/api/runs/ghost",
        &[("Content-Type", "application/json")],
        &ghost_body("two-sum", "python", code(), SOURCE, &json!([1, 2])),
    )
    .await;
    assert_eq!(reply.status, 401);
    assert_eq!(error_code(&reply), "unauthenticated");
}

/// Posts a normal run as `user`, returning its `run_id`. Used to create the source
/// submission a ghost run points at.
async fn post_plain_run(address: std::net::SocketAddr, user: u32, key: &str) -> String {
    let auth = format!("Bearer test-user-{user}");
    let reply = post(
        address,
        "/api/runs",
        &[
            ("Authorization", auth.as_str()),
            ("Content-Type", "application/json"),
            ("Idempotency-Key", key),
        ],
        &json!({
            "problem": "two-sum", "lang": "python", "code": "print(1)\n", "mode": "submit",
        })
        .to_string(),
    )
    .await;
    assert_eq!(reply.status, 202, "source run must queue: {}", reply.body);
    json_body(&reply)["run_id"]
        .as_str()
        .expect("run_id")
        .to_owned()
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_ghost_run_rejects_missing_or_foreign_source(pool: PgPool) {
    seed_users(&pool).await;
    let fixture = Fixture::new();
    let address = serve(build_router(app_state(
        pool.clone(),
        config_with(&fixture.dir, &[]),
    )))
    .await;

    let foreign = post_plain_run(address, OTHER, "foreign-source").await;
    for source in [foreign.as_str(), "00000000-0000-0000-0000-0000000000ee"] {
        let reply = submit(
            address,
            USER,
            None,
            &ghost_body("two-sum", "python", code(), source, &json!([1, 2])),
        )
        .await;
        assert_eq!(reply.status, 404, "{source}: {}", reply.body);
        assert_eq!(error_code(&reply), "unknown_submission");
    }
    assert_eq!(count_submissions(&pool).await, 1);
    assert_eq!(count_jobs(&pool).await, 1);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_ghost_run_accepts_and_persists_atomically(pool: PgPool) {
    seed_users(&pool).await;
    let fixture = Fixture::new();
    let address = serve(build_router(app_state(
        pool.clone(),
        config_with(&fixture.dir, &[]),
    )))
    .await;

    let source = post_plain_run(address, USER, "the-source").await;
    let input = json!([3, 1, 4]);
    let reply = submit(
        address,
        USER,
        Some("ghost-1"),
        &ghost_body("two-sum", "python", code(), &source, &input),
    )
    .await;

    assert_eq!(reply.status, 202, "{}", reply.body);
    let body = json_body(&reply);
    let run_id = body["run_id"].as_str().expect("run_id").to_owned();
    assert_ne!(run_id, source, "a ghost run is a new run id");
    assert_eq!(body["status"], "queued");
    assert_eq!(body["mode"], "run");
    assert_eq!(body["custom"], true);
    assert!(reply
        .headers
        .to_ascii_lowercase()
        .contains(&format!("location: /api/runs/{run_id}")));

    let (mode, ghost_input, ghost_source): (String, Option<Value>, Option<Uuid>) =
        sqlx::query_as("SELECT mode, ghost_input, ghost_source_id FROM submissions WHERE id = $1")
            .bind(Uuid::parse_str(&run_id).expect("run id is a uuid"))
            .fetch_one(&pool)
            .await
            .expect("ghost run row");
    assert_eq!(mode, "run");
    assert_eq!(ghost_input, Some(input));
    assert_eq!(
        ghost_source,
        Some(Uuid::parse_str(&source).expect("source id is a uuid"))
    );
    assert_eq!(count_submissions(&pool).await, 2);
    assert_eq!(count_jobs(&pool).await, 2);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_ghost_run_replays_and_rejects_key_reuse(pool: PgPool) {
    seed_users(&pool).await;
    let fixture = Fixture::new();
    let address = serve(build_router(app_state(
        pool.clone(),
        config_with(&fixture.dir, &[]),
    )))
    .await;

    let source = post_plain_run(address, USER, "replay-source").await;
    let body = ghost_body("two-sum", "python", code(), &source, &json!([1, 2]));

    let first = submit(address, USER, Some("same-key"), &body).await;
    let replay = submit(address, USER, Some("same-key"), &body).await;
    assert_eq!(first.status, 202);
    assert_eq!(replay.status, 202, "{}", replay.body);
    assert_eq!(json_body(&first)["run_id"], json_body(&replay)["run_id"]);
    assert!(replay
        .headers
        .to_ascii_lowercase()
        .contains("idempotent-replayed: true"));

    // The same key with a different ghost input is a different request, not a replay.
    let other = ghost_body("two-sum", "python", code(), &source, &json!([9, 9]));
    let reuse = submit(address, USER, Some("same-key"), &other).await;
    assert_eq!(reuse.status, 422);
    assert_eq!(error_code(&reuse), "idempotency_key_reuse");
    assert_eq!(count_submissions(&pool).await, 2);
}

/// IDE-3 / VAL-4: a ghost rerun spends the same per-user run budget as a normal run. With a
/// budget of one run per window, the source run uses it, so the rerun is refused with 429
/// `rate_limited`, a wait time, and nothing queued.
#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_ghost_run_shares_the_per_user_rate_limit(pool: PgPool) {
    seed_users(&pool).await;
    let fixture = Fixture::new();
    let config = config_with(&fixture.dir, &[("RUN_RATE_USER", "1")]);
    let address = serve(build_router(app_state(pool.clone(), config))).await;

    let source = post_plain_run(address, USER, "rate-source").await;
    let jobs_before = count_jobs(&pool).await;
    let submissions_before = count_submissions(&pool).await;

    let rerun = submit(
        address,
        USER,
        Some("rate-ghost"),
        &ghost_body("two-sum", "python", code(), &source, &json!([1])),
    )
    .await;

    assert_eq!(rerun.status, 429, "the window is spent: {}", rerun.body);
    assert_eq!(error_code(&rerun), "rate_limited");
    assert!(
        json_body(&rerun)["message"]
            .as_str()
            .is_some_and(|message| message.contains("seconds")),
        "the message states the wait: {}",
        rerun.body
    );
    assert_eq!(count_jobs(&pool).await, jobs_before, "no job was queued");
    assert_eq!(
        count_submissions(&pool).await,
        submissions_before,
        "no submission was stored"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_ghost_run_counts_against_the_active_cap(pool: PgPool) {
    seed_users(&pool).await;
    let fixture = Fixture::new();
    let config = config_with(&fixture.dir, &[("RUN_ACTIVE_CAP_PER_USER", "1")]);
    let address = serve(build_router(app_state(pool.clone(), config))).await;

    let source = post_plain_run(address, USER, "cap-source").await;
    let first = submit(
        address,
        USER,
        Some("a"),
        &ghost_body("two-sum", "python", code(), &source, &json!([1])),
    )
    .await;
    assert_eq!(
        first.status, 429,
        "the source run already fills the cap of 1"
    );
    assert_eq!(error_code(&first), "queue_cap");
}
