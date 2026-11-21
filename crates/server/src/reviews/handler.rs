//! Review endpoints (dashboard-fsrs T15 and T16).

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use serde::Deserialize;
use serde_json::Value;
use time::OffsetDateTime;

use super::{
    query::{read_due_reviews, review_concept, ReviewWriteError},
    types::{DueReview, DueReviewsResponse},
    ReviewGrade,
};
use crate::{error::ApiError, seams::CurrentUser, AppState};

/// Returns the caller's due cards and a labelled challenge for each concept.
///
/// # Errors
///
/// Returns `401 unauthenticated` without a session and `503 reviews_unavailable` when the database
/// read fails. No browser-supplied user id is accepted.
pub async fn get_due_reviews(
    State(state): State<AppState>,
    CurrentUser(owner): CurrentUser,
) -> Result<Json<DueReviewsResponse>, ApiError> {
    let reviews = read_due_reviews(&state.pool, owner, OffsetDateTime::now_utc())
        .await
        .map_err(|detail| {
            tracing::error!(error = %detail, "due review read failed");
            ApiError::new(
                axum::http::StatusCode::SERVICE_UNAVAILABLE,
                "reviews_unavailable",
                "Your reviews are unavailable right now. Try again in a moment.",
            )
        })?;
    Ok(Json(DueReviewsResponse { reviews }))
}

/// The body accepted by `POST /api/reviews/{concept}`. `user_id` is rejected, never trusted.
#[derive(Debug, Deserialize)]
pub struct ReviewRequest {
    /// One of `again`, `hard`, `good`, or `easy`.
    pub grade: String,
    /// The last card version the caller read. Omitting it allows one current-owner write.
    pub version: Option<i64>,
    /// A browser-supplied owner selector. Any value is an error.
    pub user_id: Option<Value>,
}

/// Atomically grades the caller's card for `concept` and returns the updated card and challenge.
///
/// # Errors
///
/// Returns `400` for a bad grade/version or supplied `user_id`, `404` for an unknown concept,
/// `409 review_conflict` for a stale version, `500 scheduler_invariant_failed` when the FSRS
/// adapter rejects persisted or calculated data, `503 reviews_unavailable` for database errors,
/// and `401 unauthenticated` without a session.
pub async fn post_review(
    State(state): State<AppState>,
    CurrentUser(owner): CurrentUser,
    Path(concept): Path<String>,
    Json(request): Json<ReviewRequest>,
) -> Result<Json<DueReview>, ApiError> {
    if request.user_id.is_some() {
        return Err(ApiError::bad_request(
            "unexpected_user_id",
            "Review ownership comes from your session.",
        ));
    }
    if request.version.is_some_and(|version| version < 0) {
        return Err(ApiError::bad_request(
            "invalid_version",
            "Review version must not be negative.",
        ));
    }
    let grade = ReviewGrade::parse(&request.grade).map_err(|_| {
        ApiError::bad_request("invalid_grade", "Grade must be again, hard, good, or easy.")
    })?;

    review_concept(
        &state.pool,
        owner,
        &concept,
        grade,
        request.version,
        OffsetDateTime::now_utc(),
    )
    .await
    .map(Json)
    .map_err(review_write_error)
}

fn review_write_error(error: ReviewWriteError) -> ApiError {
    match error {
        ReviewWriteError::Conflict => ApiError::new(
            StatusCode::CONFLICT,
            "review_conflict",
            "This review changed since you loaded it. Refresh and try again.",
        ),
        ReviewWriteError::UnknownConcept => ApiError::new(
            StatusCode::NOT_FOUND,
            "unknown_concept",
            "That concept is not available for review.",
        ),
        ReviewWriteError::Scheduler(detail) => {
            tracing::error!(error = %detail, "review scheduler invariant failed");
            ApiError::new(
                StatusCode::INTERNAL_SERVER_ERROR,
                "scheduler_invariant_failed",
                "This review could not be scheduled safely.",
            )
        }
        ReviewWriteError::Database(detail) => {
            tracing::error!(error = %detail, "review write failed");
            ApiError::new(
                StatusCode::SERVICE_UNAVAILABLE,
                "reviews_unavailable",
                "Your reviews are unavailable right now. Try again in a moment.",
            )
        }
    }
}
