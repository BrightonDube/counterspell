//! Metadata model for meta.toml (MET-*), plus the slug rule (LAY-06).
//!
//! Unknown keys are rejected at every level, so a typo cannot slip past review.
//! Field types are held as plain values here; the signature, limits and schema
//! checks in later modules give them their launch meaning.

use crate::codes;
use crate::limits::Limits;
use crate::report::{Diagnostic, Report};
use counterspell_engine::{Comparator, Schema};
use serde::Deserialize;
use serde_json::Value;

/// The parsed `meta.toml`. Every field is required unless it is marked optional.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Meta {
    /// The problem id. Must equal the directory name.
    pub slug: String,
    /// The catalogue title, 1 to 80 characters.
    pub title: String,
    /// `draft` or `published`.
    pub status: String,
    /// `easy`, `medium` or `hard`.
    pub difficulty: String,
    /// The catalogue track, kebab-case.
    pub track: String,
    /// One to five unique kebab-case tags.
    pub concepts: Vec<String>,
    /// Must be `original`.
    pub origin: String,
    /// The author byline, 1 to 60 characters.
    pub author: String,
    /// Must be `MIT`.
    pub licence: String,
    /// The pack version, 1 or more.
    pub pack_version: i64,
    /// The Python function name the learner implements.
    pub function: String,
    /// The parameters, in call order.
    pub params: Vec<Param>,
    /// The return type name.
    pub returns: String,
    /// `exact` or `unordered`.
    pub comparator: String,
    /// The per-test time and memory caps.
    pub limits: Limits,
    /// The engine schema for the argument tuple.
    pub input_schema: Schema,
    /// The seed and hidden-test count.
    pub generator: Generator,
    /// The public examples.
    pub samples: Vec<Sample>,
    /// The declared complexity class, when there is a bench.
    #[serde(default)]
    pub reference_complexity: Option<String>,
    /// The bench table, when a complexity is declared.
    #[serde(default)]
    pub bench: Option<Bench>,
    /// A short note on complexity, at most 240 characters.
    #[serde(default)]
    pub complexity_note: Option<String>,
}

/// One declared parameter.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Param {
    /// The parameter name, unique within the function.
    pub name: String,
    /// The launch type name, such as `int` or `list[int]`.
    #[serde(rename = "type")]
    pub kind: String,
}

/// The generator settings for hidden tests.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Generator {
    /// The seed. TOML integers are signed 64-bit, so this is 0 to 2^63 - 1.
    pub seed: i64,
    /// The number of hidden tests, 5 to 50.
    pub hidden_count: i64,
}

/// One public example.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Sample {
    /// The argument list in parameter order.
    pub input: Value,
    /// The expected return value.
    pub expected: Value,
}

/// The benchmark table, present only when a complexity is declared.
#[derive(Clone, Debug, Eq, PartialEq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bench {
    /// The parameter whose size grows.
    pub size_param: String,
    /// The sizes to time, strictly increasing.
    pub sizes: Vec<i64>,
}

impl Meta {
    /// The engine comparator named by the `comparator` field.
    ///
    /// Any name other than `unordered` means exact. MET-07 already rejects unknown
    /// names, so a checked pack never reaches the fallback.
    #[must_use]
    pub fn engine_comparator(&self) -> Comparator {
        if self.comparator == "unordered" {
            Comparator::Unordered
        } else {
            Comparator::Exact
        }
    }
}

/// Parses and checks `meta.toml`. Returns the model when the file parses, even if a
/// field has a bad value; the diagnostics say what is wrong. Returns `None` when the
/// file cannot be read as the model at all.
#[must_use]
pub fn check(text: &str, dir_name: &str, report: &mut Report) -> Option<Meta> {
    if let Err(error) = toml::from_str::<toml::Table>(text) {
        report.push(Diagnostic::new(
            codes::MET_01,
            FILE,
            None,
            format!(
                "TOML syntax error at {}: {}",
                location(text, error.span()),
                error.message()
            ),
        ));
        return None;
    }
    let meta = match toml::from_str::<Meta>(text) {
        Ok(meta) => meta,
        Err(error) => {
            let code = if error.message().contains("unknown field") {
                codes::MET_01
            } else {
                codes::MET_02
            };
            report.push(Diagnostic::new(
                code,
                FILE,
                None,
                format!("{} at {}", error.message(), location(text, error.span())),
            ));
            return None;
        }
    };
    check_values(&meta, dir_name, report);
    Some(meta)
}

/// The file name this module reports on.
const FILE: &str = "meta.toml";

