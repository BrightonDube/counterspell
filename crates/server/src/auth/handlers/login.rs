//! `POST /api/auth/login`: email and password sign-in.
//!
//! Every refusal gives the same `401 invalid_credentials`, whatever the cause. A limit is checked
//! before any hashing, and only a failed attempt is counted.

use std::net::SocketAddr;

use axum::{
    body::Bytes,
    extract::{ConnectInfo, Extension, State},
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use sqlx::types::Uuid;

use crate::{
    auth::{
        cookies::CookieSlot,
        email::EmailAddress,
        errors::{database, AuthError},
        limits::{Action, Scope},
        password::Verdict,
        session::AuthMethod,
        AuthState,
    },
    client_ip::client_ip,
};

/// The JSON body of a sign-in request. Unknown fields are ignored.
#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    /// The account's email address.
    pub email: String,
    /// The password, as typed.
    pub password: String,
}

/// `POST /api/auth/login`.
///
/// # Errors
///
/// Returns [`AuthError::InvalidJson`] for a body that is not a sign-in request,
/// [`AuthError::RateLimited`] when a limit is reached (before any hashing),
/// [`AuthError::Busy`] when the hash gate is full, and [`AuthError::InvalidCredentials`] for any
/// refused sign-in.
pub async fn login(
    State(state): State<AuthState>,
    Extension(slot): Extension<CookieSlot>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, AuthError> {
    let request: LoginRequest =
        serde_json::from_slice(&body).map_err(|_| AuthError::InvalidJson)?;
    let ip = client_ip(peer.ip(), &headers, &state.trusted).to_string();
    let Ok(email) = EmailAddress::parse(&request.email) else {
        return Err(refuse(&state, &ip, None).await);
    };
    let pair = format!("{}|{ip}", email.as_str());

    state
        .limits
        .check(Action::LoginFailure, Scope::EmailAndIp, &pair)
        .await?;
    state
        .limits
        .check(Action::LoginFailure, Scope::Ip, &ip)
        .await?;

    let row: Option<(Uuid, Option<String>)> =
        sqlx::query_as("SELECT id, password_hash FROM users WHERE email = $1")
            .bind(email.as_str())
            .fetch_optional(&state.pool)
            .await
            .map_err(database)?;

    let stored = row.as_ref().and_then(|(_, hash)| hash.clone());
    let verdict = match stored {
        Some(hash) => state.hashes.verify(request.password.clone(), hash).await?,
        None => state.hashes.verify_dummy(request.password.clone()).await?,
    };
    let (Some((user_id, _)), Verdict::Verified | Verdict::NeedsRehash) = (row, verdict) else {
        return Err(refuse(&state, &ip, Some(&pair)).await);
    };

    if verdict == Verdict::NeedsRehash {
        let fresh = state.hashes.hash(request.password.clone()).await?;
        sqlx::query("UPDATE users SET password_hash = $2 WHERE id = $1")
            .bind(user_id)
            .bind(fresh)
            .execute(&state.pool)
            .await
            .map_err(database)?;
    }

    let created = state
        .sessions
        .create(
            counterspell_common::ids::UserId::from_uuid(user_id),
            AuthMethod::Password,
        )
        .await?;
    slot.set(state.cookies.session_value(created.cookie_value.expose())?);
    Ok(Json(json!({ "user_id": user_id })).into_response())
}

/// Counts a failed attempt against both limits and returns the uniform refusal.
async fn refuse(state: &AuthState, ip: &str, pair: Option<&str>) -> AuthError {
    if let Some(pair) = pair {
        // A failure is counted even if the limit is now reached; the next attempt is refused.
        let _ = state
            .limits
            .hit(Action::LoginFailure, Scope::EmailAndIp, pair)
            .await;
    }
    let _ = state.limits.hit(Action::LoginFailure, Scope::Ip, ip).await;
    AuthError::InvalidCredentials
}
