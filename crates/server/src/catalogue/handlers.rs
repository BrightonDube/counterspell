//! HTTP handlers for the public catalogue (problem-catalogue C30 and C31).

use axum::{
    extract::{Query, State},
    http::HeaderMap,
    Json,
};
use serde::Deserialize;

use super::{
    dto::ProblemListResponse,
    read::{self, ProblemFilter},
};
use crate::{error::ApiError, AppState};

/// The query string of `GET /api/problems`. Unknown keys are ignored.
#[derive(Debug, Default, Deserialize)]
pub struct ListQuery {
    /// A track slug.
    pub track: Option<String>,
    /// A concept slug.
    pub concept: Option<String>,
    /// `easy`, `medium` or `hard`.
    pub difficulty: Option<String>,
}

/// `GET /api/problems`. Anonymous callers get the list with `status: null`. A signed-in caller also
/// gets their own progress on each problem.
///
/// # Errors
///
/// Returns `400 invalid_filter` for a malformed filter, and `500` when the database query fails.
pub async fn list_problems(
    State(state): State<AppState>,
    headers: HeaderMap,
    Query(query): Query<ListQuery>,
) -> Result<Json<ProblemListResponse>, ApiError> {
    let filter = ProblemFilter::parse(
        query.track.as_deref(),
        query.concept.as_deref(),
        query.difficulty.as_deref(),
    )?;
    let viewer = state
        .user_resolver
        .resolve(&headers)
        .await
        .map(counterspell_common::ids::UserId::as_uuid);
    let problems = read::list_problems(&state.pool, viewer, &filter)
        .await
        .map_err(ApiError::internal)?;
    Ok(Json(ProblemListResponse { problems }))
}
