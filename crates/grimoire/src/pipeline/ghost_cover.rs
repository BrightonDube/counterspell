//! Ghost coverage (PIP-13, PIP-18, GHC-01 to GHC-03).
//!
//! A pack needs at least two known ghosts. Each known ghost must reproduce in the
//! S11 report, and the reference must find no counterexample. The programs are named
//! as they appear in `known_ghosts.toml`, for example `incorrect/off_by_one.py`.

use serde::Deserialize;

use super::Diagnostic;

/// The fewest known ghosts a pack may have (PIP-13).
const MIN_KNOWN_GHOSTS: usize = 2;

/// One entry of `known_ghosts.toml` that this check needs. Other fields are ignored here.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct KnownGhost {
    pub program: String,
    pub kind: String,
}

/// One entry of the S11 report: whether a program's ghost reproduced, and as what kind.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct GhostReport {
    pub program: String,
    pub kind: String,
    pub verified: bool,
}

#[derive(Deserialize)]
struct KnownGhostFile {
    #[serde(default)]
    ghost: Vec<KnownGhost>,
}

#[derive(Deserialize)]
struct ReportFile {
    #[serde(default)]
    ghost: Vec<GhostReport>,
}

/// Reads the `[[ghost]]` entries of a `known_ghosts.toml` text.
///
/// # Errors
///
/// Returns the TOML error when the text is not valid TOML or an entry lacks `program` or `kind`.
pub fn parse_known_ghosts(text: &str) -> Result<Vec<KnownGhost>, toml::de::Error> {
    toml::from_str::<KnownGhostFile>(text).map(|file| file.ghost)
}

/// Reads the `[[ghost]]` entries of an S11 report text.
///
/// # Errors
///
/// Returns the TOML error when the text is not valid TOML or an entry lacks a field.
pub fn parse_reports(text: &str) -> Result<Vec<GhostReport>, toml::de::Error> {
    toml::from_str::<ReportFile>(text).map(|file| file.ghost)
}

/// Checks the count of known ghosts, and that each one reproduced with its recorded kind.
///
/// Returns `GHC-01` when there are fewer than two ghosts, and `GHC-02` (pointing at the
/// program) for each ghost with no verified report of the same program and kind.
#[must_use]
pub fn check_coverage(known: &[KnownGhost], reports: &[GhostReport]) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    if known.len() < MIN_KNOWN_GHOSTS {
        diagnostics.push(Diagnostic {
            code: "GHC-01",
            pointer: "known_ghosts.toml".to_owned(),
        });
    }
    for ghost in known {
        let reproduced = reports.iter().any(|report| {
            report.verified && report.program == ghost.program && report.kind == ghost.kind
        });
        if !reproduced {
            diagnostics.push(Diagnostic {
                code: "GHC-02",
                pointer: ghost.program.clone(),
            });
        }
    }
    diagnostics
}

/// Returns `GHC-03` when the reference found a counterexample, because a reference must find none.
#[must_use]
pub fn check_reference(counterexample_found: bool) -> Option<Diagnostic> {
    counterexample_found.then(|| Diagnostic {
        code: "GHC-03",
        pointer: "reference/solution.py".to_owned(),
    })
}
