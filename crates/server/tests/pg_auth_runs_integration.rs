//! Integration tests that connect auth sessions to the run endpoints (auth T34, requirements R11.6,
//! R12.2, R12.5 and R18.7). Each request goes through the full router with the real session
//! resolver and the real database.
//!
//! The event stream re-checks its session every heartbeat interval and closes when the session is gone
//! (R12.5), so a logout ends a stream that is already open.
//!
//! Run with `DATABASE_URL=<throwaway postgres 17> cargo test -p counterspell-server --test pg_auth_runs_integration -- --ignored pg_`.

mod support;

use std::sync::Arc;

use axum::http::{header, HeaderMap, HeaderValue};
use counterspell_common::ids::UserId;
use counterspell_server::{
    auth::{
        clock::ManualClock,
        config::Redacted,
        cookies::CookieSettings,
        limits::AuthLimiter,
        password::HashGate,
        session::{AuthMethod, SessionService},
        AuthState,
    },
    client_ip::TrustedProxies,
    ratelimit::{Limiter, MemoryLimiter},
    testkit::{ensure_user, InMemoryProblems},
    AppState,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use support::{config_for, get, post, serve, sse_client::SseResponse, Fixture, Reply};
use time::OffsetDateTime;

const ORIGIN: &str = "https://counterspell.example";

/// The full application state with real auth, a one-problem catalogue and a movable clock.
fn state(pool: &PgPool, static_dir: &std::path::Path, clock: Arc<ManualClock>) -> AppState {
    state_with(pool, config_for(static_dir), clock)
}

/// Like [`state`], with the server settings supplied by the caller.
fn state_with(
    pool: &PgPool,
    config: counterspell_server::config::ServerConfig,
    clock: Arc<ManualClock>,
) -> AppState {
    let limits = AuthLimiter::new(
        Limiter::Memory(Arc::new(MemoryLimiter::default())),
        &Redacted::new(vec![5_u8; 32]),
    );
    let auth = AuthState::new(
        pool.clone(),
        clock,
        CookieSettings::new(false),
        limits,
        HashGate::new(4, std::time::Duration::from_secs(2)),
        TrustedProxies::parse("10.0.0.0/8").expect("valid trusted proxies"),
    );
    AppState::new(
        pool.clone(),
        config,
        Limiter::Memory(Arc::new(MemoryLimiter::default())),
    )
    .with_problems(InMemoryProblems::new(&["two-sum"]))
    .with_auth(auth, ORIGIN)
    .expect("valid origin")
}

fn clock() -> Arc<ManualClock> {
    Arc::new(ManualClock::new(
        OffsetDateTime::from_unix_timestamp(1_800_000_000).expect("valid timestamp"),
    ))
}

/// Makes `user` a real account and returns a session cookie value for it.
async fn signed_in(pool: &PgPool, clock: &Arc<ManualClock>) -> (UserId, String) {
    let user = UserId::new();
    ensure_user(pool, user).await.expect("user fixture");
    support::publish_two_sum(pool).await;
    let cookie = SessionService::new(pool.clone(), clock.clone())
        .create(user, AuthMethod::MagicLink)
        .await
        .expect("create session")
        .cookie_value
        .expose()
        .to_owned();
    (user, cookie)
}

fn cookie_header(cookie: &str) -> String {
    format!("cs_session={cookie}")
}

fn run_body() -> String {
    json!({ "problem": "two-sum", "lang": "python", "code": "print(1)\n", "mode": "submit" })
        .to_string()
}

async fn post_run(address: std::net::SocketAddr, cookie: &str) -> Reply {
    let cookie = cookie_header(cookie);
    post(
        address,
        "/api/runs",
        &[
            ("Cookie", cookie.as_str()),
            ("Origin", ORIGIN),
            ("Content-Type", "application/json"),
        ],
        &run_body(),
    )
    .await
}

async fn read_run(address: std::net::SocketAddr, run_id: &str, cookie: &str) -> Reply {
    let cookie = cookie_header(cookie);
    get(
        address,
        &format!("/api/runs/{run_id}"),
        &[("Cookie", cookie.as_str())],
    )
    .await
}

fn run_id_of(reply: &Reply) -> String {
    let body: Value = serde_json::from_str(&reply.body).expect("json body");
    body["run_id"].as_str().expect("run_id").to_owned()
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_session_cookie_authenticates_post_runs(pool: PgPool) {
    let fixture = Fixture::new();
    let clock = clock();
    let address = serve(counterspell_server::build_router(state(
        &pool,
        &fixture.dir,
        clock.clone(),
    )))
    .await;
    let (_user, cookie) = signed_in(&pool, &clock).await;

    let reply = post_run(address, &cookie).await;

    assert_eq!(reply.status, 202, "a signed-in user can save a run");
    assert!(
        uuid::Uuid::parse_str(&run_id_of(&reply)).is_ok(),
        "the reply carries a run id"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_second_user_gets_404_for_the_first_users_run(pool: PgPool) {
    let fixture = Fixture::new();
    let clock = clock();
    let address = serve(counterspell_server::build_router(state(
        &pool,
        &fixture.dir,
        clock.clone(),
    )))
    .await;
    let (_owner, owner_cookie) = signed_in(&pool, &clock).await;
    let (_other, other_cookie) = signed_in(&pool, &clock).await;
    let run_id = run_id_of(&post_run(address, &owner_cookie).await);

    let as_other = read_run(address, &run_id, &other_cookie).await;
    let as_owner = read_run(address, &run_id, &owner_cookie).await;

    assert_eq!(as_other.status, 404, "another user must not see the run");
    assert_eq!(as_owner.status, 200, "the owner can read the run");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_logout_all_refuses_the_next_request_and_spares_other_users(pool: PgPool) {
    let fixture = Fixture::new();
    let clock = clock();
    let address = serve(counterspell_server::build_router(state(
        &pool,
        &fixture.dir,
        clock.clone(),
    )))
    .await;
    let (_leaver, leaver_cookie) = signed_in(&pool, &clock).await;
    let (_stayer, stayer_cookie) = signed_in(&pool, &clock).await;
    let leaver_run = run_id_of(&post_run(address, &leaver_cookie).await);
    let stayer_run = run_id_of(&post_run(address, &stayer_cookie).await);

    let cookie = cookie_header(&leaver_cookie);
    let logout = post(
        address,
        "/api/auth/logout-all",
        &[("Cookie", cookie.as_str()), ("Origin", ORIGIN)],
        "",
    )
    .await;
    assert_eq!(logout.status, 204);

    let refused = read_run(address, &leaver_run, &leaver_cookie).await;
    let still_works = read_run(address, &stayer_run, &stayer_cookie).await;
    assert_eq!(refused.status, 401, "the signed-out session is refused");
    assert_eq!(
        still_works.status, 200,
        "another user's session is untouched"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_production_resolver_reads_the_session_not_a_fixed_identity(pool: PgPool) {
    let fixture = Fixture::new();
    let clock = clock();
    let app = state(&pool, &fixture.dir, clock.clone());
    let (user, cookie) = signed_in(&pool, &clock).await;

    let mut signed = HeaderMap::new();
    signed.insert(
        header::COOKIE,
        HeaderValue::from_str(&cookie_header(&cookie)).expect("cookie header"),
    );
    let mut forged = HeaderMap::new();
    forged.insert(
        header::COOKIE,
        HeaderValue::from_static("cs_session=not-a-real-session"),
    );

    assert_eq!(app.user_resolver.resolve(&signed).await, Some(user));
    assert_eq!(app.user_resolver.resolve(&forged).await, None);
    assert_eq!(app.user_resolver.resolve(&HeaderMap::new()).await, None);
}

/// Server settings with a one-second heartbeat and a fast poll, so a stream notices a sign-out quickly.
fn quick_stream_config(static_dir: &std::path::Path) -> counterspell_server::config::ServerConfig {
    let dir = static_dir.to_string_lossy().into_owned();
    counterspell_server::config::ServerConfig::from_lookup(move |name| match name {
        "DATABASE_URL" => Some("postgres://app@127.0.0.1:1/counterspell".to_owned()),
        "VALKEY_URL" => Some("redis://127.0.0.1:1".to_owned()),
        "STATIC_DIR" => Some(dir.clone()),
        "SSE_HEARTBEAT_SECS" => Some("1".to_owned()),
        "SSE_POLL_MS" => Some("100".to_owned()),
        _ => None,
    })
    .expect("config loads")
}

/// Reads frames until the stream ends, or panics after `limit` so a stream that never closes fails.
async fn closes_within(stream: &mut SseResponse, limit: std::time::Duration) {
    let ended = tokio::time::timeout(limit, async {
        while stream.next_frame().await.is_some() {}
    })
    .await;
    assert!(
        ended.is_ok(),
        "the stream should have closed within {limit:?}"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_logout_all_closes_an_open_event_stream_at_the_next_heartbeat(pool: PgPool) {
    let fixture = Fixture::new();
    let clock = clock();
    let address = serve(counterspell_server::build_router(state_with(
        &pool,
        quick_stream_config(&fixture.dir),
        clock.clone(),
    )))
    .await;
    let (_user, cookie) = signed_in(&pool, &clock).await;
    let run_id = run_id_of(&post_run(address, &cookie).await);
    let cookie_value = cookie_header(&cookie);
    let mut stream = SseResponse::open(
        address,
        &format!("/api/runs/{run_id}/events"),
        &[("Cookie", cookie_value.as_str())],
    )
    .await;
    assert_eq!(stream.status, 200, "an unfinished run opens a live stream");

    let logout = post(
        address,
        "/api/auth/logout-all",
        &[("Cookie", cookie_value.as_str()), ("Origin", ORIGIN)],
        "",
    )
    .await;
    assert_eq!(logout.status, 204);

    closes_within(&mut stream, std::time::Duration::from_secs(5)).await;
    assert!(
        !stream.transport_failed(),
        "the server ends the stream cleanly"
    );
    let again = SseResponse::open(
        address,
        &format!("/api/runs/{run_id}/events"),
        &[("Cookie", cookie_value.as_str())],
    )
    .await;
    assert_eq!(
        again.status, 401,
        "a new stream with the dead session is refused"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_a_valid_session_keeps_its_event_stream_open_across_heartbeats(pool: PgPool) {
    let fixture = Fixture::new();
    let clock = clock();
    let address = serve(counterspell_server::build_router(state_with(
        &pool,
        quick_stream_config(&fixture.dir),
        clock.clone(),
    )))
    .await;
    let (_user, cookie) = signed_in(&pool, &clock).await;
    let run_id = run_id_of(&post_run(address, &cookie).await);
    let cookie_value = cookie_header(&cookie);
    let mut stream = SseResponse::open(
        address,
        &format!("/api/runs/{run_id}/events"),
        &[("Cookie", cookie_value.as_str())],
    )
    .await;
    assert_eq!(stream.status, 200);

    // Three heartbeat intervals pass. The stream must still be open and still sending.
    let quiet = tokio::time::timeout(std::time::Duration::from_secs(3), async {
        while stream.next_frame().await.is_some() {}
    })
    .await;
    assert!(
        quiet.is_err(),
        "a signed-in stream must not close by itself"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_the_stream_check_is_read_only_and_never_renews_a_session(pool: PgPool) {
    let fixture = Fixture::new();
    let clock = clock();
    let app = state(&pool, &fixture.dir, clock.clone());
    let (user, cookie) = signed_in(&pool, &clock).await;
    let mut headers = HeaderMap::new();
    headers.insert(
        header::COOKIE,
        HeaderValue::from_str(&cookie_header(&cookie)).expect("cookie header"),
    );
    let renewed_before: OffsetDateTime =
        sqlx::query_scalar("SELECT last_renewed_at FROM sessions WHERE user_id = $1")
            .bind(user.as_uuid())
            .fetch_one(&pool)
            .await
            .expect("session row");

    // Two hours later a normal request would renew the session. The stream check must not.
    clock.advance(time::Duration::hours(2));
    assert!(app.user_resolver.still_signed_in(&headers, user).await);

    let renewed_after: OffsetDateTime =
        sqlx::query_scalar("SELECT last_renewed_at FROM sessions WHERE user_id = $1")
            .bind(user.as_uuid())
            .fetch_one(&pool)
            .await
            .expect("session row");
    assert_eq!(
        renewed_before, renewed_after,
        "an open stream must not keep a session alive"
    );

    let mut forged = HeaderMap::new();
    forged.insert(
        header::COOKIE,
        HeaderValue::from_static("cs_session=not-a-real-session"),
    );
    assert!(!app.user_resolver.still_signed_in(&forged, user).await);
    assert!(
        !app.user_resolver
            .still_signed_in(&HeaderMap::new(), user)
            .await
    );
}
