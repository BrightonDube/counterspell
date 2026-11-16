//! The originality guard (PIP-3, PIP-8, PIP-46): text markers and the S1 record.
//!
//! The marker list comes in as data from the private repository. It is never
//! written into this code. Diagnostics carry a code and a file pointer only, so
//! a report never repeats the text it matched.

use std::fmt;

/// The five sections the S1 record needs, matched case-insensitively (PIP-8).
///
/// The requirements name the record's contents but not its headings. These five
/// headings are the assumed choice, recorded in the pipeline blockers log.
pub const REQUIRED_SECTIONS: [&str; 5] = [
    "idea",
    "why it is original",
    "patterns used",
    "checklist answers",
    "third-party text avoided",
];

use super::Diagnostic;

/// The non-empty, lowercased originality markers.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Markers {
    lowercase: Vec<String>,
}

/// Why a marker list is refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerError {
    /// A marker is empty or only spaces, so it would match every line.
    Blank,
}

impl fmt::Display for MarkerError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "an originality marker cannot be blank")
    }
}

impl std::error::Error for MarkerError {}

impl Markers {
    /// Builds the marker list. Matching ignores case.
    ///
    /// # Errors
    ///
    /// Returns [`MarkerError::Blank`] when any marker is empty or only spaces.
    pub fn new(markers: Vec<String>) -> Result<Self, MarkerError> {
        let mut lowercase = Vec::with_capacity(markers.len());
        for marker in markers {
            if marker.trim().is_empty() {
                return Err(MarkerError::Blank);
            }
            lowercase.push(marker.trim().to_lowercase());
        }
        Ok(Self { lowercase })
    }
}

/// Reports `ORG-01` for each line of `text` that contains a marker.
#[must_use]
pub fn scan_text(markers: &Markers, file: &str, text: &str) -> Vec<Diagnostic> {
    text.lines()
        .enumerate()
        .filter(|(_, line)| {
            let lower = line.to_lowercase();
            markers
                .lowercase
                .iter()
                .any(|marker| lower.contains(marker.as_str()))
        })
        .map(|(index, _)| Diagnostic {
            code: "ORG-01",
            pointer: format!("{file} line {}", index + 1),
        })
        .collect()
}

/// Checks that the S1 record has every required section with some text in it.
///
/// Returns `ORG-02` when the record is missing a section or a section is empty.
#[must_use]
pub fn check_record(text: &str) -> Option<Diagnostic> {
    let sections = section_text_flags(text);
    let complete = REQUIRED_SECTIONS.iter().all(|required| {
        sections
            .iter()
            .any(|(title, has_text)| title == required && *has_text)
    });
    if complete {
        None
    } else {
        Some(Diagnostic {
            code: "ORG-02",
            pointer: "originality.md".to_owned(),
        })
    }
}

/// Runs the whole guard over one pack: every file is scanned for markers, and the record is checked.
///
/// `record` is `None` when `originality.md` is missing. `files` should include the record too, so its text is scanned for markers.
#[must_use]
pub fn check_pack(
    markers: &Markers,
    files: &[(&str, &str)],
    record: Option<&str>,
) -> Vec<Diagnostic> {
    let mut diagnostics: Vec<Diagnostic> = files
        .iter()
        .flat_map(|(file, text)| scan_text(markers, file, text))
        .collect();
    match record {
        Some(text) => diagnostics.extend(check_record(text)),
        None => diagnostics.push(Diagnostic {
            code: "ORG-02",
            pointer: "originality.md".to_owned(),
        }),
    }
    diagnostics
}

/// For each `## ` heading, the lowercased title and whether any non-empty line follows it before the next heading.
fn section_text_flags(text: &str) -> Vec<(String, bool)> {
    let mut sections: Vec<(String, bool)> = Vec::new();
    for line in text.lines() {
        if let Some(title) = line.strip_prefix("## ") {
            sections.push((title.trim().to_lowercase(), false));
        } else if !line.trim().is_empty() {
            if let Some(current) = sections.last_mut() {
                current.1 = true;
            }
        }
    }
    sections
}
