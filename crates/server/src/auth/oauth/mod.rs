//! OAuth sign-in and linking: the providers we accept, the profile they return and the flow that
//! protects each round trip.
//!
//! A provider adapter only talks to its provider. It never decides who signs in; the pure
//! decisions in `auth::decision` do that, so the "no automatic linking" rule lives in one place.

pub mod flow;
pub mod github;
pub mod google;

use std::time::Duration;

use async_trait::async_trait;
use oauth2::{
    basic::BasicClient, AuthType, AuthUrl, AuthorizationCode, ClientId, ClientSecret,
    PkceCodeVerifier, RedirectUrl, TokenResponse, TokenUrl,
};
use reqwest::header::HeaderMap;
use serde::de::DeserializeOwned;
use thiserror::Error;
use url::Url;

use crate::auth::{config::Redacted, email::EmailAddress};

/// The longest provider response we read, in bytes. Anything bigger is refused.
pub const MAX_PROVIDER_BODY: u64 = 64 * 1024;

/// The providers we support. The database accepts only these two names.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ProviderId {
    /// GitHub, keyed by its numeric user id.
    GitHub,
    /// Google, keyed by the `sub` claim.
    Google,
}

impl ProviderId {
    /// The name stored in the database and used in URLs.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::GitHub => "github",
            Self::Google => "google",
        }
    }

    /// Reads a provider name from a URL or a stored row. Unknown names give `None`.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "github" => Some(Self::GitHub),
            "google" => Some(Self::Google),
            _ => None,
        }
    }
}

/// What a provider tells us about the person who signed in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderProfile {
    /// The provider's stable id for this person. It never changes, even if the email does.
    pub provider_user_id: String,
    /// A display name or login name, for the account page. Optional.
    pub label: Option<String>,
    /// The email address, only when the provider says it is verified and primary.
    pub verified_email: Option<EmailAddress>,
}

/// The provider refused, timed out or returned something we cannot use. The message says why
/// in plain words, never with the provider's response body.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("the provider could not complete the sign-in")]
pub struct ProviderError;

/// One identity provider. Adapters implement the code exchange and the profile lookup.
#[async_trait]
pub trait OAuthProvider: Send + Sync {
    /// Which provider this adapter talks to.
    fn id(&self) -> ProviderId;

    /// The URL the browser is sent to. It carries `state` and the S256 challenge.
    fn authorize_url(&self, state: &str, code_challenge: &str, redirect_uri: &Url) -> Url;

    /// Exchanges the code with the PKCE verifier and reads the profile.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderError`] when the exchange or the profile request fails.
    async fn fetch_profile(
        &self,
        code: &str,
        pkce_verifier: &str,
        redirect_uri: &Url,
    ) -> Result<ProviderProfile, ProviderError>;
}

/// The client id and secret of one registered OAuth app. The secret is never printed.
pub struct Credentials {
    /// The public client id from the provider's console.
    pub client_id: String,
    /// The client secret. Sent only in the token request.
    pub client_secret: Redacted<String>,
}

/// The two URLs of the authorization-code flow for one provider. Tests point them at a mock.
#[derive(Debug, Clone)]
pub struct Endpoints {
    /// Where the browser is sent to approve the sign-in.
    pub authorize: Url,
    /// Where the server exchanges the code for an access token.
    pub token: Url,
}

/// Builds the HTTP client for provider calls. It has timeouts and never follows redirects, so a
/// provider cannot send us to another host while we hold a token.
///
/// # Errors
///
/// Returns [`ProviderError`] when the client cannot be built.
pub fn http_client(headers: HeaderMap) -> Result<reqwest::Client, ProviderError> {
    reqwest::Client::builder()
        .default_headers(headers)
        .user_agent(concat!("counterspell/", env!("CARGO_PKG_VERSION")))
        .connect_timeout(Duration::from_secs(3))
        .timeout(Duration::from_secs(10))
        .redirect(reqwest::redirect::Policy::none())
        .build()
        .map_err(|_| ProviderError)
}

/// Exchanges an authorization code for an access token, sending the PKCE verifier. The access
/// token is returned to the caller and used once; nothing here logs it.
///
/// # Errors
///
/// Returns [`ProviderError`] when the provider refuses the code or cannot be reached.
pub async fn exchange_code(
    http: &reqwest::Client,
    credentials: &Credentials,
    endpoints: &Endpoints,
    code: &str,
    pkce_verifier: &str,
    redirect_uri: &Url,
) -> Result<String, ProviderError> {
    let client = BasicClient::new(ClientId::new(credentials.client_id.clone()))
        .set_client_secret(ClientSecret::new(
            credentials.client_secret.expose().clone(),
        ))
        .set_auth_type(AuthType::RequestBody)
        .set_auth_uri(AuthUrl::from_url(endpoints.authorize.clone()))
        .set_token_uri(TokenUrl::from_url(endpoints.token.clone()))
        .set_redirect_uri(RedirectUrl::from_url(redirect_uri.clone()));
    let token = client
        .exchange_code(AuthorizationCode::new(code.to_owned()))
        .set_pkce_verifier(PkceCodeVerifier::new(pkce_verifier.to_owned()))
        .request_async(http)
        .await
        .map_err(|_| ProviderError)?;
    Ok(token.access_token().secret().to_owned())
}

/// Reads one JSON profile document with the access token, refusing non-success answers and
/// bodies over [`MAX_PROVIDER_BODY`]. A failure never carries the provider's body.
///
/// # Errors
///
/// Returns [`ProviderError`] for a network error, a non-2xx answer, an oversized body or JSON
/// that does not match `T`.
pub async fn get_json<T: DeserializeOwned>(
    http: &reqwest::Client,
    url: Url,
    access_token: &str,
) -> Result<T, ProviderError> {
    let response = http
        .get(url)
        .bearer_auth(access_token)
        .send()
        .await
        .and_then(reqwest::Response::error_for_status)
        .map_err(|_| ProviderError)?;
    if response
        .content_length()
        .is_some_and(|length| length > MAX_PROVIDER_BODY)
    {
        return Err(ProviderError);
    }
    let bytes = response.bytes().await.map_err(|_| ProviderError)?;
    if u64::try_from(bytes.len()).map_or(true, |length| length > MAX_PROVIDER_BODY) {
        return Err(ProviderError);
    }
    serde_json::from_slice(&bytes).map_err(|_| ProviderError)
}

/// Builds the authorize URL: client id, redirect, scope, `state`, S256 challenge.
#[must_use]
pub fn authorize_url(
    endpoints: &Endpoints,
    client_id: &str,
    scope: &str,
    state: &str,
    code_challenge: &str,
    redirect_uri: &Url,
) -> Url {
    let mut url = endpoints.authorize.clone();
    url.query_pairs_mut()
        .append_pair("client_id", client_id)
        .append_pair("redirect_uri", redirect_uri.as_str())
        .append_pair("response_type", "code")
        .append_pair("scope", scope)
        .append_pair("state", state)
        .append_pair("code_challenge", code_challenge)
        .append_pair("code_challenge_method", "S256");
    url
}
