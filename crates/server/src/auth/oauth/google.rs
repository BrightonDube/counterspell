//! Google sign-in: the authorization-code flow with PKCE, then the userinfo document.
//!
//! The `sub` claim is the stable key. The email is used only when Google marks it verified.
//! No ID token is parsed, so no JWT library is needed.

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

/// The scopes that give the stable id, the email and the display name.
const SCOPE: &str = "openid email profile";

/// The userinfo document. Unknown fields are ignored.
#[derive(Debug, Deserialize)]
struct GoogleUserinfo {
    sub: String,
    name: Option<String>,
    email: Option<String>,
    #[serde(default)]
    email_verified: bool,
}

/// The Google adapter. Tests point its URLs at a mock server.
pub struct GoogleProvider {
    credentials: Credentials,
    endpoints: Endpoints,
    userinfo_url: Url,
    http: reqwest::Client,
}

impl GoogleProvider {
    /// Builds the adapter with explicit URLs.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderError`] when the HTTP client cannot be built.
    pub fn new(
        credentials: Credentials,
        endpoints: Endpoints,
        userinfo_url: Url,
    ) -> Result<Self, ProviderError> {
        let mut headers = HeaderMap::new();
        headers.insert(ACCEPT, HeaderValue::from_static("application/json"));
        Ok(Self {
            credentials,
            endpoints,
            userinfo_url,
            http: http_client(headers)?,
        })
    }

    /// The adapter for Google's production endpoints.
    ///
    /// # Errors
    ///
    /// Returns [`ProviderError`] when the HTTP client cannot be built.
    pub fn production(
        client_id: String,
        client_secret: Redacted<String>,
    ) -> Result<Self, ProviderError> {
        let endpoints = Endpoints {
            authorize: Url::parse("https://accounts.google.com/o/oauth2/v2/auth")
                .map_err(|_| ProviderError)?,
            token: Url::parse("https://oauth2.googleapis.com/token").map_err(|_| ProviderError)?,
        };
        Self::new(
            Credentials {
                client_id,
                client_secret,
            },
            endpoints,
            Url::parse("https://openidconnect.googleapis.com/v1/userinfo")
                .map_err(|_| ProviderError)?,
        )
    }
}

#[async_trait]
impl OAuthProvider for GoogleProvider {
    fn id(&self) -> ProviderId {
        ProviderId::Google
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
        let info: GoogleUserinfo =
            get_json(&self.http, self.userinfo_url.clone(), &access_token).await?;
        let verified_email = match (info.email, info.email_verified) {
            (Some(email), true) => EmailAddress::parse(&email).ok(),
            _ => None,
        };
        Ok(ProviderProfile {
            provider_user_id: info.sub,
            label: info.name,
            verified_email,
        })
    }
}
