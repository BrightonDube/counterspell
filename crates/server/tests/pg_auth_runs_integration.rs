//! Integration tests that connect auth sessions to the run endpoints (auth T34, requirements R11.6,
//! R12.2, R12.5 and R18.7). Each request goes through the full router with the real session
//! resolver and the real database.
//!
//! The heartbeat check of the event stream is not covered here: the run-lifecycle SSE endpoint is
//! not implemented yet. See `docs/kiro-log/auth-blockers.md`.
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
use support::{config_for, get, post, serve, Fixture, Reply};
use time::OffsetDateTime;

const ORIGIN: &str = "https://counterspell.example";

/// The full application state with real auth, a one-problem catalogue and a movable clock.
fn state(pool: &PgPool, static_dir: &std::path::Path, clock: Arc<ManualClock>) -> AppState {
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
        config_for(static_dir),
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
    assert!(uuid::Uuid::parse_str(&run_id_of(&reply)).is_ok(), "the reply carries a run id");
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
