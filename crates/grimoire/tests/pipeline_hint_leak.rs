//! PA05 (PIP-14, PIP-46, P2): a hint must not leak a sample, hidden or ghost value (HNT-03),
//! and no fenced block may run past three lines (GRM-27).
//!
//! Values are passed in memory. Diagnostics name a rung and a value index, never the value.

use std::fs;
use std::path::{Path, PathBuf};

use counterspell_grimoire::pipeline::hint_leak::{check_rungs, HiddenValues};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pipeline")
        .join(name)
        .join("rungs.txt")
}

/// Reads a fixture file whose rungs follow `=== rung N` lines.
fn rungs_from_fixture(name: &str) -> Vec<String> {
    let text = fs::read_to_string(fixture(name)).expect("fixture is readable");
    let mut rungs: Vec<String> = Vec::new();
    for line in text.lines() {
        if line.starts_with("=== rung") {
            rungs.push(String::new());
        } else if let Some(current) = rungs.last_mut() {
            current.push_str(line);
            current.push('\n');
        }
    }
    rungs
}

fn owned(values: &[&str]) -> Vec<String> {
    values.iter().map(|value| (*value).to_owned()).collect()
}

/// The values used by the fixtures: a sample `7`, a hidden `41` and a ghost `13`.
fn fixture_values<'a>(
    sample: &'a [String],
    hidden: &'a [String],
    ghost: &'a [String],
) -> HiddenValues<'a> {
    HiddenValues {
        sample,
        hidden,
        ghost_got: ghost,
    }
}

#[test]
fn a_rung_that_quotes_a_hidden_value_fails_with_exactly_hnt_03() {
    let rungs = rungs_from_fixture("hint-leaks-expected");
    let sample = owned(&["7"]);
    let hidden = owned(&["41"]);
    let ghost = owned(&["13"]);

    let diagnostics = check_rungs(&rungs, &fixture_values(&sample, &hidden, &ghost));

    let codes: Vec<(&str, &str)> = diagnostics
        .iter()
        .map(|d| (d.code, d.pointer.as_str()))
        .collect();
    assert_eq!(
        codes,
        vec![("HNT-03", "hint rung 3 matches hidden value 1")]
    );
}

#[test]
fn a_rung_that_quotes_a_ghost_got_value_fails_with_exactly_hnt_03() {
    let rungs = rungs_from_fixture("hint-leaks-ghost-got");
    let sample = owned(&["7"]);
    let hidden = owned(&["41"]);
    let ghost = owned(&["13"]);

    let diagnostics = check_rungs(&rungs, &fixture_values(&sample, &hidden, &ghost));

    let codes: Vec<(&str, &str)> = diagnostics
        .iter()
        .map(|d| (d.code, d.pointer.as_str()))
        .collect();
    assert_eq!(codes, vec![("HNT-03", "hint rung 2 matches ghost value 1")]);
}

#[test]
fn a_value_inside_a_longer_number_is_not_a_match() {
    let rungs = owned(&["Try 2026 readings first."]);
    let sample = owned(&["20"]);
    let hidden = owned(&[]);
    let ghost = owned(&[]);

    let diagnostics = check_rungs(&rungs, &fixture_values(&sample, &hidden, &ghost));

    assert_eq!(diagnostics, vec![]);
}

#[test]
fn a_value_on_its_own_as_a_token_is_a_match() {
    let rungs = owned(&["Try readings that total 20 first."]);
    let sample = owned(&["20"]);
    let hidden = owned(&[]);
    let ghost = owned(&[]);

    let diagnostics = check_rungs(&rungs, &fixture_values(&sample, &hidden, &ghost));

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].code, "HNT-03");
}

#[test]
fn a_json_literal_in_a_rung_is_a_match() {
    let rungs = owned(&["Do not compare against [0, 1] by hand."]);
    let sample = owned(&["[0, 1]"]);
    let hidden = owned(&[]);
    let ghost = owned(&[]);

    let diagnostics = check_rungs(&rungs, &fixture_values(&sample, &hidden, &ghost));

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].pointer, "hint rung 1 matches sample value 1");
}

#[test]
fn a_fenced_block_of_four_lines_fails_with_grm_27() {
    let rungs = owned(&["Here is the shape:\n```\na\nb\nc\nd\n```\n"]);
    let none = owned(&[]);

    let diagnostics = check_rungs(&rungs, &fixture_values(&none, &none, &none));

    let codes: Vec<(&str, &str)> = diagnostics
        .iter()
        .map(|d| (d.code, d.pointer.as_str()))
        .collect();
    assert_eq!(codes, vec![("GRM-27", "hint rung 1")]);
}

#[test]
fn a_fenced_block_of_three_lines_is_allowed() {
    let rungs = owned(&["Here is the shape:\n```\na\nb\nc\n```\n"]);
    let none = owned(&[]);

    let diagnostics = check_rungs(&rungs, &fixture_values(&none, &none, &none));

    assert_eq!(diagnostics, vec![]);
}

#[test]
fn the_report_never_contains_the_leaked_value() {
    let canary = "CANARY-HIDDEN-pa5";
    let rungs = owned(&["The answer looks like CANARY-HIDDEN-pa5 here."]);
    let sample = owned(&[]);
    let hidden = owned(&[canary]);
    let ghost = owned(&[]);

    let diagnostics = check_rungs(&rungs, &fixture_values(&sample, &hidden, &ghost));
    let report = format!("{diagnostics:?}");

    assert_eq!(diagnostics.len(), 1);
    assert!(
        !report.contains(canary),
        "the report must not repeat the leaked value"
    );
}
