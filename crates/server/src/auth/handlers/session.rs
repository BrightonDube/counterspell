//! Logout (this browser) and logout everywhere (every browser of the signed-in user).

use axum::{
    extract::{Extension, State},
    http::{HeaderMap, StatusCode},
};

use crate::auth::{
    capabilities::{Capabilities, Capability},
    cookies::{read_cookie, CookieSlot},
    errors::AuthError,
    extract::AuthUser,
    session::SessionKey,
    AuthState,
};

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
    Ok(StatusCode::NO_CONTENT)
}
