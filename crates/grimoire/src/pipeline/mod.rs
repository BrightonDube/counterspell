//! Pack authoring pipeline logic (`pack-authoring-pipeline` spec, design §4 and §10).
//!
//! Everything here is pure. It parses and checks text that is passed in, and it
//! never reads files, runs processes or opens sockets. Hidden values are passed in
//! memory and never returned. The scripts under `scripts/` move the files.

pub mod hint_leak;
pub mod loop_cap;
pub mod originality;
pub mod review;
pub mod state;

/// One finding from a pipeline check. `pointer` is a file and line, a rung, or a value index.
/// It never holds a hidden value or matched text (PIP-32).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Diagnostic {
    /// The stable code, for example `ORG-01` or `HNT-03`.
    pub code: &'static str,
    /// Where the finding is. Never the value that caused it.
    pub pointer: String,
}
