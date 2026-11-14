//! Magic-link request and verify over real Postgres (auth T17, requirements R5.1 to R5.9, R3.4,
//! R3.5 and R16.5).
//!
//! The mail sender records messages in memory, so each test reads the sign-in link from the
//! mail it received. Run with `DATABASE_URL` set:
//! `cargo test -p counterspell-server auth_magic -- --include-ignored`.

use std::{net::SocketAddr, sync::Arc, time::Duration as StdDuration};

use axum::{
    body::{to_bytes, Body},
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
        email::{EmailAddress, EmailSender, MemorySender},
        limits::AuthLimiter,
        password::HashGate,
        AuthState, MailPort,
    },
    client_ip::TrustedProxies,
    ratelimit::{Limiter, MemoryLimiter},
};
use serde_json::{json, Value};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use tower::ServiceExt;

const DEMO: &str = "demo@example.co";

/// The router under test, the clock that drives expiry and the mail it sends.
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

/// Sends one request and returns the response.
async fn call(app: &Router, method: Method, path: &str, body: Option<Value>) -> Response {
    let mut builder = Request::builder().method(method).uri(path);
    let body = match body {
        Some(json) => {
            builder = builder.header(header::CONTENT_TYPE, "application/json");
            Body::from(json.to_string())
        }
        None => Body::empty(),
    };
    app.clone()
        .oneshot(builder.body(body).expect("valid request"))
        .await
        .expect("the router never fails")
}

/// Reads a response into its status, headers and JSON body.
async fn read(response: Response) -> (StatusCode, HeaderMap, Value) {
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

/// Pulls the raw token out of the fragment of the sign-in link.
fn token_from(text: &str) -> String {
    let start = text
        .find("#token=")
        .expect("the link carries the token in its fragment")
        + "#token=".len();
    text[start..]
        .chars()
        .take_while(|c| c.is_ascii_alphanumeric() || *c == '-' || *c == '_')
        .collect()
}

/// Asks for a link for `email` and returns the token from the mail.
async fn request_link(harness: &Harness, email: &str, return_to: Option<&str>) -> String {
    let mut body = json!({ "email": email });
    if let Some(path) = return_to {
        body["return_to"] = json!(path);
    }
    let (status, _, _) = read(
        call(
            &harness.app,
            Method::POST,
            "/api/auth/email/magic",
            Some(body),
        )
        .await,
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    token_from(&wait_for_mail(&harness.mail).await)
}

async fn verify(app: &Router, token: &str) -> (StatusCode, HeaderMap, Value) {
    read(
        call(
            app,
            Method::POST,
            "/api/auth/email/verify",
            Some(json!({ "token": token })),
        )
        .await,
    )
    .await
}

async fn count(pool: &PgPool, sql: &str, email: &str) -> i64 {
    let (count,): (i64,) = sqlx::query_as(sql)
        .bind(email)
        .fetch_one(pool)
        .await
        .expect("count query");
    count
}

async fn existing_user(pool: &PgPool, email: &str) {
    sqlx::query(
        "INSERT INTO users (email, email_verified_at, handle, display_name)
         VALUES ($1, now(), 'known-user', 'known-user')",
    )
    .bind(email)
    .execute(pool)
    .await
    .expect("insert an existing user");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_magic_known_and_unknown_addresses_get_the_same_answer(pool: PgPool) {
    existing_user(&pool, "known@example.co").await;
    let harness = harness(&pool);

    let known = read(
        call(
            &harness.app,
            Method::POST,
            "/api/auth/email/magic",
            Some(json!({ "email": "known@example.co" })),
        )
        .await,
    )
    .await;
    let unknown = read(
        call(
            &harness.app,
            Method::POST,
            "/api/auth/email/magic",
            Some(json!({ "email": "unknown@example.co" })),
        )
        .await,
    )
    .await;

    assert_eq!(known.0, StatusCode::ACCEPTED);
    assert_eq!((known.0, known.2), (unknown.0, unknown.2));
    assert!(known.1.get(header::SET_COOKIE).is_none());
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_magic_link_signs_in_once_and_returns_the_safe_return_to(pool: PgPool) {
    let harness = harness(&pool);
    let token = request_link(&harness, "thandi@example.co", Some("/problems")).await;

    let (status, headers, body) = verify(&harness.app, &token).await;

    assert_eq!(status, StatusCode::OK);
    assert!(headers.get(header::SET_COOKIE).is_some());
    assert_eq!(body["return_to"], "/problems");
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM users WHERE email = $1",
            "thandi@example.co"
        )
        .await,
        1
    );

    let (second, _, error) = verify(&harness.app, &token).await;
    assert_eq!(second, StatusCode::BAD_REQUEST);
    assert_eq!(error["code"], "invalid_or_expired_token");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_magic_link_expires_fifteen_minutes_after_issue(pool: PgPool) {
    let harness = harness(&pool);
    let token = request_link(&harness, "late@example.co", None).await;

    harness.clock.advance(Duration::minutes(15));
    let (status, _, error) = verify(&harness.app, &token).await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(error["code"], "invalid_or_expired_token");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_magic_link_still_works_one_second_before_expiry(pool: PgPool) {
    let harness = harness(&pool);
    let token = request_link(&harness, "edge@example.co", None).await;

    harness
        .clock
        .advance(Duration::minutes(15) - Duration::seconds(1));
    let (status, _, _) = verify(&harness.app, &token).await;

    assert_eq!(status, StatusCode::OK);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_magic_concurrent_verifies_of_one_link_create_one_user(pool: PgPool) {
    let harness = harness(&pool);
    let token = request_link(&harness, "race@example.co", None).await;

    let (first, second) = tokio::join!(verify(&harness.app, &token), verify(&harness.app, &token),);

    let statuses = [first.0, second.0];
    assert_eq!(statuses.iter().filter(|s| **s == StatusCode::OK).count(), 1);
    assert_eq!(
        statuses
            .iter()
            .filter(|s| **s == StatusCode::BAD_REQUEST)
            .count(),
        1
    );
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM users WHERE email = $1",
            "race@example.co"
        )
        .await,
        1
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_magic_demo_address_gets_no_mail_and_no_token(pool: PgPool) {
    let harness = harness(&pool);

    let (status, _, body) = read(
        call(
            &harness.app,
            Method::POST,
            "/api/auth/email/magic",
            Some(json!({ "email": DEMO })),
        )
        .await,
    )
    .await;

    assert_eq!(status, StatusCode::ACCEPTED);
    assert_eq!(body, json!({ "status": "sent" }));
    assert!(harness.mail.sent().is_empty());
    assert_eq!(
        count(
            &pool,
            "SELECT count(*) FROM email_tokens WHERE email = $1",
            DEMO
        )
        .await,
        0
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_magic_get_on_the_verify_path_never_consumes_the_link(pool: PgPool) {
    let harness = harness(&pool);
    let token = request_link(&harness, "scanner@example.co", None).await;

    let get = call(
        &harness.app,
        Method::GET,
        &format!("/api/auth/email/verify?token={token}"),
        None,
    )
    .await;
    assert_eq!(get.status(), StatusCode::METHOD_NOT_ALLOWED);

    let (status, _, _) = verify(&harness.app, &token).await;
    assert_eq!(status, StatusCode::OK);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_magic_refuses_a_return_to_that_leaves_the_site(pool: PgPool) {
    let harness = harness(&pool);
    let token = request_link(&harness, "open@example.co", Some("//evil.example")).await;

    let (status, _, body) = verify(&harness.app, &token).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["return_to"], "/");
}
