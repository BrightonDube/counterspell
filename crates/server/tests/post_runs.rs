//! Tests for `POST /api/runs` (run-lifecycle T25, design Path 1 and requirements R2 to R5).
//!
//! The validation tests use a database pool that never connects. If a rejected request reached the
//! database, it would fail with `500`, so the exact error codes also prove that nothing was looked up or
//! queued. The `pg_` tests need a throwaway Postgres 17 and run with
//! `DATABASE_URL=<url> cargo test -p counterspell-server --test post_runs -- --ignored pg_`.

mod support;

use std::{path::Path, sync::Arc};

use counterspell_server::{
    build_router,
    config::ServerConfig,
    ratelimit::{Limiter, MemoryLimiter},
    testkit::{BearerTestResolver, InMemoryProblems},
    AppState,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use support::{lazy_pool, post, serve, Fixture, Reply};

/// Builds settings from the defaults plus `overrides`, with no real services behind them.
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

/// The app with the test identity, a one-problem catalogue (`two-sum`) and an in-memory limiter.
fn app_state(pool: PgPool, config: ServerConfig) -> AppState {
    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    AppState::new(pool, config, limiter)
        .with_user_resolver(BearerTestResolver)
        .with_problems(InMemoryProblems::new(&["two-sum"]))
}

fn run_body(problem: &str, lang: &str, code: &str, mode: &str) -> String {
    json!({ "problem": problem, "lang": lang, "code": code, "mode": mode }).to_string()
}

/// Posts a valid JSON run as `test-user-<user>`, with an optional `Idempotency-Key`.
async fn submit(address: std::net::SocketAddr, user: u32, key: Option<&str>, body: &str) -> Reply {
    let auth = format!("Bearer test-user-{user}");
    let mut headers = vec![
        ("Authorization", auth.as_str()),
        ("Content-Type", "application/json"),
    ];
    if let Some(key) = key {
        headers.push(("Idempotency-Key", key));
    }
    post(address, "/api/runs", &headers, body).await
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

/// One request that validation must refuse, with the error code it must produce.
struct Rejected {
    code: &'static str,
    content_type: &'static str,
    key: Option<&'static str>,
    body: String,
}

fn rejected(
    code: &'static str,
    content_type: &'static str,
    key: Option<&'static str>,
    body: String,
) -> Rejected {
    Rejected {
        code,
        content_type,
        key,
        body,
    }
}

fn rejected_requests() -> Vec<Rejected> {
    let json = "application/json";
    let code = "print(1)\n";
    vec![
        rejected(
            "source_too_large",
            json,
            None,
            run_body("two-sum", "python", &"#".repeat(65_537), "submit"),
        ),
        rejected(
            "invalid_source",
            json,
            None,
            run_body("two-sum", "python", "print(1)\u{0}\n", "submit"),
        ),
        rejected(
            "invalid_source",
            json,
            None,
            run_body("two-sum", "python", "print(1)\u{1b}\n", "submit"),
        ),
        rejected(
            "source_does_not_parse",
            json,
            None,
            run_body("two-sum", "python", "def (:\n", "submit"),
        ),
        rejected(
            "invalid_request",
            json,
            None,
            run_body("two-sum", "ruby", code, "submit"),
        ),
        rejected(
            "invalid_request",
            json,
            None,
            run_body("two-sum", "python", code, "practice"),
        ),
        rejected(
            "invalid_request",
            json,
            None,
            json!({"problem": "two-sum", "lang": "python", "code": code, "mode": "submit", "extra": true})
                .to_string(),
        ),
        rejected(
            "unsupported_media_type",
            "text/plain",
            None,
            run_body("two-sum", "python", code, "submit"),
        ),
        rejected(
            "invalid_idempotency_key",
            json,
            Some("bad key!"),
            run_body("two-sum", "python", code, "submit"),
        ),
        rejected(
            "unknown_problem",
            json,
            None,
            run_body("no-such-problem", "python", code, "submit"),
        ),
    ]
}

#[tokio::test]
async fn validation_rejects_without_side_effects() {
    let fixture = Fixture::new();
    let state = app_state(lazy_pool(), config_with(&fixture.dir, &[]));
    let address = serve(build_router(state)).await;

    for case in rejected_requests() {
        let mut headers = vec![
            ("Authorization", "Bearer test-user-1"),
            ("Content-Type", case.content_type),
        ];
        if let Some(key) = case.key {
            headers.push(("Idempotency-Key", key));
        }
        let reply = post(address, "/api/runs", &headers, &case.body).await;

        assert_eq!(
            error_code(&reply),
            case.code,
            "wrong code for reply with status {}",
            reply.status
        );
        assert!(
            (400..500).contains(&reply.status),
            "{} must be a client error, got {}",
            case.code,
            reply.status
        );
    }
}

#[tokio::test]
async fn validation_errors_use_the_documented_status() {
    let fixture = Fixture::new();
    let state = app_state(lazy_pool(), config_with(&fixture.dir, &[]));
    let address = serve(build_router(state)).await;
    let oversize = "#".repeat(65_537);

    let too_large = submit(
        address,
        1,
        None,
        &run_body("two-sum", "python", &oversize, "submit"),
    )
    .await;
    let not_parsed = submit(
        address,
        1,
        None,
        &run_body("two-sum", "python", "def (:\n", "submit"),
    )
    .await;

    assert_eq!(too_large.status, 413);
    assert_eq!(not_parsed.status, 400);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_post_requires_session(pool: PgPool) {
    let fixture = Fixture::new();
    let address = serve(build_router(app_state(
        pool.clone(),
        config_with(&fixture.dir, &[]),
    )))
    .await;

    let reply = post(
        address,
        "/api/runs",
        &[("Content-Type", "application/json")],
        &run_body("two-sum", "python", "print(1)\n", "submit"),
    )
    .await;

    assert_eq!(reply.status, 401);
    assert_eq!(error_code(&reply), "unauthorized");
    assert_eq!(count_submissions(&pool).await, 0);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_post_unknown_problem_404(pool: PgPool) {
    let fixture = Fixture::new();
    let address = serve(build_router(app_state(
        pool.clone(),
        config_with(&fixture.dir, &[]),
    )))
    .await;

    let reply = submit(
        address,
        1,
        None,
        &run_body("no-such-problem", "python", "print(1)\n", "submit"),
    )
    .await;

    assert_eq!(reply.status, 404);
    assert_eq!(error_code(&reply), "unknown_problem");
    assert_eq!(count_submissions(&pool).await, 0);
    assert_eq!(count_jobs(&pool).await, 0);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_post_creates_a_queued_run_and_returns_202(pool: PgPool) {
    let fixture = Fixture::new();
    let address = serve(build_router(app_state(
        pool.clone(),
        config_with(&fixture.dir, &[]),
    )))
    .await;

    let reply = submit(
        address,
        1,
        Some("first-try"),
        &run_body("two-sum", "python", "print(1)\n", "submit"),
    )
    .await;

    assert_eq!(reply.status, 202);
    let body = json_body(&reply);
    let run_id = body["run_id"].as_str().expect("run_id").to_owned();
    assert_eq!(body["status"], "queued");
    let location = format!("location: /api/runs/{run_id}");
    assert!(
        reply.headers.to_ascii_lowercase().contains(&location),
        "missing Location header in {}",
        reply.headers
    );
    assert!(!reply
        .headers
        .to_ascii_lowercase()
        .contains("idempotent-replayed"));
    assert_eq!(count_submissions(&pool).await, 1);
    assert_eq!(count_jobs(&pool).await, 1);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_post_rate_limited_has_retry_after(pool: PgPool) {
    let fixture = Fixture::new();
    let config = config_with(&fixture.dir, &[("RUN_RATE_USER", "1")]);
    let address = serve(build_router(app_state(pool.clone(), config))).await;
    let body = run_body("two-sum", "python", "print(1)\n", "submit");

    let first = submit(address, 1, None, &body).await;
    let second = submit(address, 1, None, &body).await;

    assert_eq!(first.status, 202);
    assert_eq!(second.status, 429);
    assert_eq!(error_code(&second), "rate_limited");
    assert!(second
        .headers
        .to_ascii_lowercase()
        .contains("retry-after: "));
    let seconds = json_body(&second)["retry_after_seconds"]
        .as_u64()
        .expect("retry_after_seconds");
    assert!(seconds >= 1);
    assert_eq!(count_submissions(&pool).await, 1);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_post_replay_header_and_no_quota_used(pool: PgPool) {
    let fixture = Fixture::new();
    let config = config_with(&fixture.dir, &[("RUN_RATE_USER", "1")]);
    let address = serve(build_router(app_state(pool.clone(), config))).await;
    let body = run_body("two-sum", "python", "print(1)\n", "submit");

    let first = submit(address, 1, Some("same-key"), &body).await;
    let replay = submit(address, 1, Some("same-key"), &body).await;

    assert_eq!(first.status, 202);
    assert_eq!(replay.status, 202, "a replay must not hit the user's limit");
    assert_eq!(json_body(&first)["run_id"], json_body(&replay)["run_id"]);
    assert!(replay
        .headers
        .to_ascii_lowercase()
        .contains("idempotent-replayed: true"));
    assert_eq!(count_submissions(&pool).await, 1);
    assert_eq!(count_jobs(&pool).await, 1);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_post_same_key_different_code_is_422(pool: PgPool) {
    let fixture = Fixture::new();
    let address = serve(build_router(app_state(
        pool.clone(),
        config_with(&fixture.dir, &[]),
    )))
    .await;

    let first = submit(
        address,
        1,
        Some("reused"),
        &run_body("two-sum", "python", "print(1)\n", "submit"),
    )
    .await;
    let reuse = submit(
        address,
        1,
        Some("reused"),
        &run_body("two-sum", "python", "print(2)\n", "submit"),
    )
    .await;

    assert_eq!(first.status, 202);
    assert_eq!(reuse.status, 422);
    assert_eq!(error_code(&reuse), "idempotency_key_reuse");
    assert_eq!(count_submissions(&pool).await, 1);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_post_active_cap_maps_to_429_queue_cap(pool: PgPool) {
    let fixture = Fixture::new();
    let config = config_with(&fixture.dir, &[("RUN_ACTIVE_CAP_PER_USER", "1")]);
    let address = serve(build_router(app_state(pool.clone(), config))).await;

    let first = submit(
        address,
        1,
        Some("a"),
        &run_body("two-sum", "python", "print(1)\n", "submit"),
    )
    .await;
    let second = submit(
        address,
        1,
        Some("b"),
        &run_body("two-sum", "python", "print(2)\n", "submit"),
    )
    .await;

    assert_eq!(first.status, 202);
    assert_eq!(second.status, 429);
    assert_eq!(error_code(&second), "queue_cap");
    assert_eq!(count_submissions(&pool).await, 1);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_post_trusted_proxy_ip(pool: PgPool) {
    let fixture = Fixture::new();

    // The test client connects from 127.0.0.1, which is a trusted proxy by default. Its
    // X-Forwarded-For names the real client, so the per-IP limit follows that address.
    let trusted = config_with(
        &fixture.dir,
        &[("RUN_RATE_IP", "1"), ("RUN_ACTIVE_CAP_PER_USER", "10")],
    );
    let address = serve(build_router(app_state(pool.clone(), trusted))).await;
    let body = run_body("two-sum", "python", "print(1)\n", "submit");
    let via_proxy = |user: u32, client: &str| {
        let auth = format!("Bearer test-user-{user}");
        let client = client.to_owned();
        let body = body.clone();
        async move {
            post(
                address,
                "/api/runs",
                &[
                    ("Authorization", auth.as_str()),
                    ("Content-Type", "application/json"),
                    ("X-Forwarded-For", client.as_str()),
                ],
                &body,
            )
            .await
        }
    };

    assert_eq!(via_proxy(11, "203.0.113.7").await.status, 202);
    assert_eq!(via_proxy(12, "203.0.113.8").await.status, 202);
    let same_client = via_proxy(13, "203.0.113.7").await;
    assert_eq!(same_client.status, 429);
    assert_eq!(error_code(&same_client), "rate_limited");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_post_ignores_forwarded_header_from_untrusted_peer(pool: PgPool) {
    let fixture = Fixture::new();

    // Trust only 10.0.0.0/8. The test client's 127.0.0.1 is then untrusted, so the header is ignored
    // and every request counts against the socket address.
    let untrusted = config_with(
        &fixture.dir,
        &[
            ("RUN_RATE_IP", "1"),
            ("RUN_ACTIVE_CAP_PER_USER", "10"),
            ("TRUSTED_PROXIES", "10.0.0.0/8"),
        ],
    );
    let address = serve(build_router(app_state(pool.clone(), untrusted))).await;
    let body = run_body("two-sum", "python", "print(1)\n", "submit");

    let auth_21 = "Bearer test-user-21";
    let first = post(
        address,
        "/api/runs",
        &[
            ("Authorization", auth_21),
            ("Content-Type", "application/json"),
            ("X-Forwarded-For", "203.0.113.20"),
        ],
        &body,
    )
    .await;
    let auth_22 = "Bearer test-user-22";
    let second = post(
        address,
        "/api/runs",
        &[
            ("Authorization", auth_22),
            ("Content-Type", "application/json"),
            ("X-Forwarded-For", "203.0.113.21"),
        ],
        &body,
    )
    .await;

    assert_eq!(first.status, 202);
    assert_eq!(
        second.status, 429,
        "spoofed forwarding must not change the IP key"
    );
}
