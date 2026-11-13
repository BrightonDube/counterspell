//! Tests for the Origin guard, the JSON and body-size layer, and the response headers (auth T16,
//! requirements R13.1 to R13.6).
//!
//! A counter handler sits behind the guard. A refused request must leave the counter unchanged,
//! which proves the handler never ran.

use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

use axum::{
    body::{to_bytes, Body},
    extract::State,
    http::{header, Method, Request, StatusCode},
    response::Redirect,
    routing::{get, post},
    Router,
};
use counterspell_server::auth::csrf::{
    apply_api_hardening, json_body_guard_layer, origin_allowed, WebOrigin,
};
use proptest::prelude::*;
use tower::ServiceExt;

const BASE: &str = "https://counterspell.example";

async fn count(State(counter): State<Arc<AtomicUsize>>) -> &'static str {
    counter.fetch_add(1, Ordering::SeqCst);
    "counted"
}

async fn redirect() -> Redirect {
    Redirect::to("/somewhere")
}

/// A probe router with the full `/api` hardening applied. Every route counts its calls.
fn hardened(counter: Arc<AtomicUsize>) -> Router {
    let origin = Arc::new(WebOrigin::from_base_url(BASE).expect("valid base URL"));
    let probe = Router::new()
        .route("/api/probe", get(count).post(count))
        .route("/api/redirect", get(redirect))
        .with_state(counter);
    apply_api_hardening(probe, origin)
}

async fn send(router: &Router, request: Request<Body>) -> (StatusCode, axum::http::HeaderMap) {
    let response = router.clone().oneshot(request).await.expect("response");
    (response.status(), response.headers().clone())
}

fn post_with(headers: &[(&str, &str)]) -> Request<Body> {
    let mut builder = Request::builder()
        .method(Method::POST)
        .uri("/api/probe")
        .header(header::CONTENT_TYPE, "application/json");
    for (name, value) in headers {
        builder = builder.header(*name, *value);
    }
    builder.body(Body::from("{}")).expect("request")
}

