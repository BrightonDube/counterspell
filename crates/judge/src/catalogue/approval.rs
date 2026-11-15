//! Approval record (problem-catalogue C11a, CAT-PT7, CAT-PT8).
//!
//! `approval.toml` names the slug, the pack version, the combined content hash that Brighton
//! approved, and the public commit. A missing or mismatched record refuses the import before the
//! sandbox runs. Unknown keys are errors.

use serde::Deserialize;
use std::fmt;

/// The parsed `approval.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Approval {
    /// The problem slug the approval is for.
    pub slug: String,
    /// The pack version the approval is for.
    pub pack_version: i32,
    /// The approved combined content hash, 64 lower-case hex characters.
    pub content_sha256: String,
    /// The public commit identifier, 40 hex characters. Recorded, not verified here.
    pub public_commit: String,
    /// Who approved the pack.
    pub approved_by: String,
    /// When the pack was approved, as written by the approver.
    pub approved_at: String,
}

/// Why an approval record is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ApprovalRefusal {
    /// No approval record exists.
    ApprovalMissing,
    /// The record is for another slug, version or content hash.
    ApprovalMismatch,
}

impl fmt::Display for ApprovalRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::ApprovalMissing => f.write_str("approval_missing"),
            Self::ApprovalMismatch => f.write_str("approval_mismatch"),
        }
    }
}

impl std::error::Error for ApprovalRefusal {}

/// Parses `approval.toml` and checks the hash fields' shape.
///
/// # Errors
///
/// Returns a message when the TOML is malformed, has unknown keys, or a hash has the wrong shape.
pub fn parse_approval(text: &str) -> Result<Approval, String> {
    let approval: Approval = toml::from_str(text).map_err(|error| error.message().to_string())?;
    if !is_lower_hex(&approval.content_sha256, 64) {
        return Err("content_sha256 must be 64 lower-case hex characters".to_string());
    }
    if !is_lower_hex(&approval.public_commit, 40) {
        return Err("public_commit must be 40 lower-case hex characters".to_string());
    }
    Ok(approval)
}

/// Checks the approval for one pack.
///
/// # Errors
///
/// Returns [`ApprovalRefusal::ApprovalMissing`] when there is no record, and
/// [`ApprovalRefusal::ApprovalMismatch`] when the slug, version or hash differs.
pub fn check_approval(
    approval: Option<&Approval>,
    slug: &str,
    pack_version: i32,
    combined_hash_hex: &str,
) -> Result<(), ApprovalRefusal> {
    let approval = approval.ok_or(ApprovalRefusal::ApprovalMissing)?;
    if approval.slug != slug
        || approval.pack_version != pack_version
        || approval.content_sha256 != combined_hash_hex
    {
        return Err(ApprovalRefusal::ApprovalMismatch);
    }
    Ok(())
}

fn is_lower_hex(text: &str, length: usize) -> bool {
    text.len() == length
        && text
            .chars()
            .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c))
}
