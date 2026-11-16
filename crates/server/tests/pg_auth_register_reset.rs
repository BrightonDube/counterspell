//! Password registration and password reset (auth T25, requirements R15.7 to R15.9, R11.5,
//! R16.4 and R16.5). Property P14 covers the uniform answers.
//!
//! Run with `DATABASE_URL` set: `cargo test -p counterspell-server pg_auth_register_reset -- --include-ignored`.

use std::{net::SocketAddr, sync::Arc, time::Duration as StdDuration};

use axum::{
    body::{to_bytes, Body},
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
        email::{EmailAddress, EmailSender, MemorySender},
        limits::AuthLimiter,
        password::HashGate,
        session::{AuthMethod, SessionService},
        AuthState, MailPort,
    },
    client_ip::TrustedProxies,
    ratelimit::{Limiter, MemoryLimiter},
};
use serde_json::{json, Value};
use sqlx::{types::Uuid, PgPool};
use time::OffsetDateTime;
use tower::ServiceExt;

const DEMO: &str = "demo@example.co";
const OLD_PASSWORD: &str = "the-old-passphrase";
const NEW_PASSWORD: &str = "a-brand-new-passphrase";

/// The router under test, its clock and the mail it sent.
struct Harness {
    app: Router,
    clock: Arc<ManualClock>,
    mail: Arc<MemorySender>,
}

fn harness(pool: &PgPool) -> Harness {
    let clock = Arc::new(ManualClock::new(
        OffsetDateTime::from_unix_timestamp(1_800_000_000).expect("valid timestamp"),
    ));
    let limits = AuthLimiter::new(
        Limiter::Memory(Arc::new(MemoryLimiter::default())),
        &Redacted::new(vec![5_u8; 32]),
    );
    let mail = Arc::new(MemorySender::default());
    let sender: Arc<dyn EmailSender> = mail.clone();
    let state = AuthState::new(
        pool.clone(),
        clock.clone(),
        CookieSettings::new(false),
        limits,
        HashGate::new(4, StdDuration::from_secs(2)),
        TrustedProxies::parse("10.0.0.0/8").expect("valid trusted proxies"),
    )
    .with_mail(MailPort {
        sender,
        public_base_url: "https://counterspell.example".to_owned(),
        demo_email: Some(EmailAddress::parse(DEMO).expect("valid demo address")),
    });
    let peer = SocketAddr::from(([203, 0, 113, 9], 40_000));
    Harness {
        app: counterspell_server::auth::router(state).layer(MockConnectInfo(peer)),
        clock,
        mail,
    }
}

/// Sends one JSON request, optionally signed in, and returns status, headers and body.
async fn call(
    app: &Router,
    method: Method,
    path: &str,
    body: Option<Value>,
    session: Option<&str>,
) -> (StatusCode, HeaderMap, Value) {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(raw) = session {
        builder = builder.header(header::COOKIE, format!("cs_session={raw}"));
    }
    let body = match body {
        Some(json) => {
            builder = builder.header(header::CONTENT_TYPE, "application/json");
            Body::from(json.to_string())
        }
        None => Body::empty(),
    };
    let response: Response = app
        .clone()
        .oneshot(builder.body(body).expect("valid request"))
        .await
        .expect("the router never fails");
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("a small body");
    let json = if bytes.is_empty() {
        Value::Null
    } else {
        serde_json::from_slice(&bytes).expect("JSON body")
    };
    (status, headers, json)
}

/// Creates a real session for a user and returns the raw cookie value.
async fn session_for(pool: &PgPool, clock: &Arc<ManualClock>, user: UserId) -> String {
    SessionService::new(pool.clone(), clock.clone())
        .create(user, AuthMethod::Password)
        .await
        .expect("a session")
        .cookie_value
        .expose()
        .to_owned()
}

/// An account with a password hash and a verified email, as if it had registered before.
async fn account_with_password(pool: &PgPool, handle: &str, email: &str) -> UserId {
    let hash =
        counterspell_server::auth::password::hash_password(OLD_PASSWORD).expect("a password hash");
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, email_verified_at, password_hash, handle, display_name)
         VALUES ($1, now(), $2, $3, $3) RETURNING id",
    )
    .bind(email)
    .bind(hash)
    .bind(handle)
    .fetch_one(pool)
    .await
    .expect("insert account");
    UserId::from_uuid(id)
}

async fn count(pool: &PgPool, sql: &str) -> i64 {
    let (count,): (i64,) = sqlx::query_as(sql).fetch_one(pool).await.expect("count");
    count
}

