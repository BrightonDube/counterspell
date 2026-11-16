//! OAuth sign-in start and callback over real Postgres, with a fake provider (auth T21,
//! properties P6 and P7, requirements R6.1 to R6.11, R7.1 to R7.7 and R16.5).
//!
//! Run with `DATABASE_URL` set: `cargo test -p counterspell-server pg_auth_oauth_model -- --include-ignored`.

use std::{net::SocketAddr, sync::Arc, time::Duration as StdDuration};

use axum::{
    body::Body,
    extract::connect_info::MockConnectInfo,
    http::{header, HeaderMap, Method, Request, StatusCode},
    response::Response,
    Router,
};
use counterspell_server::{
    auth::{
        clock::ManualClock,
        config::Redacted,
        cookies::CookieSettings,
        email::EmailAddress,
        limits::AuthLimiter,
        oauth::{fake::FakeProvider, OAuthProvider, ProviderId, ProviderProfile},
        password::HashGate,
        AuthState,
    },
    client_ip::TrustedProxies,
    ratelimit::{Limiter, MemoryLimiter},
};
use sqlx::PgPool;
use time::OffsetDateTime;
use tower::ServiceExt;

const BASE: &str = "https://counterspell.example";

fn profile(id: &str, email: Option<&str>, verified: bool) -> ProviderProfile {
    ProviderProfile {
        provider_user_id: id.to_owned(),
        label: Some(format!("label-{id}")),
        verified_email: email
            .filter(|_| verified)
            .map(|text| EmailAddress::parse(text).expect("a valid test address")),
    }
}

/// A router with one fake GitHub provider that accepts the given codes.
fn app(pool: &PgPool, profiles: Vec<(&str, ProviderProfile)>) -> Router {
    let clock = Arc::new(ManualClock::new(
        OffsetDateTime::from_unix_timestamp(1_800_000_000).expect("valid timestamp"),
    ));
    let limits = AuthLimiter::new(
        Limiter::Memory(Arc::new(MemoryLimiter::default())),
        &Redacted::new(vec![5_u8; 32]),
    );
    let github: Arc<dyn OAuthProvider> =
        Arc::new(FakeProvider::new(ProviderId::GitHub, profiles).expect("the fake URL parses"));
    let state = AuthState::new(
        pool.clone(),
        clock,
        CookieSettings::new(false),
        limits,
        HashGate::new(4, StdDuration::from_secs(2)),
        TrustedProxies::parse("10.0.0.0/8").expect("valid trusted proxies"),
    )
    .with_oauth(BASE.to_owned(), vec![github]);
    let peer = SocketAddr::from(([203, 0, 113, 9], 40_000));
    counterspell_server::auth::router(state).layer(MockConnectInfo(peer))
}

async fn call(app: &Router, uri: &str) -> Response {
    app.clone()
        .oneshot(
            Request::builder()
                .method(Method::GET)
                .uri(uri)
                .body(Body::empty())
                .expect("valid request"),
        )
        .await
        .expect("the router never fails")
}

/// The `Set-Cookie` values of a response, as text.
fn set_cookies(headers: &HeaderMap) -> Vec<String> {
    headers
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok().map(str::to_owned))
        .collect()
}

/// Reads the value of one cookie from the response's `Set-Cookie` headers.
fn cookie_value(headers: &HeaderMap, name: &str) -> Option<String> {
    set_cookies(headers).into_iter().find_map(|line| {
        let pair = line.split(';').next()?;
        let (key, value) = pair.split_once('=')?;
        (key == name && !value.is_empty()).then(|| value.to_owned())
    })
}

