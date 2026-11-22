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
    /// The password does not meet the policy.
    WeakPassword,
    /// The request body is not valid JSON for this route.
    InvalidJson,
    /// The two password fields do not match.
    ConfirmationMismatch,
    /// There is no valid session.
    Unauthenticated,
    /// Password sign-in failed. The same answer for every cause.
    InvalidCredentials,
    /// The authenticator or recovery code was refused.
    InvalidSecurityCode,
    /// The challenge is absent, expired, or already used.
    SecurityChallengeExpired,
    /// A recent session is required to manage credentials.
    RecentSignInRequired,
    /// The security feature is unavailable on this origin or deployment.
    SecurityUnavailable,
    /// The requested credential change conflicts with the account state.
    SecurityConflict,
    /// Removing this passkey would leave no browser sign-in method.
    LastSignInMethod,
    /// The account already has the maximum number of passkeys.
    PasskeyLimit,
    /// Passkey verification failed.
    PasskeyFailed,
    /// The request's `Origin` or `Sec-Fetch-Site` is not this site.
    CsrfOriginMismatch,
    /// The demo account may not do this.
    DemoAccountRestricted,
    /// The provider has no credentials configured.
    ProviderDisabled,
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
            Self::WeakPassword
            | Self::InvalidJson
            | Self::ConfirmationMismatch
            | Self::InvalidSecurityCode
            | Self::SecurityChallengeExpired
            | Self::PasskeyFailed => StatusCode::BAD_REQUEST,
            Self::Unauthenticated | Self::InvalidCredentials => StatusCode::UNAUTHORIZED,
            Self::CsrfOriginMismatch | Self::DemoAccountRestricted | Self::RecentSignInRequired => {
                StatusCode::FORBIDDEN
            }
            Self::ProviderDisabled => StatusCode::NOT_FOUND,
            Self::AccountExists
            | Self::RunsActive
            | Self::SecurityConflict
            | Self::LastSignInMethod
            | Self::PasskeyLimit => StatusCode::CONFLICT,
            Self::BodyTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
            Self::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::RateLimited { .. } => StatusCode::TOO_MANY_REQUESTS,
            Self::Busy | Self::SecurityUnavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    /// The stable, machine-readable code, for example `invalid_or_expired_token`.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::WeakPassword => "weak_password",
            Self::InvalidJson => "invalid_json",
            Self::ConfirmationMismatch => "confirmation_mismatch",
            Self::Unauthenticated => "unauthenticated",
            Self::InvalidCredentials => "invalid_credentials",
            Self::InvalidSecurityCode => "invalid_security_code",
            Self::SecurityChallengeExpired => "security_challenge_expired",
            Self::RecentSignInRequired => "recent_sign_in_required",
            Self::SecurityUnavailable => "security_unavailable",
            Self::SecurityConflict => "security_conflict",
            Self::LastSignInMethod => "last_sign_in_method",
            Self::PasskeyLimit => "passkey_limit",
            Self::PasskeyFailed => "passkey_failed",
            Self::CsrfOriginMismatch => "csrf_origin_mismatch",
            Self::DemoAccountRestricted => "demo_account_restricted",
            Self::ProviderDisabled => "provider_disabled",
            Self::AccountExists => "account_exists",
            Self::RunsActive => "runs_active",
            Self::BodyTooLarge => "body_too_large",
            Self::UnsupportedMediaType => "unsupported_media_type",
            Self::RateLimited { .. } => "rate_limited",
            Self::Busy => "busy",
            Self::Internal => "internal_error",
        }
    }

    /// A short, fixed sentence that is safe to show a person.
    #[must_use]
    pub fn message(&self) -> &'static str {
        match self {
            Self::WeakPassword => "Choose a password of 10 to 128 characters.",
            Self::InvalidJson => "The request could not be read.",
            Self::ConfirmationMismatch => "The two passwords do not match.",
            Self::Unauthenticated => "Sign in to continue.",
            Self::InvalidCredentials => "The email or password is not correct.",
            Self::InvalidSecurityCode => {
                "That code could not be verified. Try a fresh code or an unused recovery code."
            }
            Self::SecurityChallengeExpired => "This security check has expired. Start it again.",
            Self::RecentSignInRequired => "Sign in again before changing your security settings.",
            Self::SecurityUnavailable => "This security feature is temporarily unavailable.",
            Self::SecurityConflict => {
                "This change is unavailable with your current sign-in methods."
            }
            Self::PasskeyLimit => "Remove a passkey before adding another.",
            Self::LastSignInMethod => {
                "Add another passkey or connect a sign-in provider before removing this one."
            }
            Self::PasskeyFailed => {
                "The passkey could not be verified. Try again or use another sign-in method."
            }
            Self::CsrfOriginMismatch => "This request did not come from Counterspell.",
            Self::DemoAccountRestricted => "The demo account cannot do this.",
            Self::ProviderDisabled => "This sign-in method is not available.",
            Self::AccountExists => "An account already exists for that email address.",
            Self::RunsActive => "Wait for your running submissions to finish first.",
            Self::BodyTooLarge => "The request is too large.",
            Self::UnsupportedMediaType => "Send the request as JSON.",
            Self::RateLimited { .. } => "Too many attempts. Wait a moment and try again.",
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

/// Turns a database failure into [`AuthError::Internal`]. The SQL error is logged, never returned.
#[must_use]
#[allow(
    clippy::needless_pass_by_value,
    reason = "used as map_err(database), which passes the error by value"
)]
pub fn database(error: sqlx::Error) -> AuthError {
    tracing::error!(error = %error, "auth database query failed");
    AuthError::Internal
}

/// Whether a database error is a unique-constraint violation, such as a duplicate email address.
#[must_use]
pub fn is_unique_violation(error: &sqlx::Error) -> bool {
    matches!(error, sqlx::Error::Database(db) if db.is_unique_violation())
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;

    // test-change: ACC-1
    fn every_variant() -> Vec<AuthError> {
        vec![
            AuthError::WeakPassword,
            AuthError::WeakPassword,
            AuthError::InvalidJson,
            AuthError::ConfirmationMismatch,
            AuthError::Unauthenticated,
            AuthError::InvalidCredentials,
            AuthError::InvalidSecurityCode,
            AuthError::SecurityChallengeExpired,
            AuthError::RecentSignInRequired,
            AuthError::SecurityUnavailable,
            AuthError::SecurityConflict,
            AuthError::LastSignInMethod,
            AuthError::PasskeyLimit,
            AuthError::PasskeyFailed,
            AuthError::CsrfOriginMismatch,
            AuthError::DemoAccountRestricted,
            AuthError::ProviderDisabled,
            AuthError::AccountExists,
            AuthError::RunsActive,
            AuthError::BodyTooLarge,
            AuthError::UnsupportedMediaType,
            AuthError::RateLimited {
                retry_after_seconds: 42,
            },
            AuthError::Busy,
            AuthError::Internal,
        ]
    }

    #[test]
    fn each_error_has_the_status_and_code_of_the_design_table() {
        // test-change: ACC-1
        let expected = [
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
    // test-change: ACC-1
    async fn the_response_body_has_only_code_and_message() {
        let response = AuthError::InvalidCredentials.into_response();
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);
        let bytes = to_bytes(response.into_body(), 4096).await.expect("body");
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        assert_eq!(json["code"], "invalid_credentials");
        assert_eq!(json["message"], "The email or password is not correct.");
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
