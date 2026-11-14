//! OAuth sign-in and linking: the providers we accept, the profile they return and the flow that
//! protects each round trip.
//!
//! A provider adapter only talks to its provider. It never decides who signs in; the pure
//! decisions in `auth::decision` do that, so the "no automatic linking" rule lives in one place.

pub mod flow;

use async_trait::async_trait;
use thiserror::Error;
use url::Url;

use crate::auth::email::EmailAddress;

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
