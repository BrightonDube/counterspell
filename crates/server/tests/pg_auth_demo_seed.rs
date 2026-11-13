//! Database tests for the judge demo seed and `GET /api/auth/config` (auth T26, requirements
//! R16.1 to R16.3, R16.9).
//!
//! Run with `DATABASE_URL=<throwaway postgres 17> cargo test -p counterspell-server --test pg_auth_demo_seed -- --ignored`.

use std::{net::SocketAddr, sync::Arc, time::Duration as StdDuration};

use axum::{
    body::{to_bytes, Body},
    extract::connect_info::MockConnectInfo,
    http::{header, Method, Request, StatusCode},
    Router,
};
use counterspell_server::{
    auth::{
        clock::ManualClock,
        config::{DemoConfig, Redacted},
        cookies::CookieSettings,
        demo::seed_demo,
        limits::AuthLimiter,
        password::HashGate,
        AuthState,
    },
    client_ip::TrustedProxies,
    ratelimit::{Limiter, MemoryLimiter},
};
use serde_json::{json, Value};
use sqlx::PgPool;
use time::OffsetDateTime;
use tower::ServiceExt;

const DEMO_EMAIL: &str = "judge@counterspell.example";
const DEMO_PASSWORD: &str = "demo-password-for-judges";

fn demo() -> DemoConfig {
    DemoConfig {
        email: DEMO_EMAIL.to_owned(),
        password: Redacted::new(DEMO_PASSWORD.to_owned()),
    }
}

fn gate() -> HashGate {
    HashGate::new(4, StdDuration::from_secs(2))
}

fn auth_router(pool: &PgPool) -> Router {
    let clock = Arc::new(ManualClock::new(
        OffsetDateTime::from_unix_timestamp(1_800_000_000).expect("valid timestamp"),
    ));
    let limits = AuthLimiter::new(
        Limiter::Memory(Arc::new(MemoryLimiter::default())),
        &Redacted::new(vec![4_u8; 32]),
    );
    let state = AuthState::new(
        pool.clone(),
        clock,
        CookieSettings::new(false),
        limits,
        gate(),
        TrustedProxies::parse("10.0.0.0/8").expect("valid trusted proxies"),
    );
    counterspell_server::auth::router(state).layer(MockConnectInfo(SocketAddr::from((
        [203, 0, 113, 9],
        40_000,
    ))))
}

async fn demo_count(pool: &PgPool) -> i64 {
    let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM users WHERE is_demo")
        .fetch_one(pool)
        .await
        .expect("count demo users");
    count
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_seeding_twice_leaves_exactly_one_verified_demo_user(pool: PgPool) {
    let hashes = gate();
    seed_demo(&pool, &demo(), &hashes)
        .await
        .expect("first seed");
    seed_demo(&pool, &demo(), &hashes)
        .await
        .expect("second seed");

    assert_eq!(demo_count(&pool).await, 1, "there is exactly one demo user");
    let (handle, verified): (String, bool) = sqlx::query_as(
        "SELECT handle::text, email_verified_at IS NOT NULL FROM users WHERE is_demo",
    )
    .fetch_one(&pool)
    .await
    .expect("demo row");
    assert_eq!(handle, "demo");
    assert!(verified, "the demo address is verified");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_the_demo_user_signs_in_with_the_configured_password(pool: PgPool) {
    seed_demo(&pool, &demo(), &gate()).await.expect("seed");
    let router = auth_router(&pool);
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "email": DEMO_EMAIL, "password": DEMO_PASSWORD }).to_string(),
        ))
        .expect("request");
    let response = router.oneshot(request).await.expect("response");
    assert_eq!(response.status(), StatusCode::OK, "the demo password works");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_the_demo_password_is_stored_only_as_an_argon2id_hash(pool: PgPool) {
    seed_demo(&pool, &demo(), &gate()).await.expect("seed");
    let (stored,): (String,) = sqlx::query_as("SELECT password_hash FROM users WHERE is_demo")
        .fetch_one(&pool)
        .await
        .expect("hash");
    assert!(
        stored.starts_with("$argon2id$"),
        "the value is an Argon2id PHC string"
    );
    assert!(
        !stored.contains(DEMO_PASSWORD),
        "the plain password is never stored"
    );

    let (leaks,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM users WHERE password_hash LIKE '%' || $1 || '%' OR handle::text LIKE '%' || $1 || '%'",
    )
    .bind(DEMO_PASSWORD)
    .fetch_one(&pool)
    .await
    .expect("search");
    assert_eq!(leaks, 0, "the plain password appears nowhere in users");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_without_demo_settings_nothing_is_seeded_and_config_says_so(pool: PgPool) {
    assert_eq!(
        demo_count(&pool).await,
        0,
        "no demo user exists before a seed"
    );

    let settings =
        counterspell_server::auth::handlers::config::PublicSettings::new(true, false, false);
    let response = counterspell_server::auth::handlers::config::config_router(settings)
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri("/api/auth/config")
                .body(Body::empty())
                .expect("request"),
        )
        .await
        .expect("response");
    assert_eq!(response.status(), StatusCode::OK);
    let bytes = to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("body");
    let body: Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(body["providers"]["github"], true);
    assert_eq!(body["providers"]["google"], false);
    assert_eq!(body["demo"]["enabled"], false);
    assert!(
        !String::from_utf8_lossy(&bytes).contains(DEMO_PASSWORD),
        "the config route never carries a secret"
    );
}
