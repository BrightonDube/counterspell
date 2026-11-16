//! Linking for the signed-in account: `POST /api/auth/link/{provider}/start` for a provider
//! identity, and `POST /api/auth/link/email` for an email address.
//!
//! A provider link answers with the provider's authorize URL, and the callback in `oauth.rs`
//! finishes it. An email link mails a single-use link, and the verify handler attaches the address
//! once the same account opens it. The account always comes from the session cookie.

use std::net::SocketAddr;

use axum::{
    body::Bytes,
    extract::{ConnectInfo, Path, State},
    http::HeaderMap,
    response::Response,
    Json,
};
use serde::Deserialize;
use serde_json::{json, Value};
use uuid::Uuid;

use crate::{
    auth::{
        capabilities::{Capabilities, Capability},
        email::{compose, magic_link_url, EmailAddress},
        errors::{database, AuthError},
        extract::AuthUser,
        handlers::{
            magic::{send_in_background, sent},
            oauth::callback_url,
        },
        limits::{Action, Scope},
        oauth::ProviderId,
        tokens::Purpose,
        AuthState,
    },
    client_ip::client_ip,
};
use counterspell_common::ids::UserId;

/// The account settings page. A link flow returns here, with success or an error code.
const SETTINGS_PATH: &str = "/settings/account";

/// `POST /api/auth/link/{provider}/start`. Creates a link flow bound to this session and returns
/// the provider's authorize URL.
///
/// # Errors
///
/// Returns [`AuthError::Unauthenticated`] without a session, [`AuthError::DemoAccountRestricted`]
/// for the demo account, [`AuthError::ProviderDisabled`] for an unknown or disabled provider,
/// [`AuthError::RateLimited`] over the limit, and [`AuthError::Internal`] when the flow cannot
/// be stored.
pub async fn start(
    State(state): State<AuthState>,
    user: AuthUser,
    Path(provider): Path<String>,
) -> Result<Json<Value>, AuthError> {
    Capabilities::for_user(user.is_demo).require(Capability::LinkProvider)?;
    let provider_id = ProviderId::parse(&provider).ok_or(AuthError::ProviderDisabled)?;
    let port = state.oauth_port()?;
    let adapter = port
        .providers
        .get(&provider_id)
        .cloned()
        .ok_or(AuthError::ProviderDisabled)?;
    state
        .limits
        .hit(
            Action::LinkStart,
            Scope::User,
            &user.user_id.as_uuid().to_string(),
        )
        .await?;

    let flow = state
        .flows
        .create_link(provider_id, &user.key, user.user_id, SETTINGS_PATH)
        .await?;
    let redirect = callback_url(&port.base_url, provider_id)?;
    let target = adapter.authorize_url(flow.state.expose(), &flow.challenge, &redirect);
    Ok(Json(json!({ "authorize_url": target.as_str() })))
}

/// The JSON body of a request to link an email address.
#[derive(Debug, Deserialize)]
pub struct LinkEmailRequest {
    /// The address to prove by mail.
    pub email: String,
}

/// `POST /api/auth/link/email`. Mails a link for the address when it is free, and answers `202`
/// either way, so the answer does not reveal which addresses have accounts.
///
/// # Errors
///
/// Returns [`AuthError::DemoAccountRestricted`] for the demo account, [`AuthError::InvalidJson`]
/// or [`AuthError::InvalidEmail`] for a bad body, [`AuthError::EmailAlreadySet`] when the account
/// already has an email, [`AuthError::RateLimited`] over the limits, and
/// [`AuthError::EmailUnavailable`] when the limits cannot be checked.
pub async fn email(
    State(state): State<AuthState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    user: AuthUser,
    body: Bytes,
) -> Result<Response, AuthError> {
    Capabilities::for_user(user.is_demo).require(Capability::LinkEmail)?;
    let request: LinkEmailRequest =
        serde_json::from_slice(&body).map_err(|_| AuthError::InvalidJson)?;
    let email = EmailAddress::parse(&request.email).map_err(|_| AuthError::InvalidEmail)?;
    if account_has_email(&state, user.user_id).await? {
        return Err(AuthError::EmailAlreadySet);
    }
    let ip = client_ip(peer.ip(), &headers, &state.trusted).to_string();
    state
        .limits
        .hit(Action::LinkEmail, Scope::Email, email.as_str())
        .await?;
    state.limits.hit(Action::LinkEmail, Scope::Ip, &ip).await?;

    let port = state.mail_port()?;
    if !address_is_free(&state, &email, port.demo_email.as_ref()).await? {
        return Ok(sent());
    }
    let raw = state
        .tokens
        .issue(Purpose::LinkEmail, &email, Some(user.user_id), json!({}))
        .await?;
    let link = magic_link_url(&port.public_base_url, raw.expose());
    let mail = compose(
        Purpose::LinkEmail,
        email,
        &link,
        &Uuid::new_v4().to_string(),
    );
    send_in_background(port.sender.clone(), mail);
    Ok(sent())
}

/// Whether the account already has an email address.
async fn account_has_email(state: &AuthState, user: UserId) -> Result<bool, AuthError> {
    let (has_email,): (bool,) = sqlx::query_as("SELECT email IS NOT NULL FROM users WHERE id = $1")
        .bind(user.as_uuid())
        .fetch_one(&state.pool)
        .await
        .map_err(database)?;
    Ok(has_email)
}

/// Whether the address can receive a link for this account. No account may use it, and the demo
/// address never gets mail. When it is not free, the caller answers the same way as for a free one.
async fn address_is_free(
    state: &AuthState,
    email: &EmailAddress,
    demo_email: Option<&EmailAddress>,
) -> Result<bool, AuthError> {
    if demo_email == Some(email) {
        return Ok(false);
    }
    let (taken,): (bool,) = sqlx::query_as("SELECT EXISTS (SELECT 1 FROM users WHERE email = $1)")
        .bind(email.as_str())
        .fetch_one(&state.pool)
        .await
        .map_err(database)?;
    Ok(!taken)
}
