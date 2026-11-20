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
pub mod pipeline;
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

use report::{Diagnostic, Report};
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
    validate_pack(&pack_name, dogfood_scope(root), false, &pack, &mut report);
    Ok(report)
}

/// Validates the public tree of a pack on its own (`--public`).
/// `reference/solution.py` and the other private files are private-resident
/// by policy, so a missing reference skips that stage instead of failing it;
/// every other check stays strict. Used by CI, which never sees the private
/// checkout.
///
/// # Errors
///
/// Returns an I/O error only when the pack directory itself cannot be read.
pub fn validate_static_public(root: &Path) -> io::Result<Report> {
    let pack_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_owned();
    let mut report = Report::new(pack_name.clone());
    let pack = pack::load_public(root, &mut report)?;
    validate_pack(&pack_name, dogfood_scope(root), true, &pack, &mut report);
    Ok(report)
}

/// Runs every static check on the pack split across `public_root` and
/// `private_root` ([`pack::load_pair`]). This is the production layout:
/// `reference/` and `incorrect/` live in the private tree.
///
/// # Errors
///
/// Returns an I/O error only when a tree directory cannot be read.
pub fn validate_static_pair(public_root: &Path, private_root: &Path) -> io::Result<Report> {
    let pack_name = public_root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default()
        .to_owned();
    let mut report = Report::new(pack_name.clone());
    let pack = pack::load_pair(public_root, private_root, &mut report)?;
    validate_pack(
        &pack_name,
        dogfood_scope(public_root),
        false,
        &pack,
        &mut report,
    );
    Ok(report)
}

/// The name the known-ghosts underscore rule (KNG-06) checks against. An
/// underscore marks test and dogfood packs, and it may sit on the pack leaf
/// (`packs/_demo`) or on its container (`packs/_demo/<slug>`); the leaf itself
/// must stay a valid slug (LAY-06), so the container carries the mark.
fn dogfood_scope(root: &Path) -> &str {
    let marked = |path: &Path| {
        path.file_name()
            .and_then(|name| name.to_str())
            .is_some_and(|name| name.starts_with('_'))
    };
    if marked(root) || root.parent().is_some_and(marked) {
        "_"
    } else {
        ""
    }
}

/// The shared check body behind [`validate_static`] and [`validate_static_pair`].
fn validate_pack(
    pack_name: &str,
    dogfood_scope: &str,
    public_only: bool,
    pack: &pack::Pack,
    report: &mut Report,
) {
    report.mark_checked("layout");

    match pack.text("hints.toml") {
        Some(text) => {
            hints::check(text, report);
            report.mark_checked("hints");
        }
        None => report.mark_skipped("hints"),
    }

    let Some(meta) = pack
        .text(META_FILE)
        .and_then(|text| meta::check(text, pack_name, report))
    else {
        report.mark_skipped("meta");
        for stage in META_STAGES {
            report.mark_skipped(stage);
        }
        return;
    };
    report.mark_checked("meta");

    signature::check(&meta, report);
    report.mark_checked("signature");
    limits::check(&meta, report);
    report.mark_checked("limits");

    // Amendment X1: the generator settings live in private `generator.toml`.
    // A `[generator]` table inside meta.toml was already flagged LAY-05 by the
    // meta check; the file form supplies the values instead.
    let mut meta = meta;
    match pack.text("generator.toml") {
        Some(text) => match toml::from_str::<meta::Generator>(text) {
            Ok(generator) => meta.generator = Some(generator),
            Err(error) => report.push(Diagnostic::new(
                codes::GEN_01,
                "generator.toml",
                None,
                format!("generator.toml does not parse: {error}"),
            )),
        },
        None if !public_only => report.push(Diagnostic::new(
            codes::LAY_01,
            "generator.toml",
            None,
            "required file is missing",
        )),
        None => {}
    }

    match pack.text("problem.md") {
        Some(text) => {
            statement::check(text, &meta.title, report);
            report.mark_checked("statement");
        }
        None => report.mark_skipped("statement"),
    }
    samples::check(&meta, report);
    report.mark_checked("samples");
    if public_only && meta.generator.is_none() {
        report.mark_skipped("generator");
    } else {
        generator::check(&meta, meta.generator.as_ref(), report);
        report.mark_checked("generator");
    }

    match pack.text("known_ghosts.toml") {
        Some(text) => {
            ghosts::check(text, &meta, dogfood_scope, pack, report);
            report.mark_checked("ghosts");
        }
        None => report.mark_skipped("ghosts"),
    }

    if public_only && pack.text("reference/solution.py").is_none() {
        // The reference is private-resident by design (X1); a public tree has
        // nothing to check, so the stage skips rather than warns.
        report.mark_skipped("reference");
    } else {
        reference::check(pack.text("reference/solution.py"), &meta.function, report);
        report.mark_checked("reference");
    }
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
    validate_sandbox_loader(root, runner, pack::load, validate_static)
}

/// [`validate_sandbox`] for the split public/private layout
/// ([`validate_static_pair`]).
///
/// # Errors
///
/// Returns [`SandboxFault::Io`] when a tree cannot be read, and
/// [`SandboxFault::Runner`] when the runner reports an infrastructure fault.
pub fn validate_sandbox_pair(
    public_root: &Path,
    private_root: &Path,
    runner: &mut dyn PackRunner,
) -> Result<Report, SandboxFault> {
    validate_sandbox_loader(
        public_root,
        runner,
        |root, report| pack::load_pair(root, private_root, report),
        |_| validate_static_pair(public_root, private_root),
    )
}

/// The shared body behind [`validate_sandbox`] and [`validate_sandbox_pair`].
fn validate_sandbox_loader(
    root: &Path,
    runner: &mut dyn PackRunner,
    load: impl Fn(&Path, &mut Report) -> io::Result<pack::Pack>,
    static_check: impl Fn(&Path) -> io::Result<Report>,
) -> Result<Report, SandboxFault> {
    let mut report = static_check(root).map_err(SandboxFault::Io)?;
    if report.has_errors() {
        report.mark_skipped("sandbox");
        return Ok(report);
    }
    let mut scratch = Report::new(String::new());
    let loaded = load(root, &mut scratch).map_err(SandboxFault::Io)?;
    let dir_name = root
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    let Some(mut meta) = loaded
        .text(META_FILE)
        .and_then(|text| meta::check(text, dir_name, &mut scratch))
    else {
        report.mark_skipped("sandbox");
        return Ok(report);
    };
    // The generator values live in private generator.toml (amendment X1); the
    // static pass already validated them, so this fill never reports.
    if let Some(text) = loaded.text("generator.toml") {
        if let Ok(generator) = toml::from_str::<meta::Generator>(text) {
            meta.generator = Some(generator);
        }
    }
    sandbox::check(&meta, &loaded, runner, &mut report).map_err(SandboxFault::Runner)?;
    report.mark_checked("sandbox");
    Ok(report)
}
