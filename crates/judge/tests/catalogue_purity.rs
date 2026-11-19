//! Purity guard for the catalogue row model (problem-catalogue C23, CAT-Q4, CAT-PT4).
//!
//! The importer's static path must not start processes, open network sockets, call git or read
//! credentials. Since the authoring split (the `counterspell-catalogue` CLI and its readers live
//! in `crates/catalogue-cli`), no catalogue file may name `PackRunner` at all — the judge no
//! longer depends on `counterspell-grimoire`.
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

/// Returns the forbidden tokens that appear in one source file's text.
fn forbidden_in(source: &str) -> Vec<&'static str> {
    FORBIDDEN
        .iter()
        .copied()
        .filter(|token| source.contains(token))
        .collect()
}

/// Returns true when the source names `PackRunner` — authoring machinery that
/// must stay in `crates/catalogue-cli`, never in the judge's catalogue model.
fn names_pack_runner(source: &str) -> bool {
    source.contains("PackRunner")
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
fn no_catalogue_file_names_pack_runner() {
    for file in catalogue_sources() {
        let name = file
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_owned();
        let source = fs::read_to_string(&file).expect("readable source");
        assert!(
            !names_pack_runner(&source),
            "{name} names PackRunner — authoring machinery belongs in catalogue-cli"
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
fn pack_runner_is_flagged_wherever_it_appears() {
    assert!(names_pack_runner("use x::PackRunner;"));
    assert!(!names_pack_runner("no runner here"));
}
