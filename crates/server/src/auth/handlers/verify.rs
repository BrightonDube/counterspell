//! `POST /api/auth/email/verify`: uses an emailed token.
//!
//! The token arrives in the JSON body, never in a URL that a mail scanner could open. The
//! handler first reads the token's purpose without using it, then dispatches on that purpose.
//! Only a sign-in link can sign in here. Other purposes get the same refusal as an unknown token
//! until their own tasks add them.

use axum::{
    body::Bytes,
    extract::State,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::types::Uuid;

use crate::auth::{
    accounts::new_handle,
    cookies::CookieSlot,
    email::EmailAddress,
    errors::{database, is_unique_violation, AuthError},
    extract::{AuthUser, MaybeAuthUser},
    handlers::register::finish_registration,
    return_to::sanitize_return_to,
    session::AuthMethod,
    tokens::Purpose,
    AuthState,
};
use counterspell_common::ids::UserId;

/// The JSON body of a verify request.
#[derive(Debug, Deserialize)]
pub struct VerifyRequest {
    /// The raw token from the sign-in link's fragment.
    pub token: String,
}

/// `POST /api/auth/email/verify`.
///
/// # Errors
///
/// Returns [`AuthError::InvalidJson`] for a malformed body and
/// [`AuthError::InvalidOrExpiredToken`] for every token that cannot be used, including a link
/// for another account. A `link_email` token without a session gives [`AuthError::Unauthenticated`].
pub async fn verify(
    State(state): State<AuthState>,
    axum::Extension(slot): axum::Extension<CookieSlot>,
    MaybeAuthUser(session): MaybeAuthUser,
    body: Bytes,
) -> Result<Response, AuthError> {
    let request: VerifyRequest =
        serde_json::from_slice(&body).map_err(|_| AuthError::InvalidJson)?;
    let peeked = state
        .tokens
        .peek(&request.token)
        .await?
        .ok_or(AuthError::InvalidOrExpiredToken)?;
    match peeked.purpose {
        Purpose::MagicLink => sign_in_with_link(&state, &slot, &request.token).await,
        Purpose::LinkEmail => link_email(&state, &request.token, peeked.user_id, session).await,
        Purpose::Verify => finish_registration(&state, &slot, &request.token).await,
        // A reset link is used only by the reset confirmation, so it is refused here unused.
        Purpose::Reset => Err(AuthError::InvalidOrExpiredToken),
    }
}

/// Attaches a verified address to the signed-in account that asked for it. The token must belong
/// to that account, and nothing is used up when it does not. The session itself is not changed.
async fn link_email(
    state: &AuthState,
    raw: &str,
    bound_user: Option<UserId>,
    session: Option<AuthUser>,
) -> Result<Response, AuthError> {
    let signed_in = session.ok_or(AuthError::Unauthenticated)?;
    if bound_user != Some(signed_in.user_id) {
        return Err(AuthError::InvalidOrExpiredToken);
    }
    let consumed = state
        .tokens
        .consume(raw, Purpose::LinkEmail, Some(signed_in.user_id))
        .await?;
    let updated = sqlx::query(
        "UPDATE users SET email = $2, email_verified_at = $3
          WHERE id = $1 AND email IS NULL",
    )
    .bind(signed_in.user_id.as_uuid())
    .bind(consumed.email.as_str())
    .bind(state.clock.now())
    .execute(&state.pool)
    .await;
    match updated {
        Ok(result) if result.rows_affected() == 1 => {
            Ok(Json(json!({ "status": "email_linked" })).into_response())
        }
        Ok(_) => Err(AuthError::EmailAlreadySet),
        Err(error) if is_unique_violation(&error) => Err(AuthError::EmailInUse),
        Err(error) => Err(database(error)),
    }
}

/// Uses a sign-in link, finds or creates the account, and starts a session.
async fn sign_in_with_link(
    state: &AuthState,
    slot: &CookieSlot,
    raw: &str,
) -> Result<Response, AuthError> {
    let consumed = state.tokens.consume(raw, Purpose::MagicLink, None).await?;
    let user = find_or_create_user(state, &consumed.email).await?;
    let return_to = sanitize_return_to(consumed.payload.get("return_to").and_then(Value::as_str));
    let pending = crate::auth::security::complete_primary(
        state,
        slot,
        user,
        AuthMethod::MagicLink,
        &return_to,
    )
    .await?;
    if pending {
        return Ok(Json(json!({ "return_to": crate::auth::security::STEP_UP_PATH, "two_factor_required": true })).into_response());
    }
    Ok(Json(json!({
        "user": { "id": user.as_uuid() },
        "return_to": return_to,
    }))
    .into_response())
}

/// Returns the account for a verified address, creating it on first use.
///
/// The insert ignores a conflict, so two requests for a new address cannot create two accounts.
/// The demo account is never signed in through a link.
async fn find_or_create_user(state: &AuthState, email: &EmailAddress) -> Result<UserId, AuthError> {
    let handle = new_handle();
    let now = state.clock.now();
    let mut transaction = state.pool.begin().await.map_err(database)?;
    sqlx::query(
        "INSERT INTO users (email, email_verified_at, handle, display_name)
         VALUES ($1, $2, $3, $3)
         ON CONFLICT DO NOTHING",
    )
    .bind(email.as_str())
    .bind(now)
    .bind(&handle)
    .execute(&mut *transaction)
    .await
    .map_err(database)?;
    let (id, is_demo): (Uuid, bool) =
        sqlx::query_as("SELECT id, is_demo FROM users WHERE email = $1")
            .bind(email.as_str())
            .fetch_one(&mut *transaction)
            .await
            .map_err(database)?;
    if is_demo {
        return Err(AuthError::InvalidOrExpiredToken);
    }
    transaction.commit().await.map_err(database)?;
    Ok(UserId::from_uuid(id))
}
