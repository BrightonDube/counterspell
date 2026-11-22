//! Linking a provider identity for the signed-in account:
//! `POST /api/auth/link/{provider}/start`.
//!
//! A provider link answers with the provider's authorize URL, and the callback in `oauth.rs`
//! finishes it. The account always comes from the session cookie.

use axum::{
    extract::{Path, State},
    Json,
};
use serde_json::{json, Value};

use crate::auth::{
    capabilities::{Capabilities, Capability},
    errors::AuthError,
    extract::AuthUser,
    handlers::oauth::callback_url,
    limits::{Action, Scope},
    oauth::ProviderId,
    AuthState,
};

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
