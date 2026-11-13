//! Extractors that give a handler the signed-in user. Nothing in them reads a user id from the
//! request body, the path, the query string or any header other than the session cookie.

use axum::{extract::FromRequestParts, http::request::Parts};
use counterspell_common::ids::UserId;

use super::{
    cookies::{read_cookie, CookieSlot},
    errors::AuthError,
    session::SessionKey,
    AuthState,
};

/// The signed-in user. A missing or unusable session becomes `401 unauthenticated`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthUser {
    /// The user the session belongs to.
    pub user_id: UserId,
    /// The stored key of the session that was used. Logout needs it.
    pub key: SessionKey,
    /// Whether the user is the judge demo account.
    pub is_demo: bool,
}

/// The signed-in user if there is one, and `None` otherwise. Use it for routes that work either way.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MaybeAuthUser(pub Option<AuthUser>);

impl FromRequestParts<AuthState> for AuthUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AuthState,
    ) -> Result<Self, Self::Rejection> {
        resolve_session(parts, state)
            .await?
            .ok_or(AuthError::Unauthenticated)
    }
}

impl FromRequestParts<AuthState> for MaybeAuthUser {
    type Rejection = AuthError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AuthState,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(resolve_session(parts, state).await?))
    }
}

/// Checks the session cookie. A renewed session re-issues its cookie. An unusable cookie is
/// cleared, and the clearing header goes out with whatever response the handler gives.
async fn resolve_session(parts: &Parts, state: &AuthState) -> Result<Option<AuthUser>, AuthError> {
    let Some(raw) = read_cookie(&parts.headers, state.cookies.session_name()) else {
        return Ok(None);
    };
    let slot = parts.extensions.get::<CookieSlot>();
    let Some(found) = state.sessions.authenticate(raw).await? else {
        if let Some(slot) = slot {
            slot.set(state.cookies.clear_session()?);
        }
        return Ok(None);
    };
    if found.renewed {
        if let Some(slot) = slot {
            slot.set(state.cookies.session_value(raw)?);
        }
    }
    Ok(Some(AuthUser {
        user_id: found.user_id,
        key: found.key,
        is_demo: found.is_demo,
    }))
}
