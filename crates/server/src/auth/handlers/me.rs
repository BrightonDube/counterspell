//! `GET /api/me`: the signed-in user's own account, linked providers and what the account may do.

use axum::{
    extract::State,
    http::header,
    response::{IntoResponse, Response},
    Json,
};
use counterspell_common::ids::UserId;
use serde::Serialize;

use crate::auth::{
    capabilities::Capabilities,
    errors::{database, AuthError},
    extract::AuthUser,
    AuthState,
};

/// The account fields that the signed-in user may see.
#[derive(Debug, Serialize)]
pub struct MeUser {
    /// The user id.
    pub id: UserId,
    /// The public handle.
    pub handle: String,
    /// The display name.
    pub display_name: String,
    /// The verified email address, when there is one.
    pub email: Option<String>,
    /// Whether this is the judge demo account.
    pub is_demo: bool,
    /// The linked sign-in providers, for example `github`.
    pub providers: Vec<String>,
}

/// The body of `GET /api/me`.
#[derive(Debug, Serialize)]
pub struct MeView {
    /// The account.
    pub user: MeUser,
    /// What the account may do. The UI reads this instead of checking for the demo account.
    pub capabilities: Capabilities,
}

/// `GET /api/me`. Answers with `Cache-Control: no-store`, and only with the caller's own data.
///
/// # Errors
///
/// Returns [`AuthError::Unauthenticated`] when the account no longer exists, and
/// [`AuthError::Internal`] when the database fails.
pub async fn me(State(state): State<AuthState>, user: AuthUser) -> Result<Response, AuthError> {
    let row: Option<(String, String, Option<String>, bool)> = sqlx::query_as(
        "SELECT handle::text, display_name, email::text, is_demo FROM users WHERE id = $1",
    )
    .bind(user.user_id.as_uuid())
    .fetch_optional(&state.pool)
    .await
    .map_err(database)?;
    let Some((handle, display_name, email, is_demo)) = row else {
        return Err(AuthError::Unauthenticated);
    };

    let providers: Vec<(String,)> =
        sqlx::query_as("SELECT provider FROM identities WHERE user_id = $1 ORDER BY provider")
            .bind(user.user_id.as_uuid())
            .fetch_all(&state.pool)
            .await
            .map_err(database)?;

    let view = MeView {
        user: MeUser {
            id: user.user_id,
            handle,
            display_name,
            email,
            is_demo,
            providers: providers.into_iter().map(|(name,)| name).collect(),
        },
        capabilities: Capabilities::for_user(is_demo),
    };
    Ok(([(header::CACHE_CONTROL, "no-store")], Json(view)).into_response())
}