/// Waits for the detached send to deliver one message and returns its text body.
async fn wait_for_mail(mail: &MemorySender) -> String {
    for _ in 0..200 {
        if let Some(message) = mail.sent().last() {
            return message.text.clone();
        }
        tokio::time::sleep(StdDuration::from_millis(10)).await;
    }
    panic!("no mail arrived");
}

/// Pulls the raw token out of the fragment of the link in the mail text.
fn token_from(text: &str) -> String {
    let start = text
        .find("#token=")
        .expect("the mail carries a token fragment")
        + "#token=".len();
    text[start..]
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

/// Registers an address and password. The answer is returned as status and body.
async fn register(h: &Harness, email: &str, password: &str) -> (StatusCode, Value) {
    let (status, _, body) = call(
        &h.app,
        Method::POST,
        "/api/auth/register",
        Some(json!({ "email": email, "password": password })),
        None,
    )
    .await;
    (status, body)
}

/// Uses a verify token. The answer is returned as status, body and headers.
async fn verify(h: &Harness, token: &str) -> (StatusCode, HeaderMap, Value) {
    call(
        &h.app,
        Method::POST,
        "/api/auth/email/verify",
        Some(json!({ "token": token })),
        None,
    )
    .await
}

async fn request_reset(h: &Harness, email: &str) -> (StatusCode, Value) {
    let (status, _, body) = call(
        &h.app,
        Method::POST,
        "/api/auth/password/reset",
        Some(json!({ "email": email })),
        None,
    )
    .await;
    (status, body)
}

async fn confirm_reset(h: &Harness, token: &str, password: &str) -> (StatusCode, Value) {
    let (status, _, body) = call(
        &h.app,
        Method::POST,
        "/api/auth/password/reset/confirm",
        Some(json!({ "token": token, "new_password": password })),
        None,
    )
    .await;
    (status, body)
}

async fn login(h: &Harness, email: &str, password: &str) -> StatusCode {
    let (status, _, _) = call(
        &h.app,
        Method::POST,
        "/api/auth/login",
        Some(json!({ "email": email, "password": password })),
        None,
    )
    .await;
    status
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_register_reset_no_account_exists_before_the_mailbox_is_proven(pool: PgPool) {
    let h = harness(&pool);

    let (status, _) = register(&h, "fresh@example.co", NEW_PASSWORD).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let before = count(
        &pool,
        "SELECT count(*) FROM users WHERE email = 'fresh@example.co'",
    )
    .await;
    let token = token_from(&wait_for_mail(&h.mail).await);
    let (status, headers, body) = verify(&h, &token).await;

    assert_eq!(before, 0, "registering creates no account row");
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["return_to"], "/");
    assert!(headers.get(header::SET_COOKIE).is_some());
    assert_eq!(
        count(&pool, "SELECT count(*) FROM users WHERE email = 'fresh@example.co' AND password_hash IS NOT NULL").await,
        1
    );
    assert_eq!(
        login(&h, "fresh@example.co", NEW_PASSWORD).await,
        StatusCode::OK
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_register_reset_registering_a_taken_address_changes_nothing(pool: PgPool) {
    let owner = account_with_password(&pool, "owner", "owned@example.co").await;
    let h = harness(&pool);
    let (hash_before,): (Option<String>,) =
        sqlx::query_as("SELECT password_hash FROM users WHERE id = $1")
            .bind(owner.as_uuid())
            .fetch_one(&pool)
            .await
            .expect("the account");

    register(&h, "owned@example.co", NEW_PASSWORD).await;
    let token = token_from(&wait_for_mail(&h.mail).await);
    let (status, headers, body) = verify(&h, &token).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "account_exists");
    assert!(
        headers.get(header::SET_COOKIE).is_none(),
        "nobody is signed in"
    );
    let (hash_after,): (Option<String>,) =
        sqlx::query_as("SELECT password_hash FROM users WHERE id = $1")
            .bind(owner.as_uuid())
            .fetch_one(&pool)
            .await
            .expect("the account");
    assert_eq!(
        hash_after, hash_before,
        "the existing password is unchanged"
    );
    assert_eq!(count(&pool, "SELECT count(*) FROM users").await, 1);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_register_reset_registration_answers_match_for_known_and_unknown(pool: PgPool) {
    account_with_password(&pool, "known", "known@example.co").await;
    let h = harness(&pool);

    let known = register(&h, "known@example.co", NEW_PASSWORD).await;
    let unknown = register(&h, "unknown@example.co", NEW_PASSWORD).await;

    assert_eq!(known.0, StatusCode::ACCEPTED);
    assert_eq!(known, unknown);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_register_reset_a_weak_password_is_refused_before_any_mail(pool: PgPool) {
    let h = harness(&pool);

    let (status, body) = register(&h, "weak@example.co", "short").await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "weak_password");
    assert_eq!(count(&pool, "SELECT count(*) FROM email_tokens").await, 0);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_register_reset_reset_revokes_every_session_and_replaces_the_password(
    pool: PgPool,
) {
    let user = account_with_password(&pool, "walker", "walker@example.co").await;
    let h = harness(&pool);
    session_for(&pool, &h.clock, user).await;
    session_for(&pool, &h.clock, user).await;
    let sessions_sql = format!(
        "SELECT count(*) FROM sessions WHERE user_id = '{}'",
        user.as_uuid()
    );
    assert_eq!(
        count(&pool, &sessions_sql).await,
        2,
        "two sessions before the reset"
    );
    request_reset(&h, "walker@example.co").await;
    let token = token_from(&wait_for_mail(&h.mail).await);

    let (status, _) = confirm_reset(&h, &token, NEW_PASSWORD).await;

    assert_eq!(status, StatusCode::NO_CONTENT);
    assert_eq!(
        count(&pool, &sessions_sql).await,
        0,
        "every session of the user is gone"
    );
    assert_eq!(
        login(&h, "walker@example.co", OLD_PASSWORD).await,
        StatusCode::UNAUTHORIZED
    );
    assert_eq!(
        login(&h, "walker@example.co", NEW_PASSWORD).await,
        StatusCode::OK
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_register_reset_a_reset_link_is_single_use(pool: PgPool) {
    account_with_password(&pool, "once", "once@example.co").await;
    let h = harness(&pool);
    request_reset(&h, "once@example.co").await;
    let token = token_from(&wait_for_mail(&h.mail).await);

    let (first, _) = confirm_reset(&h, &token, NEW_PASSWORD).await;
    let (second, _) = confirm_reset(&h, &token, "yet-another-passphrase").await;

    assert_eq!(first, StatusCode::NO_CONTENT);
    assert_eq!(second, StatusCode::BAD_REQUEST);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_register_reset_reset_sends_nothing_for_passwordless_or_demo_accounts(
    pool: PgPool,
) {
    sqlx::query("INSERT INTO users (email, email_verified_at, handle, display_name) VALUES ('nopass@example.co', now(), 'nopass', 'nopass')")
        .execute(&pool)
        .await
        .expect("a passwordless account");
    sqlx::query("SELECT cs_seed_demo_user($1, 'placeholder-hash')")
        .bind(DEMO)
        .execute(&pool)
        .await
        .expect("seed the demo account");
    let h = harness(&pool);

    let (passwordless, _) = request_reset(&h, "nopass@example.co").await;
    let (demo, _) = request_reset(&h, DEMO).await;
    tokio::time::sleep(StdDuration::from_millis(50)).await;

    assert_eq!(passwordless, StatusCode::ACCEPTED);
    assert_eq!(demo, StatusCode::ACCEPTED);
    assert!(h.mail.sent().is_empty(), "no reset mail for these accounts");
    assert_eq!(count(&pool, "SELECT count(*) FROM email_tokens").await, 0);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_register_reset_reset_answers_match_for_known_and_unknown(pool: PgPool) {
    account_with_password(&pool, "seen", "seen@example.co").await;
    let h = harness(&pool);

    let known = request_reset(&h, "seen@example.co").await;
    let unknown = request_reset(&h, "nobody@example.co").await;

    assert_eq!(known.0, StatusCode::ACCEPTED);
    assert_eq!(known, unknown);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_register_reset_a_magic_link_never_adds_a_password(pool: PgPool) {
    let h = harness(&pool);
    call(
        &h.app,
        Method::POST,
        "/api/auth/email/magic",
        Some(json!({ "email": "passless@example.co" })),
        None,
    )
    .await;
    let token = token_from(&wait_for_mail(&h.mail).await);
    let (status, _, _) = verify(&h, &token).await;
    assert_eq!(status, StatusCode::OK);

    let (has_password,): (bool,) = sqlx::query_as(
        "SELECT password_hash IS NOT NULL FROM users WHERE email = 'passless@example.co'",
    )
    .fetch_one(&pool)
    .await
    .expect("the account");
    assert!(!has_password, "only register and reset may set a password");
}
