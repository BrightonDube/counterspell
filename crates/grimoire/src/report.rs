//! Diagnostics and the validation report, with text and JSON output.

use crate::codes::Code;
use serde::Serialize;
use serde_json::{json, Value};

/// How serious a diagnostic is. Errors fail validation; warnings do not, unless
/// the caller asks for `--deny-warnings`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    /// The pack is invalid.
    Error,
    /// The pack is valid, but a reviewer should look.
    Warning,
}

impl Severity {
    /// The lower-case word used in JSON output.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
        }
    }

    /// The upper-case word used in text output.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Error => "ERROR",
            Self::Warning => "WARNING",
        }
    }
}

/// One finding about one file.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    /// The stable code, such as `MET-02`.
    pub code: &'static str,
    /// The severity taken from the code.
    pub severity: Severity,
    /// The file name inside the pack, such as `meta.toml`.
    pub file: String,
    /// A JSON Pointer or similar location, when there is one.
    pub pointer: Option<String>,
    /// A plain-English message. It never contains hidden test data.
    pub message: String,
}

impl Diagnostic {
    /// Creates a diagnostic for `code`. The severity comes from the code table.
    #[must_use]
    pub fn new(
        code: Code,
        file: impl Into<String>,
        pointer: Option<&str>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            code: code.id,
            severity: code.severity,
            file: file.into(),
            pointer: pointer.map(str::to_owned),
            message: message.into(),
        }
    }

    /// One line for a terminal, such as `ERROR MET-02 meta.toml /limits/time_ms: ...`.
    #[must_use]
    pub fn to_text_line(&self) -> String {
        match &self.pointer {
            Some(pointer) => format!(
                "{} {} {} {}: {}",
                self.severity.label(),
                self.code,
                self.file,
                pointer,
                self.message
            ),
            None => format!(
                "{} {} {}: {}",
                self.severity.label(),
                self.code,
                self.file,
                self.message
            ),
        }
    }

    fn to_json(&self) -> Value {
        json!({
            "code": self.code,
            "severity": self.severity.as_str(),
            "file": self.file,
            "pointer": self.pointer,
            "message": self.message,
        })
    }
}

/// The result of validating one pack.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Report {
    pack: String,
    checked: Vec<String>,
    skipped: Vec<String>,
    diagnostics: Vec<Diagnostic>,
}

impl Report {
    /// Starts an empty report for the pack named `pack`.
    #[must_use]
    pub fn new(pack: impl Into<String>) -> Self {
        Self {
            pack: pack.into(),
            ..Self::default()
        }
    }

    /// Adds one diagnostic.
    pub fn push(&mut self, diagnostic: Diagnostic) {
        self.diagnostics.push(diagnostic);
    }

    /// Records that a validation stage ran.
    pub fn mark_checked(&mut self, stage: &str) {
        self.checked.push(stage.to_owned());
    }

    /// Records that a validation stage was skipped, so an empty error list never
    /// looks like a full pass.
    pub fn mark_skipped(&mut self, stage: &str) {
        self.skipped.push(stage.to_owned());
    }

    /// The stages that ran, in order.
    #[must_use]
    pub fn checked(&self) -> &[String] {
        &self.checked
    }

    /// The stages that were skipped, in order.
    #[must_use]
    pub fn skipped(&self) -> &[String] {
        &self.skipped
    }

    /// All diagnostics in the order they were added.
    #[must_use]
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// The number of error diagnostics.
    #[must_use]
    pub fn error_count(&self) -> usize {
        self.count(Severity::Error)
    }

    /// The number of warning diagnostics.
    #[must_use]
    pub fn warning_count(&self) -> usize {
        self.count(Severity::Warning)
    }

    /// Whether any error diagnostic is present.
    #[must_use]
    pub fn has_errors(&self) -> bool {
        self.error_count() > 0
    }

    /// The JSON form used by `--json` and by the MCP tool.
    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({
            "pack": self.pack,
            "checked": self.checked,
            "skipped": self.skipped,
            "diagnostics": self.diagnostics.iter().map(Diagnostic::to_json).collect::<Vec<_>>(),
            "summary": {
                "errors": self.error_count(),
                "warnings": self.warning_count(),
            },
        })
    }

    fn count(&self, severity: Severity) -> usize {
        self.diagnostics
            .iter()
            .filter(|diagnostic| diagnostic.severity == severity)
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::codes;
    use serde_json::json;

    #[test]
    fn report_counts_errors_and_warnings_separately() {
        let mut report = Report::new("count-pairs");
        report.push(Diagnostic::new(codes::LAY_01, "meta.toml", None, "missing"));
        report.push(Diagnostic::new(codes::MET_02, "meta.toml", None, "bad"));
        report.push(Diagnostic::new(
            codes::SBX_04,
            "known_ghosts.toml",
            None,
            "not minimal",
        ));

        assert_eq!(report.error_count(), 2);
        assert_eq!(report.warning_count(), 1);
        assert!(report.has_errors());
    }

    #[test]
    fn report_without_diagnostics_has_no_errors() {
        let report = Report::new("count-pairs");

        assert!(!report.has_errors());
        assert_eq!(report.warning_count(), 0);
    }

    #[test]
    fn report_json_has_the_contract_fields() {
        let mut report = Report::new("count-pairs");
        report.mark_checked("layout");
        report.mark_skipped("statement");
        report.push(Diagnostic::new(
            codes::MET_02,
            "meta.toml",
            Some("/limits/time_ms"),
            "time_ms must be 100 to 2000",
        ));

        let value = report.to_json();

        assert_eq!(
            value,
            json!({
                "pack": "count-pairs",
                "checked": ["layout"],
                "skipped": ["statement"],
                "diagnostics": [{
                    "code": "MET-02",
                    "severity": "error",
                    "file": "meta.toml",
                    "pointer": "/limits/time_ms",
                    "message": "time_ms must be 100 to 2000"
                }],
                "summary": { "errors": 1, "warnings": 0 }
            })
        );
    }

    #[test]
    fn text_line_names_severity_code_file_and_pointer() {
        let diagnostic = Diagnostic::new(
            codes::MET_02,
            "meta.toml",
            Some("/limits/time_ms"),
            "time_ms must be 100 to 2000",
        );

        assert_eq!(
            diagnostic.to_text_line(),
            "ERROR MET-02 meta.toml /limits/time_ms: time_ms must be 100 to 2000"
        );
    }

    #[test]
    fn text_line_omits_pointer_when_there_is_none() {
        let diagnostic = Diagnostic::new(codes::SBX_04, "known_ghosts.toml", None, "not minimal");

        assert_eq!(
            diagnostic.to_text_line(),
            "WARNING SBX-04 known_ghosts.toml: not minimal"
        );
    }

    #[test]
    fn diagnostic_severity_comes_from_its_code() {
        let warning = Diagnostic::new(codes::SBX_04, "known_ghosts.toml", None, "x");
        let error = Diagnostic::new(codes::KNG_04, "known_ghosts.toml", None, "x");

        assert_eq!(warning.severity, Severity::Warning);
        assert_eq!(error.severity, Severity::Error);
    }
}
