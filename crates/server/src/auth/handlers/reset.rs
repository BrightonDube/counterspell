//! Password reset: `POST /api/auth/password/reset` asks for a link, and
//! `POST /api/auth/password/reset/confirm` sets the new password.
//!
//! The request answers `202` for every valid address and mails only accounts that have a password
//! and are not the demo account. A confirmed reset replaces the hash and ends every session of the
//! user. It never signs anybody in (R15.8).

use std::net::SocketAddr;

use axum::{
    body::Bytes,
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
};
use serde::Deserialize;
use serde_json::json;
use sqlx::types::Uuid as DbUuid;
use uuid::Uuid;

use crate::{
    auth::{
        email::{compose, magic_link_url, EmailAddress},
        errors::{database, AuthError},
        handlers::magic::{send_in_background, sent},
        limits::{Action, Scope},
        password::check_policy,
        tokens::Purpose,
        AuthState,
    },
    client_ip::client_ip,
};
use counterspell_common::ids::UserId;

/// The JSON body of a reset request.
#[derive(Debug, Deserialize)]
pub struct ResetRequest {
    /// The address whose password should be reset.
    pub email: String,
}

/// The JSON body of a reset confirmation.
#[derive(Debug, Deserialize)]
pub struct ResetConfirm {
    /// The raw token from the reset link.
    pub token: String,
    /// The new password. It is hashed before anything is stored.
    pub new_password: String,
}

/// `POST /api/auth/password/reset`. Answers `202` for every valid address.
///
/// # Errors
///
/// Returns [`AuthError::InvalidJson`] for a malformed body, [`AuthError::InvalidEmail`] for an
/// address that does not parse, [`AuthError::RateLimited`] or [`AuthError::EmailUnavailable`] from
/// the limits, and [`AuthError::Internal`] when storage fails.
pub async fn request_reset(
    State(state): State<AuthState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, AuthError> {
    let request: ResetRequest =
        serde_json::from_slice(&body).map_err(|_| AuthError::InvalidJson)?;
    let email = EmailAddress::parse(&request.email).map_err(|_| AuthError::InvalidEmail)?;
    let ip = client_ip(peer.ip(), &headers, &state.trusted).to_string();
    state
        .limits
        .hit(Action::Reset, Scope::Email, email.as_str())
        .await?;
    state.limits.hit(Action::Reset, Scope::Ip, &ip).await?;

    let port = state.mail_port()?;
    let account: Option<(DbUuid, bool)> = sqlx::query_as(
        "SELECT id, is_demo FROM users WHERE email = $1 AND password_hash IS NOT NULL",
    )
    .bind(email.as_str())
    .fetch_optional(&state.pool)
    .await
    .map_err(database)?;
    // Only an ordinary account with a password gets mail. Everyone else gets the same answer.
    if let Some((id, false)) = account {
        let raw = state
            .tokens
            .issue(
                Purpose::Reset,
                &email,
                Some(UserId::from_uuid(id)),
                json!({}),
            )
            .await?;
        let link = magic_link_url(&port.public_base_url, raw.expose());
        let mail = compose(Purpose::Reset, email, &link, &Uuid::new_v4().to_string());
        send_in_background(port.sender.clone(), mail);
    }
    Ok(sent())
}

/// `POST /api/auth/password/reset/confirm`. Sets the new password and ends every session of the
/// account. The answer is `204` and nobody is signed in.
///
/// # Errors
///
/// Returns [`AuthError::InvalidJson`] for a malformed body, [`AuthError::WeakPassword`] for a
/// password outside the policy, [`AuthError::InvalidOrExpiredToken`] for an unusable link, and
/// [`AuthError::Internal`] when storage or hashing fails.
pub async fn confirm_reset(
    State(state): State<AuthState>,
    body: Bytes,
) -> Result<Response, AuthError> {
    let request: ResetConfirm =
        serde_json::from_slice(&body).map_err(|_| AuthError::InvalidJson)?;
    check_policy(&request.new_password)?;
    let consumed = state
        .tokens
        .consume(&request.token, Purpose::Reset, None)
        .await?;
    let user = consumed.user_id.ok_or(AuthError::InvalidOrExpiredToken)?;
    let hash = state.hashes.hash(request.new_password).await?;
    sqlx::query("UPDATE users SET password_hash = $2 WHERE id = $1")
        .bind(user.as_uuid())
        .bind(hash)
        .execute(&state.pool)
        .await
        .map_err(database)?;
    state.sessions.revoke_all(user).await?;
    Ok(StatusCode::NO_CONTENT.into_response())
}
