//! The signed-in learner dashboard (dashboard-fsrs).
//!
//! The dashboard is a read model: one PostgreSQL query scoped to the signed-in user produces the JSON
//! response. This module holds the response types, the dashboard error codes and the extractor that
//! names the owner. The handler and the query arrive in later tasks.

pub mod errors;
pub mod types;

use axum::{extract::FromRequestParts, http::request::Parts};
use counterspell_common::ids::UserId;

use crate::{seams::CurrentUser, AppState};
use errors::DashboardError;

/// The signed-in user who owns the dashboard. It comes only from the session, never from the URL or body.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DashboardUser(pub UserId);

impl FromRequestParts<AppState> for DashboardUser {
    type Rejection = DashboardError;

    /// Reuses the shared `CurrentUser` seam. The only change is the error code the dashboard sends.
    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        match CurrentUser::from_request_parts(parts, state).await {
            Ok(CurrentUser(user)) => Ok(Self(user)),
            Err(_) => Err(DashboardError::Unauthenticated),
        }
    }
}
