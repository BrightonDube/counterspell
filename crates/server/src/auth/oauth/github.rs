//! GitHub sign-in: the authorization-code flow with PKCE, then the user and their emails.
//!
//! GitHub's numeric user id is the stable key, because logins can be renamed. Only the primary,
//! verified email is used, and only as information. It never decides which account signs in.

use async_trait::async_trait;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT};
use serde::Deserialize;
use url::Url;

use crate::auth::{
    config::Redacted,
    email::EmailAddress,
    oauth::{
        authorize_url, exchange_code, get_json, http_client, Credentials, Endpoints, OAuthProvider,
        ProviderError, ProviderId, ProviderProfile,
    },
};

/// The scopes needed to read the user and their emails.
const SCOPE: &str = "read:user user:email";

/// One user record from `GET /user`. Unknown fields are ignored.
#[derive(Debug, Deserialize)]
struct GithubUser {
    id: u64,
    login: Option<String>,
}

/// One entry from `GET /user/emails`.
#[derive(Debug, Deserialize)]
struct GithubEmail {
    email: String,
    primary: bool,
    verified: bool,
}

/// The GitHub adapter. Tests point its URLs at a mock server.
pub struct GithubProvider {
    credentials: Credentials,
    endpoints: Endpoints,
    user_url: Url,
    emails_url: Url,
    http: reqwest::Client,
}

impl GithubProvider {
    /// Builds the adapter with explicit URLs.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderError`] when the HTTP client cannot be built.
    pub fn new(
        credentials: Credentials,
        endpoints: Endpoints,
        user_url: Url,
        emails_url: Url,
    ) -> Result<Self, ProviderError> {
        let mut headers = HeaderMap::new();
        // GitHub answers the token request in form encoding unless JSON is asked for.
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        Ok(Self {
            credentials,
            endpoints,
            user_url,
            emails_url,
            http: http_client(headers)?,
        })
    }

    /// The adapter for github.com and api.github.com.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderError`] when the HTTP client cannot be built.
    pub fn production(
        client_id: String,
        client_secret: Redacted<String>,
    ) -> Result<Self, ProviderError> {
        let endpoints = Endpoints {
            authorize: Url::parse("https://github.com/login/oauth/authorize")
                .map_err(|_| ProviderError)?,
            token: Url::parse("https://github.com/login/oauth/access_token")
                .map_err(|_| ProviderError)?,
        };
        Self::new(
            Credentials {
                client_id,
                client_secret,
            },
            endpoints,
            Url::parse("https://api.github.com/user").map_err(|_| ProviderError)?,
            Url::parse("https://api.github.com/user/emails").map_err(|_| ProviderError)?,
        )
    }
}

/// Picks the email GitHub marks as both primary and verified, if there is one.
fn primary_verified(emails: &[GithubEmail]) -> Option<EmailAddress> {
    emails
        .iter()
        .find(|entry| entry.primary && entry.verified)
        .and_then(|entry| EmailAddress::parse(&entry.email).ok())
}

#[async_trait]
impl OAuthProvider for GithubProvider {
    fn id(&self) -> ProviderId {
        ProviderId::GitHub
    }

    fn authorize_url(&self, state: &str, code_challenge: &str, redirect_uri: &Url) -> Url {
        authorize_url(
            &self.endpoints,
            &self.credentials.client_id,
            SCOPE,
            state,
            code_challenge,
            redirect_uri,
        )
    }

    async fn fetch_profile(
        &self,
        code: &str,
        pkce_verifier: &str,
        redirect_uri: &Url,
    ) -> Result<ProviderProfile, ProviderError> {
        let access_token = exchange_code(
            &self.http,
            &self.credentials,
            &self.endpoints,
            code,
            pkce_verifier,
            redirect_uri,
        )
        .await?;
        let user: GithubUser = get_json(&self.http, self.user_url.clone(), &access_token).await?;
        let emails: Vec<GithubEmail> =
            get_json(&self.http, self.emails_url.clone(), &access_token).await?;
        Ok(ProviderProfile {
            provider_user_id: user.id.to_string(),
            label: user.login,
            verified_email: primary_verified(&emails),
        })
    }
}
