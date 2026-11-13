//! Database tests for password sign-in (auth T24, requirements R15.1 to R15.6, R14.1, R14.2,
//! R14.5).
//!
//! The hash gate counts how many verifications it started, so a test can prove that a limited
//! request never reached the hasher.
//!
//! Run with `DATABASE_URL=<throwaway postgres 17> cargo test -p counterspell-server --test pg_auth_login -- --ignored`.

use std::{net::SocketAddr, sync::Arc, time::Duration as StdDuration};

use axum::{
    body::{to_bytes, Body},
    extract::connect_info::MockConnectInfo,
    http::{header, Method, Request, StatusCode},
    Router,
};
use counterspell_common::ids::UserId;
use counterspell_server::{
    auth::{
        clock::ManualClock,
        config::Redacted,
        cookies::CookieSettings,
        limits::AuthLimiter,
        password::{hash_password, HashGate},
        AuthState,
    },
    ratelimit::{Limiter, MemoryLimiter},
};
use serde_json::{json, Value};
use sqlx::PgPool;
use time::OffsetDateTime;
use tower::ServiceExt;
use uuid::Uuid;

const PASSWORD: &str = "correct horse battery";

fn app(pool: &PgPool, gate: HashGate) -> (Router, AuthState) {
    let clock = Arc::new(ManualClock::new(
        OffsetDateTime::from_unix_timestamp(1_800_000_000).expect("valid timestamp"),
    ));
    let limits = AuthLimiter::new(
        Limiter::Memory(Arc::new(MemoryLimiter::default())),
        &Redacted::new(vec![5_u8; 32]),
    );
    let state = AuthState::new(
        pool.clone(),
        clock,
        CookieSettings::new(false),
        limits,
        gate,
        counterspell_server::client_ip::TrustedProxies::parse("10.0.0.0/8")
            .expect("valid trusted proxies"),
    );
    let peer = SocketAddr::from(([203, 0, 113, 9], 40_000));
    (
        counterspell_server::auth::router(state.clone()).layer(MockConnectInfo(peer)),
        state,
    )
}

fn gate() -> HashGate {
    HashGate::new(4, StdDuration::from_secs(2))
}

/// Inserts a verified user with a password (`Some`) or without one (`None`).
async fn user_with_password(pool: &PgPool, handle: &str, password: Option<&str>) -> UserId {
    let hash = password.map(|p| hash_password(p).expect("hash"));
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, email_verified_at, password_hash, handle, display_name)
         VALUES ($1, now(), $2, $3, $3) RETURNING id",
    )
    .bind(format!("{handle}@example.co"))
    .bind(hash)
    .bind(handle)
    .fetch_one(pool)
    .await
    .expect("insert user");
    UserId::from_uuid(id)
}

async fn login(
    router: &Router,
    email: &str,
    password: &str,
) -> (StatusCode, axum::http::HeaderMap, Value) {
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/login")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(
            json!({ "email": email, "password": password }).to_string(),
        ))
        .expect("request");
    let response = router.clone().oneshot(request).await.expect("response");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("body");
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, headers, body)
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_the_right_password_signs_in_and_sets_a_session(pool: PgPool) {
    let user = user_with_password(&pool, "password-ok", Some(PASSWORD)).await;
    let (router, _) = app(&pool, gate());
    let (status, headers, body) = login(&router, "password-ok@example.co", PASSWORD).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user_id"], user.to_string());
    let cookie = headers
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|v| v.to_str().ok())
        .find(|v| v.starts_with("cs_session="))
        .expect("a session cookie is set")
        .to_owned();
    assert!(cookie.contains("HttpOnly") && cookie.contains("SameSite=Lax"));
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_unknown_wrong_and_passwordless_accounts_get_identical_answers(pool: PgPool) {
    user_with_password(&pool, "known-user", Some(PASSWORD)).await;
    user_with_password(&pool, "no-password", None).await;
    let (router, _) = app(&pool, gate());

    let unknown = login(&router, "nobody@example.co", "whatever password").await;
    let wrong = login(&router, "known-user@example.co", "not the password").await;
    let passwordless = login(&router, "no-password@example.co", "whatever password").await;

    for (status, _, body) in [&unknown, &wrong, &passwordless] {
        assert_eq!(*status, StatusCode::UNAUTHORIZED);
        assert_eq!(body["code"], "invalid_credentials");
    }
    assert_eq!(
        unknown.2, wrong.2,
        "unknown and wrong passwords answer the same"
    );
    assert_eq!(
        wrong.2, passwordless.2,
        "a passwordless account answers the same"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_five_failures_are_limited_before_any_hashing(pool: PgPool) {
    user_with_password(&pool, "limited-user", Some(PASSWORD)).await;
    let gate = gate();
    let (router, _) = app(&pool, gate.clone());
    for _ in 0..5 {
        let (status, _, _) = login(&router, "limited-user@example.co", "wrong guess").await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
    }
    let started_before = gate.verifications_started();

    let (status, _, body) = login(&router, "limited-user@example.co", PASSWORD).await;
    assert_eq!(
        status,
        StatusCode::TOO_MANY_REQUESTS,
        "the sixth attempt is limited"
    );
    assert_eq!(body["code"], "rate_limited");
    assert_eq!(
        gate.verifications_started(),
        started_before,
        "a limited request never reaches the hasher"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_a_saturated_hash_gate_answers_busy(pool: PgPool) {
    user_with_password(&pool, "busy-user", Some(PASSWORD)).await;
    let gate = HashGate::new(4, StdDuration::from_millis(50));
    let mut held = Vec::new();
    for _ in 0..4 {
        held.push(gate.acquire().await.expect("a free permit"));
    }
    let (router, _) = app(&pool, gate);
    let (status, _, body) = login(&router, "busy-user@example.co", PASSWORD).await;
    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(body["code"], "busy");
}
