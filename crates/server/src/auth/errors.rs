//! Errors the auth module returns to clients.
//!
//! Each variant maps to one HTTP status, one stable `code` and one fixed, safe message, as in the
//! error table of the auth design. Internal failures are logged by the caller and reach the
//! client as a generic `internal_error`, so no SQL, path or secret leaves the server.

use axum::{
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

/// Something that went wrong in the auth module, with a status, code and safe message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthError {
    /// The email address is not valid.
    InvalidEmail,
    /// The password does not meet the policy.
    WeakPassword,
    /// The request body is not valid JSON for this route.
    InvalidJson,
    /// The two password fields do not match.
    ConfirmationMismatch,
    /// A token is unknown, used, expired or for another purpose. Deliberately one case.
    InvalidOrExpiredToken,
    /// There is no valid session.
    Unauthenticated,
    /// Password sign-in failed. The same answer for every cause.
    InvalidCredentials,
    /// The request's `Origin` or `Sec-Fetch-Site` is not this site.
    CsrfOriginMismatch,
    /// The demo account may not do this.
    DemoAccountRestricted,
    /// The provider has no credentials configured.
    ProviderDisabled,
    /// The account already has an email address.
    EmailAlreadySet,
    /// The email address belongs to another account.
    EmailInUse,
    /// An account with that address already exists.
    AccountExists,
    /// The account has a run that is still queued or running.
    RunsActive,
    /// The request body is over the size limit.
    BodyTooLarge,
    /// The request is not JSON.
    UnsupportedMediaType,
    /// Too many requests. The client should wait this many seconds.
    RateLimited {
        /// Seconds until the client may try again.
        retry_after_seconds: u64,
    },
    /// Sending email is switched off for now (the daily cap is reached, or the limiter is down).
    EmailUnavailable,
    /// The server is too busy to start this work. Try again later.
    Busy,
    /// Something failed inside the server. The details are logged, never returned.
    Internal,
}

#[derive(Serialize)]
struct Envelope<'a> {
    code: &'a str,
    message: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    retry_after_seconds: Option<u64>,
}

impl AuthError {
    /// The HTTP status for this error.
    #[must_use]
    pub fn status(&self) -> StatusCode {
        match self {
            Self::InvalidEmail
            | Self::WeakPassword
            | Self::InvalidJson
            | Self::ConfirmationMismatch
            | Self::InvalidOrExpiredToken => StatusCode::BAD_REQUEST,
            Self::Unauthenticated | Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::CsrfOriginMismatch | Self::DemoAccountRestricted => StatusCode::FORBIDDEN,
            Self::ProviderDisabled => StatusCode::NOT_FOUND,
            Self::EmailAlreadySet | Self::EmailInUse | Self::AccountExists | Self::RunsActive => {
                StatusCode::CONFLICT
            }
            Self::BodyTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::EmailUnavailable | Self::Busy => StatusCode::SERVICE_UNAVAILABLE,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// The stable, machine-readable code, for example `invalid_or_expired_token`.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::InvalidEmail => "invalid_email",
            Self::WeakPassword => "weak_password",
            Self::InvalidJson => "invalid_json",
            Self::ConfirmationMismatch => "confirmation_mismatch",
            Self::InvalidOrExpiredToken => "invalid_or_expired_token",
            Self::Unauthenticated => "unauthenticated",
            Self::InvalidCredentials => "invalid_credentials",
            Self::CsrfOriginMismatch => "csrf_origin_mismatch",
            Self::DemoAccountRestricted => "demo_account_restricted",
            Self::ProviderDisabled => "provider_disabled",
            Self::EmailAlreadySet => "email_already_set",
            Self::EmailInUse => "email_in_use",
            Self::AccountExists => "account_exists",
            Self::RunsActive => "runs_active",
            Self::BodyTooLarge => "body_too_large",
            Self::UnsupportedMediaType => "unsupported_media_type",
            Self::RateLimited { .. } => "rate_limited",
            Self::EmailUnavailable => "email_unavailable",
            Self::Busy => "busy",
            Self::Internal => "internal_error",
        }
    }

