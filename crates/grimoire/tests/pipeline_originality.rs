//! PA04 (PIP-3, PIP-8, PIP-46, P6): the originality guard.
//!
//! Markers in these tests are synthetic (`SYNTH-MARKER-ALPHA`). They are not third-party text.

use std::fs;
use std::path::{Path, PathBuf};

use counterspell_grimoire::pipeline::originality::{
    check_pack, check_record, scan_text, MarkerError, Markers,
};

/// The synthetic marker used by the copied-marker fixture.
const SYNTH_MARKER: &str = "SYNTH-MARKER-ALPHA";

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pipeline")
        .join(name)
}

/// Every `.md` file in a fixture, as (file name, text), sorted by name.
fn fixture_files(name: &str) -> Vec<(String, String)> {
    let mut files: Vec<(String, String)> = fs::read_dir(fixture(name))
        .expect("fixture directory is readable")
        .map(|entry| {
            let path = entry.expect("directory entry is readable").path();
            let file_name = path
                .file_name()
                .and_then(|name| name.to_str())
                .expect("fixture file name is UTF-8")
                .to_owned();
            let text = fs::read_to_string(&path).expect("fixture file is UTF-8");
            (file_name, text)
        })
        .filter(|(file_name, _)| {
            Path::new(file_name)
                .extension()
                .is_some_and(|ext| ext == "md")
        })
        .collect();
    files.sort();
    files
}

fn markers() -> Markers {
    Markers::new(vec![SYNTH_MARKER.to_owned()]).expect("a synthetic marker is valid")
}

/// Runs the guard over a fixture directory. Returns the diagnostics and the record text.
fn run_fixture(name: &str) -> (Vec<(&'static str, String)>, Option<String>) {
    let files = fixture_files(name);
    let borrowed: Vec<(&str, &str)> = files
        .iter()
        .map(|(file, text)| (file.as_str(), text.as_str()))
        .collect();
    let record = files
        .iter()
        .find(|(file, _)| file == "originality.md")
        .map(|(_, text)| text.clone());
    let diagnostics = check_pack(&markers(), &borrowed, record.as_deref())
        .into_iter()
        .map(|diagnostic| (diagnostic.code, diagnostic.pointer))
        .collect();
    (diagnostics, record)
}

#[test]
fn a_copied_marker_fixture_fails_with_exactly_org_01_at_its_line() {
    let (diagnostics, _) = run_fixture("copied-marker");

    assert_eq!(
        diagnostics,
        vec![("ORG-01", "problem.md line 4".to_owned())]
    );
}

#[test]
fn a_fixture_without_a_record_fails_with_exactly_org_02() {
    let (diagnostics, record) = run_fixture("no-originality-record");

    assert!(record.is_none());
    assert_eq!(diagnostics, vec![("ORG-02", "originality.md".to_owned())]);
}

#[test]
fn a_record_with_all_five_sections_filled_passes() {
    let (_, record) = run_fixture("copied-marker");
    let record = record.expect("the copied-marker fixture has a record");

    assert_eq!(check_record(&record), None);
}

#[test]
fn a_record_with_an_empty_section_fails_with_org_02() {
    let text = "## Idea\nA counting idea.\n\n## Why it is original\nWritten from scratch.\n\n\
                ## Patterns used\nA hash map.\n\n## Checklist answers\nAll yes.\n\n\
                ## Third-party text avoided\n";

    let diagnostic = check_record(text).expect("the empty section is a finding");

    assert_eq!(diagnostic.code, "ORG-02");
    assert_eq!(diagnostic.pointer, "originality.md");
}

#[test]
fn a_marker_match_ignores_case() {
    let diagnostics = scan_text(
        &markers(),
        "statement.md",
        "contains synth-marker-alpha here",
    );

    assert_eq!(diagnostics.len(), 1);
    assert_eq!(diagnostics[0].pointer, "statement.md line 1");
}

#[test]
fn the_report_never_contains_the_matched_marker_text() {
    let (diagnostics, _) = run_fixture("copied-marker");
    let report = format!("{diagnostics:?}");

    assert!(
        !report.contains(SYNTH_MARKER),
        "the report must not repeat the marker"
    );
    assert!(!report.to_lowercase().contains(&SYNTH_MARKER.to_lowercase()));
}

#[test]
fn a_blank_marker_is_refused_because_it_would_match_every_line() {
    assert_eq!(
        Markers::new(vec!["   ".to_owned()]),
        Err(MarkerError::Blank)
    );
}
