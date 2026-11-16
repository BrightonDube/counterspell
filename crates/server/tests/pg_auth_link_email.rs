//! Adding an email address to a signed-in account by mailbox proof (auth T23, requirements R9.1
//! to R9.6 and R14.1).
//!
//! Run with `DATABASE_URL` set: `cargo test -p counterspell-server pg_auth_link_email -- --include-ignored`.

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
use time::{Duration, OffsetDateTime};
use tower::ServiceExt;

const DEMO: &str = "demo@example.co";

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

/// Sends one request, optionally signed in, and returns the response parts.
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
        .create(user, AuthMethod::MagicLink)
        .await
        .expect("a session")
        .cookie_value
        .expose()
        .to_owned()
}

/// An account that signed in with a provider and has no email address.
async fn account_without_email(pool: &PgPool, handle: &str) -> UserId {
    let (id,): (Uuid,) =
        sqlx::query_as("INSERT INTO users (handle, display_name) VALUES ($1, $1) RETURNING id")
            .bind(handle)
            .fetch_one(pool)
            .await
            .expect("insert account");
    UserId::from_uuid(id)
}

/// An account that already has a verified email address.
async fn account_with_email(pool: &PgPool, handle: &str, email: &str) -> UserId {
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, email_verified_at, handle, display_name)
         VALUES ($1, now(), $2, $2) RETURNING id",
    )
    .bind(email)
    .bind(handle)
    .fetch_one(pool)
    .await
    .expect("insert account with email");
    UserId::from_uuid(id)
}

