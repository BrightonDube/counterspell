//! Dashboard error codes. Each error becomes the shared JSON envelope with a stable code and a safe message.

use std::fmt::Display;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

use crate::error::ApiError;

/// An error the dashboard can return. Database and mapping detail is logged, never sent to a client.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DashboardError {
    /// There is no valid session. Sends `401 unauthenticated`.
    Unauthenticated,
    /// PostgreSQL could not produce a complete dashboard. Sends `503 dashboard_unavailable`.
    Unavailable,
    /// A mapping or configuration failure that the learner cannot fix. Sends `500 internal_error`.
    Internal,
}

impl DashboardError {
    /// Logs why the dashboard read failed, then returns the safe `Unavailable` error.
    /// Call it at the database boundary so the SQL detail never reaches the client.
    #[must_use]
    pub fn unavailable(detail: impl Display) -> Self {
        tracing::error!(error = %detail, "dashboard read failed");
        Self::Unavailable
    }

    /// Logs an internal mapping failure, then returns the safe `Internal` error.
    #[must_use]
    pub fn internal(detail: impl Display) -> Self {
        tracing::error!(error = %detail, "dashboard mapping failed");
        Self::Internal
    }
}

impl From<DashboardError> for ApiError {
    fn from(error: DashboardError) -> Self {
        match error {
            DashboardError::Unauthenticated => ApiError::new(
                StatusCode::UNAUTHORIZED,
                "unauthenticated",
                "Sign in to view your dashboard.",
            ),
            DashboardError::Unavailable => ApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "dashboard_unavailable",
                "Your dashboard is unavailable right now. Try again in a moment.",
            ),
            DashboardError::Internal => ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal_error",
                "Something went wrong while building your dashboard.",
            ),
        }
    }
}

impl IntoResponse for DashboardError {
    fn into_response(self) -> Response {
        ApiError::from(self).into_response()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::body::to_bytes;
    use axum::http::StatusCode;

    async fn envelope(error: DashboardError) -> (StatusCode, String) {
        let response = error.into_response();
        let status = response.status();
        let bytes = to_bytes(response.into_body(), 4096).await.expect("body");
        (status, String::from_utf8(bytes.to_vec()).expect("utf8"))
    }

    #[tokio::test]
    async fn unauthenticated_is_401_with_the_stable_code() {
        let (status, body) = envelope(DashboardError::Unauthenticated).await;
        assert_eq!(status, StatusCode::UNAUTHORIZED);
        let json: serde_json::Value = serde_json::from_str(&body).expect("json");
        assert_eq!(json["code"], "unauthenticated");
    }

    #[tokio::test]
    async fn unavailable_is_503_and_never_sends_the_database_detail() {
        let error = DashboardError::unavailable("relation \"secret_projects\" does not exist");
        let (status, body) = envelope(error).await;
        assert_eq!(status, StatusCode::SERVICE_UNAVAILABLE);
        let json: serde_json::Value = serde_json::from_str(&body).expect("json");
        assert_eq!(json["code"], "dashboard_unavailable");
        assert!(
            !body.contains("secret_projects"),
            "body leaked detail: {body}"
        );
    }

    #[tokio::test]
    async fn internal_is_500_with_the_internal_error_code() {
        let (status, body) = envelope(DashboardError::internal("bad score column")).await;
        assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
        let json: serde_json::Value = serde_json::from_str(&body).expect("json");
        assert_eq!(json["code"], "internal_error");
        assert!(
            !body.contains("bad score column"),
            "body leaked detail: {body}"
        );
    }
}
