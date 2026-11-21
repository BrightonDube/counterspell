//! `GET /api/reviews/due` (dashboard-fsrs T15).

use axum::{extract::State, Json};
use time::OffsetDateTime;

use super::{query::read_due_reviews, types::DueReviewsResponse};
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
