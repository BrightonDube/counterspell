//! HTTP tests for cookies, the session extractors, logout, logout everywhere and `GET /api/me`
//! (auth T11, requirements R10.1, R10.4, R10.5, R11.1 to R11.4, R12.1 to R12.4).
//!
//! Each request goes through the real auth router and the real database. Time comes from a
//! `ManualClock`. Run with `DATABASE_URL=<throwaway postgres 17> cargo test -p counterspell-server
//! --test auth_session_http -- --ignored`.

use std::sync::Arc;

use axum::{
    body::{to_bytes, Body},
    http::{header, HeaderMap, Request, StatusCode},
    Router,
};
use counterspell_common::ids::UserId;
use counterspell_server::auth::{
    clock::ManualClock,
    cookies::CookieSettings,
    resolver::SessionUserResolver,
    session::{AuthMethod, SessionService},
    AuthState,
};
use counterspell_server::auth::{config::Redacted, limits::AuthLimiter, password::HashGate};
use counterspell_server::client_ip::TrustedProxies;
use counterspell_server::ratelimit::{Limiter, MemoryLimiter};
use counterspell_server::seams::UserResolver;
use serde_json::Value;
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use tower::ServiceExt;
use uuid::Uuid;

/// Inserts a verified user and returns its id.
async fn verified_user(pool: &PgPool, handle: &str) -> UserId {
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, email_verified_at, handle, display_name)
         VALUES ($1, now(), $2, $2) RETURNING id",
    )
    .bind(format!("{handle}@example.co"))
    .bind(handle)
    .fetch_one(pool)
    .await
    .expect("insert user");
    UserId::from_uuid(id)
}

/// The auth router over this pool, with a clock the test can move. Cookies are the development
/// names, so no `Secure` attribute is needed in tests.
fn app(pool: &PgPool) -> (Router, Arc<ManualClock>) {
    let clock = Arc::new(ManualClock::new(
        OffsetDateTime::from_unix_timestamp(1_800_000_000).expect("valid timestamp"),
    ));
    let limits = AuthLimiter::new(
        Limiter::Memory(Arc::new(MemoryLimiter::default())),
        &Redacted::new(vec![5_u8; 32]),
    );
    let gate = HashGate::new(4, std::time::Duration::from_secs(2));
    let trusted = TrustedProxies::parse("10.0.0.0/8").expect("valid trusted proxies");
    let state = AuthState::new(
        pool.clone(),
        clock.clone(),
        CookieSettings::new(false),
        limits,
        gate,
        trusted,
    );
    (counterspell_server::auth::router(state), clock)
}

/// Creates a session for `user` and returns its raw cookie value.
async fn login(pool: &PgPool, clock: &Arc<ManualClock>, user: UserId) -> String {
    SessionService::new(pool.clone(), clock.clone())
        .create(user, AuthMethod::MagicLink)
        .await
        .expect("create session")
        .cookie_value
        .expose()
        .to_owned()
}

/// Sends one request and returns the status, headers and body.
async fn send(
    router: &Router,
    method: &str,
    path: &str,
    cookie: Option<&str>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(value) = cookie {
        builder = builder.header(header::COOKIE, format!("cs_session={value}"));
    }
    let request = builder.body(Body::empty()).expect("request");
    let response = router.clone().oneshot(request).await.expect("response");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("body");
    let body = serde_json::from_slice(&bytes).unwrap_or(Value::Null);
    (status, headers, body)
}

