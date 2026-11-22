//! Seams: the places where later specs plug in real behaviour.
//!
//! The run-lifecycle server only needs three answers: who is calling, whether a problem exists and
//! is published, and which languages and source sizes are allowed. Production uses the safe defaults
//! here (nobody is signed in, no problems exist, two languages). The auth and catalogue specs replace
//! them. Tests use the versions in `testkit`.

use async_trait::async_trait;
use axum::{
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap},
};
use counterspell_common::ids::UserId;

use crate::{error::ApiError, AppState};

/// Works out who is calling from the request headers. Returns `None` for anonymous callers.
///
/// It is async because the session resolver checks the database.
#[async_trait]
pub trait UserResolver: Send + Sync {
    /// Returns the signed-in user for these headers, or `None`.
    async fn resolve(&self, headers: &HeaderMap) -> Option<UserId>;

    /// Says whether `user` is still signed in for these headers. A long-lived stream calls this on
    /// each heartbeat and closes when it is `false`, so a logout ends a stream that is already open.
    ///
    /// The default asks `resolve` again. A resolver that renews sessions as a side effect should
    /// override it with a read-only check, so an open stream cannot keep a session alive.
    async fn still_signed_in(&self, headers: &HeaderMap, user: UserId) -> bool {
        self.resolve(headers).await == Some(user)
    }
}

/// The production resolver until the auth spec lands. It never identifies anyone, so every
/// signed-in route answers `401`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Unauthenticated;

#[async_trait]
impl UserResolver for Unauthenticated {
    async fn resolve(&self, _headers: &HeaderMap) -> Option<UserId> {
        None
    }
}

/// A published problem, as far as the run flow needs to know.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProblemRef {
    /// The problem's URL-safe identifier.
    pub slug: String,
    /// The version that is published now. A run is pinned to this version (CAT-L1).
    pub version: i32,
}

/// Finds published problems by slug.
#[async_trait]
pub trait ProblemLookup: Send + Sync {
    /// Returns the problem when it exists and is published, with the version to pin.
    async fn published(&self, slug: &str) -> Option<ProblemRef>;
}

/// The production catalogue until the grimoire import lands. It publishes nothing, so every run
/// request for a problem answers `404 unknown_problem`.
#[derive(Debug, Clone, Copy, Default)]
pub struct EmptyCatalogue;

#[async_trait]
impl ProblemLookup for EmptyCatalogue {
    async fn published(&self, _slug: &str) -> Option<ProblemRef> {
        None
    }
}

/// One language the server accepts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Language {
    /// The name the client sends, for example `python`. Case sensitive.
    pub name: String,
    /// The human-facing label, for example `Python`.
    pub display_name: String,
    /// The largest accepted source for this language, in bytes.
    pub max_source_bytes: u64,
}

/// The languages the server accepts: the launch set plus the stretch languages
/// whose judge-side suites passed (language-drivers T16).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageCatalog {
    languages: Vec<Language>,
}

impl LanguageCatalog {
    /// The accepted catalogue: `python`, `javascript`, `java`, and `cpp`, each
    /// with the same source limit.
    #[must_use]
    pub fn launch(max_source_bytes: u64) -> Self {
        let languages = [
            ("python", "Python"),
            ("javascript", "JavaScript"),
            ("java", "Java"),
            ("cpp", "C++"),
        ]
        .into_iter()
        .map(|(name, display_name)| Language {
            name: name.to_owned(),
            display_name: display_name.to_owned(),
            max_source_bytes,
        })
        .collect();
        Self { languages }
    }

    /// Finds an enabled language by its exact name.
    #[must_use]
    pub fn find(&self, name: &str) -> Option<&Language> {
        self.languages.iter().find(|language| language.name == name)
    }

    /// Every accepted language, in catalogue order. Disabled stretch languages
    /// are never in the catalogue, so they cannot be listed.
    #[must_use]
    pub fn entries(&self) -> &[Language] {
        &self.languages
    }
}

/// The signed-in user for a request. Use it as a handler argument; a missing user becomes `401`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentUser(pub UserId);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        state
            .user_resolver
            .resolve(&parts.headers)
            .await
            .map(CurrentUser)
            .ok_or_else(ApiError::unauthenticated)
    }
}