/// Checks the values of fields that deserialised correctly.
fn check_values(meta: &Meta, dir_name: &str, report: &mut Report) {
    if !is_slug(dir_name) || dir_name != meta.slug {
        report.push(Diagnostic::new(
            codes::LAY_06,
            FILE,
            Some("/slug"),
            "directory name must be a valid slug and equal the slug in meta.toml",
        ));
    }
    if !matches!(meta.status.as_str(), "draft" | "published") {
        report.push(bad_value("/status", "status must be draft or published"));
    }
    if !matches!(meta.difficulty.as_str(), "easy" | "medium" | "hard") {
        report.push(bad_value(
            "/difficulty",
            "difficulty must be easy, medium or hard",
        ));
    }
    let title_length = meta.title.chars().count();
    if !(1..=80).contains(&title_length) {
        report.push(bad_value("/title", "title must be 1 to 80 characters"));
    }
    let author_length = meta.author.chars().count();
    if !(1..=60).contains(&author_length) {
        report.push(bad_value("/author", "author must be 1 to 60 characters"));
    }
    if !is_kebab(&meta.track) {
        report.push(bad_value("/track", "track must be kebab-case"));
    }
    check_concepts(&meta.concepts, report);
    if meta.origin != "original" {
        report.push(Diagnostic::new(
            codes::MET_03,
            FILE,
            Some("/origin"),
            "origin must be \"original\"; only original problems are accepted",
        ));
    }
    if meta.licence != "MIT" {
        report.push(Diagnostic::new(
            codes::MET_03,
            FILE,
            Some("/licence"),
            "licence must be \"MIT\"",
        ));
    }
    if meta.pack_version < 1 {
        report.push(bad_value("/pack_version", "pack_version must be 1 or more"));
    }
    if !is_identifier(&meta.function) {
        report.push(bad_value(
            "/function",
            "function must be a lower-case Python identifier of up to 64 characters",
        ));
    }
}

/// Checks the concept list: one to five unique kebab-case tags.
fn check_concepts(concepts: &[String], report: &mut Report) {
    if !(1..=5).contains(&concepts.len()) {
        report.push(bad_value("/concepts", "concepts must have 1 to 5 tags"));
        return;
    }
    if concepts.iter().any(|concept| !is_kebab(concept)) {
        report.push(bad_value("/concepts", "each concept must be kebab-case"));
    }
    let mut seen = std::collections::BTreeSet::new();
    if !concepts.iter().all(|concept| seen.insert(concept)) {
        report.push(bad_value("/concepts", "concepts must be unique"));
    }
}

/// A MET-02 diagnostic for a field with a bad value.
fn bad_value(pointer: &str, message: &str) -> Diagnostic {
    Diagnostic::new(codes::MET_02, FILE, Some(pointer), message)
}

/// Describes a byte span as `line L, column C`, or says the position is unknown.
fn location(text: &str, span: Option<std::ops::Range<usize>>) -> String {
    let Some(span) = span else {
        return "an unknown position".to_owned();
    };
    let before = &text[..span.start.min(text.len())];
    let line = before.matches('\n').count() + 1;
    let column = before
        .rfind('\n')
        .map_or(before.len(), |newline| before.len() - newline - 1)
        + 1;
    format!("line {line}, column {column}")
}

/// Whether `text` is a valid slug: lower-case, 3 to 40 characters, digits and hyphens.
fn is_slug(text: &str) -> bool {
    let length = text.chars().count();
    (3..=40).contains(&length)
        && text.starts_with(|character: char| character.is_ascii_lowercase())
        && text.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '-'
        })
}

/// Whether `text` is kebab-case: lower-case words of letters or digits joined by hyphens.
pub(crate) fn is_kebab(text: &str) -> bool {
    text.starts_with(|character: char| character.is_ascii_lowercase())
        && text.split('-').all(|part| {
            !part.is_empty()
                && part
                    .chars()
                    .all(|character| character.is_ascii_lowercase() || character.is_ascii_digit())
        })
}

