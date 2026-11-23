//! OAuth sign-in and linking: `GET /api/auth/oauth/{provider}/start` and `.../callback`.
//!
//! The callback never returns JSON. Success redirects to the saved place, and every failure
//! redirects to `/login?error=<code>`, or to `/settings/account?error=<code>` for a link flow.
//! The flow is taken first, so each state works once. The browser binding of a sign-in is
//! checked before the code is exchanged, and the binding cookie is cleared on every outcome.

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
        decision::{
            decide_link, decide_oauth_login, Actor, LinkDecision, OAuthDecision, ProfileEmail,
        },
        email::EmailAddress,
        errors::{is_unique_violation, AuthError},
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

/// Why a callback did not finish. The code is shown on the page the browser lands on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Refusal {
    /// The state is unknown, reused, expired, or the browser binding or session does not match.
    StateInvalid,
    /// The person refused at the provider.
    Denied,
    /// The code exchange or the profile request failed, or the server could not finish.
    Failed,
    /// The provider's verified email belongs to another account, so nothing was linked.
    AccountExists,
    /// The provider identity is linked to a different account. Nothing changed.
    IdentityAlreadyLinked,
    /// The account already has an identity for this provider. Nothing changed.
    ProviderAlreadyLinked,
    /// The provider is not enabled on this server.
    ProviderDisabled,
}

impl Refusal {
    /// The error code that the page reads from the query string.
    fn code(self) -> &'static str {
        match self {
            Self::StateInvalid => "state_invalid",
            Self::Denied => "oauth_denied",
            Self::Failed => "oauth_failed",
            Self::AccountExists => "account_exists",
            Self::IdentityAlreadyLinked => "identity_already_linked",
            Self::ProviderAlreadyLinked => "provider_already_linked",
            Self::ProviderDisabled => "provider_disabled",
        }
    }
}

/// A refused callback. The purpose is known only once the flow has been taken, so a refusal
/// before that point goes to the sign-in page.
struct Refused {
    /// The flow's purpose, or `None` when no flow was taken.
    purpose: Option<FlowPurpose>,
    /// The reason.
    refusal: Refusal,
}

impl Refused {
    /// A refusal before any flow was taken.
    fn early(refusal: Refusal) -> Self {
        Self {
            purpose: None,
            refusal,
        }
    }

    /// The page the browser lands on. A link flow returns to the account settings page.
    fn landing(&self) -> String {
        match self.purpose {
            Some(FlowPurpose::Link) => {
                format!("/settings/account?error={}", self.refusal.code())
            }
            _ => format!("/login?error={}", self.refusal.code()),
        }
    }
}

/// `GET /api/auth/oauth/{provider}/start`: sends the browser to the provider to sign in.
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
        Ok(landing) => Redirect::to(&landing).into_response(),
        Err(refused) => {
            tracing::info!(reason = refused.refusal.code(), "oauth callback refused");
            Redirect::to(&refused.landing()).into_response()
        }
    }
}

/// Takes the flow, then runs the steps for its purpose. Returns where to go on success.
async fn finish(
    state: &AuthState,
    slot: &CookieSlot,
    headers: &HeaderMap,
    provider_name: &str,
    query: CallbackQuery,
) -> Result<String, Refused> {
    let flow = take_flow(state, provider_name, query.state.as_deref()).await?;
    let purpose = flow.purpose();
    run_flow(state, slot, headers, &flow, query)
        .await
        .map_err(|refusal| Refused {
            purpose: Some(purpose),
            refusal,
        })
}

/// Takes the flow for the `state` value. The row is deleted even when the checks below fail.
async fn take_flow(
    state: &AuthState,
    provider_name: &str,
    raw_state: Option<&str>,
) -> Result<Flow, Refused> {
    let provider = ProviderId::parse(provider_name).ok_or(Refused::early(Refusal::StateInvalid))?;
    let raw = raw_state.ok_or(Refused::early(Refusal::StateInvalid))?;
    state
        .flows
        .consume(raw, provider)
        .await
        .map_err(|_| Refused::early(Refusal::Failed))?
        .ok_or(Refused::early(Refusal::StateInvalid))
}

