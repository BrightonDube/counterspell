//! Log-capture test for the auth flows (auth T35, property P21, requirements R19.1 to R19.4 and R21.3).
//!
//! The test installs a capturing subscriber, runs the flows, and checks that no secret value
//! (the sign-in token, the session cookie value, the password, the address) appears in any
//! captured line. A sentinel line proves the capture works, so an empty log cannot pass.
//!
//! Not covered here: OAuth `code`, `state`, verifier and access token, and the Resend key. Those
//! flows need the provider adapters and a mail API stub, which are checked by their own tests.
//!
//! Run with `DATABASE_URL` set: `cargo test -p counterspell-server --test auth_logs -- --ignored pg_`.

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::Duration as StdDuration,
};

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
use time::OffsetDateTime;
use tower::ServiceExt;

const SENTINEL: &str = "log-capture-sentinel";

/// Collects everything the subscriber writes. Clones share one buffer.
#[derive(Clone, Default)]
struct Capture(Arc<Mutex<Vec<u8>>>);

impl Capture {
    fn text(&self) -> String {
        String::from_utf8_lossy(&self.0.lock().expect("capture lock")).into_owned()
    }
}

impl std::io::Write for Capture {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.lock().expect("capture lock").extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

/// Installs the capturing subscriber for this test binary. The binary has one test, so one global
/// subscriber is enough, and it also sees events written from blocking threads.
fn capture_logs() -> Capture {
    let capture = Capture::default();
    let writer = capture.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_max_level(tracing::Level::TRACE)
        .with_writer(move || writer.clone())
        .finish();
    tracing::subscriber::set_global_default(subscriber)
        .expect("one global subscriber per test binary");
    capture
}

struct Harness {
    app: Router,
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
        demo_email: Some(EmailAddress::parse("demo@example.co").expect("valid demo address")),
    });
    let peer = SocketAddr::from(([203, 0, 113, 9], 40_000));
    Harness {
        app: counterspell_server::auth::router(state).layer(MockConnectInfo(peer)),
        mail,
    }
}

/// Sends one request, with an optional cookie header, and returns the response.
async fn call(
    app: &Router,
    method: Method,
    path: &str,
    body: Option<Value>,
    cookie: Option<&str>,
) -> Response {
    let mut builder = Request::builder().method(method).uri(path);
    if let Some(value) = cookie {
        builder = builder.header(header::COOKIE, format!("cs_session={value}"));
    }
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

async fn read(response: Response) -> (StatusCode, HeaderMap) {
    let status = response.status();
    let headers = response.headers().clone();
    to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("a small body");
    (status, headers)
}

/// Waits until the mail count grows past `before` and returns the newest message's text.
async fn next_mail(mail: &MemorySender, before: usize) -> String {
    for _ in 0..300 {
        let sent = mail.sent();
        if sent.len() > before {
            return sent.last().expect("a message").text.clone();
        }
        tokio::time::sleep(StdDuration::from_millis(10)).await;
    }
    panic!("no mail arrived");
}

/// Pulls the raw token out of the fragment of a link.
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

/// The session cookie value a response set.
fn session_value(headers: &HeaderMap) -> String {
    headers
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find_map(|cookie| {
            cookie
                .strip_prefix("cs_session=")
                .map(|rest| rest.split(';').next().unwrap_or_default().to_owned())
        })
        .expect("a session cookie was set")
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_no_auth_flow_writes_a_secret_to_the_logs(pool: PgPool) {
    let logs = capture_logs();
    tracing::info!("{SENTINEL}");
    let h = harness(&pool);
    let mut secrets: Vec<String> = Vec::new();

    // Magic link: request, then verify.
    let magic_email = "canary.magic@example.co";
    secrets.push(magic_email.to_owned());
    let before = h.mail.sent().len();
    let (status, _) = read(
        call(
            &h.app,
            Method::POST,
            "/api/auth/email/magic",
            Some(json!({ "email": magic_email })),
            None,
        )
        .await,
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let magic_token = token_from(&next_mail(&h.mail, before).await);
    secrets.push(magic_token.clone());
    let (status, headers) = read(
        call(
            &h.app,
            Method::POST,
            "/api/auth/email/verify",
            Some(json!({ "token": magic_token })),
            None,
        )
        .await,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "the magic link signs in");
    secrets.push(session_value(&headers));

    // Password: register (mail proves the address), verify, then sign in again.
    let password_email = "canary.password@example.co";
    let password = "Canary-Passphrase-7731"; // gitleaks:allow (test canary, not a credential)
    secrets.push(password_email.to_owned());
    secrets.push(password.to_owned());
    let before = h.mail.sent().len();
    let (status, _) = read(
        call(
            &h.app,
            Method::POST,
            "/api/auth/register",
            Some(json!({ "email": password_email, "password": password })),
            None,
        )
        .await,
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let register_token = token_from(&next_mail(&h.mail, before).await);
    secrets.push(register_token.clone());
    let (status, _) = read(
        call(
            &h.app,
            Method::POST,
            "/api/auth/email/verify",
            Some(json!({ "token": register_token })),
            None,
        )
        .await,
    )
    .await;
    assert!(status.is_success(), "the registration verifies: {status}");

    let (status, headers) = read(
        call(
            &h.app,
            Method::POST,
            "/api/auth/login",
            Some(json!({ "email": password_email, "password": password })),
            None,
        )
        .await,
    )
    .await;
    assert_eq!(status, StatusCode::OK, "the password signs in");
    let password_session = session_value(&headers);
    secrets.push(password_session.clone());

    // Password reset request for the existing account.
    let before = h.mail.sent().len();
    let (status, _) = read(
        call(
            &h.app,
            Method::POST,
            "/api/auth/password/reset",
            Some(json!({ "email": password_email })),
            None,
        )
        .await,
    )
    .await;
    assert_eq!(status, StatusCode::ACCEPTED);
    let reset_token = token_from(&next_mail(&h.mail, before).await);
    secrets.push(reset_token);

    // Logout everywhere, with the signed-in cookie.
    let (status, _) = read(
        call(
            &h.app,
            Method::POST,
            "/api/auth/logout-all",
            None,
            Some(&password_session),
        )
        .await,
    )
    .await;
    assert_eq!(status, StatusCode::NO_CONTENT);

    let captured = logs.text();
    assert!(
        captured.contains(SENTINEL),
        "the capture must see events, or the check proves nothing"
    );
    for secret in &secrets {
        assert!(
            !captured.contains(secret.as_str()),
            "a secret value reached the logs: {secret}"
        );
    }
}
