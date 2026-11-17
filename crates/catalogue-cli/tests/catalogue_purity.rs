//! Purity guard for the catalogue importer (problem-catalogue C23, CAT-Q4, CAT-PT4).
//!
//! The importer must not start processes, open network sockets, call git or read
//! credentials: it reads the two pack trees and writes only to Postgres. This is
//! the `crates/catalogue-cli` counterpart of `crates/judge/tests/catalogue_purity.rs`,
//! which guards the judge's remaining catalogue row model.
//! A source-text scan is a cheap guard for this architecture rule. It does not
//! replace the type system, which already keeps the static path off the sandbox.

use std::fs;
use std::path::{Path, PathBuf};

/// Text that the importer source must never contain. `Command::new` and
/// `process::Command` are listed precisely — the binary legitimately names
/// `std::process::ExitCode` and its own `Command` verb enum.
const FORBIDDEN: [&str; 10] = [
    "process::Command",
    "Command::new",
    "std::net",
    "TcpStream",
    "\"git\"",
    "TOKEN",
    "DEPLOY_KEY",
    "SSH_AUTH_SOCK",
    "GIT_SSH",
    "reqwest",
];

/// Returns the forbidden tokens that appear in one source file's text.
fn forbidden_in(source: &str) -> Vec<&'static str> {
    FORBIDDEN
        .iter()
        .copied()
        .filter(|token| source.contains(token))
        .collect()
}

/// Every `.rs` file under `src/` of this crate (library and binaries).
fn cli_sources() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut files = Vec::new();
    let mut pending = vec![root];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).expect("catalogue-cli source directory") {
            let path = entry.expect("directory entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
    }
    files.sort();
    files
}

#[test]
fn the_importer_source_has_no_forbidden_tokens() {
    let mut found = Vec::new();
    for file in cli_sources() {
        let source = fs::read_to_string(&file).expect("readable source");
        for token in forbidden_in(&source) {
            found.push(format!("{}: {token}", file.display()));
        }
    }
    assert_eq!(found, Vec::<String>::new());
}

#[test]
fn a_scratch_copy_that_spawns_a_process_is_flagged() {
    let scratch =
        "fn run() { let _ = std::process::Command::new(\"git\").arg(\"pull\").output(); }";

    assert_eq!(
        forbidden_in(scratch),
        vec!["process::Command", "Command::new", "\"git\""]
    );
}

#[test]
fn a_scratch_copy_that_reads_a_credential_is_flagged() {
    let scratch = "const KEY: &str = env!(\"DEPLOY_KEY\"); // TOKEN";

    assert_eq!(forbidden_in(scratch), vec!["TOKEN", "DEPLOY_KEY"]);
}
