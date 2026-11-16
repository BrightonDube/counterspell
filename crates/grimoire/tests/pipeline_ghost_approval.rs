//! PA06 (PIP-13, PIP-18, PIP-28, PIP-29, PIP-36, P1, P5): ghost coverage and the approval checks.
//!
//! Fixtures are synthetic. The manifest hash is passed in as text. It is computed by
//! `scripts/pack-assemble.sh` with `sha256sum`, not here, so this crate needs no hashing
//! dependency. The one-byte change test lives in the assembly script test (PA07).

use std::fs;
use std::path::{Path, PathBuf};

use counterspell_grimoire::pipeline::approval::{check_publish, meta_status, Approval};
use counterspell_grimoire::pipeline::ghost_cover::{
    check_coverage, check_reference, parse_known_ghosts, parse_reports, KnownGhost,
};
use counterspell_grimoire::pipeline::Diagnostic;

fn fixture(name: &str, file: &str) -> String {
    let path: PathBuf = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/pipeline")
        .join(name)
        .join(file);
    fs::read_to_string(path).expect("fixture is readable")
}

/// The codes and pointers of a list of diagnostics, for exact comparison.
fn codes(diagnostics: &[Diagnostic]) -> Vec<(&'static str, String)> {
    diagnostics
        .iter()
        .map(|diagnostic| (diagnostic.code, diagnostic.pointer.clone()))
        .collect()
}

fn ghost(program: &str, kind: &str) -> KnownGhost {
    KnownGhost {
        program: program.to_owned(),
        kind: kind.to_owned(),
    }
}

#[test]
fn a_fixture_with_one_known_ghost_fails_with_exactly_ghc_01() {
    let known =
        parse_known_ghosts(&fixture("one-ghost", "known_ghosts.toml")).expect("the fixture parses");
    let reports = parse_reports(&fixture("one-ghost", "s11-report.toml")).expect("report parses");

    let diagnostics = check_coverage(&known, &reports);

    assert_eq!(
        codes(&diagnostics),
        vec![("GHC-01", "known_ghosts.toml".to_owned())]
    );
}

#[test]
fn a_ghost_that_does_not_reproduce_fails_with_exactly_ghc_02() {
    let known = parse_known_ghosts(&fixture("ghost-not-reproducing", "known_ghosts.toml"))
        .expect("the fixture parses");
    let reports =
        parse_reports(&fixture("ghost-not-reproducing", "s11-report.toml")).expect("report parses");

    let diagnostics = check_coverage(&known, &reports);

    assert_eq!(
        codes(&diagnostics),
        vec![("GHC-02", "incorrect/wrong_start.py".to_owned())]
    );
}

#[test]
fn two_ghosts_that_both_reproduce_pass() {
    let known = vec![
        ghost("incorrect/off_by_one.py", "wrong"),
        ghost("incorrect/wrong_start.py", "wrong"),
    ];
    let reports = parse_reports(
        "[[ghost]]\nprogram = \"incorrect/off_by_one.py\"\nkind = \"wrong\"\nverified = true\n\n\
         [[ghost]]\nprogram = \"incorrect/wrong_start.py\"\nkind = \"wrong\"\nverified = true\n",
    )
    .expect("report parses");

    assert_eq!(check_coverage(&known, &reports), vec![]);
}

#[test]
fn a_ghost_of_a_different_kind_does_not_count_as_reproduced() {
    let known = vec![
        ghost("incorrect/off_by_one.py", "timeout"),
        ghost("incorrect/wrong_start.py", "wrong"),
    ];
    let reports = parse_reports(
        "[[ghost]]\nprogram = \"incorrect/off_by_one.py\"\nkind = \"wrong\"\nverified = true\n\n\
         [[ghost]]\nprogram = \"incorrect/wrong_start.py\"\nkind = \"wrong\"\nverified = true\n",
    )
    .expect("report parses");

    assert_eq!(
        codes(&check_coverage(&known, &reports)),
        vec![("GHC-02", "incorrect/off_by_one.py".to_owned())]
    );
}

#[test]
fn a_reference_that_finds_a_counterexample_fails_with_ghc_03() {
    assert_eq!(
        check_reference(true),
        Some(Diagnostic {
            code: "GHC-03",
            pointer: "reference/solution.py".to_owned(),
        })
    );
    assert_eq!(check_reference(false), None);
}

#[test]
fn a_published_pack_with_no_approval_fails_with_exactly_pub_01() {
    let status =
        meta_status(&fixture("publish-without-approval", "meta.toml")).expect("meta parses");

    let diagnostics = check_publish(status.as_deref(), None, "aaaa");

    assert_eq!(
        codes(&diagnostics),
        vec![("PUB-01", "approval.toml".to_owned())]
    );
}

#[test]
fn an_approval_for_another_hash_fails_with_exactly_apr_01() {
    let status = meta_status(&fixture("approval-hash-mismatch", "meta.toml")).expect("meta parses");
    let approval = Approval::from_toml(&fixture("approval-hash-mismatch", "approval.toml"))
        .expect("approval parses");

    let diagnostics = check_publish(status.as_deref(), Some(&approval), "aaaa");

    assert_eq!(
        codes(&diagnostics),
        vec![("APR-01", "approval.toml content_sha256".to_owned())]
    );
}

#[test]
fn an_approval_for_the_same_hash_passes() {
    let approval = Approval::from_toml(
        "slug = \"count-pairs\"\npack_version = 2\ncontent_sha256 = \"aaaa\"\n\
         public_commit = \"9c2e4f1\"\napproved_by = \"Brighton\"\napproved_at = \"2026-11-26\"\n",
    )
    .expect("approval parses");

    assert_eq!(
        check_publish(Some("published"), Some(&approval), "aaaa"),
        vec![]
    );
}

#[test]
fn a_draft_pack_needs_no_approval() {
    assert_eq!(check_publish(Some("draft"), None, "aaaa"), vec![]);
}

#[test]
fn an_approval_becomes_stale_when_the_manifest_hash_changes() {
    let approval = Approval::from_toml(
        "slug = \"count-pairs\"\npack_version = 2\ncontent_sha256 = \"aaaa\"\n\
         public_commit = \"9c2e4f1\"\napproved_by = \"Brighton\"\napproved_at = \"2026-11-26\"\n",
    )
    .expect("approval parses");

    assert_eq!(
        codes(&check_publish(Some("published"), Some(&approval), "aaab")),
        vec![("APR-01", "approval.toml content_sha256".to_owned())]
    );
}

#[test]
fn an_approval_file_with_an_unknown_key_is_refused() {
    let text = "slug = \"count-pairs\"\npack_version = 2\ncontent_sha256 = \"aaaa\"\n\
                public_commit = \"9c2e4f1\"\napproved_by = \"Brighton\"\n\
                approved_at = \"2026-11-26\"\nhidden_expected = \"[0, 1]\"\n";

    assert!(Approval::from_toml(text).is_err());
}
