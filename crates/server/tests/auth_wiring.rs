//! The auth routes mounted on the server router (auth T33). No database is used: the pool never
//! connects, and the requests below stop before any query.
//!
//! Run with `cargo test -p counterspell-server auth_wiring`.

use std::{collections::HashMap, net::SocketAddr, sync::Arc};

use axum::{
    body::{to_bytes, Body},
    extract::connect_info::MockConnectInfo,
    http::{header, Method, Request, StatusCode},
    Router,
};
use counterspell_server::{
    auth::{config::AuthConfig, setup::build_state},
    build_router,
    client_ip::TrustedProxies,
    config::ServerConfig,
    ratelimit::{Limiter, MemoryLimiter},
    AppState,
};
use sqlx::postgres::PgPoolOptions;
use tower::ServiceExt;

const BASE: &str = "https://counterspell.example";

fn auth_config() -> AuthConfig {
    let vars: HashMap<String, String> = [
        ("APP_ENV", "production"),
        ("PUBLIC_BASE_URL", BASE),
        ("DATABASE_URL", "postgres://server:fake-pw@127.0.0.1:1/cs"),
        ("VALKEY_URL", "redis://127.0.0.1:1"),
        ("SESSION_SECRET", "fake-session-secret-value-0123456789"),
        ("RESEND_API_KEY", "re_fake_key_value"),
        ("MAIL_FROM", "Counterspell <login@counterspell.example>"),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_owned(), value.to_owned()))
    .collect();
    AuthConfig::from_map(&vars).expect("a valid production config")
}

fn server_config() -> ServerConfig {
    let vars: HashMap<String, String> = [
        ("STATIC_DIR", "/nonexistent-static-dir"),
        ("DATABASE_URL", "postgres://server:fake-pw@127.0.0.1:1/cs"),
        ("VALKEY_URL", "redis://127.0.0.1:1"),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_owned(), value.to_owned()))
    .collect();
    ServerConfig::from_lookup(|name| vars.get(name).cloned()).expect("a valid server config")
}

fn lazy_pool() -> sqlx::PgPool {
    PgPoolOptions::new()
        .connect_lazy("postgres://server:fake-pw@127.0.0.1:1/cs")
        .expect("a lazy pool never connects")
}

/// The router with auth mounted, as `main` builds it.
fn auth_config_without_mail() -> AuthConfig {
    let vars: HashMap<String, String> = [
        ("APP_ENV", "production"),
        ("PUBLIC_BASE_URL", BASE),
        ("DATABASE_URL", "postgres://server:fake-pw@127.0.0.1:1/cs"),
        ("VALKEY_URL", "redis://127.0.0.1:1"),
        ("SESSION_SECRET", "fake-session-secret-value-0123456789"),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_owned(), value.to_owned()))
    .collect();
    AuthConfig::from_map(&vars).expect("a valid production config without mail")
}

fn app_with_config(config: &AuthConfig) -> Router {
    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    let auth = build_state(
        lazy_pool(),
        config,
        limiter.clone(),
        TrustedProxies::default(),
    )
    .expect("the auth state builds");
    let state = AppState::new(lazy_pool(), server_config(), limiter)
        .with_auth(auth, BASE)
        .expect("the public base URL is an origin");
    build_router(state).layer(MockConnectInfo(SocketAddr::from((
        [203, 0, 113, 9],
        40_000,
    ))))
}

fn app_with_auth() -> Router {
    app_with_config(&auth_config())
}

async fn send(app: &Router, request: Request<Body>) -> (StatusCode, serde_json::Value) {
    let response = app
        .clone()
        .oneshot(request)
        .await
        .expect("the router never fails");
    let status = response.status();
    let bytes = to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("a small body");
    let json = serde_json::from_slice(&bytes).unwrap_or(serde_json::Value::Null);
    (status, json)
}

#[tokio::test]
async fn auth_wiring_the_public_auth_config_is_served_without_a_session() {
    let app = app_with_auth();
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/auth/config")
        .body(Body::empty())
        .expect("valid request");

    let (status, json) = send(&app, request).await;

    assert_eq!(status, StatusCode::OK);
    assert!(
        json.get("demo").is_some(),
        "the config answer describes the demo account"
    );
    // Email sign-in and the demo account are retired: the page offers providers only, even
    // when mail and demo credentials are configured.
    assert_eq!(json["demo"]["enabled"], false);
    assert_eq!(json["email"]["magic_link"], false);
    assert_eq!(json["email"]["password"], false);
}

#[tokio::test]
async fn auth_wiring_session_status_is_public_without_a_session() {
    let app = app_with_auth();
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/auth/session")
        .body(Body::empty())
        .expect("valid request");

    let (status, json) = send(&app, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["signed_in"], false);
}

#[tokio::test]
async fn auth_wiring_me_without_a_session_is_401_json() {
    let app = app_with_auth();
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/me")
        .body(Body::empty())
        .expect("valid request");

    let (status, json) = send(&app, request).await;

    assert_eq!(status, StatusCode::UNAUTHORIZED);
    assert_eq!(json["code"], "unauthenticated");
}

#[tokio::test]
async fn auth_wiring_a_cross_site_write_is_refused_before_any_handler() {
    let app = app_with_auth();
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/logout")
        .header(header::ORIGIN, "https://evil.example")
        .body(Body::empty())
        .expect("valid request");

    let (status, json) = send(&app, request).await;

    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(json["code"], "csrf_origin_mismatch");
}

#[tokio::test]
async fn auth_wiring_config_never_advertises_email_sign_in() {
    let app = app_with_config(&auth_config_without_mail());
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/auth/config")
        .body(Body::empty())
        .expect("valid request");

    let (status, json) = send(&app, request).await;

    assert_eq!(status, StatusCode::OK);
    assert_eq!(json["email"]["magic_link"], false);
    assert_eq!(json["email"]["password"], false);
    assert_eq!(json["providers"]["github"], false);
    assert_eq!(json["providers"]["google"], false);
}

#[tokio::test]
async fn auth_wiring_magic_link_is_unavailable_when_mail_is_disabled() {
    let app = app_with_config(&auth_config_without_mail());
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/auth/email/magic")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(r#"{"email":"someone@example.co"}"#))
        .expect("valid request");

    let (status, json) = send(&app, request).await;

    assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
    assert_eq!(json["code"], "email_unavailable");
}

#[tokio::test]
async fn auth_wiring_without_auth_no_auth_route_exists() {
    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    let app = build_router(AppState::new(lazy_pool(), server_config(), limiter)).layer(
        MockConnectInfo(SocketAddr::from(([203, 0, 113, 9], 40_000))),
    );
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/me")
        .body(Body::empty())
        .expect("valid request");

    let (status, _) = send(&app, request).await;

    assert_eq!(
        status,
        StatusCode::NOT_FOUND,
        "no auth routes are mounted without auth"
    );
}