#[tokio::test]
async fn a_post_from_the_site_origin_reaches_the_handler() {
    let counter = Arc::new(AtomicUsize::new(0));
    let router = hardened(counter.clone());
    let (status, _) = send(&router, post_with(&[("origin", BASE)])).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn a_post_from_another_origin_is_refused_before_the_handler() {
    let counter = Arc::new(AtomicUsize::new(0));
    let router = hardened(counter.clone());
    let (status, _) = send(
        &router,
        post_with(&[
            ("origin", "https://evil.example"),
            ("cookie", "cs_session=abc"),
        ]),
    )
    .await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(counter.load(Ordering::SeqCst), 0, "the handler never ran");
}

#[tokio::test]
async fn a_cross_site_post_without_origin_is_refused() {
    let counter = Arc::new(AtomicUsize::new(0));
    let router = hardened(counter.clone());
    let (status, _) = send(&router, post_with(&[("sec-fetch-site", "cross-site")])).await;
    assert_eq!(status, StatusCode::FORBIDDEN);
    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn a_post_with_a_cookie_and_no_origin_headers_is_refused() {
    let counter = Arc::new(AtomicUsize::new(0));
    let router = hardened(counter.clone());
    let (status, _) = send(&router, post_with(&[("cookie", "cs_session=abc")])).await;
    assert_eq!(
        status,
        StatusCode::FORBIDDEN,
        "a cookie with no origin proof is refused"
    );
    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn a_get_is_never_blocked_by_the_origin_guard() {
    let counter = Arc::new(AtomicUsize::new(0));
    let router = hardened(counter.clone());
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/probe")
        .header("origin", "https://evil.example")
        .header("cookie", "cs_session=abc")
        .body(Body::empty())
        .expect("request");
    let (status, _) = send(&router, request).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

#[tokio::test]
async fn responses_are_never_cached() {
    let counter = Arc::new(AtomicUsize::new(0));
    let router = hardened(counter);
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/probe")
        .body(Body::empty())
        .expect("request");
    let (_, headers) = send(&router, request).await;
    assert_eq!(
        headers
            .get(header::CACHE_CONTROL)
            .and_then(|v| v.to_str().ok()),
        Some("no-store")
    );
}

#[tokio::test]
async fn a_redirect_does_not_leak_the_referrer() {
    let counter = Arc::new(AtomicUsize::new(0));
    let router = hardened(counter);
    let request = Request::builder()
        .method(Method::GET)
        .uri("/api/redirect")
        .body(Body::empty())
        .expect("request");
    let (status, headers) = send(&router, request).await;
    assert!(status.is_redirection());
    assert_eq!(
        headers.get("referrer-policy").and_then(|v| v.to_str().ok()),
        Some("no-referrer")
    );
}

#[tokio::test]
async fn a_body_that_is_not_json_is_refused_with_415() {
    let counter = Arc::new(AtomicUsize::new(0));
    let probe = Router::new()
        .route("/api/probe", post(count))
        .with_state(counter.clone());
    let router = json_body_guard_layer(probe);
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/probe")
        .header(header::CONTENT_TYPE, "text/plain")
        .body(Body::from("hello"))
        .expect("request");
    let (status, _) = send(&router, request).await;
    assert_eq!(status, StatusCode::UNSUPPORTED_MEDIA_TYPE);
    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn a_body_over_sixteen_kib_is_refused_with_413() {
    let counter = Arc::new(AtomicUsize::new(0));
    let probe = Router::new()
        .route("/api/probe", post(count))
        .with_state(counter.clone());
    let router = json_body_guard_layer(probe);
    let big = "x".repeat(17 * 1024);
    let request = Request::builder()
        .method(Method::POST)
        .uri("/api/probe")
        .header(header::CONTENT_TYPE, "application/json")
        .body(Body::from(format!("{{\"v\":\"{big}\"}}")))
        .expect("request");
    let (status, _) = send(&router, request).await;
    assert_eq!(status, StatusCode::PAYLOAD_TOO_LARGE);
    assert_eq!(counter.load(Ordering::SeqCst), 0);
}

#[tokio::test]
async fn a_small_json_body_passes_the_body_guard() {
    let counter = Arc::new(AtomicUsize::new(0));
    let probe = Router::new()
        .route("/api/probe", post(count))
        .with_state(counter.clone());
    let router = json_body_guard_layer(probe);
    let (status, body) = {
        let response = router
            .clone()
            .oneshot(
                Request::builder()
                    .method(Method::POST)
                    .uri("/api/probe")
                    .header(header::CONTENT_TYPE, "application/json")
                    .body(Body::from("{}"))
                    .expect("request"),
            )
            .await
            .expect("response");
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 1024).await.expect("body");
        (status, bytes)
    };
    assert_eq!(status, StatusCode::OK);
    assert_eq!(&body[..], b"counted");
    assert_eq!(counter.load(Ordering::SeqCst), 1);
}

/// The guard's rule, written out independently for the property test.
fn expected(method: &Method, origin: Option<&str>, site: Option<&str>, cookie: bool) -> bool {
    let mutating = matches!(
        *method,
        Method::POST | Method::PUT | Method::PATCH | Method::DELETE
    );
    if !mutating {
        return true;
    }
    match (origin, site) {
        (Some(value), _) => value == BASE,
        (None, Some(value)) => value == "same-origin" || value == "none",
        (None, None) => !cookie,
    }
}

fn method_strategy() -> impl Strategy<Value = Method> {
    prop_oneof![
        Just(Method::GET),
        Just(Method::HEAD),
        Just(Method::POST),
        Just(Method::PUT),
        Just(Method::PATCH),
        Just(Method::DELETE),
    ]
}

proptest! {
    #[test]
    fn origin_rule_matches_the_written_rule(
        method in method_strategy(),
        origin in prop_oneof![Just(None), Just(Some(BASE)), Just(Some("https://evil.example")), Just(Some("null"))],
        site in prop_oneof![
            Just(None),
            Just(Some("same-origin")),
            Just(Some("same-site")),
            Just(Some("cross-site")),
            Just(Some("none")),
        ],
        cookie in any::<bool>(),
    ) {
        prop_assert_eq!(
            origin_allowed(&method, origin, site, cookie, BASE),
            expected(&method, origin, site, cookie)
        );
    }

    #[test]
    fn a_wrong_origin_is_always_refused_for_writes(
        method in prop_oneof![Just(Method::POST), Just(Method::PUT), Just(Method::PATCH), Just(Method::DELETE)],
        site in prop_oneof![Just(None), Just(Some("same-origin")), Just(Some("none"))],
        cookie in any::<bool>(),
    ) {
        prop_assert!(!origin_allowed(&method, Some("https://evil.example"), site, cookie, BASE));
    }
}
