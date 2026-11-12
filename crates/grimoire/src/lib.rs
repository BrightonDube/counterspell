//! Grimoire pack validation for `packs/<slug>/`.
//!
//! Static checks read files and never execute them. Sandbox checks go through
//! the [`runner::PackRunner`] trait, which the trusted judge implements.

use std::io;
use std::path::Path;

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

#[cfg(test)]
mod fixtures;

use report::Report;
use runner::PackRunner;
pub use sandbox::SandboxFault;

/// The file that holds the metadata.
const META_FILE: &str = "meta.toml";

/// Stages that need a parsed `meta.toml`. They are skipped when it fails.
const META_STAGES: [&str; 7] = [
    "signature",
    "limits",
    "statement",
    "samples",
    "generator",
    "ghosts",
    "reference",
];

/// Runs every static check on the pack at `root`, in the order the design lists.
///
/// Stages that need a parsed `meta.toml` are skipped when it fails, and the
/// report names every skipped stage, so an empty error list never looks like a
/// full pass. Reads files only; it never runs Python.
///
/// # Errors
///
/// Returns an I/O error only when the pack directory itself cannot be read.
pub fn validate_static(root: &Path) -> io::Result<Report> {
    let pack_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_owned();
    let mut report = Report::new(pack_name.clone());
    let pack = pack::load(root, &mut report)?;
    report.mark_checked("layout");

    match pack.text("hints.toml") {
        Some(text) => {
            hints::check(text, &mut report);
            report.mark_checked("hints");
        }
        None => report.mark_skipped("hints"),
    }

    let Some(meta) = pack
        .text(META_FILE)
        .and_then(|text| meta::check(text, &pack_name, &mut report))
    else {
        report.mark_skipped("meta");
        for stage in META_STAGES {
            report.mark_skipped(stage);
        }
        return Ok(report);
    };
    report.mark_checked("meta");

    signature::check(&meta, &mut report);
    report.mark_checked("signature");
    limits::check(&meta, &mut report);
    report.mark_checked("limits");

    match pack.text("problem.md") {
        Some(text) => {
            statement::check(text, &meta.title, &mut report);
            report.mark_checked("statement");
        }
        None => report.mark_skipped("statement"),
    }
    samples::check(&meta, &mut report);
    report.mark_checked("samples");
    generator::check(&meta, &mut report);
    report.mark_checked("generator");

    match pack.text("known_ghosts.toml") {
        Some(text) => {
            ghosts::check(text, &meta, &pack_name, &pack, &mut report);
            report.mark_checked("ghosts");
        }
        None => report.mark_skipped("ghosts"),
    }

    reference::check(
        pack.text("reference/solution.py"),
        &meta.function,
        &mut report,
    );
    report.mark_checked("reference");
    Ok(report)
}

/// Runs the static checks, then the sandbox checks through `runner`.
///
/// The sandbox stage runs only when the static checks have no errors, so a broken
/// pack never reaches the runner. Its report lists `sandbox` as skipped in that case.
///
/// # Errors
///
/// Returns [`SandboxFault::Io`] when a pack file cannot be read, and
/// [`SandboxFault::Runner`] when the runner reports an infrastructure fault.
pub fn validate_sandbox(root: &Path, runner: &mut dyn PackRunner) -> Result<Report, SandboxFault> {
    let mut report = validate_static(root).map_err(SandboxFault::Io)?;
    if report.has_errors() {
        report.mark_skipped("sandbox");
        return Ok(report);
    }
    let mut scratch = Report::new(String::new());
    let loaded = pack::load(root, &mut scratch).map_err(SandboxFault::Io)?;
    let dir_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let Some(meta) = loaded
        .text(META_FILE)
        .and_then(|text| meta::check(text, dir_name, &mut scratch))
    else {
        report.mark_skipped("sandbox");
        return Ok(report);
    };
    sandbox::check(&meta, &loaded, runner, &mut report).map_err(SandboxFault::Runner)?;
    report.mark_checked("sandbox");
    Ok(report)
}