async fn token_count(pool: &PgPool) -> i64 {
    let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM email_tokens")
        .fetch_one(pool)
        .await
        .expect("count tokens");
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

/// Pulls the raw token out of the fragment of the sign-in link in the mail text.
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

/// Asks to link `email` to the signed-in account.
async fn request_link(harness: &Harness, email: &str, session: &str) -> (StatusCode, Value) {
    let (status, _, body) = call(
        &harness.app,
        Method::POST,
        "/api/auth/link/email",
        Some(json!({ "email": email })),
        Some(session),
    )
    .await;
    (status, body)
}

/// Uses an emailed token from a session.
async fn verify(app: &Router, token: &str, session: Option<&str>) -> (StatusCode, Value) {
    let (status, _, body) = call(
        app,
        Method::POST,
        "/api/auth/email/verify",
        Some(json!({ "token": token })),
        session,
    )
    .await;
    (status, body)
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_link_email_the_mailed_link_attaches_the_address_once(pool: PgPool) {
    let h = harness(&pool);
    let owner = account_without_email(&pool, "owner").await;
    let session = session_for(&pool, &h.clock, owner).await;

    let (status, _) = request_link(&h, "new@example.co", &session).await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let token = token_from(&wait_for_mail(&h.mail).await);

    let (status, body) = verify(&h.app, &token, Some(&session)).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "email_linked");
    let (email, verified): (Option<String>, Option<OffsetDateTime>) =
        sqlx::query_as("SELECT email::text, email_verified_at FROM users WHERE id = $1")
            .bind(owner.as_uuid())
            .fetch_one(&pool)
            .await
            .expect("the account");
    assert_eq!(email.as_deref(), Some("new@example.co"));
    assert!(verified.is_some());

    let (again, _) = verify(&h.app, &token, Some(&session)).await;
    assert_eq!(again, StatusCode::BAD_REQUEST, "the second use fails");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_link_email_free_and_taken_addresses_get_the_same_answer(pool: PgPool) {
    let h = harness(&pool);
    account_with_email(&pool, "holder", "taken@example.co").await;
    let asker = account_without_email(&pool, "asker").await;
    let session = session_for(&pool, &h.clock, asker).await;

    let (taken_status, taken_body) = request_link(&h, "taken@example.co", &session).await;
    tokio::time::sleep(StdDuration::from_millis(50)).await;
    let mail_for_taken = h.mail.sent().len();
    let (free_status, free_body) = request_link(&h, "free@example.co", &session).await;

    assert_eq!(taken_status, free_status);
    assert_eq!(taken_body, free_body);
    assert_eq!(free_status, StatusCode::ACCEPTED);
    assert_eq!(mail_for_taken, 0, "no mail goes to a taken address");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_link_email_a_link_for_another_session_does_not_burn_the_token(pool: PgPool) {
    let h = harness(&pool);
    let owner = account_without_email(&pool, "owner").await;
    let stranger = account_without_email(&pool, "stranger").await;
    let owner_session = session_for(&pool, &h.clock, owner).await;
    let stranger_session = session_for(&pool, &h.clock, stranger).await;
    request_link(&h, "mine@example.co", &owner_session).await;
    let token = token_from(&wait_for_mail(&h.mail).await);

    let (no_session, _) = verify(&h.app, &token, None).await;
    let (wrong_session, _) = verify(&h.app, &token, Some(&stranger_session)).await;
    let (right_session, _) = verify(&h.app, &token, Some(&owner_session)).await;

    assert_eq!(no_session, StatusCode::UNAUTHORIZED);
    assert_eq!(wrong_session, StatusCode::BAD_REQUEST);
    assert_eq!(right_session, StatusCode::OK);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_link_email_the_link_expires_after_fifteen_minutes(pool: PgPool) {
    let h = harness(&pool);
    let owner = account_without_email(&pool, "owner").await;
    let session = session_for(&pool, &h.clock, owner).await;
    request_link(&h, "late@example.co", &session).await;
    let token = token_from(&wait_for_mail(&h.mail).await);
    h.clock.advance(Duration::minutes(15));

    let (status, _) = verify(&h.app, &token, Some(&session)).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_link_email_an_address_taken_after_the_link_was_sent_gives_409(pool: PgPool) {
    let h = harness(&pool);
    let owner = account_without_email(&pool, "owner").await;
    let session = session_for(&pool, &h.clock, owner).await;
    request_link(&h, "race@example.co", &session).await;
    let token = token_from(&wait_for_mail(&h.mail).await);
    account_with_email(&pool, "other", "race@example.co").await;

    let (status, body) = verify(&h.app, &token, Some(&session)).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "email_in_use");
    let (email,): (Option<String>,) = sqlx::query_as("SELECT email::text FROM users WHERE id = $1")
        .bind(owner.as_uuid())
        .fetch_one(&pool)
        .await
        .expect("the account");
    assert_eq!(email, None);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_link_email_an_account_with_an_email_gets_409(pool: PgPool) {
    let h = harness(&pool);
    let user = account_with_email(&pool, "has", "has@example.co").await;
    let session = session_for(&pool, &h.clock, user).await;

    let (status, body) = request_link(&h, "second@example.co", &session).await;

    assert_eq!(status, StatusCode::CONFLICT);
    assert_eq!(body["code"], "email_already_set");
    assert_eq!(token_count(&pool).await, 0);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_link_email_the_demo_account_is_refused(pool: PgPool) {
    sqlx::query("SELECT cs_seed_demo_user($1, 'placeholder-hash')")
        .bind(DEMO)
        .execute(&pool)
        .await
        .expect("seed the demo account");
    let (demo,): (Uuid,) = sqlx::query_as("SELECT id FROM users WHERE is_demo")
        .fetch_one(&pool)
        .await
        .expect("the demo user");
    let h = harness(&pool);
    let session = session_for(&pool, &h.clock, UserId::from_uuid(demo)).await;

    let (status, body) = request_link(&h, "demo-other@example.co", &session).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(body["code"], "demo_account_restricted");
    assert_eq!(token_count(&pool).await, 0);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_link_email_the_start_needs_a_session(pool: PgPool) {
    let h = harness(&pool);

    let (status, _, _) = call(
        &h.app,
        Method::POST,
        "/api/auth/link/email",
        Some(json!({ "email": "anyone@example.co" })),
        None,
    )
    .await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
}
