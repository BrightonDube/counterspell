//! Log-capture test for the auth flows (auth T35, property P21, requirements R19.1 to R19.4 and R21.3).
//!
//! The test installs a capturing subscriber, runs the flows, and checks that no secret value
//! (the sign-in token, the session cookie value, the password, the address) appears in any
//! captured line. A sentinel line proves the capture works, so an empty log cannot pass.
//!
//! Not covered here: the OAuth verifier and access token are covered by the OAuth and provider
//! tests below; the Resend key is covered by a sender test below. Their canaries are checked here.
//!
//! Run with `DATABASE_URL` set: `cargo test -p counterspell-server --test auth_logs -- --ignored pg_`.

use std::{
    net::SocketAddr,
    sync::{Arc, Mutex, OnceLock},
    time::Duration as StdDuration,
};

use async_trait::async_trait;
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
        email::EmailAddress,
        limits::AuthLimiter,
        oauth::{
            fake::FakeProvider, github::GithubProvider, Credentials, Endpoints, OAuthProvider,
            ProviderError, ProviderId, ProviderProfile,
        },
        password::HashGate,
        AuthState,
    },
    client_ip::TrustedProxies,
    ratelimit::{Limiter, MemoryLimiter},
};
use serde_json::{json, Value};
use sqlx::PgPool;
use time::OffsetDateTime;
use tower::ServiceExt;
use url::Url;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

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

/// The one capture buffer for this test binary. Tests run in parallel, so the subscriber is
/// installed once and every test reads the same buffer. A leak from any flow fails the check.
fn shared_logs() -> &'static Capture {
    static CAPTURE: OnceLock<Capture> = OnceLock::new();
    CAPTURE.get_or_init(|| {
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
    })
}

// test-change: AUTH-EMAIL-REMOVAL
struct Harness {
    app: Router,
}

