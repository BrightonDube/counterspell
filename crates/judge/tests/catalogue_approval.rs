//! Approval record and retired-value tests (problem-catalogue C11a, CAT-PT7, CAT-PT8, CAT-PT13).
//! Pure: no database and no filesystem. Hash literals were computed with `sha256sum` from the exact
//! canonical bytes, `{"a":[2,3],"b":1}`, NUL, `4`, not by the code under test.

use std::fmt::Write as _;

use counterspell_judge::catalogue::approval::{check_approval, parse_approval, ApprovalRefusal};
use counterspell_judge::catalogue::retired::{
    canonical_hash, check_retired, parse_retired, RetiredRefusal,
};
use serde_json::json;

const CANONICAL_PAIR_SHA256: &str =
    "b9aa0771d4a2409ca3dd9a5c7166112c51d4991b062891c4d0da58923a818f85";
const APPROVAL_HASH: &str = "1111111111111111111111111111111111111111111111111111111111111111";
const COMMIT: &str = "2222222222222222222222222222222222222222";

fn approval_text(slug: &str, pack_version: i32) -> String {
    format!(
        "slug = \"{slug}\"\npack_version = {pack_version}\ncontent_sha256 = \"{APPROVAL_HASH}\"\n\
         public_commit = \"{COMMIT}\"\napproved_by = \"Brighton\"\napproved_at = \"2026-11-15\"\n"
    )
}

#[test]
fn approval_parses_and_matches() {
    let approval =
        parse_approval(&approval_text("two-sum", 1)).expect("a valid approval must parse");
    assert!(check_approval(Some(&approval), "two-sum", 1, APPROVAL_HASH).is_ok());
}

#[test]
fn approval_missing_is_refused() {
    assert_eq!(
        check_approval(None, "two-sum", 1, APPROVAL_HASH),
        Err(ApprovalRefusal::ApprovalMissing)
    );
}

#[test]
fn approval_wrong_pack_version_is_mismatch() {
    let approval = parse_approval(&approval_text("two-sum", 1)).expect("parse");
    assert_eq!(
        check_approval(Some(&approval), "two-sum", 2, APPROVAL_HASH),
        Err(ApprovalRefusal::ApprovalMismatch)
    );
}

#[test]
fn approval_hash_mismatch_is_refused() {
    let approval = parse_approval(&approval_text("two-sum", 1)).expect("parse");
    let other = "3".repeat(64);
    assert_eq!(
        check_approval(Some(&approval), "two-sum", 1, &other),
        Err(ApprovalRefusal::ApprovalMismatch)
    );
}

#[test]
fn approval_rejects_unknown_key() {
    let text = format!("{}extra = 1\n", approval_text("two-sum", 1));
    assert!(parse_approval(&text).is_err());
}

#[test]
fn canonical_hash_matches_hand_computed_literal() {
    // Keys are written out of order on purpose; the canonical form sorts them.
    let input = json!({"b": 1, "a": [2, 3]});
    let digest = canonical_hash(&input, &json!(4));
    let hex: String = digest.iter().fold(String::new(), |mut out, b| {
        let _ = write!(out, "{b:02x}");
        out
    });
    assert_eq!(hex, CANONICAL_PAIR_SHA256);
}

#[test]
fn retired_value_match_shows_only_a_prefix() {
    let file = parse_retired(
        "[[retired]]\nslug = \"two-sum\"\nsha256 = \"abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789\"\n\
         [[rotated]]\nslug = \"two-sum\"\npack_version = 2\nrotated_on = \"2026-11-15\"\n",
    )
    .expect("valid retired file");
    let refusal = check_retired(
        &file,
        "two-sum",
        2,
        &["abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789".to_string()],
    )
    .expect_err("a retired value must be refused");
    let text = refusal.to_string();
    assert!(
        text.contains("abcdef012345"),
        "the 12-character prefix is shown"
    );
    assert!(
        !text.contains("abcdef0123456789abcdef"),
        "the full value is not shown: {text}"
    );
    assert!(matches!(refusal, RetiredRefusal::RetiredValue { .. }));
}

#[test]
fn retired_slug_without_rotation_record_is_refused() {
    let file = parse_retired(
        "[[retired]]\nslug = \"two-sum\"\nsha256 = \"abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789\"\n",
    )
    .expect("valid retired file");
    let refusal = check_retired(&file, "two-sum", 2, &[]).expect_err("rotation is required");
    assert!(matches!(refusal, RetiredRefusal::RotationMissing { .. }));
}

#[test]
fn retired_check_ignores_other_slugs() {
    let file = parse_retired("").expect("empty retired file");
    assert!(check_retired(&file, "two-sum", 1, &["x".to_string()]).is_ok());
}
