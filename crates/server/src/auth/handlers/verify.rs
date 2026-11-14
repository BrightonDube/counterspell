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
    errors::{database, AuthError},
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
/// [`AuthError::InvalidOrExpiredToken`] for every token that cannot be used, including tokens
/// for purposes this route does not handle yet.
pub async fn verify(
    State(state): State<AuthState>,
    axum::Extension(slot): axum::Extension<CookieSlot>,
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
        // Account confirmation, password reset and email linking are added by later tasks.
        // The token is left unused, so nothing is lost when they arrive.
        Purpose::Verify | Purpose::Reset | Purpose::LinkEmail => {
            Err(AuthError::InvalidOrExpiredToken)
        }
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
    let created = state.sessions.create(user, AuthMethod::MagicLink).await?;
    slot.set(state.cookies.session_value(created.cookie_value.expose())?);
    let return_to = sanitize_return_to(consumed.payload.get("return_to").and_then(Value::as_str));
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
