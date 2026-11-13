//! The JSON error envelope: every API error is `{"code", "message"}`, with optional retry fields, and never leaks internal detail.

use axum::{
    body::to_bytes,
    http::{header, StatusCode},
    response::IntoResponse,
};
use counterspell_server::error::ApiError;
use serde_json::{json, Value};

async fn parts(error: ApiError) -> (StatusCode, axum::http::HeaderMap, Value) {
    let response = error.into_response();
    let status = response.status();
    let headers = response.headers().clone();
    let bytes = to_bytes(response.into_body(), 64 * 1024)
        .await
        .expect("body reads");
    let body: Value = serde_json::from_slice(&bytes).expect("body is JSON");
    (status, headers, body)
}

#[tokio::test]
async fn not_found_has_only_code_and_message() {
    let (status, headers, body) = parts(ApiError::not_found("API route not found")).await;

    assert_eq!(status, StatusCode::NOT_FOUND);
    assert_eq!(
        body,
        json!({ "code": "not_found", "message": "API route not found" })
    );
    assert!(headers.get(header::RETRY_AFTER).is_none());
}

#[tokio::test]
async fn rate_limit_adds_retry_after_in_header_and_body() {
    let (status, headers, body) = parts(ApiError::rate_limited(42)).await;

    assert_eq!(status, StatusCode::TOO_MANY_REQUESTS);
    assert_eq!(
        headers
            .get(header::RETRY_AFTER)
            .and_then(|v| v.to_str().ok()),
        Some("42")
    );
    assert_eq!(body["code"], "rate_limited");
    assert_eq!(body["retry_after_seconds"], 42);
}

#[tokio::test]
async fn internal_error_hides_the_underlying_detail() {
    let secret_detail = "relation \"submissions\" does not exist at /srv/app/db.rs";
    let (status, _, body) = parts(ApiError::internal(secret_detail)).await;

    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert_eq!(
        body,
        json!({ "code": "internal", "message": "internal error" })
    );
    assert!(!body.to_string().contains("submissions"));
}

#[tokio::test]
async fn bad_request_carries_its_stable_code() {
    let (status, _, body) = parts(ApiError::bad_request(
        "invalid_idempotency_key",
        "Idempotency-Key is required",
    ))
    .await;

    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert_eq!(body["code"], "invalid_idempotency_key");
}
