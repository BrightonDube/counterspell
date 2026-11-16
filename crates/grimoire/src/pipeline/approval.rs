//! Publish and approval checks (PIP-28, PIP-29, PIP-36, D9, P1).
//!
//! A pack is `published` only with an approval whose hash equals the hash of the
//! assembled manifest. The hash itself comes from `scripts/pack-assemble.sh`. This
//! module only compares it. Any change to a file changes the manifest hash, so an
//! approval made before the change no longer matches (APR-01).

use serde::Deserialize;

use super::Diagnostic;

/// The `status` value that marks a pack as approved for import.
const PUBLISHED: &str = "published";

/// The contents of `approval.toml`, written by Brighton in the private tree (PIP-28).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Approval {
    pub slug: String,
    pub pack_version: u32,
    pub content_sha256: String,
    pub public_commit: String,
    pub approved_by: String,
    pub approved_at: String,
}

impl Approval {
    /// Parses an `approval.toml` text. Unknown keys are refused.
    ///
    /// # Errors
    ///
    /// Returns the TOML error when a field is missing, has the wrong type, or is unknown.
    pub fn from_toml(text: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(text)
    }
}

/// Reads the `status` value from a `meta.toml` text, if there is one.
///
/// # Errors
///
/// Returns the TOML error when the text is not valid TOML.
pub fn meta_status(meta_text: &str) -> Result<Option<String>, toml::de::Error> {
    let table: toml::Table = toml::from_str(meta_text)?;
    Ok(table
        .get("status")
        .and_then(toml::Value::as_str)
        .map(str::to_owned))
}

/// Checks a `published` pack against its approval and the assembled manifest hash.
///
/// Returns `PUB-01` when there is no approval, and `APR-01` when the approval names another
/// hash. A `draft` pack needs no approval and gives no finding.
#[must_use]
pub fn check_publish(
    status: Option<&str>,
    approval: Option<&Approval>,
    manifest_sha256: &str,
) -> Vec<Diagnostic> {
    if status != Some(PUBLISHED) {
        return Vec::new();
    }
    match approval {
        None => vec![Diagnostic {
            code: "PUB-01",
            pointer: "approval.toml".to_owned(),
        }],
        Some(approval) if approval.content_sha256 != manifest_sha256 => vec![Diagnostic {
            code: "APR-01",
            pointer: "approval.toml content_sha256".to_owned(),
        }],
        Some(_) => Vec::new(),
    }
}
