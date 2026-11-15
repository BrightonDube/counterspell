//! Response structs for the public catalogue API (problem-catalogue CAT-A8).
//!
//! Every field a client can see is listed here. Rows from the database never go straight to JSON,
//! so a column that is missing from these structs cannot leak.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// The body of `GET /api/problems/{slug}`. It holds the public parts of one published version
/// (CAT-A4, CAT-A5). Hidden tests, reference code, known ghosts and hint bodies have no field here.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProblemDetail {
    /// The URL-safe identifier.
    pub slug: String,
    /// The display title of the pinned version.
    pub title: String,
    /// The track the problem belongs to.
    pub track: TrackRef,
    /// `easy`, `medium` or `hard`.
    pub difficulty: String,
    /// The concept tags, sorted by slug.
    pub concepts: Vec<ConceptRef>,
    /// The statement in Markdown.
    pub statement_markdown: String,
    /// The bullet points under the statement's Constraints heading.
    pub constraints: Vec<String>,
    /// The public sample tests, in order.
    pub examples: Vec<Example>,
    /// The function a learner implements.
    pub signature: SignatureView,
    /// The starter code per language, keyed by language id.
    pub starters: BTreeMap<String, String>,
    /// The starter languages the server accepts right now.
    pub enabled_languages: Vec<String>,
    /// The per-test limits of the pinned version.
    pub limits: LimitsView,
    /// The number of authored hints. The hint bodies are never sent.
    pub hint_count: i64,
}

/// One public sample test.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Example {
    /// The sample input, as JSON.
    pub input: Value,
    /// The expected output for that input, as JSON.
    pub expected: Value,
}

/// The function signature a learner implements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SignatureView {
    /// The function name in the learner's code.
    pub function_name: String,
    /// The parameters, in call order.
    pub parameters: Vec<ParameterView>,
    /// The launch type name of the return value.
    pub return_type: String,
}

/// One parameter of the signature.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ParameterView {
    /// The parameter name.
    pub name: String,
    /// The launch type name, such as `int` or `list[int]`.
    #[serde(rename = "type")]
    pub kind: String,
}

/// The time and memory limits of a version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LimitsView {
    /// The per-test time limit in milliseconds.
    pub time_ms: i64,
    /// The memory limit in megabytes.
    pub mem_mb: i64,
}

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
