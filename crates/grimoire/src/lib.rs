//! Grimoire pack validation for `packs/<slug>/`.
//!
//! Static checks read files and never execute them. Sandbox checks go through
//! the [`runner::PackRunner`] trait, which the trusted judge implements.

/// The package format version reserved for future pack validation.
pub const GRIMOIRE_VERSION: &str = "0.1.0";

pub mod codes;
pub mod generator;
pub mod ghosts;
pub mod hints;
pub mod limits;
pub mod meta;
pub mod pack;
pub mod record;
pub mod reference;
pub mod report;
pub mod runner;
pub mod samples;
pub mod sandbox;
pub mod signature;
pub mod statement;
