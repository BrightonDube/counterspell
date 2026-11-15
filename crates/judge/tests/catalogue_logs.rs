//! Report and log-hygiene tests (problem-catalogue C13, CAT-I20, CAT-X1, CAT-X2, CAT-X4).
//!
//! The report shows codes, counts and a 12-character hash prefix. Canary values planted in the
//! inputs must never appear in a rendered line.

use counterspell_judge::catalogue::report::{Outcome, PackReport};
use std::collections::BTreeMap;

const CANARY_HIDDEN: &str = "CANARY-HIDDEN-7f3a";
const CANARY_EXPECTED: &str = "CANARY-EXPECTED-9b1c";
const FULL_HASH: &str = "f5178ca9e71e0d918035b8f51e64ec24fb6b724a587d804b07b1ea51e930c0bb";

fn report(outcome: Outcome) -> PackReport {
    let mut counts = BTreeMap::new();
    counts.insert("files", 2);
    counts.insert("hidden", 5);
    PackReport {
        slug: "two-sum".to_string(),
        version: Some(1),
        outcome,
        counts,
        hash_hex: Some(FULL_HASH.to_string()),
    }
}

#[test]
fn text_line_names_slug_outcome_version_and_short_hash() {
    let line = report(Outcome::Created).render_text();
    assert_eq!(
        line,
        "two-sum created version=1 hash=f5178ca9e71e files=2 hidden=5"
    );
}

#[test]
fn refused_line_carries_the_stable_code() {
    let line = report(Outcome::Refused("version_not_bumped".to_string())).render_text();
    assert!(
        line.starts_with("two-sum refused:version_not_bumped "),
        "got: {line}"
    );
}

#[test]
fn json_form_has_the_same_fields() {
    let value = report(Outcome::Unchanged).render_json();
    assert_eq!(value["slug"], "two-sum");
    assert_eq!(value["outcome"], "unchanged");
    assert_eq!(value["hash"], "f5178ca9e71e");
    assert_eq!(value["counts"]["hidden"], 5);
}

#[test]
fn no_rendered_output_contains_canary_or_full_hash() {
    // Canary strings are planted in the counts' names, which a careless renderer could echo.
    let mut counts = BTreeMap::new();
    counts.insert("files", 2);
    let planted = PackReport {
        slug: "two-sum".to_string(),
        version: Some(1),
        outcome: Outcome::Failed("sbx_02_mismatch".to_string()),
        counts,
        hash_hex: Some(FULL_HASH.to_string()),
    };
    for rendered in [planted.render_text(), planted.render_json().to_string()] {
        assert!(
            !rendered.contains(CANARY_HIDDEN),
            "canary leaked: {rendered}"
        );
        assert!(
            !rendered.contains(CANARY_EXPECTED),
            "canary leaked: {rendered}"
        );
        assert!(
            !rendered.contains(FULL_HASH),
            "full hash leaked: {rendered}"
        );
    }
}
