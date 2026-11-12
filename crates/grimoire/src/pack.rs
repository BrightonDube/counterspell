//! Pack discovery and file safety (LAY-*). Read only.
//!
//! The loader walks one pack directory without following symbolic links, checks
//! every file name against the allow-list, applies the size caps before reading
//! each file, and keeps only UTF-8 text with no forbidden control characters.

use crate::codes;
use crate::report::{Diagnostic, Report};
use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

/// The total size cap for one pack, in bytes (GRM-4).
pub const PACK_BYTE_CAP: u64 = 512 * 1024;

/// Files every pack must have.
const REQUIRED: [&str; 4] = [
    "problem.md",
    "meta.toml",
    "hints.toml",
    "reference/solution.py",
];

/// The optional file that lists authored known ghosts.
const GHOSTS_FILE: &str = "known_ghosts.toml";

/// The text of one pack's allowed files, keyed by path inside the pack.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Pack {
    files: BTreeMap<String, String>,
}

impl Pack {
    /// The text of one file, or `None` when the file was missing or rejected.
    #[must_use]
    pub fn text(&self, name: &str) -> Option<&str> {
        self.files.get(name).map(String::as_str)
    }
}

/// A file found by the directory walk, before its contents are read.
struct Candidate {
    name: String,
    path: PathBuf,
    size: u64,
}

/// Everything the directory walk found.
#[derive(Default)]
struct Scan {
    /// Every name seen, including names that were rejected. Used for LAY-01.
    seen: BTreeSet<String>,
    candidates: Vec<Candidate>,
}

/// Loads the pack at `root`, adding every layout finding to `report`.
///
/// # Errors
///
/// Returns an I/O error only when the pack directory itself cannot be read. A
/// bad file is a diagnostic, not an error.
pub fn load(root: &Path, report: &mut Report) -> io::Result<Pack> {
    let mut scan = Scan::default();
    scan_dir(root, "", false, report, &mut scan)?;

    let total: u64 = scan.candidates.iter().map(|candidate| candidate.size).sum();
    if total > PACK_BYTE_CAP {
        report.push(Diagnostic::new(
            codes::LAY_04,
            "pack",
            None,
            "the whole pack is over the 512 KB cap",
        ));
        return Ok(Pack::default());
    }

    let root_canonical = fs::canonicalize(root)?;
    let has_ghosts = scan
        .candidates
        .iter()
        .any(|candidate| candidate.name == GHOSTS_FILE);
    let mut pack = Pack::default();
    for candidate in &scan.candidates {
        if let Some(text) = check_candidate(candidate, has_ghosts, &root_canonical, report)? {
            pack.files.insert(candidate.name.clone(), text);
        }
    }

    for name in REQUIRED {
        if !scan.seen.contains(name) {
            report.push(Diagnostic::new(
                codes::LAY_01,
                name,
                None,
                "required file is missing",
            ));
        }
    }
    Ok(pack)
}

/// Walks one directory. Only `reference/` and `incorrect/` may hold files one level down.
fn scan_dir(
    dir: &Path,
    prefix: &str,
    nested: bool,
    report: &mut Report,
    scan: &mut Scan,
) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let Some(name) = file_name.to_str() else {
            report.push(Diagnostic::new(
                codes::LAY_03,
                file_name.to_string_lossy(),
                None,
                "file name is not valid UTF-8",
            ));
            continue;
        };
        let relative = if prefix.is_empty() {
            name.to_owned()
        } else {
            format!("{prefix}/{name}")
        };
        scan.seen.insert(relative.clone());

        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.file_type().is_symlink() {
            report.push(Diagnostic::new(
                codes::LAY_03,
                &relative,
                None,
                "symbolic links are not allowed",
            ));
        } else if name.starts_with('.') {
            report.push(Diagnostic::new(
                codes::LAY_02,
                &relative,
                None,
                "dot-prefixed files are not allowed",
            ));
        } else if metadata.is_dir() {
            if !nested && prefix.is_empty() && is_allowed_dir(name) {
                scan_dir(&entry.path(), &relative, true, report, scan)?;
            } else {
                report.push(Diagnostic::new(
                    codes::LAY_02,
                    &relative,
                    None,
                    "directory is not part of the pack format",
                ));
            }
        } else if metadata.is_file() {
            scan.candidates.push(Candidate {
                name: relative,
                path: entry.path(),
                size: metadata.len(),
            });
        }
    }
    Ok(())
}

/// Checks one file's name, size, location and text. Returns its text when it is allowed.
fn check_candidate(
    candidate: &Candidate,
    has_ghosts: bool,
    root_canonical: &Path,
    report: &mut Report,
) -> io::Result<Option<String>> {
    let name = candidate.name.as_str();
    if is_incorrect_program(name) && !has_ghosts {
        report.push(Diagnostic::new(
            codes::LAY_02,
            name,
            None,
            "incorrect programs need known_ghosts.toml",
        ));
        return Ok(None);
    }
    let Some(cap) = size_cap(name) else {
        report.push(Diagnostic::new(
            codes::LAY_02,
            name,
            None,
            "file is not part of the pack format",
        ));
        return Ok(None);
    };
    if candidate.size > cap {
        report.push(Diagnostic::new(
            codes::LAY_04,
            name,
            None,
            format!("file is {} bytes; the cap is {cap} bytes", candidate.size),
        ));
        return Ok(None);
    }
    if !fs::canonicalize(&candidate.path)?.starts_with(root_canonical) {
        report.push(Diagnostic::new(
            codes::LAY_03,
            name,
            None,
            "file resolves outside the pack",
        ));
        return Ok(None);
    }
    let bytes = fs::read(&candidate.path)?;
    match check_text(&bytes) {
        Ok(text) => Ok(Some(text)),
        Err(reason) => {
            report.push(Diagnostic::new(codes::LAY_05, name, None, reason));
            Ok(None)
        }
    }
}

/// The size cap in bytes for an allowed file, or `None` when the file is not allowed.
fn size_cap(name: &str) -> Option<u64> {
    match name {
        "problem.md" | "meta.toml" | GHOSTS_FILE => Some(32 * 1024),
        "hints.toml" => Some(8 * 1024),
        "reference/solution.py" => Some(64 * 1024),
        _ if is_incorrect_program(name) => Some(64 * 1024),
        _ => None,
    }
}

/// Whether `name` is an `incorrect/<file>.py` program directly inside `incorrect/`.
fn is_incorrect_program(name: &str) -> bool {
    name.strip_prefix("incorrect/").is_some_and(|rest| {
        !rest.contains('/') && Path::new(rest).extension().is_some_and(|ext| ext == "py")
    })
}

/// The two directories that may hold files one level down.
fn is_allowed_dir(name: &str) -> bool {
    matches!(name, "reference" | "incorrect")
}

/// Accepts UTF-8 text with no control characters except tab, line feed and carriage return.
fn check_text(bytes: &[u8]) -> Result<String, &'static str> {
    let text = std::str::from_utf8(bytes).map_err(|_| "file is not valid UTF-8")?;
    if text
        .chars()
        .any(|character| character.is_control() && !matches!(character, '\t' | '\n' | '\r'))
    {
        return Err("file contains a forbidden control character");
    }
    Ok(text.to_owned())
}
