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

/// The files a public-tree pack must have when its private side is absent
/// ( validation):  is private-resident by policy.
const REQUIRED_PUBLIC: [&str; 3] = ["problem.md", "meta.toml", "hints.toml"];

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
    /// The tree root this file was found under (public or private), for the
    /// symlink-containment check.
    tree_root: PathBuf,
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
    load_scan(root, None, &REQUIRED, report)
}

/// Loads the pack split across a public tree and its private tree.
///
/// The private tree contributes only `reference/`, `incorrect/` and
/// `known_ghosts.toml` to the pack file set; catalogue files there
/// (`hidden.toml`, `generator.toml`, `approval.toml`, `retired-values.toml`)
/// are not pack files and are ignored. Private copies shadow public ones,
/// mirroring the catalogue's private-first convention.
///
/// # Errors
///
/// Returns an I/O error only when a tree directory itself cannot be read.
pub fn load_pair(public_root: &Path, private_root: &Path, report: &mut Report) -> io::Result<Pack> {
    load_scan(public_root, Some(private_root), &REQUIRED, report)
}

/// [] for a public tree checked on its own.  is
/// private-resident by policy, so it is not required here; a public copy is
/// still checked when present.
///
/// # Errors
///
/// Returns an I/O error only when the tree directory itself cannot be read.
pub fn load_public(root: &Path, report: &mut Report) -> io::Result<Pack> {
    load_scan(root, None, &REQUIRED_PUBLIC, report)
}

/// The shared walk behind [`load`] and [`load_pair`].
fn load_scan(
    public_root: &Path,
    private_root: Option<&Path>,
    required: &[&str],
    report: &mut Report,
) -> io::Result<Pack> {
    let mut scan = Scan::default();
    scan_dir(public_root, "", false, public_root, report, &mut scan)?;
    if let Some(private_root) = private_root {
        scan_private_root(private_root, report, &mut scan)?;
    }

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

    let has_ghosts = scan
        .candidates
        .iter()
        .any(|candidate| candidate.name == GHOSTS_FILE);
    let mut pack = Pack::default();
    for candidate in &scan.candidates {
        if let Some(text) = check_candidate(candidate, has_ghosts, report)? {
            pack.files.insert(candidate.name.clone(), text);
        }
    }

    for &name in required {
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

/// Walks the private tree for pack files. Only `reference/`, `incorrect/` and
/// `known_ghosts.toml` join the pack set — every other entry is catalogue data
/// and is skipped, though symbolic links are still refused.
fn scan_private_root(dir: &Path, report: &mut Report, scan: &mut Scan) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_name = entry.file_name();
        let Some(name) = file_name.to_str() else {
            continue;
        };
        let metadata = fs::symlink_metadata(entry.path())?;
        if metadata.file_type().is_symlink() {
            report.push(Diagnostic::new(
                codes::LAY_03,
                name,
                None,
                "symbolic links are not allowed",
            ));
            continue;
        }
        if name.starts_with('.') {
            continue;
        }
        if metadata.is_dir() {
            if name == "reference" || name == "incorrect" {
                scan_dir(&entry.path(), name, true, dir, report, scan)?;
            }
        } else if metadata.is_file() && name == GHOSTS_FILE {
            scan.seen.insert(name.to_owned());
            scan.candidates.push(Candidate {
                name: name.to_owned(),
                path: entry.path(),
                size: metadata.len(),
                tree_root: dir.to_path_buf(),
            });
        }
    }
    Ok(())
}

/// Walks one directory. Only `reference/` and `incorrect/` may hold files one level down.
fn scan_dir(
    dir: &Path,
    prefix: &str,
    nested: bool,
    tree_root: &Path,
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
            // `fixtures/` nests one level deeper: `fixtures/<lang>/<file>`.
            if (!nested && prefix.is_empty() && is_allowed_dir(name))
                || (nested && prefix == "fixtures")
            {
                scan_dir(&entry.path(), &relative, true, tree_root, report, scan)?;
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
                tree_root: tree_root.to_path_buf(),
            });
        }
    }
    Ok(())
}

/// Checks one file's name, size, location and text. Returns its text when it is allowed.
fn check_candidate(
    candidate: &Candidate,
    has_ghosts: bool,
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
    if !fs::canonicalize(&candidate.path)?.starts_with(fs::canonicalize(&candidate.tree_root)?) {
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
        _ if is_fixture_program(name) => Some(64 * 1024),
        _ => None,
    }
}

/// Whether `name` is a `fixtures/<lang>/known_correct.<ext>` test fixture
/// (language-drivers T10). Fixtures are test-only programs; they never join
/// the reference or incorrect sets.
fn is_fixture_program(name: &str) -> bool {
    let Some(rest) = name.strip_prefix("fixtures/") else {
        return false;
    };
    let Some((lang, file)) = rest.split_once('/') else {
        return false;
    };
    !lang.is_empty()
        && !file.contains('/')
        && file
            .strip_prefix("known_correct.")
            .is_some_and(|ext| !ext.is_empty() && !ext.contains('.'))
}

/// Whether `name` is an `incorrect/<file>.py` program directly inside `incorrect/`.
fn is_incorrect_program(name: &str) -> bool {
    name.strip_prefix("incorrect/").is_some_and(|rest| {
        !rest.contains('/') && Path::new(rest).extension().is_some_and(|ext| ext == "py")
    })
}

/// The three directories that may hold files one level down. `fixtures/`
/// holds one `<lang>` directory per enabled language (language-drivers T10).
fn is_allowed_dir(name: &str) -> bool {
    matches!(name, "reference" | "incorrect" | "fixtures")
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

#[cfg(test)]
impl Pack {
    /// Builds a pack from `(name, text)` pairs. Used only by unit tests.
    pub(crate) fn with_text(files: &[(&str, &str)]) -> Self {
        Self {
            files: files
                .iter()
                .map(|(name, text)| ((*name).to_owned(), (*text).to_owned()))
                .collect(),
        }
    }
}
