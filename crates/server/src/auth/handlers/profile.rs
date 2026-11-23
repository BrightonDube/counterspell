//! `PATCH /api/me/profile`: the only profile write path for settings and onboarding.

use axum::{
    extract::State,
    http::header,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;

use crate::auth::{
    capabilities::{Capabilities, Capability},
    errors::{database, AuthError},
    extract::AuthUser,
    profile::{update_profile, ProfilePatch},
    AuthState,
};

use super::me::me_view;

/// The JSON body of `PATCH /api/me/profile`.
#[derive(Debug, Deserialize)]
pub struct ProfileBody {
    /// The learner's display name.
    pub display_name: String,
    /// Optional two-letter country code. `null` clears it.
    #[serde(default)]
    pub country: Option<String>,
}

/// `PATCH /api/me/profile`. Answers with the same view as `GET /api/me`.
///
/// # Errors
///
/// Returns [`AuthError::Unauthenticated`] without a session, [`AuthError::DemoAccountRestricted`]
/// for the demo account, [`AuthError::InvalidJson`] for malformed fields, and
/// [`AuthError::Internal`] when the database fails.
pub async fn update(
    State(state): State<AuthState>,
    user: AuthUser,
    Json(body): Json<ProfileBody>,
) -> Result<Response, AuthError> {
    Capabilities::for_user(user.is_demo).require(Capability::EditProfile)?;
    let patch = ProfilePatch::try_new(&body.display_name, body.country.as_deref())?;
    update_profile(&state.pool, user.user_id, &patch, state.clock.now())
        .await
        .map_err(database)?;
    let view = me_view(&state, user.user_id).await?;
    Ok(([(header::CACHE_CONTROL, "no-store")], Json(view)).into_response())
}