fn location(headers: &HeaderMap) -> String {
    headers
        .get(header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("a redirect with a Location header")
        .to_owned()
}

/// Starts a sign-in and returns the `state` and the binding cookie value.
async fn start(app: &Router, provider: &str, return_to: &str) -> (String, String) {
    let response = call(
        app,
        &format!("/api/auth/oauth/{provider}/start?return_to={return_to}"),
    )
    .await;
    assert_eq!(response.status(), StatusCode::SEE_OTHER);
    let target = location(response.headers());
    let url = url::Url::parse(&target).expect("an absolute authorize URL");
    let state = url
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.into_owned())
        .expect("the authorize URL carries state");
    let binding = cookie_value(response.headers(), "cs_oauth").expect("a binding cookie");
    (state, binding)
}

/// Calls the callback with an optional binding cookie.
async fn back(
    app: &Router,
    provider: &str,
    query: &str,
    binding: Option<&str>,
) -> (StatusCode, HeaderMap, String) {
    let mut builder = Request::builder()
        .method(Method::GET)
        .uri(format!("/api/auth/oauth/{provider}/callback?{query}"));
    if let Some(value) = binding {
        builder = builder.header(header::COOKIE, format!("cs_oauth={value}"));
    }
    let response = app
        .clone()
        .oneshot(builder.body(Body::empty()).expect("valid request"))
        .await
        .expect("the router never fails");
    let status = response.status();
    let headers = response.headers().clone();
    let target = location(&headers);
    (status, headers, target)
}

async fn count(pool: &PgPool, table: &str) -> i64 {
    let sql = format!("SELECT count(*) FROM {table}");
    let (count,): (i64,) = sqlx::query_as(&sql).fetch_one(pool).await.expect("count");
    count
}