fn set_cookie_values(headers: &HeaderMap) -> Vec<String> {
    headers
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok().map(str::to_owned))
        .collect()
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_a_request_without_a_cookie_is_unauthorised(pool: PgPool) {
    let (router, _clock) = app(&pool);
    let (status, _headers, body) = send(&router, "GET", "/api/me", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["code"], "unauthenticated");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_a_garbage_cookie_is_refused_and_cleared(pool: PgPool) {
    let (router, _clock) = app(&pool);
    let (status, headers, _body) = send(&router, "GET", "/api/me", Some("not-a-session")).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    let cookies = set_cookie_values(&headers);
    assert!(
        cookies
            .iter()
            .any(|value| value.starts_with("cs_session=;") && value.contains("Max-Age=0")),
        "an unusable cookie is cleared, got {cookies:?}"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_an_expired_session_is_refused_and_cleared(pool: PgPool) {
    let user = verified_user(&pool, "expired-http").await;
    let (router, clock) = app(&pool);
    let raw = login(&pool, &clock, user).await;
    let (ok, _, _) = send(&router, "GET", "/api/me", Some(&raw)).await;
    assert_eq!(ok, StatusCode::OK, "the session works before it expires");

    clock.advance(Duration::days(31));
    let (status, headers, _) = send(&router, "GET", "/api/me", Some(&raw)).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert!(set_cookie_values(&headers)
        .iter()
        .any(|value| value.contains("Max-Age=0")));
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_me_returns_only_the_callers_own_data(pool: PgPool) {
    let first = verified_user(&pool, "caller-one").await;
    let second = verified_user(&pool, "caller-two").await;
    let (router, clock) = app(&pool);
    let first_cookie = login(&pool, &clock, first).await;
    let second_cookie = login(&pool, &clock, second).await;

    let (status, headers, body) = send(&router, "GET", "/api/me", Some(&first_cookie)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["user"]["handle"], "caller-one");
    assert_eq!(body["user"]["id"], first.to_string());
    assert!(
        !body.to_string().contains("caller-two"),
        "the other user's data never appears in the response"
    );
    assert_eq!(
        headers
            .get(header::CACHE_CONTROL)
            .and_then(|v| v.to_str().ok()),
        Some("no-store"),
        "account data is never cached"
    );

    let (_, _, other) = send(&router, "GET", "/api/me", Some(&second_cookie)).await;
    assert_eq!(other["user"]["handle"], "caller-two");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_a_used_session_is_renewed_and_its_cookie_is_reissued(pool: PgPool) {
    let user = verified_user(&pool, "renewed-http").await;
    let (router, clock) = app(&pool);
    let raw = login(&pool, &clock, user).await;

    clock.advance(Duration::hours(2));
    let (status, headers, _) = send(&router, "GET", "/api/me", Some(&raw)).await;
    assert_eq!(status, StatusCode::OK);
    let expected = format!("cs_session={raw};");
    assert!(
        set_cookie_values(&headers)
            .iter()
            .any(|value| { value.starts_with(&expected) && value.contains("Max-Age=2592000") }),
        "a renewed session re-issues its cookie for 30 days"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_logout_is_idempotent(pool: PgPool) {
    let user = verified_user(&pool, "logout-once").await;
    let (router, clock) = app(&pool);
    let raw = login(&pool, &clock, user).await;

    let (first, first_headers, _) = send(&router, "POST", "/api/auth/logout", Some(&raw)).await;
    assert_eq!(first, StatusCode::NO_CONTENT);
    assert!(set_cookie_values(&first_headers)
        .iter()
        .any(|value| value.contains("Max-Age=0")));

    let (second, _, _) = send(&router, "POST", "/api/auth/logout", Some(&raw)).await;
    assert_eq!(
        second,
        StatusCode::NO_CONTENT,
        "a second logout is harmless"
    );

    let (none, _, _) = send(&router, "POST", "/api/auth/logout", None).await;
    assert_eq!(
        none,
        StatusCode::NO_CONTENT,
        "logout without a cookie is harmless"
    );

    let (after, _, _) = send(&router, "GET", "/api/me", Some(&raw)).await;
    assert_eq!(
        after,
        StatusCode::UNAUTHORIZED,
        "the session is gone on the next request"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_logout_all_ends_only_the_callers_sessions(pool: PgPool) {
    let owner = verified_user(&pool, "everywhere-a").await;
    let other = verified_user(&pool, "everywhere-b").await;
    let (router, clock) = app(&pool);
    let laptop = login(&pool, &clock, owner).await;
    let phone = login(&pool, &clock, owner).await;
    let bystander = login(&pool, &clock, other).await;

    let (status, _, _) = send(&router, "POST", "/api/auth/logout-all", Some(&laptop)).await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let (phone_status, _, _) = send(&router, "GET", "/api/me", Some(&phone)).await;
    assert_eq!(
        phone_status,
        StatusCode::UNAUTHORIZED,
        "the second device is signed out too"
    );
    let (laptop_status, _, _) = send(&router, "GET", "/api/me", Some(&laptop)).await;
    assert_eq!(laptop_status, StatusCode::UNAUTHORIZED);
    let (other_status, _, _) = send(&router, "GET", "/api/me", Some(&bystander)).await;
    assert_eq!(
        other_status,
        StatusCode::OK,
        "another user's session is untouched"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_logout_all_needs_a_session(pool: PgPool) {
    let (router, _clock) = app(&pool);
    let (status, _, body) = send(&router, "POST", "/api/auth/logout-all", None).await;
    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(body["code"], "unauthenticated");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_session_user_resolver_matches_the_extractor(pool: PgPool) {
    let user = verified_user(&pool, "resolver-http").await;
    let (router, clock) = app(&pool);
    let raw = login(&pool, &clock, user).await;

    let resolver = SessionUserResolver::new(
        SessionService::new(pool.clone(), clock.clone()),
        "cs_session".to_owned(),
    );
    let mut headers = HeaderMap::new();
    headers.insert(
        header::COOKIE,
        format!("cs_session={raw}").parse().expect("header"),
    );
    assert_eq!(resolver.resolve(&headers).await, Some(user));

    let (_, _, body) = send(&router, "GET", "/api/me", Some(&raw)).await;
    assert_eq!(
        body["user"]["id"],
        user.to_string(),
        "both paths see the same user"
    );

    assert_eq!(
        resolver.resolve(&HeaderMap::new()).await,
        None,
        "no cookie means no user"
    );
}
