//! OAuth sign-in: `GET /api/auth/oauth/{provider}/start` and `.../callback`.
//!
//! The callback never returns JSON. Success redirects to the saved return path, and every failure
//! redirects to `/login?error=<code>`. The flow is taken first, so each state works once. The
//! browser binding is checked before the code is exchanged, and the binding cookie is cleared on
//! every outcome.

use std::net::SocketAddr;

use axum::{
    extract::{ConnectInfo, Extension, Path, Query, State},
    http::HeaderMap,
    response::{IntoResponse, Redirect, Response},
};
use serde::Deserialize;
use sqlx::{types::Uuid, PgPool};
use url::Url;

use crate::{
    auth::{
        accounts::new_handle,
        cookies::{read_cookie, CookieSlot},
        decision::{decide_oauth_login, OAuthDecision, ProfileEmail},
        email::EmailAddress,
        errors::AuthError,
        limits::{Action, Scope},
        oauth::{
            flow::{Flow, FlowPurpose},
            OAuthProvider, ProviderId, ProviderProfile,
        },
        return_to::sanitize_return_to,
        session::AuthMethod,
        AuthState,
    },
    client_ip::client_ip,
};
use counterspell_common::ids::UserId;

/// The optional `return_to` query of the start request.
#[derive(Debug, Deserialize)]
pub struct StartQuery {
    /// Where to go after sign-in. It is checked again before it is used.
    #[serde(default)]
    pub return_to: Option<String>,
}

/// The query the provider sends back to the callback.
#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    /// The one-time authorization code.
    pub code: Option<String>,
    /// The `state` value we sent.
    pub state: Option<String>,
    /// Set by the provider when the person refused, for example `access_denied`.
    pub error: Option<String>,
}

/// Why a callback did not sign anyone in. The code is shown on the login page.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Refusal {
    /// The state is unknown, reused, expired, or the browser binding does not match.
    StateInvalid,
    /// The person refused at the provider.
    Denied,
    /// The code exchange or the profile request failed, or the server could not finish.
    Failed,
    /// The provider's verified email belongs to another account, so nothing was linked.
    AccountExists,
}

impl Refusal {
    /// The error code that the login page reads from the query string.
    fn code(self) -> &'static str {
        match self {
            Self::StateInvalid => "state_invalid",
            Self::Denied => "oauth_denied",
            Self::Failed => "oauth_failed",
            Self::AccountExists => "account_exists",
        }
    }
}

/// `GET /api/auth/oauth/{provider}/start`: sends the browser to the provider.
///
/// # Errors
///
/// Returns [`AuthError::ProviderDisabled`] for an unknown or disabled provider,
/// [`AuthError::RateLimited`] over the start limit, and [`AuthError::Internal`] when the flow
/// cannot be stored.
pub async fn start(
    State(state): State<AuthState>,
    Extension(slot): Extension<CookieSlot>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    Path(provider): Path<String>,
    Query(query): Query<StartQuery>,
) -> Result<Response, AuthError> {
    let provider_id = ProviderId::parse(&provider).ok_or(AuthError::ProviderDisabled)?;
    let port = state.oauth_port()?;
    let adapter = port
        .providers
        .get(&provider_id)
        .cloned()
        .ok_or(AuthError::ProviderDisabled)?;
    let ip = client_ip(peer.ip(), &headers, &state.trusted).to_string();
    state.limits.hit(Action::OAuthStart, Scope::Ip, &ip).await?;

    let flow = state
        .flows
        .create_login(provider_id, query.return_to.as_deref().unwrap_or("/"))
        .await?;
    let binding = flow.binding.as_ref().ok_or(AuthError::Internal)?;
    let redirect = callback_url(&port.base_url, provider_id)?;
    let target = adapter.authorize_url(flow.state.expose(), &flow.challenge, &redirect);
    slot.push(state.cookies.binding_value(binding.expose())?);
    Ok(Redirect::to(target.as_str()).into_response())
}

/// `GET /api/auth/oauth/{provider}/callback`. Always a redirect, never JSON.
pub async fn callback(
    State(state): State<AuthState>,
    Extension(slot): Extension<CookieSlot>,
    headers: HeaderMap,
    Path(provider): Path<String>,
    Query(query): Query<CallbackQuery>,
) -> Response {
    // The binding cookie is single-use, so it is cleared whatever happened.
    if let Ok(clear) = state.cookies.clear_binding() {
        slot.push(clear);
    }
    match finish(&state, &slot, &headers, &provider, query).await {
        Ok(return_to) => Redirect::to(&return_to).into_response(),
        Err(refusal) => {
            tracing::info!(reason = refusal.code(), "oauth sign-in refused");
            Redirect::to(&format!("/login?error={}", refusal.code())).into_response()
        }
    }
}

/// Runs the callback steps in their fixed order and returns the place to go on success.
async fn finish(
    state: &AuthState,
    slot: &CookieSlot,
    headers: &HeaderMap,
    provider_name: &str,
    query: CallbackQuery,
) -> Result<String, Refusal> {
    let provider = ProviderId::parse(provider_name).ok_or(Refusal::StateInvalid)?;
    let port = state.oauth_port().map_err(|_| Refusal::Failed)?;
    let adapter = port
        .providers
        .get(&provider)
        .cloned()
        .ok_or(Refusal::StateInvalid)?;
    let raw_state = query.state.as_deref().ok_or(Refusal::StateInvalid)?;

    // Taking the flow uses it up. A replayed or altered state finds nothing.
    let flow = state
        .flows
        .consume(raw_state, provider)
        .await
        .map_err(|_| Refusal::Failed)?
        .ok_or(Refusal::StateInvalid)?;
    if flow.purpose() == FlowPurpose::Login {
        let binding = read_cookie(headers, state.cookies.binding_name());
        if !flow.binding_matches(binding) {
            return Err(Refusal::StateInvalid);
        }
    }
    if query.error.is_some() {
        return Err(Refusal::Denied);
    }
    let code = query.code.as_deref().ok_or(Refusal::Failed)?;
    let redirect = callback_url(&port.base_url, provider).map_err(|_| Refusal::Failed)?;

    match flow.purpose() {
        FlowPurpose::Login => sign_in(state, slot, adapter.as_ref(), &flow, code, &redirect).await,
        // Linking a provider to a signed-in account is added in T22.
        FlowPurpose::Link => Err(Refusal::Failed),
    }
}