/// The checks and the work for a taken flow, in their fixed order.
async fn run_flow(
    state: &AuthState,
    slot: &CookieSlot,
    headers: &HeaderMap,
    flow: &Flow,
    query: CallbackQuery,
) -> Result<String, Refusal> {
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
    let port = state.oauth_port().map_err(|_| Refusal::Failed)?;
    let adapter = port
        .providers
        .get(&flow.provider())
        .cloned()
        .ok_or(Refusal::ProviderDisabled)?;
    let redirect = callback_url(&port.base_url, flow.provider()).map_err(|_| Refusal::Failed)?;

    match flow.purpose() {
        FlowPurpose::Login => sign_in(state, slot, adapter.as_ref(), flow, code, &redirect).await,
        FlowPurpose::Link => {
            link_provider(
                state,
                slot,
                headers,
                adapter.as_ref(),
                flow,
                code,
                &redirect,
            )
            .await
        }
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
    let (user, created) = match decide_oauth_login(owner, email) {
        OAuthDecision::SignIn(user) => (user, false),
        OAuthDecision::RefuseEmailTaken => return Err(Refusal::AccountExists),
        OAuthDecision::CreateAccount { email } => (
            create_account(state, flow.provider(), &profile, email).await?,
            true,
        ),
    };

    let method = match flow.provider() {
        ProviderId::GitHub => AuthMethod::OauthGithub,
        ProviderId::Google => AuthMethod::OauthGoogle,
    };
    let pending =
        crate::auth::security::complete_primary(state, slot, user, method, flow.return_to())
            .await
            .map_err(|_| Refusal::Failed)?;
    if pending {
        return Ok(crate::auth::security::STEP_UP_PATH.to_owned());
    }
    let destination = sanitize_return_to(Some(flow.return_to()));
    if needs_onboarding(&state.pool, user, created).await? {
        let query = url::form_urlencoded::Serializer::new(String::new())
            .append_pair("return_to", &destination)
            .finish();
        return Ok(format!("/onboarding?{query}"));
    }
    Ok(destination)
}

/// Links the provider identity to the signed-in account that started the flow. Only the very
/// session that started the flow may finish it. The provider's email is never used.
async fn link_provider(
    state: &AuthState,
    slot: &CookieSlot,
    headers: &HeaderMap,
    adapter: &dyn OAuthProvider,
    flow: &Flow,
    code: &str,
    redirect: &Url,
) -> Result<String, Refusal> {
    let actor = flow_session_user(state, slot, headers, flow).await?;
    let profile = adapter
        .fetch_profile(code, flow.pkce_verifier(), redirect)
        .await
        .map_err(|_| Refusal::Failed)?;
    let provider = flow.provider();
    let owner = identity_owner(&state.pool, provider, &profile.provider_user_id).await?;
    let has_provider = actor_has_provider(&state.pool, actor.id, provider).await?;
    match decide_link(&actor, owner, has_provider) {
        LinkDecision::Link => insert_identity(state, actor.id, provider, &profile).await?,
        LinkDecision::AlreadyLinkedHere => {}
        LinkDecision::RefuseLinkedElsewhere => return Err(Refusal::IdentityAlreadyLinked),
        LinkDecision::RefuseProviderAlreadyLinked => {
            return Err(Refusal::ProviderAlreadyLinked);
        }
        // The demo account cannot start a link flow, so this is only a safe default.
        LinkDecision::RefuseDemo => return Err(Refusal::StateInvalid),
    }
    Ok(format!("/settings/account?linked={}", provider.as_str()))
}

/// The account of the browser's current session. The session must be the one the flow was
/// started from, so a flow cannot be finished from another browser or after logout.
async fn flow_session_user(
    state: &AuthState,
    slot: &CookieSlot,
    headers: &HeaderMap,
    flow: &Flow,
) -> Result<Actor, Refusal> {
    let raw = read_cookie(headers, state.cookies.session_name()).ok_or(Refusal::StateInvalid)?;
    let found = state
        .sessions
        .authenticate(raw)
        .await
        .map_err(|_| Refusal::Failed)?
        .ok_or(Refusal::StateInvalid)?;
    let same_session = flow.session_key() == Some(found.key.as_str());
    let same_user = flow.user() == Some(found.user_id);
    if !(same_session && same_user) {
        return Err(Refusal::StateInvalid);
    }
    if found.renewed {
        let cookie = state
            .cookies
            .session_value(raw)
            .map_err(|_| Refusal::Failed)?;
        slot.push(cookie);
    }
    Ok(Actor {
        id: found.user_id,
        is_demo: found.is_demo,
    })
}

/// Whether the account already has an identity for this provider.
async fn actor_has_provider(
    pool: &PgPool,
    user: UserId,
    provider: ProviderId,
) -> Result<bool, Refusal> {
    let (exists,): (bool,) = sqlx::query_as(
        "SELECT EXISTS (SELECT 1 FROM identities WHERE user_id = $1 AND provider = $2)",
    )
    .bind(user.as_uuid())
    .bind(provider.as_str())
    .fetch_one(pool)
    .await
    .map_err(|_| Refusal::Failed)?;
    Ok(exists)
}

/// Adds the identity to the account. When a parallel request linked first, the outcome is read
/// again, so the answer matches what the database now holds.
async fn insert_identity(
    state: &AuthState,
    user: UserId,
    provider: ProviderId,
    profile: &ProviderProfile,
) -> Result<(), Refusal> {
    let inserted = sqlx::query(
        "INSERT INTO identities (user_id, provider, provider_user_id, provider_label)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(user.as_uuid())
    .bind(provider.as_str())
    .bind(&profile.provider_user_id)
    .bind(provider_label(profile))
    .execute(&state.pool)
    .await;
    match inserted {
        Ok(_) => Ok(()),
        Err(error) if is_unique_violation(&error) => {
            match identity_owner(&state.pool, provider, &profile.provider_user_id).await? {
                Some(owner) if owner == user => Ok(()),
                Some(_) => Err(Refusal::IdentityAlreadyLinked),
                None => Err(Refusal::ProviderAlreadyLinked),
            }
        }
        Err(_) => Err(Refusal::Failed),
    }
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

/// Whether this login must pass through profile onboarding.
async fn needs_onboarding(pool: &PgPool, user: UserId, created: bool) -> Result<bool, Refusal> {
    if created {
        return Ok(true);
    }
    let (incomplete,): (bool,) =
        sqlx::query_as("SELECT profile_completed_at IS NULL FROM users WHERE id = $1")
            .bind(user.as_uuid())
            .fetch_one(pool)
            .await
            .map_err(|_| Refusal::Failed)?;
    Ok(incomplete)
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
    sqlx::query(
        "INSERT INTO identities (user_id, provider, provider_user_id, provider_label)
         VALUES ($1, $2, $3, $4)",
    )
    .bind(user_id)
    .bind(provider.as_str())
    .bind(&profile.provider_user_id)
    .bind(provider_label(profile))
    .execute(&mut *transaction)
    .await
    .map_err(|_| Refusal::Failed)?;
    transaction.commit().await.map_err(|_| Refusal::Failed)?;
    Ok(UserId::from_uuid(user_id))
}

/// The provider's display name, cut to the length the table allows. It is never used to find an
/// account.
fn provider_label(profile: &ProviderProfile) -> Option<String> {
    profile
        .label
        .as_ref()
        .map(|text| text.chars().take(100).collect())
}

/// The callback URL for one provider. It is built from configuration only, never from the request.
pub(crate) fn callback_url(base_url: &str, provider: ProviderId) -> Result<Url, AuthError> {
    let text = format!(
        "{}/api/auth/oauth/{}/callback",
        base_url.trim_end_matches('/'),
        provider.as_str()
    );
    Url::parse(&text).map_err(|_| AuthError::Internal)
}
