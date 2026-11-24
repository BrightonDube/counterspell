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
    ///
    /// # Errors
    /// Returns [`ApiError::internal`] when the backing store cannot answer: a lookup failure must
    /// not reach the client as `400 unsupported_language`, which would blame a valid request.
    async fn supports_language(&self, problem: &ProblemRef, lang: &str) -> Result<bool, ApiError>;
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

    async fn supports_language(
        &self,
        _problem: &ProblemRef,
        _lang: &str,
    ) -> Result<bool, ApiError> {
        Ok(false)
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
    ///
    /// Order is the launch order (python first — it is the workspace's default pick), with any
    /// future catalogue names appended in the file's own order.
    ///
    /// `server_cap` is `Some` only when the operator explicitly set `MAX_SOURCE_BYTES`: the cap
    /// then clamps each driver's own limit, so an operator override is never widened. When it is
    /// `None` every driver keeps its declared limit — the 64 KiB figure in config is a default,
    /// not a ceiling (system-design source-size contract).
    ///
    /// Two bounds always apply so an advertised limit stays reachable:
    /// `request_body_limit_bytes / 2` preserves the launch 2:1 source-to-body headroom, and
    /// [`MAX_ADVERTISED_SOURCE_BYTES`] keeps every entry inside the API/client contract range.
    #[must_use]
    pub fn from_enabled(
        catalogue: &counterspell_common::languages::LanguageCatalog,
        server_cap: Option<u64>,
        request_body_limit_bytes: u64,
    ) -> Self {
        let mut names: Vec<&str> = catalogue.names().collect();
        names.sort_by_key(|name| {
            LAUNCH_ORDER
                .iter()
                .position(|known| known == name)
                .unwrap_or(LAUNCH_ORDER.len())
        });
        let reachable_cap = (request_body_limit_bytes / 2).min(MAX_ADVERTISED_SOURCE_BYTES);
        let languages = names
            .into_iter()
            .filter_map(|name| {
                let definition = catalogue.get(name)?;
                definition.is_enabled().then(|| {
                    let driver_limit = definition.max_source_bytes();
                    Language {
                        name: name.to_owned(),
                        display_name: display_name(name).to_owned(),
                        max_source_bytes: server_cap
                            .map_or(driver_limit, |cap| driver_limit.min(cap))
                            .min(reachable_cap),
                    }
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

/// The product order for the accepted languages: python leads, matching `launch`.
const LAUNCH_ORDER: [&str; 4] = ["python", "javascript", "java", "cpp"];

/// The largest source limit the API may advertise: `MAX_SOURCE_BYTES` config range and the
/// client's `parseLanguageSummary` both bound accepted values at 1 MiB, so a driver declaring
/// more would make the whole `/api/languages` response unparseable (LC03).
const MAX_ADVERTISED_SOURCE_BYTES: u64 = 1_048_576;

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

    /// Builds a catalogue from a synthetic `languages.toml` with one enabled and one disabled
    /// entry. The enabled entry's driver must exist, so the base directory is the repo's real
    /// `languages/` tree — no temp directory (insecure-predictable-path rule).
    fn catalogue_with(
        enabled_name: &str,
        disabled_name: &str,
    ) -> counterspell_common::languages::LanguageCatalog {
        let base = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../languages"));
        let entry = |name: &str, enabled: bool, source_cap: u64| {
            format!(
                "[{name}]\nenabled = {enabled}\nimage = \"python:3.13-slim@sha256:e48f59cdba69a3cbd9ed2cf0f348d90d45a8d350d98134cf1070bc106ee9ed90\"\nfile = \"sol.py\"\ndriver = \"drivers/python/driver.py\"\ncompile = []\nrun = [\"python3\", \"main.py\"]\nbench_args = []\nmem_mb = 64\ntime_ms_per_test = 1000\nmax_time_ms_per_test = 5000\ncompile_budget_ms = 5000\nmax_source_bytes = {source_cap}\nexec_scratch = false\nprotocol_version = 1\n"
            )
        };
        let text = entry(enabled_name, true, 1234) + &entry(disabled_name, false, 99);
        counterspell_common::languages::LanguageCatalog::parse(&text, base)
            .expect("synthetic catalogue parses")
    }

    #[test]
    fn from_enabled_lists_only_enabled_drivers_with_their_own_limits() {
        let judge = catalogue_with("java", "cpp");
        let catalogue = LanguageCatalog::from_enabled(&judge, Some(65_536), 131_072);

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
    fn from_enabled_keeps_a_configured_cap_below_the_driver_limit() {
        let judge = catalogue_with("java", "cpp");
        let catalogue = LanguageCatalog::from_enabled(&judge, Some(512), 131_072);

        let java = catalogue.find("java").expect("java listed");
        assert_eq!(
            java.max_source_bytes, 512,
            "the operator's cap is a ceiling, not a floor"
        );
    }

    #[test]
    fn from_enabled_lets_a_wider_driver_limit_stand_when_no_cap_is_set() {
        let judge = catalogue_with("java", "cpp");
        let catalogue = LanguageCatalog::from_enabled(&judge, None, 131_072);

        let java = catalogue.find("java").expect("java listed");
        assert_eq!(
            java.max_source_bytes, 1234,
            "unset MAX_SOURCE_BYTES is not a ceiling — the driver's declared limit applies"
        );
    }

    #[test]
    fn from_enabled_never_advertises_above_the_request_capacity() {
        // A 2 MiB driver limit against the default 128 KiB request body: no source near
        // the driver limit fits the body, so the advertised limit drops to half the body
        // (the launch 2:1 source-to-body invariant).
        let base = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../languages"));
        let text = "[java]\nenabled = true\nimage = \"python:3.13-slim@sha256:e48f59cdba69a3cbd9ed2cf0f348d90d45a8d350d98134cf1070bc106ee9ed90\"\nfile = \"sol.py\"\ndriver = \"drivers/python/driver.py\"\ncompile = []\nrun = [\"python3\", \"main.py\"]\nbench_args = []\nmem_mb = 64\ntime_ms_per_test = 1000\nmax_time_ms_per_test = 5000\ncompile_budget_ms = 5000\nmax_source_bytes = 2097152\nexec_scratch = false\nprotocol_version = 1\n";
        let judge = counterspell_common::languages::LanguageCatalog::parse(text, base)
            .expect("synthetic catalogue parses");
        let catalogue = LanguageCatalog::from_enabled(&judge, None, 131_072);
        let java = catalogue.find("java").expect("java listed");
        assert_eq!(java.max_source_bytes, 65_536);
    }

    #[test]
    fn from_enabled_never_advertises_above_the_client_contract_bound() {
        let base = std::path::Path::new(concat!(env!("CARGO_MANIFEST_DIR"), "/../../languages"));
        let text = "[java]\nenabled = true\nimage = \"python:3.13-slim@sha256:e48f59cdba69a3cbd9ed2cf0f348d90d45a8d350d98134cf1070bc106ee9ed90\"\nfile = \"sol.py\"\ndriver = \"drivers/python/driver.py\"\ncompile = []\nrun = [\"python3\", \"main.py\"]\nbench_args = []\nmem_mb = 64\ntime_ms_per_test = 1000\nmax_time_ms_per_test = 5000\ncompile_budget_ms = 5000\nmax_source_bytes = 2097152\nexec_scratch = false\nprotocol_version = 1\n";
        let judge = counterspell_common::languages::LanguageCatalog::parse(text, base)
            .expect("synthetic catalogue parses");
        let catalogue = LanguageCatalog::from_enabled(&judge, None, 4_194_304);

        let java = catalogue.find("java").expect("java listed");
        assert_eq!(
            java.max_source_bytes, 1_048_576,
            "the client rejects advertised limits above 1 MiB"
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
        let catalogue = LanguageCatalog::from_enabled(&judge, Some(65_536), 131_072);

        let names: Vec<&str> = catalogue
            .entries()
            .iter()
            .map(|lang| lang.name.as_str())
            .collect();
        assert_eq!(
            names,
            ["python", "javascript", "java", "cpp"],
            "launch order, python first"
        );
    }
}
