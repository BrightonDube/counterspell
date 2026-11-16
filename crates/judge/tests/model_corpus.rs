// SPDX-License-Identifier: MIT
//! The corpus-is-data guard (judge-sandbox T25, R16.2 and R16.3).
//!
//! Hostile corpus files are only ever data: the tests read them as text and deliver
//! them into a sandbox container as submitted source. This test asserts no test or
//! harness source calls process-spawning APIs, and that the corpus directory contains
//! only program files and its README.

use std::fs;
use std::path::{Path, PathBuf};

// Compiles the shared real-daemon harness so it cannot rot before the scenario tests
// land. Its contents are empty without the `sandbox` feature.
mod sbx;

/// The crate's root directory.
fn crate_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Every `.rs` file under `dir`, recursively.
fn rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    let mut pending = vec![dir.to_path_buf()];
    while let Some(dir) = pending.pop() {
        for entry in
            fs::read_dir(&dir).unwrap_or_else(|error| panic!("reading {}: {error}", dir.display()))
        {
            let path = entry.expect("directory entry").path();
            if path.is_dir() {
                pending.push(path);
            } else if path.extension().is_some_and(|ext| ext == "rs") {
                files.push(path);
            }
        }
    }
    files
}

/// The tokens that would spawn or run a program on the host. Written split so this
/// file's own text does not match them. `std::process::ExitCode` is a type, not a
/// spawn, and stays allowed.
const SPAWN_TOKENS: &[&str] = &[
    concat!("Com", "mand::new"),
    concat!("process::Com", "mand"),
    concat!("tokio::pro", "cess::"),
    concat!("process::ex", "it("),
];

/// The markers a line carries when it touches a corpus path.
const CORPUS_MARKERS: &[&str] = &["hostile", "fixtures", "solutions", ".py"];

#[test]
fn hostile_files_are_data() {
    let root = crate_root();

    // The corpus directory holds only program files and its README.
    let hostile_dir = root.join("tests/hostile");
    for entry in fs::read_dir(&hostile_dir).expect("the hostile corpus directory exists") {
        let name = entry.expect("corpus entry").file_name();
        let name = name.to_str().expect("a UTF-8 file name");
        let is_program = Path::new(name)
            .extension()
            .is_some_and(|ext| ext.eq_ignore_ascii_case("py"));
        assert!(
            is_program || name == "README.md",
            "only .py programs and the README belong in tests/hostile: {name}"
        );
    }

    // No line of any test or harness source may both call a host-spawning API and name
    // a corpus path: corpus files are delivered as bytes, never run on the host.
    // Scanner tests hold these tokens as data on lines that name no corpus file.
    for dir in [root.join("src"), root.join("tests")] {
        for file in rust_files(&dir) {
            let text = fs::read_to_string(&file)
                .unwrap_or_else(|error| panic!("reading {}: {error}", file.display()));
            for (line_number, line) in text.lines().enumerate() {
                let spawns = SPAWN_TOKENS.iter().any(|token| line.contains(token));
                let touches_corpus = CORPUS_MARKERS.iter().any(|marker| line.contains(marker));
                assert!(
                    !(spawns && touches_corpus),
                    "{}:{} calls a host-spawning API on a corpus path: {line}",
                    file.display(),
                    line_number + 1
                );
            }
        }
    }
}
