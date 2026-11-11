//! Placeholder Grimoire pack model.
//!
//! Pack parsing and validation are deferred. This package owns both the future
//! library and its `validate` binary so no second validator crate is needed.

/// The package format version reserved for future pack validation.
pub const GRIMOIRE_VERSION: &str = "0.1.0";
