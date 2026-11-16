//! Password registration: `POST /api/auth/register` mails a link that carries the password hash.
//!
//! No account exists until the link is opened, so a mailbox owner has to prove the address first
//! (R15.7). Opening the link either creates the account with that hash and signs in, or refuses
//! with `account_exists` and changes nothing.

use std::net::SocketAddr;

use axum::{
    body::Bytes,
    extract::{ConnectInfo, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use sqlx::types::Uuid as DbUuid;
use uuid::Uuid;

use crate::{
    auth::{
        accounts::new_handle,
        cookies::CookieSlot,
        email::{compose, magic_link_url, EmailAddress},
        errors::{database, is_unique_violation, AuthError},
        handlers::magic::{send_in_background, sent},
        limits::{Action, Scope},
        password::check_policy,
        return_to::sanitize_return_to,
        session::AuthMethod,
        tokens::Purpose,
        AuthState,
    },
    client_ip::client_ip,
};
use counterspell_common::ids::UserId;

/// The JSON body of a registration request.
#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    /// The address to prove by mail.
    pub email: String,
    /// The password to set once the address is proven. It is hashed before anything is stored.
    pub password: String,
    /// Where to go after sign-in. Anything that would leave the site becomes `/`.
    #[serde(default)]
    pub return_to: Option<String>,
}

/// `POST /api/auth/register`. Answers `202` for every valid address and mails a link.
///
/// # Errors
///
/// Returns [`AuthError::InvalidJson`] for a malformed body, [`AuthError::InvalidEmail`] and
/// [`AuthError::WeakPassword`] for bad input, [`AuthError::RateLimited`] or
/// [`AuthError::EmailUnavailable`] from the limits, and [`AuthError::Internal`] when hashing or
/// storage fails.
pub async fn register(
    State(state): State<AuthState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, AuthError> {
    let request: RegisterRequest =
        serde_json::from_slice(&body).map_err(|_| AuthError::InvalidJson)?;
    let email = EmailAddress::parse(&request.email).map_err(|_| AuthError::InvalidEmail)?;
    check_policy(&request.password)?;
    let ip = client_ip(peer.ip(), &headers, &state.trusted).to_string();
    state
        .limits
        .hit(Action::Register, Scope::Email, email.as_str())
        .await?;
    state.limits.hit(Action::Register, Scope::Ip, &ip).await?;

    let port = state.mail_port()?;
    if port.demo_email.as_ref() == Some(&email) {
        return Ok(sent());
    }
    let hash = state.hashes.hash(request.password).await?;
    let return_to = sanitize_return_to(request.return_to.as_deref());
    let raw = state
        .tokens
        .issue(
            Purpose::Verify,
            &email,
            None,
            json!({ "password_hash": hash, "return_to": return_to }),
        )
        .await?;
    let link = magic_link_url(&port.public_base_url, raw.expose());
    let mail = compose(Purpose::Verify, email, &link, &Uuid::new_v4().to_string());
    send_in_background(port.sender.clone(), mail);
    Ok(sent())
}

/// Uses a registration link. Creates the account with the stored hash and signs it in, or
/// refuses with `account_exists` when the address already belongs to an account.
///
/// # Errors
///
/// Returns [`AuthError::InvalidOrExpiredToken`] for an unusable link, [`AuthError::AccountExists`]
/// when the address is taken, and [`AuthError::Internal`] for storage failures.
pub async fn finish_registration(
    state: &AuthState,
    slot: &CookieSlot,
    raw: &str,
) -> Result<Response, AuthError> {
    let consumed = state.tokens.consume(raw, Purpose::Verify, None).await?;
    let hash = consumed
        .payload
        .get("password_hash")
        .and_then(Value::as_str)
        .ok_or(AuthError::Internal)?;
    let return_to = sanitize_return_to(consumed.payload.get("return_to").and_then(Value::as_str));
    let user = insert_account(state, consumed.email.as_str(), hash).await?;
    let created = state.sessions.create(user, AuthMethod::Register).await?;
    slot.set(state.cookies.session_value(created.cookie_value.expose())?);
    Ok(Json(json!({
        "user": { "id": user.as_uuid() },
        "return_to": return_to,
    }))
    .into_response())
}

/// Inserts the verified account. The unique email constraint refuses an address that is taken,
/// including the demo address, so a race can never create a second account.
async fn insert_account(state: &AuthState, email: &str, hash: &str) -> Result<UserId, AuthError> {
    let inserted: Result<(DbUuid,), sqlx::Error> = sqlx::query_as(
        "INSERT INTO users (email, email_verified_at, password_hash, handle, display_name)
         VALUES ($1, $2, $3, $4, $4) RETURNING id",
    )
    .bind(email)
    .bind(state.clock.now())
    .bind(hash)
    .bind(new_handle())
    .fetch_one(&state.pool)
    .await;
    match inserted {
        Ok((id,)) => Ok(UserId::from_uuid(id)),
        Err(error) if is_unique_violation(&error) => Err(AuthError::AccountExists),
        Err(error) => Err(database(error)),
    }
}
