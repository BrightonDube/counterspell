//! Typed handoff record for the importer (GRM-54).
//!
//! A [`ProblemRecord`] is built only from a pack that passed every static check.
//! The crate has no database types; the server's import step turns the record
//! into a `problems` row.

use crate::limits::Limits;
use crate::meta::{self, Bench, Param};
use crate::pack;
use crate::report::Report;
use crate::validate_static;
use counterspell_engine::{Comparator, Schema};
use std::io;
use std::path::Path;

/// The difficulty levels a pack may declare.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Difficulty {
    /// For beginners.
    Easy,
    /// For practised learners.
    Medium,
    /// For interview-level problems.
    Hard,
}

impl Difficulty {
    /// Reads a difficulty name. Returns `None` for any other word.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "easy" => Some(Self::Easy),
            "medium" => Some(Self::Medium),
            "hard" => Some(Self::Hard),
            _ => None,
        }
    }
}

/// The function a learner implements.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Signature {
    /// The Python function name.
    pub function: String,
    /// The parameters, in call order.
    pub params: Vec<Param>,
    /// The return type name.
    pub returns: String,
}

/// Everything the importer needs about one valid pack.
#[derive(Clone, Debug, PartialEq)]
pub struct ProblemRecord {
    /// The problem id, equal to the directory name.
    pub slug: String,
    /// The catalogue title.
    pub title: String,
    /// The catalogue track.
    pub track: String,
    /// The difficulty level.
    pub difficulty: Difficulty,
    /// The statement markdown from `problem.md`.
    pub statement_md: String,
    /// The function signature.
    pub signature: Signature,
    /// The engine schema for the argument tuple.
    pub input_schema: Schema,
    /// The engine comparator.
    pub comparator: Comparator,
    /// The per-test limits.
    pub limits: Limits,
    /// The declared complexity class, when there is a bench.
    pub reference_complexity: Option<String>,
    /// The bench table, when there is one.
    pub bench: Option<Bench>,
    /// A short complexity note.
    pub complexity_note: Option<String>,
    /// The pack version.
    pub version: u32,
    /// The concept tags.
    pub concepts: Vec<String>,
}

/// Why a record could not be built.
#[derive(Debug)]
pub enum RecordError {
    /// The pack has at least one error. The report says which.
    Invalid(Report),
    /// The pack directory could not be read at all.
    Unreadable(io::Error),
}

/// Builds the record for the pack at `root`. Only a pack with zero errors gives a record.
///
/// # Errors
///
/// Returns [`RecordError::Invalid`] with the report when the pack has any error, and
/// [`RecordError::Unreadable`] when the directory cannot be read.
pub fn build(root: &Path) -> Result<ProblemRecord, RecordError> {
    let report = validate_static(root).map_err(RecordError::Unreadable)?;
    if report.has_errors() {
        return Err(RecordError::Invalid(report));
    }
    let mut scratch = Report::new(String::new());
    let pack = pack::load(root, &mut scratch).map_err(RecordError::Unreadable)?;
    let dir_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let meta = pack
        .text("meta.toml")
        .and_then(|text| meta::check(text, dir_name, &mut scratch))
        .ok_or_else(|| RecordError::Invalid(report.clone()))?;
    let difficulty =
        Difficulty::parse(&meta.difficulty).ok_or_else(|| RecordError::Invalid(report.clone()))?;
    let version =
        u32::try_from(meta.pack_version).map_err(|_| RecordError::Invalid(report.clone()))?;
    let comparator = if meta.comparator == "unordered" {
        Comparator::Unordered
    } else {
        Comparator::Exact
    };
    Ok(ProblemRecord {
        slug: meta.slug.clone(),
        title: meta.title.clone(),
        track: meta.track.clone(),
        difficulty,
        statement_md: pack.text("problem.md").unwrap_or_default().to_owned(),
        signature: Signature {
            function: meta.function.clone(),
            params: meta.params.clone(),
            returns: meta.returns.clone(),
        },
        input_schema: meta.input_schema.clone(),
        comparator,
        limits: meta.limits,
        reference_complexity: meta.reference_complexity.clone(),
        bench: meta.bench.clone(),
        complexity_note: meta.complexity_note.clone(),
        version,
        concepts: meta.concepts.clone(),
    })
}
