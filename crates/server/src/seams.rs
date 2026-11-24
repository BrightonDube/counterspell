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

    /// Whether `problem`'s published version offers `lang` — that is, a starter row exists for it.
    /// The caller has already checked that `lang` is an enabled catalogue language (LC05).
    async fn supports_language(&self, problem: &ProblemRef, lang: &str) -> bool;
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

    async fn supports_language(&self, _problem: &ProblemRef, _lang: &str) -> bool {
        false
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

    /// The accepted catalogue derived from the judge's language file: only languages with an
    /// enabled driver are listed, so a disabled stretch language is never advertised or accepted
    /// (language-consistency LC03). Source limits come from the same file the executor enforces.
    /// The parser lives in `counterspell-common` — the server never depends on the judge.
    #[must_use]
    pub fn from_enabled(catalogue: &counterspell_common::languages::LanguageCatalog) -> Self {
        let languages = catalogue
            .names()
            .filter_map(|name| {
                let definition = catalogue.get(name)?;
                definition.is_enabled().then(|| Language {
                    name: name.to_owned(),
                    display_name: display_name(name).to_owned(),
                    max_source_bytes: definition.max_source_bytes(),
                })
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

/// The human-facing label for a catalogue language name; unknown names keep their id.
fn display_name(name: &str) -> &str {
    match name {
        "python" => "Python",
        "javascript" => "JavaScript",
        "java" => "Java",
        "cpp" => "C++",
        _ => name,
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

#[cfg(test)]
mod tests {
    use super::LanguageCatalog;
    use std::fs;

    /// Writes a minimal valid `languages.toml` with `enabled`/`disabled` entries.
    /// The enabled entry needs a real driver file under the base directory.
    fn catalogue_with(
        enabled_name: &str,
        disabled_name: &str,
    ) -> (
        std::path::PathBuf,
        counterspell_common::languages::LanguageCatalog,
    ) {
        let base = std::env::temp_dir().join(format!("lc-seams-{}", std::process::id()));
        fs::create_dir_all(&base).expect("temp dir");
        fs::write(base.join("driver.py"), "# driver\n").expect("driver file");
        let entry = |name: &str, enabled: bool, source_cap: u64| {
            format!(
                "[{name}]\nenabled = {enabled}\nimage = \"python:3.13-slim@sha256:e48f59cdba69a3cbd9ed2cf0f348d90d45a8d350d98134cf1070bc106ee9ed90\"\nfile = \"sol.py\"\ndriver = \"driver.py\"\ncompile = []\nrun = [\"python3\", \"driver.py\"]\nbench_args = []\nmem_mb = 64\ntime_ms_per_test = 1000\nmax_time_ms_per_test = 5000\ncompile_budget_ms = 5000\nmax_source_bytes = {source_cap}\nexec_scratch = false\nprotocol_version = 1\n"
            )
        };
        let text = entry(enabled_name, true, 1234) + &entry(disabled_name, false, 99);
        let catalogue = counterspell_common::languages::LanguageCatalog::parse(&text, &base)
            .expect("synthetic catalogue parses");
        (base, catalogue)
    }

    #[test]
    fn from_enabled_lists_only_enabled_drivers_with_their_own_limits() {
        let (_base, judge) = catalogue_with("java", "cpp");
        let catalogue = LanguageCatalog::from_enabled(&judge);

        let names: Vec<&str> = catalogue
            .entries()
            .iter()
            .map(|lang| lang.name.as_str())
            .collect();
        assert_eq!(names, ["java"], "disabled drivers must not be listed");
        let java = catalogue.find("java").expect("java listed");
        assert_eq!(java.display_name, "Java");
        assert_eq!(
            java.max_source_bytes, 1234,
            "the driver's own limit, not a server default"
        );
        assert!(
            catalogue.find("cpp").is_none(),
            "cpp is disabled and refused"
        );
    }

    #[test]
    fn from_enabled_over_the_repo_catalogue_lists_all_four_drivers() {
        let path = std::path::Path::new(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../languages/languages.toml"
        ));
        let judge = counterspell_common::languages::LanguageCatalog::load(path)
            .expect("repo catalogue loads");
        let catalogue = LanguageCatalog::from_enabled(&judge);

        let names: Vec<&str> = catalogue
            .entries()
            .iter()
            .map(|lang| lang.name.as_str())
            .collect();
        assert_eq!(names, ["cpp", "java", "javascript", "python"]);
    }
}
