//! `DELETE /api/me`: deletes the signed-in user's account and everything that depends on it
//! (auth T36, requirements R17.1 to R17.5).

use axum::{extract::State, http::StatusCode, Extension, Json};
use serde::Deserialize;

use crate::auth::{
    capabilities::{Capabilities, Capability},
    cookies::CookieSlot,
    errors::{database, AuthError},
    extract::AuthUser,
    AuthState,
};

/// The body of `DELETE /api/me`. The caller must type their own handle to confirm.
#[derive(Debug, Deserialize)]
pub struct DeleteBody {
    /// The handle of the account being deleted. It must match exactly.
    pub confirm_handle: String,
}

/// `DELETE /api/me`. The account, its identities, sessions, tokens, flows, submissions and run
/// rows are removed in one transaction. Foreign keys with `ON DELETE CASCADE` do the rest.
///
/// # Errors
///
/// Returns [`AuthError::DemoAccountRestricted`] for the demo account,
/// [`AuthError::ConfirmationMismatch`] when the handle does not match,
/// [`AuthError::RunsActive`] while a run is queued or claimed, and
/// [`AuthError::Internal`] when the database fails. A failed request changes nothing.
pub async fn delete_account(
    State(state): State<AuthState>,
    Extension(slot): Extension<CookieSlot>,
    user: AuthUser,
    Json(body): Json<DeleteBody>,
) -> Result<StatusCode, AuthError> {
    Capabilities::for_user(user.is_demo).require(Capability::DeleteAccount)?;

    let mut tx = state.pool.begin().await.map_err(database)?;
    let row: Option<(String, Option<String>)> =
        sqlx::query_as("SELECT handle::text, email::text FROM users WHERE id = $1 FOR UPDATE")
            .bind(user.user_id.as_uuid())
            .fetch_optional(&mut *tx)
            .await
            .map_err(database)?;
    let Some((handle, email)) = row else {
        return Err(AuthError::Unauthenticated);
    };
    if body.confirm_handle != handle {
        return Err(AuthError::ConfirmationMismatch);
    }

    let active: bool = sqlx::query_scalar(
        "SELECT EXISTS (
             SELECT 1 FROM jobs j
             JOIN submissions s ON s.id = j.submission_id
             WHERE s.user_id = $1 AND j.state IN ('queued', 'claimed')
         )",
    )
    .bind(user.user_id.as_uuid())
    .fetch_one(&mut *tx)
    .await
    .map_err(database)?;
    if active {
        return Err(AuthError::RunsActive);
    }

    if let Some(address) = email {
        sqlx::query("DELETE FROM email_tokens WHERE email = $1::citext AND used_at IS NULL")
            .bind(address)
            .execute(&mut *tx)
            .await
            .map_err(database)?;
    }
    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user.user_id.as_uuid())
        .execute(&mut *tx)
        .await
        .map_err(database)?;
    tx.commit().await.map_err(database)?;

    slot.set(state.cookies.clear_session()?);
    Ok(StatusCode::NO_CONTENT)
}