/// Whether `text` is a lower-case Python identifier of at most 64 characters.
fn is_identifier(text: &str) -> bool {
    let length = text.chars().count();
    (1..=64).contains(&length)
        && text.starts_with(|character: char| character.is_ascii_lowercase() || character == '_')
        && text.chars().all(|character| {
            character.is_ascii_lowercase() || character.is_ascii_digit() || character == '_'
        })
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A complete, valid `meta.toml` for the slug `count-pairs`.
    fn valid() -> String {
        r#"slug = "count-pairs"
title = "Count the Pairs"
status = "draft"
difficulty = "easy"
track = "arrays"
concepts = ["arrays", "pairs", "nested-loops"]
origin = "original"
author = "Brighton"
licence = "MIT"
pack_version = 1
function = "count_pairs"
returns = "int"
comparator = "exact"

[[params]]
name = "values"
type = "list[int]"

[[params]]
name = "target"
type = "int"

[limits]
time_ms = 2000
mem_mb = 256

[input_schema]
type = "tuple"
items = [
  { type = "int_array", min_len = 0, max_len = 30, min = -50, max = 50 },
  { type = "int", min = -100, max = 100 },
]

[generator]
seed = 1729
hidden_count = 12

[[samples]]
input = [[1, 2, 3, 2], 4]
expected = 2
"#
        .to_owned()
    }

    fn codes_for(text: &str, dir_name: &str) -> (Option<Meta>, Vec<String>) {
        let mut report = Report::new("count-pairs");
        let meta = check(text, dir_name, &mut report);
        let codes = report
            .diagnostics()
            .iter()
            .map(|diagnostic| diagnostic.code.to_owned())
            .collect();
        (meta, codes)
    }

    #[test]
    fn valid_meta_parses_with_no_diagnostics() {
        let (meta, codes) = codes_for(&valid(), "count-pairs");

        assert!(meta.is_some());
        assert!(codes.is_empty(), "unexpected codes: {codes:?}");
    }

    #[test]
    fn unknown_key_is_met_01_with_its_line() {
        let text = valid().replace("licence = \"MIT\"", "licence = \"MIT\"\nsource_url = \"x\"");
        let mut report = Report::new("count-pairs");

        let meta = check(&text, "count-pairs", &mut report);

        assert!(meta.is_none());
        assert_eq!(report.diagnostics()[0].code, "MET-01");
        assert!(report.diagnostics()[0].message.contains("line 10"));
    }

    #[test]
    fn toml_syntax_error_is_met_01() {
        let (meta, codes) = codes_for("slug = \n", "count-pairs");

        assert!(meta.is_none());
        assert_eq!(codes, ["MET-01"]);
    }

    #[test]
    fn missing_licence_is_met_02() {
        let text = valid().replace("licence = \"MIT\"\n", "");

        let (meta, codes) = codes_for(&text, "count-pairs");

        assert!(meta.is_none());
        assert_eq!(codes, ["MET-02"]);
    }

    #[test]
    fn licence_other_than_mit_is_met_03() {
        let text = valid().replace("licence = \"MIT\"", "licence = \"GPL-3.0\"");

        let (_, codes) = codes_for(&text, "count-pairs");

        assert_eq!(codes, ["MET-03"]);
    }

    #[test]
    fn origin_other_than_original_is_met_03() {
        let text = valid().replace("origin = \"original\"", "origin = \"hackerrank\"");

        let (_, codes) = codes_for(&text, "count-pairs");

        assert_eq!(codes, ["MET-03"]);
    }

    #[test]
    fn slug_that_differs_from_directory_is_lay_06() {
        let (_, codes) = codes_for(&valid(), "count-couples");

        assert_eq!(codes, ["LAY-06"]);
    }

    #[test]
    fn directory_name_that_is_not_a_slug_is_lay_06() {
        let text = valid().replace("count-pairs", "Count_Pairs");

        let (_, codes) = codes_for(&text, "Count_Pairs");

        assert_eq!(codes, ["LAY-06"]);
    }

    #[test]
    fn title_over_80_characters_is_met_02() {
        let text = valid().replace("Count the Pairs", &"t".repeat(81));

        let (_, codes) = codes_for(&text, "count-pairs");

        assert_eq!(codes, ["MET-02"]);
    }

    #[test]
    fn title_of_exactly_80_characters_is_allowed() {
        let text = valid().replace("Count the Pairs", &"t".repeat(80));

        let (_, codes) = codes_for(&text, "count-pairs");

        assert!(codes.is_empty(), "unexpected codes: {codes:?}");
    }

    #[test]
    fn six_concepts_is_met_02() {
        let text = valid().replace(
            "[\"arrays\", \"pairs\", \"nested-loops\"]",
            "[\"a\", \"b\", \"c\", \"d\", \"e\", \"f\"]",
        );

        let (_, codes) = codes_for(&text, "count-pairs");

        assert_eq!(codes, ["MET-02"]);
    }

    #[test]
    fn duplicate_concepts_are_met_02() {
        let text = valid().replace(
            "[\"arrays\", \"pairs\", \"nested-loops\"]",
            "[\"arrays\", \"arrays\"]",
        );

        let (_, codes) = codes_for(&text, "count-pairs");

        assert_eq!(codes, ["MET-02"]);
    }

    #[test]
    fn concept_that_is_not_kebab_case_is_met_02() {
        let text = valid().replace("\"nested-loops\"", "\"Nested_Loops\"");

        let (_, codes) = codes_for(&text, "count-pairs");

        assert_eq!(codes, ["MET-02"]);
    }

    #[test]
    fn pack_version_zero_is_met_02() {
        let text = valid().replace("pack_version = 1", "pack_version = 0");

        let (_, codes) = codes_for(&text, "count-pairs");

        assert_eq!(codes, ["MET-02"]);
    }

    #[test]
    fn function_name_with_capitals_is_met_02() {
        let text = valid().replace("function = \"count_pairs\"", "function = \"CountPairs\"");

        let (_, codes) = codes_for(&text, "count-pairs");

        assert_eq!(codes, ["MET-02"]);
    }

    #[test]
    fn unknown_difficulty_is_met_02() {
        let text = valid().replace("difficulty = \"easy\"", "difficulty = \"impossible\"");

        let (_, codes) = codes_for(&text, "count-pairs");

        assert_eq!(codes, ["MET-02"]);
    }

    #[test]
    fn unknown_status_is_met_02() {
        let text = valid().replace("status = \"draft\"", "status = \"archived\"");

        let (_, codes) = codes_for(&text, "count-pairs");

        assert_eq!(codes, ["MET-02"]);
    }
}
