//! `POST /api/auth/email/magic`: asks for a sign-in link by email.
//!
//! The answer is `202 {"status":"sent"}` whether or not the address has an account, so the
//! response does not reveal who is registered (R5.1, R5.2). The mail is sent in the background,
//! so the response time does not depend on the account either.

use std::{net::SocketAddr, sync::Arc};

use axum::{
    body::Bytes,
    extract::{ConnectInfo, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Deserialize;
use serde_json::json;
use uuid::Uuid;

use crate::{
    auth::{
        email::{compose, magic_link_url, EmailAddress, EmailSender, OutgoingMail},
        errors::AuthError,
        limits::{Action, Scope},
        return_to::sanitize_return_to,
        tokens::Purpose,
        AuthState,
    },
    client_ip::client_ip,
};

/// The JSON body of a link request. `return_to` is optional and checked before it is stored.
#[derive(Debug, Deserialize)]
pub struct MagicRequest {
    /// The address to send the link to.
    pub email: String,
    /// Where to go after sign-in. Anything that would leave the site becomes `/`.
    #[serde(default)]
    pub return_to: Option<String>,
}

/// `POST /api/auth/email/magic`.
///
/// # Errors
///
/// Returns [`AuthError::InvalidJson`] for a malformed body, [`AuthError::InvalidEmail`] for an
/// address that does not parse, [`AuthError::RateLimited`] or [`AuthError::EmailUnavailable`]
/// from the limits, and [`AuthError::Internal`] when the token store or mail is not available.
pub async fn request_magic(
    State(state): State<AuthState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, AuthError> {
    let request: MagicRequest =
        serde_json::from_slice(&body).map_err(|_| AuthError::InvalidJson)?;
    let email = EmailAddress::parse(&request.email).map_err(|_| AuthError::InvalidEmail)?;
    let ip = client_ip(peer.ip(), &headers, &state.trusted).to_string();
    state
        .limits
        .hit(Action::Magic, Scope::Email, email.as_str())
        .await?;
    state.limits.hit(Action::Magic, Scope::Ip, &ip).await?;

    let port = state.mail_port()?;
    if port.demo_email.as_ref() == Some(&email) {
        // The demo account never gets a link. The caller sees the same answer as for any address.
        return Ok(sent());
    }

    let return_to = sanitize_return_to(request.return_to.as_deref());
    let raw = state
        .tokens
        .issue(
            Purpose::MagicLink,
            &email,
            None,
            json!({ "return_to": return_to }),
        )
        .await?;
    let link = magic_link_url(&port.public_base_url, raw.expose());
    let mail = compose(
        Purpose::MagicLink,
        email,
        &link,
        &Uuid::new_v4().to_string(),
    );
    send_in_background(port.sender.clone(), mail);
    Ok(sent())
}

/// The uniform answer for every address.
pub(crate) fn sent() -> Response {
    (StatusCode::ACCEPTED, Json(json!({ "status": "sent" }))).into_response()
}

/// Sends one mail without making the caller wait. A failure is logged without the link or the
/// address. The token simply expires.
pub(crate) fn send_in_background(sender: Arc<dyn EmailSender>, mail: OutgoingMail) {
    tokio::spawn(async move {
        if sender.send(mail).await.is_err() {
            tracing::warn!("a mail could not be sent");
        }
    });
}
