//! Stable, bounded error categories for every tool result.
//!
//! Messages never carry absolute host paths, secrets, hidden test data or unbounded
//! program output. A user's wrong answer is never an error here. It is a normal result.

use serde::Serialize;
use serde_json::{json, Value};
use std::path::Path;

/// The longest message returned to a client, in bytes.
pub const MESSAGE_CAP: usize = 512;

/// The category a client can branch on.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCategory {
    /// The request is malformed, outside the schema, or names a path that is not allowed.
    InvalidRequest,
    /// The pack, program or known ghost does not exist.
    NotFound,
    /// No approved sandbox endpoint is configured. Code-running tools refuse to run.
    SandboxNotConfigured,
    /// The sandbox, its image, or its cleanup failed. This is never a user verdict.
    SandboxInfrastructure,
    /// The engine stopped the operation safely, with a reason.
    EngineAborted,
    /// An unexpected adapter failure.
    Internal,
}

/// A tool error that is safe to show to a client.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ToolError {
    /// The stable category.
    pub category: ErrorCategory,
    /// A short message, already bounded and redacted.
    pub message: String,
}

impl ToolError {
    /// Builds an error. The message is redacted against `root` and bounded.
    #[must_use]
    pub fn new(category: ErrorCategory, message: &str, root: &Path) -> Self {
        Self {
            category,
            message: bounded(&redact(message, root), MESSAGE_CAP),
        }
    }

    /// The error as the JSON body of a tool error result.
    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({ "error": self.category, "message": self.message })
    }
}

/// Replaces the workspace root in `text` with `<workspace>`, so no host path leaks.
#[must_use]
pub fn redact(text: &str, root: &Path) -> String {
    let root_text = root.display().to_string();
    if root_text.is_empty() {
        return text.to_string();
    }
    text.replace(&root_text, "<workspace>")
}

/// Cuts `text` to at most `cap` bytes, on a character boundary, and marks the cut.
#[must_use]
pub fn bounded(text: &str, cap: usize) -> String {
    if text.len() <= cap {
        return text.to_string();
    }
    let mut end = cap;
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    format!("{}...", &text[..end])
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn a_workspace_path_in_a_message_is_redacted() {
        let root = PathBuf::from("/home/dev/counterspell");
        let text = "cannot read /home/dev/counterspell/packs/x/meta.toml";
        assert_eq!(
            redact(text, &root),
            "cannot read <workspace>/packs/x/meta.toml"
        );
    }

    #[test]
    fn a_long_message_is_cut_to_the_cap_with_a_marker() {
        let text = "a".repeat(600);
        let cut = bounded(&text, MESSAGE_CAP);
        assert_eq!(cut.len(), MESSAGE_CAP + 3);
        assert!(cut.ends_with("..."));
    }

    #[test]
    fn a_cut_never_splits_a_multibyte_character() {
        // Each "é" is two bytes, so a cap of 3 falls inside the second one.
        let cut = bounded("ééé", 3);
        assert_eq!(cut, "é...");
    }

    #[test]
    fn categories_serialise_as_snake_case_names() {
        let error = ToolError::new(
            ErrorCategory::SandboxNotConfigured,
            "no endpoint",
            Path::new("/x"),
        );
        assert_eq!(error.to_json()["error"], "sandbox_not_configured");
        assert_eq!(error.to_json()["message"], "no endpoint");
    }
}
