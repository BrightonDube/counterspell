//! Seams: the places where later specs plug in real behaviour.
//!
//! The run-lifecycle server only needs three answers: who is calling, whether a problem exists and
//! is published, and which languages and source sizes are allowed. Production uses the safe defaults
//! here (nobody is signed in, no problems exist, two languages). The auth and catalogue specs replace
//! them. Tests use the versions in `testkit`.

use std::future::Future;

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, HeaderMap},
};
use counterspell_common::ids::UserId;

use crate::{error::ApiError, AppState};

/// Works out who is calling from the request headers. Returns `None` for anonymous callers.
pub trait UserResolver: Send + Sync {
    /// Returns the signed-in user for these headers, or `None`.
    fn resolve(&self, headers: &HeaderMap) -> Option<UserId>;
}

/// The production resolver until the auth spec lands. It never identifies anyone, so every
/// signed-in route answers `401`.
#[derive(Debug, Clone, Copy, Default)]
pub struct Unauthenticated;

impl UserResolver for Unauthenticated {
    fn resolve(&self, _headers: &HeaderMap) -> Option<UserId> {
        None
    }
}

/// A published problem, as far as the run flow needs to know.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProblemRef {
    /// The problem's URL-safe identifier.
    pub slug: String,
}

/// Finds published problems by slug.
pub trait ProblemLookup: Send + Sync {
    /// Returns the problem when it exists and is published.
    fn published(&self, slug: &str) -> Option<ProblemRef>;
}

/// The production catalogue until the grimoire import lands. It publishes nothing, so every run
/// request for a problem answers `404 unknown_problem`.
#[derive(Debug, Clone, Copy, Default)]
pub struct EmptyCatalogue;

impl ProblemLookup for EmptyCatalogue {
    fn published(&self, _slug: &str) -> Option<ProblemRef> {
        None
    }
}

/// One language the server accepts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Language {
    /// The name the client sends, for example `python`. Case sensitive.
    pub name: String,
    /// The largest accepted source for this language, in bytes.
    pub max_source_bytes: u64,
}

/// The languages the server accepts. The launch set is Python and JavaScript.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageCatalog {
    languages: Vec<Language>,
}

impl LanguageCatalog {
    /// The launch catalogue: `python` and `javascript`, each with the same source limit.
    #[must_use]
    pub fn launch(max_source_bytes: u64) -> Self {
        let languages = ["python", "javascript"]
            .into_iter()
            .map(|name| Language {
                name: name.to_owned(),
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
}

/// The signed-in user for a request. Use it as a handler argument; a missing user becomes `401`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrentUser(pub UserId);

impl FromRequestParts<AppState> for CurrentUser {
    type Rejection = ApiError;

    fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        let result = state
            .user_resolver
            .resolve(&parts.headers)
            .map(CurrentUser)
            .ok_or_else(ApiError::unauthorized);
        std::future::ready(result)
    }
}
