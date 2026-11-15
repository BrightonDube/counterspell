//! Report model and renderers (problem-catalogue C13, CAT-I20, CAT-X1, CAT-X2, CAT-X4).
//!
//! A report holds only slugs, versions, stable codes, counts and a 12-character hash prefix. It
//! never holds pack bytes, hidden values or program output, so printing it cannot leak them.
//! The renderers take `&PackReport`, so printing never moves data out of the report.

use std::collections::BTreeMap;

/// Hex characters of the content hash that a report may show.
pub const HASH_PREFIX_CHARS: usize = 12;

/// What happened to one pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    /// A new version was written.
    Created,
    /// The stored version already matches. Nothing was written.
    Unchanged,
    /// The import was refused for a stable code.
    Refused(String),
    /// The import failed for a stable code.
    Failed(String),
}

impl Outcome {
    /// Returns the label used in text output, such as `created` or `refused:version_not_bumped`.
    #[must_use]
    pub fn label(&self) -> String {
        match self {
            Self::Created => "created".to_string(),
            Self::Unchanged => "unchanged".to_string(),
            Self::Refused(code) => format!("refused:{code}"),
            Self::Failed(code) => format!("failed:{code}"),
        }
    }

    /// Returns the short outcome name used in JSON output.
    #[must_use]
    pub const fn kind(&self) -> &'static str {
        match self {
            Self::Created => "created",
            Self::Unchanged => "unchanged",
            Self::Refused(_) => "refused",
            Self::Failed(_) => "failed",
        }
    }
}

/// The report for one pack.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PackReport {
    /// The problem slug.
    pub slug: String,
    /// The version involved, when there is one.
    pub version: Option<i32>,
    /// What happened.
    pub outcome: Outcome,
    /// Named counts, for example files and hidden tests.
    pub counts: BTreeMap<&'static str, usize>,
    /// The full content hash as hex. Only the prefix is ever rendered.
    pub hash_hex: Option<String>,
}

impl PackReport {
    /// Renders one line of text. Shows the slug, outcome, version, hash prefix and counts.
    #[must_use]
    pub fn render_text(&self) -> String {
        let mut parts = vec![format!("{} {}", self.slug, self.outcome.label())];
        if let Some(version) = self.version {
            parts.push(format!("version={version}"));
        }
        if let Some(prefix) = self.hash_prefix() {
            parts.push(format!("hash={prefix}"));
        }
        for (name, count) in &self.counts {
            parts.push(format!("{name}={count}"));
        }
        parts.join(" ")
    }

    /// Renders the report as a JSON object with the same fields.
    #[must_use]
    pub fn render_json(&self) -> serde_json::Value {
        serde_json::json!({
            "slug": self.slug,
            "version": self.version,
            "outcome": self.outcome.kind(),
            "code": match &self.outcome {
                Outcome::Refused(code) | Outcome::Failed(code) => Some(code.as_str()),
                Outcome::Created | Outcome::Unchanged => None,
            },
            "hash": self.hash_prefix(),
            "counts": self.counts,
        })
    }

    fn hash_prefix(&self) -> Option<String> {
        self.hash_hex
            .as_ref()
            .map(|hash| hash.chars().take(HASH_PREFIX_CHARS).collect())
    }
}
