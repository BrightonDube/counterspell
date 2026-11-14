//! Linking a GitHub or Google identity to a signed-in account (auth T22, property P8,
//! requirements R8.1 to R8.8).
//!
//! Run with `DATABASE_URL` set: `cargo test -p counterspell-server pg_auth_link_provider -- --include-ignored`.

use std::{net::SocketAddr, sync::Arc, time::Duration as StdDuration};

use axum::{
    body::Body,
    extract::connect_info::MockConnectInfo,
    http::{header, HeaderMap, Method, Request, StatusCode},
    response::Response,
    Router,
};
use counterspell_common::ids::UserId;
use counterspell_server::{
    auth::{
        clock::ManualClock,
        config::Redacted,
        cookies::CookieSettings,
        email::EmailAddress,
        limits::AuthLimiter,
        oauth::{fake::FakeProvider, OAuthProvider, ProviderId, ProviderProfile},
        password::HashGate,
        session::{AuthMethod, SessionService},
        AuthState,
    },
    client_ip::TrustedProxies,
    ratelimit::{Limiter, MemoryLimiter},
};
use sqlx::{types::Uuid, PgPool};
use time::OffsetDateTime;
use tower::ServiceExt;

const BASE: &str = "https://counterspell.example";
const START: i64 = 1_800_000_000;

fn start_time() -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(START).expect("valid timestamp")
}

fn profile(id: &str, email: Option<&str>) -> ProviderProfile {
    ProviderProfile {
        provider_user_id: id.to_owned(),
        label: Some(format!("label-{id}")),
        verified_email: email.map(|text| EmailAddress::parse(text).expect("a valid test address")),
    }
}

/// The auth router with one fake GitHub provider. Its profiles are keyed by the callback code.
fn app(pool: &PgPool, clock: &Arc<ManualClock>, profiles: Vec<(&str, ProviderProfile)>) -> Router {
    let limits = AuthLimiter::new(
        Limiter::Memory(Arc::new(MemoryLimiter::default())),
        &Redacted::new(vec![5_u8; 32]),
    );
    let github: Arc<dyn OAuthProvider> =
        Arc::new(FakeProvider::new(ProviderId::GitHub, profiles).expect("the fake URL parses"));
    let state = AuthState::new(
        pool.clone(),
        clock.clone(),
        CookieSettings::new(false),
        limits,
        HashGate::new(4, StdDuration::from_secs(2)),
        TrustedProxies::parse("10.0.0.0/8").expect("valid trusted proxies"),
    )
    .with_oauth(BASE.to_owned(), vec![github]);
    let peer = SocketAddr::from(([203, 0, 113, 9], 40_000));
    counterspell_server::auth::router(state).layer(MockConnectInfo(peer))
}

async fn send(app: &Router, request: Request<Body>) -> (StatusCode, HeaderMap, String) {
    let response: Response = app
        .clone()
        .oneshot(request)
        .await
        .expect("the router never fails");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = axum::body::to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("a small body");
    (
        status,
        headers,
        String::from_utf8_lossy(&bytes).into_owned(),
    )
}

/// Creates a real session for a user and returns the raw cookie value.
async fn session_for(pool: &PgPool, clock: &Arc<ManualClock>, user: UserId) -> String {
    let service = SessionService::new(pool.clone(), clock.clone());
    let created = service
        .create(user, AuthMethod::MagicLink)
        .await
        .expect("a session");
    created.cookie_value.expose().to_owned()
}

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

/// Gives a user an existing GitHub identity, as if it had been linked before.
async fn link_existing(pool: &PgPool, user: UserId, provider_user_id: &str) {
    sqlx::query(
        "INSERT INTO identities (user_id, provider, provider_user_id) VALUES ($1, 'github', $2)",
    )
    .bind(user.as_uuid())
    .bind(provider_user_id)
    .execute(pool)
    .await
    .expect("an existing identity");
}

async fn identity_count(pool: &PgPool, user: UserId) -> i64 {
    let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM identities WHERE user_id = $1")
        .bind(user.as_uuid())
        .fetch_one(pool)
        .await
        .expect("count identities");
    count
}

async fn total_identities(pool: &PgPool) -> i64 {
    let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM identities")
        .fetch_one(pool)
        .await
        .expect("count identities");
    count
}

fn with_cookie(method: Method, uri: &str, session: Option<&str>) -> Request<Body> {
    let mut builder = Request::builder().method(method).uri(uri);
    if let Some(raw) = session {
        builder = builder.header(header::COOKIE, format!("cs_session={raw}"));
    }
    builder.body(Body::empty()).expect("valid request")
}

