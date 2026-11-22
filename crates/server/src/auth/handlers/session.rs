//! Logout (this browser) and logout everywhere (every browser of the signed-in user).

use axum::{
    extract::{Extension, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Serialize;

use crate::auth::{
    capabilities::{Capabilities, Capability},
    cookies::{read_cookie, CookieSlot},
    errors::AuthError,
    extract::{AuthUser, MaybeAuthUser},
    session::SessionKey,
    AuthState,
};

/// The body of `GET /api/auth/session`.
#[derive(Debug, Serialize)]
pub struct SessionStatus {
    /// Whether the request carried a valid session cookie.
    pub signed_in: bool,
}

/// `GET /api/auth/session`. Lets public pages distinguish signed-in visitors without making an
/// expected signed-out visit produce a `401`.
///
/// # Errors
///
/// Returns [`AuthError::Internal`] when the database fails.
pub async fn status(user: MaybeAuthUser) -> Json<SessionStatus> {
    Json(SessionStatus {
        signed_in: user.0.is_some(),
    })
}

/// `POST /api/auth/logout`. Ends the session in the cookie, if there is one, and clears the cookie.
/// Calling it again, or with no cookie, is harmless and still answers `204`.
///
/// # Errors
///
/// Returns [`AuthError::Internal`] when the database fails.
pub async fn logout(
    State(state): State<AuthState>,
    Extension(slot): Extension<CookieSlot>,
    headers: HeaderMap,
) -> Result<StatusCode, AuthError> {
    if let Some(raw) = read_cookie(&headers, state.cookies.session_name()) {
        state.sessions.revoke(&SessionKey::of(raw)).await?;
    }
    slot.set(state.cookies.clear_session()?);
    crate::auth::security::end_pending(&state, &slot, &headers).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /api/auth/logout-all`. Ends every session of the signed-in user. The demo account may
/// not do this.
///
/// # Errors
///
/// Returns [`AuthError::Unauthenticated`] without a session, [`AuthError::DemoAccountRestricted`]
/// for the demo account, and [`AuthError::Internal`] when the database fails.
pub async fn logout_all(
    State(state): State<AuthState>,
    Extension(slot): Extension<CookieSlot>,
    user: AuthUser,
) -> Result<StatusCode, AuthError> {
    Capabilities::for_user(user.is_demo).require(Capability::LogoutEverywhere)?;
    state.sessions.revoke_all(user.user_id).await?;
    slot.set(state.cookies.clear_session()?);
    slot.push(state.cookies.clear_step_up()?);
    Ok(StatusCode::NO_CONTENT)
}
