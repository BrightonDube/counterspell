//! A stand-in OAuth provider for tests. It accepts only the codes it was given, returns the
//! profile stored for each code, and never makes a network call.

use std::collections::HashMap;

use async_trait::async_trait;
use url::Url;

use crate::auth::oauth::{
    authorize_url, Endpoints, OAuthProvider, ProviderError, ProviderId, ProviderProfile,
};

/// A provider that answers from a fixed table of codes and profiles.
pub struct FakeProvider {
    id: ProviderId,
    endpoints: Endpoints,
    profiles: HashMap<String, ProviderProfile>,
}

impl FakeProvider {
    /// Builds the fake for one provider, with one profile per accepted code.
    ///
    /// # Errors
    ///
    /// Returns [`url::ParseError`] only if the fixed fake URL stops parsing.
    pub fn new(
        id: ProviderId,
        profiles: Vec<(&str, ProviderProfile)>,
    ) -> Result<Self, url::ParseError> {
        let authorize = Url::parse("https://fake.example/authorize")?;
        Ok(Self {
            id,
            endpoints: Endpoints {
                authorize: authorize.clone(),
                token: authorize,
            },
            profiles: profiles
                .into_iter()
                .map(|(code, profile)| (code.to_owned(), profile))
                .collect(),
        })
    }
}

#[async_trait]
impl OAuthProvider for FakeProvider {
    fn id(&self) -> ProviderId {
        self.id
    }

    fn authorize_url(&self, state: &str, code_challenge: &str, redirect_uri: &Url) -> Url {
        authorize_url(
            &self.endpoints,
            "fake-client",
            "fake",
            state,
            code_challenge,
            redirect_uri,
        )
    }

    async fn fetch_profile(
        &self,
        code: &str,
        _pkce_verifier: &str,
        _redirect_uri: &Url,
    ) -> Result<ProviderProfile, ProviderError> {
        self.profiles.get(code).cloned().ok_or(ProviderError)
    }
}
