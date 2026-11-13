//! The JSON error envelope for the API.
//!
//! Every error is `{"code": "...", "message": "..."}`. The code is stable and machine-readable.
//! The message is safe to show to a person. Rate-limit errors also send `retry_after_seconds` and a
//! `Retry-After` header. Internal errors are logged and replaced with a generic message, so no SQL,
//! path or stack trace reaches a client.

use std::fmt::Display;

use axum::{
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// An error that becomes a JSON reply with a status code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ApiError {
    status: StatusCode,
    code: &'static str,
    message: String,
    retry_after_seconds: Option<u64>,
}

#[derive(Serialize)]
struct Envelope<'a> {
    code: &'a str,
    message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    retry_after_seconds: Option<u64>,
}

impl ApiError {
    /// Builds an error with any status, code and safe message.
    #[must_use]
    pub fn new(status: StatusCode, code: &'static str, message: impl Into<String>) -> Self {
        Self {
            status,
            code,
            message: message.into(),
            retry_after_seconds: None,
        }
    }

    /// A `400` error for a request the client must fix.
    #[must_use]
    pub fn bad_request(code: &'static str, message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, message)
    }

    /// A `404` error. Use it for unknown routes and for runs the caller may not see.
    #[must_use]
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, "not_found", message)
    }

    /// A `429` error that tells the client how many seconds to wait.
    #[must_use]
    pub fn rate_limited(retry_after_seconds: u64) -> Self {
        Self::throttled(
            "rate_limited",
            format!("Too many requests. Try again in {retry_after_seconds} seconds."),
            retry_after_seconds,
        )
    }

    /// A `429` error with its own stable code, for queue caps, and a wait time.
    #[must_use]
    pub fn throttled(
        code: &'static str,
        message: impl Into<String>,
        retry_after_seconds: u64,
    ) -> Self {
        Self {
            status: StatusCode::TOO_MANY_REQUESTS,
            code,
            message: message.into(),
            retry_after_seconds: Some(retry_after_seconds),
        }
    }

    /// A `401` error for a request with no valid session.
    #[must_use]
    pub fn unauthorized() -> Self {
        Self::new(
            StatusCode::UNAUTHORIZED,
            "unauthorized",
            "Sign in to continue",
        )
    }

    /// A `500` error. The detail is logged here and never sent to the client.
    #[must_use]
    pub fn internal(detail: impl Display) -> Self {
        tracing::error!(error = %detail, "internal server error");
        Self::new(
            StatusCode::INTERNAL_SERVER_ERROR,
            "internal",
            "internal error",
        )
    }

    /// The HTTP status this error will be sent with.
    #[must_use]
    pub fn status(&self) -> StatusCode {
        self.status
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let envelope = Envelope {
            code: self.code,
            message: &self.message,
            retry_after_seconds: self.retry_after_seconds,
        };
        let mut response = (self.status, Json(envelope)).into_response();
        if let Some(seconds) = self.retry_after_seconds {
            if let Ok(value) = HeaderValue::from_str(&seconds.to_string()) {
                response.headers_mut().insert(header::RETRY_AFTER, value);
            }
        }
        response
    }
}
