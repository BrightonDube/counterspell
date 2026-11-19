//! Retired-value check (problem-catalogue C11a, CAT-PT8, CAT-PT13).
//!
//! `retired-values.toml` lists hidden-test values that must never be reused, by hash. A retired
//! value is shown only by its first 12 hex characters. A slug with retired values needs a rotation
//! record before the pack can be imported again.

use serde::Deserialize;
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::fmt;

/// One retired value, identified by its hash.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Retired {
    /// The problem slug.
    pub slug: String,
    /// The retired value's SHA-256, lower-case hex.
    pub sha256: String,
}

/// A record that a slug's hidden data was rotated for a pack version.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Rotated {
    /// The problem slug.
    pub slug: String,
    /// The pack version the rotation is for.
    pub pack_version: i32,
    /// The date the rotation was recorded.
    pub rotated_on: String,
}

/// The parsed `retired-values.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct RetiredFile {
    /// Retired values.
    #[serde(default)]
    pub retired: Vec<Retired>,
    /// Rotation records.
    #[serde(default)]
    pub rotated: Vec<Rotated>,
}

/// Why an import is refused by the retired-value check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetiredRefusal {
    /// A hidden value matches a retired hash. Shows only the first 12 hex characters.
    RetiredValue {
        /// The first 12 hex characters of the retired hash.
        prefix: String,
    },
    /// The slug has retired values but no rotation record for this pack version.
    RotationMissing {
        /// The slug.
        slug: String,
    },
}

impl fmt::Display for RetiredRefusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RetiredValue { prefix } => {
                write!(f, "hidden value matches retired value sha256:{prefix}...")
            }
            Self::RotationMissing { slug } => {
                write!(
                    f,
                    "{slug} has retired values and no rotation record for this version"
                )
            }
        }
    }
}

impl std::error::Error for RetiredRefusal {}

/// Parses `retired-values.toml`.
///
/// # Errors
///
/// Returns a message when the TOML is malformed or has unknown keys.
pub fn parse_retired(text: &str) -> Result<RetiredFile, String> {
    toml::from_str(text).map_err(|error| error.message().to_string())
}

/// Returns the SHA-256 of an input and its expected output, in canonical form.
///
/// Object keys are sorted and no whitespace is added. The two canonical strings are joined with a
/// NUL byte, so `input` and `expected` cannot run into each other.
#[must_use]
pub fn canonical_hash(input: &Value, expected: &Value) -> [u8; 32] {
    let mut bytes = canonical_json(input).into_bytes();
    bytes.push(0);
    bytes.extend_from_slice(canonical_json(expected).as_bytes());
    Sha256::digest(&bytes).into()
}

/// Writes a JSON value in canonical form: sorted object keys, no whitespace.
///
/// The sandbox fill also uses this to compare inputs across sources, so a curated
/// case and a generated case that carry the same value canonicalise identically.
#[must_use]
pub fn canonical_json(value: &Value) -> String {
    match value {
        Value::Object(map) => {
            let mut keys: Vec<&String> = map.keys().collect();
            keys.sort();
            let parts: Vec<String> = keys
                .into_iter()
                .map(|key| {
                    let name = serde_json::to_string(key).unwrap_or_default();
                    format!("{name}:{}", canonical_json(&map[key]))
                })
                .collect();
            format!("{{{}}}", parts.join(","))
        }
        Value::Array(items) => {
            let parts: Vec<String> = items.iter().map(canonical_json).collect();
            format!("[{}]", parts.join(","))
        }
        other => other.to_string(),
    }
}

/// Checks a pack's hidden values against the retired list.
///
/// `case_hashes` holds the SHA-256 of each hidden case in canonical form, as lower-case hex.
///
/// # Errors
///
/// Returns [`RetiredRefusal::RetiredValue`] when a case matches, and
/// [`RetiredRefusal::RotationMissing`] when the slug has retired values but no rotation record for
/// `pack_version`.
pub fn check_retired(
    file: &RetiredFile,
    slug: &str,
    pack_version: i32,
    case_hashes: &[String],
) -> Result<(), RetiredRefusal> {
    let retired_for_slug: Vec<&Retired> = file.retired.iter().filter(|r| r.slug == slug).collect();
    for retired in &retired_for_slug {
        if case_hashes.contains(&retired.sha256) {
            return Err(RetiredRefusal::RetiredValue {
                prefix: retired.sha256.chars().take(12).collect(),
            });
        }
    }
    if !retired_for_slug.is_empty()
        && !file
            .rotated
            .iter()
            .any(|r| r.slug == slug && r.pack_version == pack_version)
    {
        return Err(RetiredRefusal::RotationMissing {
            slug: slug.to_string(),
        });
    }
    Ok(())
}
