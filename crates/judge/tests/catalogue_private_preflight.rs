//! Private-tree preflight and join tests (problem-catalogue C08a, CAT-PT1 to CAT-PT3, CAT-X4).
//! Pure: no database, no filesystem. The caller resolves the root state before it calls these.

use counterspell_judge::catalogue::private_tree::{
    join, preflight, JoinOutcome, PreflightError, RootState,
};

#[test]
fn preflight_refuses_missing_root_with_exact_code() {
    let error = preflight(RootState::Missing).expect_err("a missing root must be refused");
    assert_eq!(error, PreflightError::Missing);
    assert_eq!(error.code(), "private_tree_missing");
    assert_eq!(error.exit_status(), 2);
}

#[test]
fn preflight_refuses_unreadable_root_with_exact_code() {
    let error = preflight(RootState::Unreadable).expect_err("an unreadable root must be refused");
    assert_eq!(error.code(), "private_tree_unreadable");
    assert_eq!(error.exit_status(), 2);
}

#[test]
fn preflight_refuses_root_inside_public_with_exact_code() {
    let error = preflight(RootState::Readable {
        inside_public: true,
    })
    .expect_err("a private root inside the public tree must be refused");
    assert_eq!(error.code(), "private_root_inside_public");
    assert_eq!(error.exit_status(), 2);
}

#[test]
fn preflight_accepts_a_readable_root_outside_public() {
    assert!(preflight(RootState::Readable {
        inside_public: false
    })
    .is_ok());
}

#[test]
fn join_pairs_matching_slugs() {
    let outcomes = join(&["count-pairs"], &["count-pairs"]);
    assert_eq!(
        outcomes,
        vec![JoinOutcome::Paired("count-pairs".to_string())]
    );
}

#[test]
fn join_reports_public_slug_without_private_part() {
    let outcomes = join(&["count-pairs"], &[]);
    assert_eq!(
        outcomes,
        vec![JoinOutcome::PrivatePartMissing("count-pairs".to_string())]
    );
    assert_eq!(
        JoinOutcome::PrivatePartMissing("x".into()).code(),
        "private_part_missing"
    );
}

#[test]
fn join_reports_private_directory_without_public_pack() {
    let outcomes = join(&[], &["ghost-only"]);
    assert_eq!(
        outcomes,
        vec![JoinOutcome::OrphanPrivatePack("ghost-only".to_string())]
    );
    assert_eq!(
        JoinOutcome::OrphanPrivatePack("x".into()).code(),
        "orphan_private_pack"
    );
}
