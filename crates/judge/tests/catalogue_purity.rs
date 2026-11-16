//! Purity guard for the pack importer (problem-catalogue C23, requirements CAT-Q4 and CAT-PT4).
//!
//! The importer must not start processes, open network sockets, call git or read credentials.
//! Only `sandbox_fill.rs` may name `PackRunner`, so the static path cannot reach the sandbox.
//! A source-text scan is a cheap guard for this architecture rule. It does not replace the type
//! system, which already keeps the static path from calling into the sandbox.
//!
//! Run with `cargo test -p counterspell-judge --test catalogue_purity`.

use std::fs;
use std::path::{Path, PathBuf};

/// Text that the importer source must never contain.
const FORBIDDEN: [&str; 9] = [
    "std::process",
    "Command::",
    "std::net",
    "TcpStream",
    "\"git\"",
    "TOKEN",
    "DEPLOY_KEY",
    "SSH_AUTH_SOCK",
    "GIT_SSH",
];

/// The only file allowed to name the sandbox runner trait.
const SANDBOX_FILE: &str = "sandbox_fill.rs";

/// Returns the forbidden tokens that appear in one source file's text.
fn forbidden_in(source: &str) -> Vec<&'static str> {
    FORBIDDEN
        .iter()
        .copied()
        .filter(|token| source.contains(token))
        .collect()
}

/// Returns true when `file_name` is outside the sandbox path and names `PackRunner`.
fn names_pack_runner_outside_sandbox(file_name: &str, source: &str) -> bool {
    file_name != SANDBOX_FILE && source.contains("PackRunner")
}

fn catalogue_sources() -> Vec<PathBuf> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/catalogue");
    let mut files = Vec::new();
    let mut pending = vec![root];
    while let Some(dir) = pending.pop() {
        for entry in fs::read_dir(&dir).expect("catalogue source directory") {
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
    for file in catalogue_sources() {
        let source = fs::read_to_string(&file).expect("readable source");
        for token in forbidden_in(&source) {
            found.push(format!("{}: {token}", file.display()));
        }
    }
    assert_eq!(found, Vec::<String>::new());
}

#[test]
fn only_the_sandbox_file_names_pack_runner() {
    for file in catalogue_sources() {
        let name = file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_owned();
        let source = fs::read_to_string(&file).expect("readable source");
        assert!(
            !names_pack_runner_outside_sandbox(&name, &source),
            "{name} names PackRunner outside the sandbox file"
        );
    }
}

#[test]
fn a_scratch_copy_that_spawns_a_process_is_flagged() {
    let scratch =
        "fn run() { let _ = std::process::Command::new(\"git\").arg(\"pull\").output(); }";

    assert_eq!(
        forbidden_in(scratch),
        vec!["std::process", "Command::", "\"git\""]
    );
}

#[test]
fn a_scratch_copy_that_reads_a_credential_is_flagged() {
    let scratch = "const KEY: &str = env!(\"DEPLOY_KEY\"); // TOKEN";

    assert_eq!(forbidden_in(scratch), vec!["TOKEN", "DEPLOY_KEY"]);
}

#[test]
fn pack_runner_is_flagged_outside_the_sandbox_file_only() {
    assert!(names_pack_runner_outside_sandbox(
        "static_import.rs",
        "use x::PackRunner;"
    ));
    assert!(!names_pack_runner_outside_sandbox(
        SANDBOX_FILE,
        "use x::PackRunner;"
    ));
    assert!(!names_pack_runner_outside_sandbox(
        "write.rs",
        "no runner here"
    ));
}
