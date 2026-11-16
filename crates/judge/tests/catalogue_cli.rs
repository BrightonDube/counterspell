//! Exit-code tests for `counterspell-catalogue` (problem-catalogue C19).
//!
//! These run the real binary and assert the process exit code — an
//! `ExitCode` is opaque inside the process, so the 0/1/2 contract can only
//! be proven from outside. Every case here exits before any database work,
//! so no `CATALOGUE_DATABASE_URL` is needed.

use std::path::Path;
use std::process::Command;

const BIN: &str = env!("CARGO_BIN_EXE_counterspell-catalogue");

#[allow(clippy::disallowed_methods)] // a test harness may spawn the CLI under test; production code may not
fn run(args: &[&str]) -> i32 {
    Command::new(BIN)
        .args(args)
        // The variable must not leak in from the developer shell; these tests
        // prove the exits happen before the database is consulted.
        .env_remove("CATALOGUE_DATABASE_URL")
        .output()
        .expect("binary runs")
        .status
        .code()
        .expect("exit code")
}

#[test]
fn no_args_is_usage() {
    assert_eq!(run(&[]), 2);
}

#[test]
fn unknown_verb_is_usage() {
    assert_eq!(run(&["frobnicate"]), 2);
}

#[test]
fn extra_positional_is_usage() {
    assert_eq!(run(&["import", "count-pairs"]), 2);
}

#[test]
fn missing_private_root_exits_two_without_a_database() {
    assert_eq!(run(&["import", "--static-only"]), 2);
}

#[test]
fn missing_private_root_dir_exits_two() {
    assert_eq!(
        run(&[
            "import",
            "--static-only",
            "--private-root",
            "/definitely/not/a/real/path",
        ]),
        2
    );
}

#[test]
fn private_root_inside_public_exits_two() {
    let root = tempfile::tempdir().unwrap();
    let packs = root.path().join("packs");
    std::fs::create_dir_all(&packs).unwrap();
    let inside = packs.join("private");
    std::fs::create_dir_all(&inside).unwrap();
    assert_eq!(
        run(&[
            "import",
            "--static-only",
            "--packs",
            packs.to_str().unwrap(),
            "--private-root",
            inside.to_str().unwrap(),
        ]),
        2
    );
}

#[test]
fn sandbox_import_without_a_runner_is_sbx_07() {
    let root = tempfile::tempdir().unwrap();
    let packs = root.path().join("packs");
    let private = root.path().join("private");
    std::fs::create_dir_all(&packs).unwrap();
    std::fs::create_dir_all(&private).unwrap();
    // A valid private root, no runner wired: SBX-07 is an exit-2 refusal.
    assert_eq!(
        run(&[
            "import",
            "--packs",
            packs.to_str().unwrap(),
            "--private-root",
            private.to_str().unwrap(),
        ]),
        2
    );
}

#[test]
fn validate_sandbox_without_a_runner_is_sbx_07() {
    let root = tempfile::tempdir().unwrap();
    let packs = root.path().join("packs");
    let private = root.path().join("private");
    std::fs::create_dir_all(&packs).unwrap();
    std::fs::create_dir_all(&private).unwrap();
    assert_eq!(
        run(&[
            "validate-sandbox",
            "count-pairs",
            "--packs",
            packs.to_str().unwrap(),
            "--private-root",
            private.to_str().unwrap(),
        ]),
        2
    );
}

/// The `--packs` default resolves against the working directory; the tests
/// pass absolute roots so nothing depends on it.
#[allow(dead_code)]
fn path_str(path: &Path) -> &str {
    path.to_str().unwrap()
}
