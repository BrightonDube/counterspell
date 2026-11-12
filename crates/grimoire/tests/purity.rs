//! Architecture guard (GRM-35, GRM-31, GHO-6): the validator reads files and never
//! runs code, opens sockets or writes packs, and its wording never claims minimality.
//!
//! The scan reads `src/` as text. It is a cheap tripwire, not a proof, and it runs
//! with the normal test suite so a stray `Command::new` fails the build.

use std::fs;
use std::path::{Path, PathBuf};

/// APIs the validator must never use. Reading files is allowed, writing and running are not.
const FORBIDDEN_APIS: [&str; 8] = [
    "std::process::Command",
    "Command::",
    "std::net",
    "TcpStream",
    "File::create",
    "fs::write",
    "OpenOptions",
    "std::os::unix::process",
];

/// Phrases a diagnostic or doc comment must never use (the engine's R9.6 list).
const FORBIDDEN_PHRASES: [&str; 6] = [
    "smallest possible",
    "global",
    "proven",
    "optimal",
    "the minimum",
    "smallest",
];

/// The source directory of this crate.
fn source_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Every `.rs` file under `src/`, with its text.
fn source_files() -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();
    for entry in fs::read_dir(source_dir()).expect("src directory is readable") {
        let path = entry.expect("directory entry is readable").path();
        if path.extension().is_some_and(|ext| ext == "rs") {
            let text = fs::read_to_string(&path).expect("source file is UTF-8");
            files.push((path, text));
        }
    }
    files
}

/// Returns the forbidden API names that appear in `text`.
fn api_violations(text: &str) -> Vec<&'static str> {
    FORBIDDEN_APIS
        .into_iter()
        .filter(|api| text.contains(api))
        .collect()
}

/// Returns the forbidden phrases that appear in `text`, ignoring case.
fn phrase_violations(text: &str) -> Vec<&'static str> {
    let lower = text.to_lowercase();
    FORBIDDEN_PHRASES
        .into_iter()
        .filter(|phrase| lower.contains(phrase))
        .collect()
}

#[test]
fn no_source_file_runs_processes_opens_sockets_or_writes_files() {
    for (path, text) in source_files() {
        let found = api_violations(&text);
        assert!(found.is_empty(), "{} uses {found:?}", path.display());
    }
}

#[test]
fn scanner_catches_a_planted_process_call() {
    let planted = "fn spawn() { let _ = std::process::Command::new(\"python3\"); }";

    assert!(!api_violations(planted).is_empty());
}

#[test]
fn scanner_catches_a_planted_file_write() {
    assert_eq!(api_violations("std::fs::write(path, bytes)"), ["fs::write"]);
}

#[test]
fn no_source_text_uses_a_forbidden_minimality_phrase() {
    for (path, text) in source_files() {
        if path.ends_with("ghosts.rs") {
            // This file defines the list itself, so it is checked by its own tests.
            continue;
        }
        let found = phrase_violations(&text);
        assert!(found.is_empty(), "{} uses {found:?}", path.display());
    }
}

#[test]
fn phrase_scanner_catches_a_claim_of_minimality() {
    assert_eq!(
        phrase_violations("This is the smallest failing input."),
        ["smallest"]
    );
}