/// One complete sign-in: start, then the callback with the same state and cookie.
async fn sign_in(app: &Router, code: &str) -> (StatusCode, HeaderMap, String) {
    let (state, binding) = start(app, "github", "/problems").await;
    back(
        app,
        "github",
        &format!("code={code}&state={state}"),
        Some(&binding),
    )
    .await
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_oauth_model_a_new_identity_creates_one_account_then_signs_in_to_it(pool: PgPool) {
    let app = app(
        &pool,
        vec![("c1", profile("42", Some("octo@example.co"), true))],
    );

    let (status, headers, _) = sign_in(&app, "c1").await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    assert_eq!(location(&headers), "/problems");
    assert!(cookie_value(&headers, "cs_session").is_some());
    assert_eq!(count(&pool, "users").await, 1);
    assert_eq!(count(&pool, "identities").await, 1);

    let (_, second, _) = sign_in(&app, "c1").await;
    assert_eq!(location(&second), "/problems");
    assert_eq!(
        count(&pool, "users").await,
        1,
        "the second sign-in reuses the account"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_oauth_model_a_matching_email_refuses_and_changes_nothing(pool: PgPool) {
    sqlx::query("INSERT INTO users (email, email_verified_at, handle, display_name) VALUES ('taken@example.co', now(), 'owner', 'owner')")
        .execute(&pool)
        .await
        .expect("an existing account");
    let app = app(
        &pool,
        vec![("c2", profile("99", Some("taken@example.co"), true))],
    );
    let (users, identities, sessions) = (
        count(&pool, "users").await,
        count(&pool, "identities").await,
        count(&pool, "sessions").await,
    );

    let (_, headers, where_to) = sign_in(&app, "c2").await;

    assert_eq!(where_to, "/login?error=account_exists");
    assert!(cookie_value(&headers, "cs_session").is_none());
    assert_eq!(count(&pool, "users").await, users);
    assert_eq!(count(&pool, "identities").await, identities);
    assert_eq!(count(&pool, "sessions").await, sessions);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_oauth_model_a_replayed_state_fails(pool: PgPool) {
    let app = app(&pool, vec![("c3", profile("7", None, false))]);
    let (state, binding) = start(&app, "github", "/").await;
    let query = format!("code=c3&state={state}");
    let (_, _, first) = back(&app, "github", &query, Some(&binding)).await;
    assert_eq!(first, "/");

    let (_, _, second) = back(&app, "github", &query, Some(&binding)).await;

    assert_eq!(second, "/login?error=state_invalid");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_oauth_model_a_missing_binding_fails_and_still_uses_the_flow(pool: PgPool) {
    let app = app(&pool, vec![("c4", profile("8", None, false))]);
    let (state, binding) = start(&app, "github", "/").await;
    let query = format!("code=c4&state={state}");

    let (_, _, without) = back(&app, "github", &query, None).await;
    let (_, _, with_it) = back(&app, "github", &query, Some(&binding)).await;

    assert_eq!(without, "/login?error=state_invalid");
    assert_eq!(
        with_it, "/login?error=state_invalid",
        "the flow was used up"
    );
    assert_eq!(count(&pool, "users").await, 0);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_oauth_model_a_refused_consent_redirects_with_oauth_denied(pool: PgPool) {
    let app = app(&pool, vec![]);
    let (state, binding) = start(&app, "github", "/").await;

    let (_, _, where_to) = back(
        &app,
        "github",
        &format!("error=access_denied&state={state}"),
        Some(&binding),
    )
    .await;

    assert_eq!(where_to, "/login?error=oauth_denied");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_oauth_model_an_unverified_provider_email_creates_an_account_without_email(
    pool: PgPool,
) {
    let app = app(
        &pool,
        vec![("c5", profile("9", Some("unverified@example.co"), false))],
    );

    let (_, headers, _) = sign_in(&app, "c5").await;

    assert!(cookie_value(&headers, "cs_session").is_some());
    let (email,): (Option<String>,) = sqlx::query_as("SELECT email::text FROM users LIMIT 1")
        .fetch_one(&pool)
        .await
        .expect("the new account");
    assert_eq!(email, None);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_oauth_model_the_demo_address_counts_as_taken(pool: PgPool) {
    sqlx::query("SELECT cs_seed_demo_user('demo@example.co', 'placeholder-hash')")
        .execute(&pool)
        .await
        .expect("seed the demo account");
    let app = app(
        &pool,
        vec![("c6", profile("10", Some("demo@example.co"), true))],
    );

    let (_, headers, where_to) = sign_in(&app, "c6").await;

    assert_eq!(where_to, "/login?error=account_exists");
    assert!(cookie_value(&headers, "cs_session").is_none());
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_oauth_model_a_disabled_provider_is_not_found(pool: PgPool) {
    let app = app(&pool, vec![]);

    let response = call(&app, "/api/auth/oauth/google/start").await;

    assert_eq!(response.status(), StatusCode::NOT_FOUND);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_oauth_model_every_callback_clears_the_binding_cookie(pool: PgPool) {
    let app = app(&pool, vec![("c7", profile("11", None, false))]);
    let (state, binding) = start(&app, "github", "/").await;

    let (_, headers, _) = back(
        &app,
        "github",
        &format!("code=c7&state={state}"),
        Some(&binding),
    )
    .await;

    let cleared = set_cookies(&headers)
        .into_iter()
        .any(|line| line.starts_with("cs_oauth=") && line.contains("Max-Age=0"));
    assert!(cleared);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_oauth_model_a_sequence_of_callbacks_keeps_one_account_per_identity(pool: PgPool) {
    // Accounts A and C are new, B reuses A's verified address, and A signs in again.
    let app = app(
        &pool,
        vec![
            ("a", profile("1", Some("shared@example.co"), true)),
            ("b", profile("2", Some("shared@example.co"), true)),
            ("c", profile("3", None, false)),
        ],
    );

    let first = sign_in(&app, "a").await;
    let second = sign_in(&app, "b").await;
    let third = sign_in(&app, "a").await;
    let fourth = sign_in(&app, "c").await;

    assert!(cookie_value(&first.1, "cs_session").is_some());
    assert_eq!(second.2, "/login?error=account_exists");
    assert!(cookie_value(&third.1, "cs_session").is_some());
    assert!(cookie_value(&fourth.1, "cs_session").is_some());
    assert_eq!(count(&pool, "users").await, 2);
    assert_eq!(count(&pool, "identities").await, 2);
}
