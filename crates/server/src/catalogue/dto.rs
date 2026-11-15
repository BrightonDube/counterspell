//! Response structs for the public catalogue API (problem-catalogue CAT-A8).
//!
//! Every field a client can see is listed here. Rows from the database never go straight to JSON,
//! so a column that is missing from these structs cannot leak.

use serde::{Deserialize, Serialize};

/// The body of `GET /api/problems`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProblemListResponse {
    /// The published problems that match the filters, in track order then slug order.
    pub problems: Vec<ProblemSummary>,
}

/// One problem in the list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProblemSummary {
    /// The URL-safe identifier.
    pub slug: String,
    /// The display title.
    pub title: String,
    /// The track the problem belongs to.
    pub track: TrackRef,
    /// `easy`, `medium` or `hard`.
    pub difficulty: String,
    /// The concept tags, sorted by slug.
    pub concepts: Vec<ConceptRef>,
    /// The caller's own progress, or `null` for an anonymous caller.
    pub status: Option<SolveStatus>,
}

/// A track, as the catalogue shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrackRef {
    /// The track's identifier.
    pub slug: String,
    /// The track's display name.
    pub name: String,
}

/// A concept tag, as the catalogue shows it.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConceptRef {
    /// The concept's identifier.
    pub slug: String,
    /// The concept's display name.
    pub name: String,
}

/// The caller's progress on one problem. It comes only from the caller's own submissions (CAT-A2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SolveStatus {
    /// A finished run was accepted.
    Solved,
    /// The caller has a run, and none was accepted.
    Attempted,
    /// The caller has no run for this problem.
    Unattempted,
}

impl SolveStatus {
    /// Reads the word the SQL status expression returns. Any other word gives `None`.
    #[must_use]
    pub fn from_sql(word: &str) -> Option<Self> {
        match word {
            "solved" => Some(Self::Solved),
            "attempted" => Some(Self::Attempted),
            "unattempted" => Some(Self::Unattempted),
            _ => None,
        }
    }
}