fn location(headers: &HeaderMap) -> String {
    headers
        .get(header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("a redirect with a Location header")
        .to_owned()
}

/// Starts linking for a session and returns the `state` from the authorize URL.
async fn start_link(app: &Router, provider: &str, session: Option<&str>) -> (StatusCode, String) {
    let request = with_cookie(
        Method::POST,
        &format!("/api/auth/link/{provider}/start"),
        session,
    );
    let (status, _, body) = send(app, request).await;
    if status != StatusCode::OK {
        return (status, String::new());
    }
    let json: serde_json::Value = serde_json::from_str(&body).expect("JSON body");
    let target = json["authorize_url"]
        .as_str()
        .expect("an authorize_url string");
    let url = url::Url::parse(target).expect("an absolute authorize URL");
    let state = url
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.into_owned())
        .expect("the authorize URL carries state");
    (status, state)
}

/// Calls the provider callback with a code and a state, optionally signed in as someone.
async fn back(
    app: &Router,
    provider: &str,
    code: &str,
    state: &str,
    session: Option<&str>,
) -> String {
    let uri = format!("/api/auth/oauth/{provider}/callback?code={code}&state={state}");
    let (status, headers, _) = send(app, with_cookie(Method::GET, &uri, session)).await;
    assert_eq!(status, StatusCode::SEE_OTHER);
    location(&headers)
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_link_provider_linking_adds_exactly_one_identity_to_the_signed_in_user(
    pool: PgPool,
) {
    let clock = Arc::new(ManualClock::new(start_time()));
    let alice = verified_user(&pool, "alice").await;
    let session = session_for(&pool, &clock, alice).await;
    let app = app(&pool, &clock, vec![("l1", profile("500", None))]);

    let (status, state) = start_link(&app, "github", Some(&session)).await;
    assert_eq!(status, StatusCode::OK);
    let where_to = back(&app, "github", "l1", &state, Some(&session)).await;

    assert_eq!(where_to, "/settings/account?linked=github");
    assert_eq!(identity_count(&pool, alice).await, 1);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_link_provider_a_callback_from_another_session_changes_nothing(pool: PgPool) {
    let clock = Arc::new(ManualClock::new(start_time()));
    let alice = verified_user(&pool, "alice").await;
    let bob = verified_user(&pool, "bobby").await;
    let alice_session = session_for(&pool, &clock, alice).await;
    let bob_session = session_for(&pool, &clock, bob).await;
    let app = app(&pool, &clock, vec![("l2", profile("501", None))]);
    let (_, state) = start_link(&app, "github", Some(&alice_session)).await;

    let where_to = back(&app, "github", "l2", &state, Some(&bob_session)).await;

    assert_eq!(where_to, "/settings/account?error=state_invalid");
    assert_eq!(total_identities(&pool).await, 0);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_link_provider_a_callback_without_a_session_fails(pool: PgPool) {
    let clock = Arc::new(ManualClock::new(start_time()));
    let alice = verified_user(&pool, "alice").await;
    let alice_session = session_for(&pool, &clock, alice).await;
    let app = app(&pool, &clock, vec![("l3", profile("502", None))]);
    let (_, state) = start_link(&app, "github", Some(&alice_session)).await;

    let where_to = back(&app, "github", "l3", &state, None).await;

    assert_eq!(where_to, "/settings/account?error=state_invalid");
    assert_eq!(total_identities(&pool).await, 0);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_link_provider_logging_out_before_the_callback_kills_the_flow(pool: PgPool) {
    let clock = Arc::new(ManualClock::new(start_time()));
    let alice = verified_user(&pool, "alice").await;
    let session = session_for(&pool, &clock, alice).await;
    let app = app(&pool, &clock, vec![("l4", profile("503", None))]);
    let (_, state) = start_link(&app, "github", Some(&session)).await;
    let (logout, _, _) = send(
        &app,
        with_cookie(Method::POST, "/api/auth/logout", Some(&session)),
    )
    .await;
    assert_eq!(logout, StatusCode::NO_CONTENT);

    let where_to = back(&app, "github", "l4", &state, Some(&session)).await;

    // The cascade deleted the flow with its session, so the callback no longer knows it was a
    // link flow and uses the sign-in page for the error. What matters is that it fails.
    assert_eq!(where_to, "/login?error=state_invalid");
    assert_eq!(identity_count(&pool, alice).await, 0);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_link_provider_an_identity_of_another_account_is_refused_unchanged(pool: PgPool) {
    let clock = Arc::new(ManualClock::new(start_time()));
    let owner = verified_user(&pool, "owner").await;
    link_existing(&pool, owner, "504").await;
    let mallory = verified_user(&pool, "mallory").await;
    let session = session_for(&pool, &clock, mallory).await;
    let app = app(&pool, &clock, vec![("l5", profile("504", None))]);
    let (_, state) = start_link(&app, "github", Some(&session)).await;

    let where_to = back(&app, "github", "l5", &state, Some(&session)).await;

    assert_eq!(where_to, "/settings/account?error=identity_already_linked");
    assert_eq!(identity_count(&pool, mallory).await, 0);
    assert_eq!(identity_count(&pool, owner).await, 1);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_link_provider_the_same_identity_again_is_a_no_op(pool: PgPool) {
    let clock = Arc::new(ManualClock::new(start_time()));
    let owner = verified_user(&pool, "owner").await;
    link_existing(&pool, owner, "505").await;
    let session = session_for(&pool, &clock, owner).await;
    let app = app(&pool, &clock, vec![("l6", profile("505", None))]);
    let (_, state) = start_link(&app, "github", Some(&session)).await;

    let where_to = back(&app, "github", "l6", &state, Some(&session)).await;

    assert_eq!(where_to, "/settings/account?linked=github");
    assert_eq!(identity_count(&pool, owner).await, 1);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_link_provider_a_second_identity_of_one_provider_is_refused(pool: PgPool) {
    let clock = Arc::new(ManualClock::new(start_time()));
    let owner = verified_user(&pool, "owner").await;
    link_existing(&pool, owner, "506").await;
    let session = session_for(&pool, &clock, owner).await;
    let app = app(&pool, &clock, vec![("l7", profile("507", None))]);
    let (_, state) = start_link(&app, "github", Some(&session)).await;

    let where_to = back(&app, "github", "l7", &state, Some(&session)).await;

    assert_eq!(where_to, "/settings/account?error=provider_already_linked");
    assert_eq!(identity_count(&pool, owner).await, 1);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_link_provider_the_provider_email_is_ignored_for_linking(pool: PgPool) {
    let clock = Arc::new(ManualClock::new(start_time()));
    let alice = verified_user(&pool, "alice").await;
    let bob = verified_user(&pool, "bobby").await;
    let session = session_for(&pool, &clock, alice).await;
    // The provider reports Bob's address. That must not change either account.
    let app = app(
        &pool,
        &clock,
        vec![("l8", profile("508", Some("bobby@example.co")))],
    );
    let (_, state) = start_link(&app, "github", Some(&session)).await;

    let where_to = back(&app, "github", "l8", &state, Some(&session)).await;

    assert_eq!(where_to, "/settings/account?linked=github");
    assert_eq!(identity_count(&pool, alice).await, 1);
    assert_eq!(identity_count(&pool, bob).await, 0);
    let (alice_email,): (Option<String>,) =
        sqlx::query_as("SELECT email::text FROM users WHERE id = $1")
            .bind(alice.as_uuid())
            .fetch_one(&pool)
            .await
            .expect("alice");
    assert_eq!(alice_email.as_deref(), Some("alice@example.co"));
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_link_provider_the_demo_session_is_refused_with_403(pool: PgPool) {
    sqlx::query("SELECT cs_seed_demo_user('demo@example.co', 'placeholder-hash')")
        .execute(&pool)
        .await
        .expect("seed the demo account");
    let (demo_id,): (Uuid,) = sqlx::query_as("SELECT id FROM users WHERE is_demo")
        .fetch_one(&pool)
        .await
        .expect("the demo user");
    let clock = Arc::new(ManualClock::new(start_time()));
    let session = session_for(&pool, &clock, UserId::from_uuid(demo_id)).await;
    let app = app(&pool, &clock, vec![]);

    let (status, _) = start_link(&app, "github", Some(&session)).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    let (flows,): (i64,) = sqlx::query_as("SELECT count(*) FROM oauth_flows")
        .fetch_one(&pool)
        .await
        .expect("count flows");
    assert_eq!(flows, 0, "no flow is created for the demo account");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_link_provider_start_without_a_session_is_unauthenticated(pool: PgPool) {
    let clock = Arc::new(ManualClock::new(start_time()));
    let app = app(&pool, &clock, vec![]);

    let (status, _) = start_link(&app, "github", None).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
