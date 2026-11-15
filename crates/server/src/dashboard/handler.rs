//! `GET /api/dashboard`: the signed-in learner's dashboard (dashboard-fsrs T05).
//!
//! The owner comes only from the session through `DashboardUser`. The route has no user selector,
//! so a browser cannot ask for another learner's dashboard.

use axum::{extract::State, Json};
use time::OffsetDateTime;

use super::{
    errors::DashboardError, query::read_dashboard, types::DashboardResponse, DashboardUser,
};
use crate::AppState;

/// Returns the caller's dashboard in one database read.
///
/// The streak is measured against the UTC date of this request. A failed read returns
/// `503 dashboard_unavailable` and never a partial dashboard.
///
/// # Errors
///
/// Returns `401 unauthenticated` when there is no valid session, and
/// `503 dashboard_unavailable` when the database read or its mapping fails.
pub async fn get_dashboard(
    State(state): State<AppState>,
    DashboardUser(owner): DashboardUser,
) -> Result<Json<DashboardResponse>, DashboardError> {
    let today = OffsetDateTime::now_utc().date();
    let response = read_dashboard(&state.pool, owner, today)
        .await
        .map_err(DashboardError::unavailable)?;
    Ok(Json(response))
}
