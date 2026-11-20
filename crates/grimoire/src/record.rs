//! Typed handoff record for the importer (GRM-54).
//!
//! A [`ProblemRecord`] is built only from a pack that passed every static check.
//! The crate has no database types; the server's import step turns the record
//! into a `problems` row.

use crate::limits::Limits;
use crate::meta::{self, Bench, Param};
use crate::pack;
use crate::report::Report;
use crate::signature::LaunchType;
use crate::validate_static;
use counterspell_engine::{Comparator, Schema};
use serde_json::{json, Value};
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

/// Maps a `meta.toml` type name to a driver-protocol `{"kind": ...}` descriptor.
fn type_descriptor(name: &str) -> Result<Value, String> {
    match LaunchType::parse(name) {
        Some(LaunchType::Int) => Ok(json!({"kind": "int"})),
        Some(LaunchType::Str) => Ok(json!({"kind": "string"})),
        Some(LaunchType::IntArray) => Ok(json!({"kind": "list", "items": {"kind": "int"}})),
        Some(LaunchType::Matrix) => Ok(json!({
            "kind": "list",
            "items": {"kind": "list", "items": {"kind": "int"}}
        })),
        None => Err(name.to_owned()),
    }
}

/// Builds the driver-protocol signature the judge writes to `signature.json` in the sandbox.
///
/// Parameters and the return type carry `{"kind": ...}` descriptors, not `meta.toml` type
/// strings.
///
/// # Errors
///
/// Returns the name of the first type that is not a launch type.
pub fn signature_descriptor(signature: &Signature) -> Result<Value, String> {
    let parameters = signature
        .params
        .iter()
        .map(|param| {
            type_descriptor(&param.kind).map(|kind| json!({"name": param.name, "type": kind}))
        })
        .collect::<Result<Vec<_>, _>>()?;
    Ok(json!({
        "protocol_version": 1,
        "function": signature.function,
        "parameters": parameters,
        "returns": type_descriptor(&signature.returns)?,
    }))
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
    build_with(
        || validate_static(root),
        |scratch| pack::load(root, scratch),
        root.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default(),
    )
}

/// [`build`] for the split public/private layout ([`crate::pack::load_pair`]):
/// `reference/` and `incorrect/` come from the private tree.
///
/// # Errors
///
/// Returns [`RecordError::Unreadable`] when a tree cannot be read and
/// [`RecordError::Invalid`] when the merged pack has static errors.
pub fn build_pair(public_root: &Path, private_root: &Path) -> Result<ProblemRecord, RecordError> {
    build_with(
        || crate::validate_static_pair(public_root, private_root),
        |scratch| pack::load_pair(public_root, private_root, scratch),
        public_root
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default(),
    )
}

/// The shared body behind [`build`] and [`build_pair`].
fn build_with(
    validate: impl FnOnce() -> io::Result<Report>,
    load: impl FnOnce(&mut Report) -> io::Result<pack::Pack>,
    dir_name: &str,
) -> Result<ProblemRecord, RecordError> {
    let report = validate().map_err(RecordError::Unreadable)?;
    if report.has_errors() {
        return Err(RecordError::Invalid(report));
    }
    let mut scratch = Report::new(String::new());
    let pack = load(&mut scratch).map_err(RecordError::Unreadable)?;
    let meta = pack
        .text("meta.toml")
        .and_then(|text| meta::check(text, dir_name, &mut scratch))
        .ok_or_else(|| RecordError::Invalid(report.clone()))?;
    let difficulty =
        Difficulty::parse(&meta.difficulty).ok_or_else(|| RecordError::Invalid(report.clone()))?;
    let version =
        u32::try_from(meta.pack_version).map_err(|_| RecordError::Invalid(report.clone()))?;
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
        comparator: meta.engine_comparator(),
        limits: meta.limits,
        reference_complexity: meta.reference_complexity.clone(),
        bench: meta.bench.clone(),
        complexity_note: meta.complexity_note.clone(),
        version,
        concepts: meta.concepts.clone(),
    })
}