fn harness(pool: &PgPool) -> Harness {
    let clock = Arc::new(ManualClock::new(
        OffsetDateTime::from_unix_timestamp(1_800_000_000).expect("valid timestamp"),
    ));
    let limits = AuthLimiter::new(
        Limiter::Memory(Arc::new(MemoryLimiter::default())),
        &Redacted::new(vec![5_u8; 32]),
    );
    let state = AuthState::new(
        pool.clone(),
        clock.clone(),
        CookieSettings::new(false),
        limits,
        HashGate::new(4, StdDuration::from_secs(2)),
        TrustedProxies::parse("10.0.0.0/8").expect("valid trusted proxies"),
    );
    let peer = SocketAddr::from(([203, 0, 113, 9], 40_000));
    Harness {
        app: counterspell_server::auth::router(state).layer(MockConnectInfo(peer)),
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

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
// test-change: AUTH-EMAIL-REMOVAL
async fn pg_no_auth_flow_writes_a_secret_to_the_logs(pool: PgPool) {
    let logs = shared_logs();
    tracing::info!("{SENTINEL}");
    let h = harness(&pool);
    let mut secrets: Vec<String> = Vec::new();

    // Password sign-in is accepted only for the demo address; a wrong password and a
    // non-demo address both refuse the same way.
    let password = "Canary-Passphrase-7731"; // gitleaks:allow (test canary, not a credential)
    secrets.push(password.to_owned());
    for email in ["canary.demo@example.co", "canary.other@example.co"] {
        secrets.push(email.to_owned());
        let (status, _) = read(
            call(
                &h.app,
                Method::POST,
                "/api/auth/login",
                Some(json!({ "email": email, "password": password })),
                None,
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::UNAUTHORIZED, "{email} cannot sign in");
    }

    // The retired email routes do not exist.
    for path in [
        "/api/auth/email/magic",
        "/api/auth/email/verify",
        "/api/auth/register",
        "/api/auth/password/reset",
        "/api/auth/link/email",
    ] {
        let (status, _) = read(
            call(
                &h.app,
                Method::POST,
                path,
                Some(json!({ "email": "canary.retired@example.co" })),
                None,
            )
            .await,
        )
        .await;
        assert_eq!(status, StatusCode::NOT_FOUND, "{path} is retired");
    }

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

/// Canary values for the OAuth flows. They are long and unusual, so a match in the logs cannot
/// be a coincidence.
const SIGN_IN_CODE: &str = "canary-oauth-code-sign-in";
const LINK_CODE: &str = "canary-oauth-code-link";
const LINK_EMAIL: &str = "canary.oauth@example.co";

/// Wraps the fake provider and keeps every PKCE verifier it is handed. The verifiers are
/// random per flow, so the test reads them back and checks none of them reaches a log.
struct RecordingProvider {
    inner: FakeProvider,
    verifiers: Arc<Mutex<Vec<String>>>,
}

#[async_trait]
impl OAuthProvider for RecordingProvider {
    fn id(&self) -> ProviderId {
        self.inner.id()
    }

    fn authorize_url(&self, state: &str, code_challenge: &str, redirect_uri: &Url) -> Url {
        self.inner
            .authorize_url(state, code_challenge, redirect_uri)
    }

    async fn fetch_profile(
        &self,
        code: &str,
        pkce_verifier: &str,
        redirect_uri: &Url,
    ) -> Result<ProviderProfile, ProviderError> {
        self.verifiers
            .lock()
            .expect("verifier lock")
            .push(pkce_verifier.to_owned());
        self.inner
            .fetch_profile(code, pkce_verifier, redirect_uri)
            .await
    }
}

/// A router with one fake GitHub provider that accepts the sign-in code and the link code.
fn oauth_app(pool: &PgPool, verifiers: Arc<Mutex<Vec<String>>>) -> Router {
    let clock = Arc::new(ManualClock::new(
        OffsetDateTime::from_unix_timestamp(1_800_000_000).expect("valid timestamp"),
    ));
    let limits = AuthLimiter::new(
        Limiter::Memory(Arc::new(MemoryLimiter::default())),
        &Redacted::new(vec![5_u8; 32]),
    );
    let fake = FakeProvider::new(
        ProviderId::GitHub,
        vec![
            (
                SIGN_IN_CODE,
                ProviderProfile {
                    provider_user_id: "canary-github-sign-in".to_owned(),
                    label: Some("canary-label".to_owned()),
                    verified_email: Some(EmailAddress::parse(LINK_EMAIL).expect("an address")),
                },
            ),
            (
                LINK_CODE,
                ProviderProfile {
                    provider_user_id: "canary-github-link".to_owned(),
                    label: None,
                    verified_email: None,
                },
            ),
        ],
    )
    .expect("the fake URL parses");
    let github: Arc<dyn OAuthProvider> = Arc::new(RecordingProvider {
        inner: fake,
        verifiers,
    });
    let state = AuthState::new(
        pool.clone(),
        clock,
        CookieSettings::new(false),
        limits,
        HashGate::new(4, StdDuration::from_secs(2)),
        TrustedProxies::parse("10.0.0.0/8").expect("valid trusted proxies"),
    )
    .with_oauth("https://counterspell.example".to_owned(), vec![github]);
    let peer = SocketAddr::from(([203, 0, 113, 9], 40_000));
    counterspell_server::auth::router(state).layer(MockConnectInfo(peer))
}

/// Sends a GET with an optional raw `Cookie` header.
async fn get_with_cookies(app: &Router, uri: &str, cookies: Option<String>) -> Response {
    let mut builder = Request::builder().method(Method::GET).uri(uri);
    if let Some(value) = cookies {
        builder = builder.header(header::COOKIE, value);
    }
    app.clone()
        .oneshot(builder.body(Body::empty()).expect("valid request"))
        .await
        .expect("the router never fails")
}

/// The value of one cookie that a response set.
fn cookie_from(headers: &HeaderMap, name: &str) -> String {
    headers
        .get_all(header::SET_COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .find_map(|line| {
            let pair = line.split(';').next()?;
            let (key, value) = pair.split_once('=')?;
            (key == name && !value.is_empty()).then(|| value.to_owned())
        })
        .unwrap_or_else(|| panic!("a {name} cookie was set"))
}

/// The `Location` header of a redirect.
fn location_of(headers: &HeaderMap) -> String {
    headers
        .get(header::LOCATION)
        .and_then(|value| value.to_str().ok())
        .expect("a redirect with a Location header")
        .to_owned()
}

/// The `state` value inside an authorize URL.
fn state_in(authorize_url: &str) -> String {
    Url::parse(authorize_url)
        .expect("an absolute authorize URL")
        .query_pairs()
        .find(|(key, _)| key == "state")
        .map(|(_, value)| value.into_owned())
        .expect("the authorize URL carries state")
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_oauth_sign_in_and_link_write_no_secret_to_the_logs(pool: PgPool) {
    let logs = shared_logs();
    tracing::info!("{SENTINEL}");
    let verifiers = Arc::new(Mutex::new(Vec::new()));
    let app = oauth_app(&pool, verifiers.clone());
    let mut secrets: Vec<String> = vec![
        SIGN_IN_CODE.to_owned(),
        LINK_CODE.to_owned(),
        LINK_EMAIL.to_owned(),
    ];

    // Sign-in: start (sets the binding cookie), then the callback with the code.
    let start = get_with_cookies(&app, "/api/auth/oauth/github/start?return_to=/", None).await;
    assert_eq!(start.status(), StatusCode::SEE_OTHER);
    let start_headers = start.headers().clone();
    let sign_in_state = state_in(&location_of(&start_headers));
    let binding = cookie_from(&start_headers, "cs_oauth");
    secrets.push(sign_in_state.clone());
    secrets.push(binding.clone());

    let callback = get_with_cookies(
        &app,
        &format!("/api/auth/oauth/github/callback?code={SIGN_IN_CODE}&state={sign_in_state}"),
        Some(format!("cs_oauth={binding}")),
    )
    .await;
    assert_eq!(
        callback.status(),
        StatusCode::SEE_OTHER,
        "the sign-in succeeds"
    );
    let session = cookie_from(callback.headers(), "cs_session");
    secrets.push(session.clone());

    // Link GitHub to the signed-in account. A link flow is bound to the session, not a cookie.
    let link_start = call(
        &app,
        Method::POST,
        "/api/auth/link/github/start",
        None,
        Some(&session),
    )
    .await;
    assert_eq!(link_start.status(), StatusCode::OK);
    let body = to_bytes(link_start.into_body(), 64 * 1024)
        .await
        .expect("a small body");
    let link_json: Value = serde_json::from_slice(&body).expect("a JSON body");
    let link_state = state_in(
        link_json["authorize_url"]
            .as_str()
            .expect("an authorize URL"),
    );
    secrets.push(link_state.clone());

    let link_callback = get_with_cookies(
        &app,
        &format!("/api/auth/oauth/github/callback?code={LINK_CODE}&state={link_state}"),
        Some(format!("cs_session={session}")),
    )
    .await;
    // The sign-in already linked this GitHub identity, so the link is refused. The refusal still
    // runs the whole link callback, including the provider call, which is what this test checks.
    assert!(
        location_of(link_callback.headers()).contains("error=provider_already_linked"),
        "the repeat link is refused, got {}",
        location_of(link_callback.headers())
    );

    // Every verifier the provider received is a secret too. The check is only meaningful if the
    // provider was actually called, so an empty list fails the test.
    let recorded = verifiers.lock().expect("verifier lock").clone();
    assert_eq!(
        recorded.len(),
        2,
        "both the sign-in and the link reached the provider"
    );
    secrets.extend(recorded);

    let captured = logs.text();
    assert!(
        captured.contains(SENTINEL),
        "the capture must see events, or the check proves nothing"
    );
    for secret in &secrets {
        assert!(
            !captured.contains(secret.as_str()),
            "an OAuth value reached the logs: {secret}"
        );
    }
}

/// Canary values for the GitHub adapter. The client secret and the access token must never be
/// logged, and neither must the PKCE verifier or the code.
const GITHUB_CLIENT_SECRET: &str = "canary-github-client-secret-9d2e";
const GITHUB_ACCESS_TOKEN: &str = "canary-github-access-token-7f3a";
const GITHUB_VERIFIER: &str = "canary-github-pkce-verifier-4b1c";
const GITHUB_CODE: &str = "canary-github-code-61aa";
const GITHUB_MAIL: &str = "canary.github@example.co";

#[tokio::test]
async fn auth_logs_github_exchange_keeps_secret_and_token_out_of_logs() {
    let logs = shared_logs();
    tracing::info!("{SENTINEL}");
    let server = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/login/oauth/access_token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "access_token": GITHUB_ACCESS_TOKEN,
            "token_type": "bearer",
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/user"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!({
            "id": 4242,
            "login": "canary-user",
        })))
        .mount(&server)
        .await;
    Mock::given(method("GET"))
        .and(path("/user/emails"))
        .respond_with(ResponseTemplate::new(200).set_body_json(json!([{
            "email": GITHUB_MAIL,
            "primary": true,
            "verified": true,
        }])))
        .mount(&server)
        .await;

    let base = server.uri();
    let provider = GithubProvider::new(
        Credentials {
            client_id: "canary-client-id".to_owned(),
            client_secret: Redacted::new(GITHUB_CLIENT_SECRET.to_owned()),
        },
        Endpoints {
            authorize: Url::parse(&format!("{base}/login/oauth/authorize")).expect("url"),
            token: Url::parse(&format!("{base}/login/oauth/access_token")).expect("url"),
        },
        Url::parse(&format!("{base}/user")).expect("url"),
        Url::parse(&format!("{base}/user/emails")).expect("url"),
    )
    .expect("the adapter builds");
    let redirect =
        Url::parse("https://counterspell.example/api/auth/oauth/github/callback").expect("url");

    let profile = provider
        .fetch_profile(GITHUB_CODE, GITHUB_VERIFIER, &redirect)
        .await
        .expect("the exchange succeeds");
    assert!(
        profile.verified_email.is_some(),
        "the primary email came through"
    );

    let captured = logs.text();
    assert!(captured.contains(SENTINEL), "the capture must see events");
    for secret in [
        GITHUB_CLIENT_SECRET,
        GITHUB_ACCESS_TOKEN,
        GITHUB_VERIFIER,
        GITHUB_CODE,
        GITHUB_MAIL,
    ] {
        assert!(
            !captured.contains(secret),
            "a GitHub value reached the logs: {secret}"
        );
    }
}