    /// A short, fixed sentence that is safe to show a person.
    #[must_use]
    pub fn message(&self) -> &'static str {
        match self {
            Self::InvalidEmail => "Enter a valid email address.",
            Self::WeakPassword => "Choose a password of 10 to 128 characters.",
            Self::InvalidJson => "The request could not be read.",
            Self::ConfirmationMismatch => "The two passwords do not match.",
            Self::InvalidOrExpiredToken => {
                "This link is invalid or has expired. Request a new one."
            }
            Self::Unauthenticated => "Sign in to continue.",
            Self::InvalidCredentials => "The email or password is not correct.",
            Self::CsrfOriginMismatch => "This request did not come from Counterspell.",
            Self::DemoAccountRestricted => "The demo account cannot do this.",
            Self::ProviderDisabled => "This sign-in method is not available.",
            Self::EmailAlreadySet => "This account already has an email address.",
            Self::EmailInUse => "That email address is already used by another account.",
            Self::AccountExists => "An account already exists for that email address.",
            Self::RunsActive => "Wait for your running submissions to finish first.",
            Self::BodyTooLarge => "The request is too large.",
            Self::UnsupportedMediaType => "Send the request as JSON.",
            Self::RateLimited { .. } => "Too many attempts. Wait a moment and try again.",
            Self::EmailUnavailable => {
                "Sign-in emails are paused for now. Try another sign-in method."
            }
            Self::Busy => "The server is busy. Try again in a moment.",
            Self::Internal => "Something went wrong. Try again later.",
        }
    }
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        let retry_after_seconds = match self {
            Self::RateLimited {
                retry_after_seconds,
            } => Some(retry_after_seconds),
            _ => None,
        };
        let body = Envelope {
            code: self.code(),
            message: self.message(),
            retry_after_seconds,
        };
        let mut response = (self.status(), Json(body)).into_response();
        if let Some(seconds) = retry_after_seconds {
            if let Ok(value) = HeaderValue::from_str(&seconds.to_string()) {
                response.headers_mut().insert(header::RETRY_AFTER, value);
            }
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    fn every_variant() -> Vec<AuthError> {
        vec![
            AuthError::InvalidEmail,
            AuthError::WeakPassword,
            AuthError::InvalidJson,
            AuthError::ConfirmationMismatch,
            AuthError::InvalidOrExpiredToken,
            AuthError::Unauthenticated,
            AuthError::InvalidCredentials,
            AuthError::CsrfOriginMismatch,
            AuthError::DemoAccountRestricted,
            AuthError::ProviderDisabled,
            AuthError::EmailAlreadySet,
            AuthError::EmailInUse,
            AuthError::AccountExists,
            AuthError::RunsActive,
            AuthError::BodyTooLarge,
            AuthError::UnsupportedMediaType,
            AuthError::RateLimited {
                retry_after_seconds: 42,
            },
            AuthError::EmailUnavailable,
            AuthError::Busy,
            AuthError::Internal,
        ]
    }

    #[test]
    fn each_error_has_the_status_and_code_of_the_design_table() {
        let expected = [
            (AuthError::InvalidEmail, 400, "invalid_email"),
            (
                AuthError::InvalidOrExpiredToken,
                400,
                "invalid_or_expired_token",
            ),
            (AuthError::Unauthenticated, 401, "unauthenticated"),
            (AuthError::InvalidCredentials, 401, "invalid_credentials"),
            (AuthError::CsrfOriginMismatch, 403, "csrf_origin_mismatch"),
            (
                AuthError::DemoAccountRestricted,
                403,
                "demo_account_restricted",
            ),
            (AuthError::ProviderDisabled, 404, "provider_disabled"),
            (AuthError::AccountExists, 409, "account_exists"),
            (AuthError::BodyTooLarge, 413, "body_too_large"),
            (
                AuthError::UnsupportedMediaType,
                415,
                "unsupported_media_type",
            ),
            (
                AuthError::RateLimited {
                    retry_after_seconds: 1,
                },
                429,
                "rate_limited",
            ),
            (AuthError::EmailUnavailable, 503, "email_unavailable"),
            (AuthError::Busy, 503, "busy"),
        ];
        for (error, status, code) in expected {
            assert_eq!(error.status().as_u16(), status, "status of {code}");
            assert_eq!(error.code(), code);
        }
    }

    #[test]
    fn every_code_is_stable_snake_case() {
        for error in every_variant() {
            let code = error.code();
            assert!(
                code.chars().all(|c| c.is_ascii_lowercase() || c == '_'),
                "{code}"
            );
        }
    }

    #[tokio::test]
    async fn the_response_body_has_only_code_and_message() {
        let response = AuthError::InvalidOrExpiredToken.into_response();
        assert_eq!(response.status(), StatusCode::BAD_REQUEST);
        let bytes = to_bytes(response.into_body(), 4096).await.expect("body");
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        assert_eq!(json["code"], "invalid_or_expired_token");
        assert_eq!(
            json["message"],
            "This link is invalid or has expired. Request a new one."
        );
        assert!(json.get("retry_after_seconds").is_none());
    }

    #[tokio::test]
    async fn rate_limit_sends_retry_after_and_seconds() {
        let response = AuthError::RateLimited {
            retry_after_seconds: 42,
        }
        .into_response();
        assert_eq!(response.status(), StatusCode::TOO_MANY_REQUESTS);
        assert_eq!(response.headers()[header::RETRY_AFTER], "42");
        let bytes = to_bytes(response.into_body(), 4096).await.expect("body");
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        assert_eq!(json["retry_after_seconds"], 42);
    }

    #[tokio::test]
    async fn internal_errors_hide_their_details() {
        let response = AuthError::Internal.into_response();
        assert_eq!(response.status(), StatusCode::INTERNAL_SERVER_ERROR);
        let bytes = to_bytes(response.into_body(), 4096).await.expect("body");
        let text = String::from_utf8(bytes.to_vec()).expect("utf8");
        assert!(text.contains("internal_error"));
        assert!(!text.to_lowercase().contains("sql"));
    }
}