/// Exchanges the code, decides what the profile means, and starts a session.
async fn sign_in(
    state: &AuthState,
    slot: &CookieSlot,
    adapter: &dyn OAuthProvider,
    flow: &Flow,
    code: &str,
    redirect: &Url,
) -> Result<String, Refusal> {
    let profile = adapter
        .fetch_profile(code, flow.pkce_verifier(), redirect)
        .await
        .map_err(|_| Refusal::Failed)?;
    let owner = identity_owner(&state.pool, flow.provider(), &profile.provider_user_id).await?;
    let email = profile_email(&state.pool, profile.verified_email.clone()).await?;
    let user = match decide_oauth_login(owner, email) {
        OAuthDecision::SignIn(user) => user,
        OAuthDecision::RefuseEmailTaken => return Err(Refusal::AccountExists),
        OAuthDecision::CreateAccount { email } => {
            create_account(state, flow.provider(), &profile, email).await?
        }
    };

    let method = match flow.provider() {
        ProviderId::GitHub => AuthMethod::OauthGithub,
        ProviderId::Google => AuthMethod::OauthGoogle,
    };
    let created = state
        .sessions
        .create(user, method)
        .await
        .map_err(|_| Refusal::Failed)?;
    let cookie = state
        .cookies
        .session_value(created.cookie_value.expose())
        .map_err(|_| Refusal::Failed)?;
    slot.push(cookie);
    Ok(sanitize_return_to(Some(flow.return_to())))
}

/// The account that already holds this provider identity, if any.
async fn identity_owner(
    pool: &PgPool,
    provider: ProviderId,
    provider_user_id: &str,
) -> Result<Option<UserId>, Refusal> {
    let row: Option<(Uuid,)> = sqlx::query_as(
        "SELECT user_id FROM identities WHERE provider = $1 AND provider_user_id = $2",
    )
    .bind(provider.as_str())
    .bind(provider_user_id)
    .fetch_optional(pool)
    .await
    .map_err(|_| Refusal::Failed)?;
    Ok(row.map(|(id,)| UserId::from_uuid(id)))
}

/// Whether the provider's verified email is free. An address held by any account, including the
/// demo account, counts as taken, so it never links or signs in to that account.
async fn profile_email(
    pool: &PgPool,
    email: Option<EmailAddress>,
) -> Result<ProfileEmail, Refusal> {
    let Some(address) = email else {
        return Ok(ProfileEmail::Absent);
    };
    let (taken,): (bool,) = sqlx::query_as("SELECT EXISTS (SELECT 1 FROM users WHERE email = $1)")
        .bind(address.as_str())
        .fetch_one(pool)
        .await
        .map_err(|_| Refusal::Failed)?;
    Ok(if taken {
        ProfileEmail::Taken
    } else {
        ProfileEmail::Free(address)
    })
}

/// Creates the account and its identity in one transaction. A unique violation (another request
/// took the same address first) gives `AccountExists` and leaves nothing behind.
async fn create_account(
    state: &AuthState,
    provider: ProviderId,
    profile: &ProviderProfile,
    email: Option<EmailAddress>,
) -> Result<UserId, Refusal> {
    let now = state.clock.now();
    let mut transaction = state.pool.begin().await.map_err(|_| Refusal::Failed)?;
    let inserted: Result<(Uuid,), sqlx::Error> = sqlx::query_as(
        "INSERT INTO users (email, email_verified_at, handle, display_name)
         VALUES ($1, $2, $3, $3) RETURNING id",
    )
    .bind(email.as_ref().map(EmailAddress::as_str))
    .bind(email.as_ref().map(|_| now))
    .bind(new_handle())
    .fetch_one(&mut *transaction)
    .await;
    let (user_id,) = match inserted {
        Ok(row) => row,
        Err(error) if is_unique_violation(&error) => return Err(Refusal::AccountExists),
        Err(_) => return Err(Refusal::Failed),
    };
    let label: Option<String> = profile
        .label
        .as_ref()
        .map(|text| text.chars().take(100).collect());
    sqlx::query(
        "INSERT INTO identities (user_id, provider, provider_user_id, provider_label)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(user_id)
    .bind(provider.as_str())
    .bind(&profile.provider_user_id)
    .bind(label)
    .execute(&mut *transaction)
    .await
    .map_err(|_| Refusal::Failed)?;
    transaction.commit().await.map_err(|_| Refusal::Failed)?;
    Ok(UserId::from_uuid(user_id))
}

/// Whether a database error is a unique-constraint violation.
fn is_unique_violation(error: &sqlx::Error) -> bool {
    matches!(error, sqlx::Error::Database(db) if db.is_unique_violation())
}

/// The callback URL for one provider. It is built from configuration only, never from the request.
fn callback_url(base_url: &str, provider: ProviderId) -> Result<Url, AuthError> {
    let text = format!(
        "{}/api/auth/oauth/{}/callback",
        base_url.trim_end_matches('/'),
        provider.as_str()
    );
    Url::parse(&text).map_err(|_| AuthError::Internal)
}
