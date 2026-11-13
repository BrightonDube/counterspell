//! Known ghost checks, static part (KNG-01 to KNG-06).

use crate::codes;
use crate::meta::{is_kebab, Meta};
use crate::pack::Pack;
use crate::reference::defines_function;
use crate::report::{Diagnostic, Report};
use crate::samples::accepts_return;
use crate::signature::LaunchType;
use counterspell_engine::{FailureKind, KnownGhost};
use serde::Deserialize;
use serde_json::Value;

/// The shape of `known_ghosts.toml`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GhostFile {
    #[serde(default)]
    ghost: Vec<GhostEntry>,
}

/// One authored known ghost.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct GhostEntry {
    id: String,
    program: String,
    kind: String,
    input: Value,
    #[serde(default)]
    got: Option<Value>,
    #[serde(default)]
    exception_class: Option<String>,
    explanation: String,
    #[serde(default)]
    concepts: Vec<String>,
    #[serde(default)]
    dogfood: bool,
}

/// The most known ghosts a pack may hold.
const MAX_GHOSTS: usize = 10;

/// The longest explanation, in bytes.
const MAX_EXPLANATION_BYTES: usize = 400;

/// Phrases an explanation may not use. Copied from the engine's R9.6 list; a test checks the copy.
pub const FORBIDDEN_PHRASES: [&str; 6] = [
    "smallest possible",
    "global",
    "proven",
    "optimal",
    "the minimum",
    "smallest",
];

/// One known ghost as the sandbox needs it: the program that shows it and the engine's record.
pub(crate) struct SandboxGhost {
    /// The `incorrect/` file that the ghost runs.
    pub(crate) program: String,
    /// The ghost as the engine's classifier and matcher read it.
    pub(crate) known: KnownGhost,
}

/// Reads the known ghosts a sandbox run can use.
///
/// A ghost with an unknown kind, or a file that does not parse, is left out here.
/// The static checks already report both, and they run first.
pub(crate) fn sandbox_ghosts(text: &str) -> Vec<SandboxGhost> {
    let Ok(file) = toml::from_str::<GhostFile>(text) else {
        return Vec::new();
    };
    file.ghost
        .into_iter()
        .filter_map(|entry| {
            let kind = kind_of(&entry.kind)?;
            Some(SandboxGhost {
                program: entry.program,
                known: KnownGhost {
                    id: entry.id,
                    input: entry.input,
                    kind,
                    exception_class: entry.exception_class,
                    got: entry.got,
                    explanation: entry.explanation,
                    concept_tags: entry.concepts,
                },
            })
        })
        .collect()
}

/// Maps the TOML kind name to the engine's failure kind.
fn kind_of(name: &str) -> Option<FailureKind> {
    match name {
        "wrong" => Some(FailureKind::WrongAnswer),
        "crash" => Some(FailureKind::RuntimeError),
        "timeout" => Some(FailureKind::Timeout),
        _ => None,
    }
}

/// Checks `known_ghosts.toml` against the schema, the programs, the kind rules and the wording.
pub fn check(text: &str, meta: &Meta, dir_name: &str, pack: &Pack, report: &mut Report) {
    let file = match toml::from_str::<GhostFile>(text) {
        Ok(file) => file,
        Err(error) => {
            report.push(Diagnostic::new(
                codes::KNG_02,
                "known_ghosts.toml",
                None,
                format!("known_ghosts.toml does not parse: {}", error.message()),
            ));
            return;
        }
    };
    if file.ghost.len() > MAX_GHOSTS {
        report.push(Diagnostic::new(
            codes::KNG_01,
            "known_ghosts.toml",
            None,
            "a pack may hold at most 10 known ghosts",
        ));
    }
    let mut ids = std::collections::BTreeSet::new();
    for ghost in &file.ghost {
        if !is_kebab(&ghost.id) || !ids.insert(ghost.id.as_str()) {
            report.push(Diagnostic::new(
                codes::KNG_01,
                "known_ghosts.toml",
                None,
                format!("ghost id {:?} must be unique kebab-case", ghost.id),
            ));
        }
        check_ghost(ghost, meta, dir_name, pack, report);
    }
}

/// Checks one ghost against the schema, its program, its kind rules, its wording and its concepts.
fn check_ghost(ghost: &GhostEntry, meta: &Meta, dir_name: &str, pack: &Pack, report: &mut Report) {
    let id = ghost.id.as_str();
    if meta.input_schema.contains(&ghost.input).is_err() {
        report.push(Diagnostic::new(
            codes::KNG_02,
            "known_ghosts.toml",
            None,
            format!("ghost {id:?} input does not fit the input schema"),
        ));
    }
    let program_defines = ghost.program.starts_with("incorrect/")
        && pack
            .text(&ghost.program)
            .is_some_and(|source| defines_function(source, &meta.function));
    if !program_defines {
        report.push(Diagnostic::new(
            codes::KNG_02,
            "known_ghosts.toml",
            None,
            format!(
                "ghost {id:?} program must be an incorrect/ file that defines {}",
                meta.function
            ),
        ));
    }
    check_kind(ghost, meta, report);
    let explanation = ghost.explanation.as_str();
    let lower = explanation.to_lowercase();
    if !(1..=MAX_EXPLANATION_BYTES).contains(&explanation.len())
        || FORBIDDEN_PHRASES
            .iter()
            .any(|phrase| lower.contains(phrase))
    {
        report.push(Diagnostic::new(
            codes::KNG_04,
            "known_ghosts.toml",
            None,
            format!(
                "ghost {id:?} explanation must be 1 to 400 bytes and must not claim minimality"
            ),
        ));
    }
    if !ghost
        .concepts
        .iter()
        .all(|concept| meta.concepts.contains(concept))
    {
        report.push(Diagnostic::new(
            codes::KNG_05,
            "known_ghosts.toml",
            None,
            format!("ghost {id:?} uses a concept that is not in the pack"),
        ));
    }
    if ghost.dogfood && !dir_name.starts_with('_') {
        report.push(Diagnostic::new(
            codes::KNG_06,
            "known_ghosts.toml",
            None,
            format!("ghost {id:?} sets dogfood outside an underscore pack"),
        ));
    }
}

/// Checks the `got` and `exception_class` fields against the ghost's kind.
fn check_kind(ghost: &GhostEntry, meta: &Meta, report: &mut Report) {
    let id = ghost.id.as_str();
    let bad = |message: &str| {
        Diagnostic::new(
            codes::KNG_03,
            "known_ghosts.toml",
            None,
            format!("ghost {id:?} {message}"),
        )
    };
    match ghost.kind.as_str() {
        "wrong" => {
            match &ghost.got {
                None => report.push(bad("is wrong-answer but has no got value")),
                Some(got) => {
                    if !LaunchType::parse(&meta.returns)
                        .is_some_and(|kind| accepts_return(kind, got))
                    {
                        report.push(bad("has a got value that does not match the return type"));
                    }
                }
            }
            if ghost.exception_class.is_some() {
                report.push(bad("is wrong-answer and must not have an exception_class"));
            }
        }
        "crash" => {
            if ghost.got.is_some() {
                report.push(bad("is a crash and must not have a got value"));
            }
            if !ghost
                .exception_class
                .as_deref()
                .is_some_and(valid_exception_class)
            {
                report.push(bad("is a crash and needs a valid exception_class"));
            }
        }
        "timeout" => {
            if ghost.got.is_some() || ghost.exception_class.is_some() {
                report.push(bad(
                    "is a timeout and must have neither got nor exception_class",
                ));
            }
        }
        _ => report.push(bad("has an unknown kind")),
    }
}

/// Whether `name` looks like a Python exception class name, such as `ValueError`.
fn valid_exception_class(name: &str) -> bool {
    let mut characters = name.chars();
    name.len() <= 64
        && characters
            .next()
            .is_some_and(|first| first.is_ascii_alphabetic() || first == '_')
        && characters.all(|character| {
            character.is_ascii_alphanumeric() || character == '_' || character == '.'
        })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;

    const PROGRAM: &str = "def count_pairs(values, target):\n    return 1\n";

    /// A pack that holds the one incorrect program the ghosts refer to.
    fn pack() -> Pack {
        Pack::with_text(&[("incorrect/same-index.py", PROGRAM)])
    }

    /// One valid wrong-answer ghost, as a TOML entry.
    fn ghost(id: &str) -> String {
        format!(
            "[[ghost]]\nid = \"{id}\"\nprogram = \"incorrect/same-index.py\"\nkind = \"wrong\"\ninput = [[0], 0]\ngot = 1\nconcepts = [\"pairs\"]\nexplanation = \"A value cannot pair with itself.\"\n\n"
        )
    }

    fn codes_for(ghosts: &str, dir_name: &str, pack: &Pack) -> Vec<String> {
        let meta = fixtures::parse(&fixtures::valid_text());
        let mut report = Report::new(dir_name);
        check(ghosts, &meta, dir_name, pack, &mut report);
        report
            .diagnostics()
            .iter()
            .map(|d| d.code.to_owned())
            .collect()
    }

    #[test]
    fn valid_ghost_has_no_codes() {
        assert_eq!(
            codes_for(&ghost("same-index"), "count-pairs", &pack()),
            Vec::<String>::new()
        );
    }

    #[test]
    fn eleven_ghosts_is_kng_01() {
        let text: String = (0..11)
            .map(|index| ghost(&format!("ghost-{index}")))
            .collect();

        assert_eq!(codes_for(&text, "count-pairs", &pack()), ["KNG-01"]);
    }

    #[test]
    fn duplicate_ghost_id_is_kng_01() {
        let text = ghost("same-index") + &ghost("same-index");

        assert_eq!(codes_for(&text, "count-pairs", &pack()), ["KNG-01"]);
    }

    #[test]
    fn ghost_input_outside_the_schema_is_kng_02() {
        let text = ghost("same-index").replace("input = [[0], 0]", "input = [[0], 500]");

        assert_eq!(codes_for(&text, "count-pairs", &pack()), ["KNG-02"]);
    }

    #[test]
    fn missing_program_file_is_kng_02() {
        let text = ghost("same-index").replace("incorrect/same-index.py", "incorrect/other.py");

        assert_eq!(codes_for(&text, "count-pairs", &pack()), ["KNG-02"]);
    }

    #[test]
    fn program_that_does_not_define_the_function_is_kng_02() {
        let pack = Pack::with_text(&[(
            "incorrect/same-index.py",
            "def count(values):\n    return 1\n",
        )]);

        assert_eq!(
            codes_for(&ghost("same-index"), "count-pairs", &pack),
            ["KNG-02"]
        );
    }

    #[test]
    fn wrong_ghost_without_got_is_kng_03() {
        let text = ghost("same-index").replace("got = 1\n", "");

        assert_eq!(codes_for(&text, "count-pairs", &pack()), ["KNG-03"]);
    }

    #[test]
    fn crash_ghost_without_exception_class_is_kng_03() {
        let text = ghost("same-index")
            .replace("kind = \"wrong\"\n", "kind = \"crash\"\n")
            .replace("got = 1\n", "");

        assert_eq!(codes_for(&text, "count-pairs", &pack()), ["KNG-03"]);
    }

    #[test]
    fn crash_ghost_with_got_is_kng_03() {
        let text = ghost("same-index").replace(
            "kind = \"wrong\"\n",
            "kind = \"crash\"\nexception_class = \"ValueError\"\n",
        );

        assert_eq!(codes_for(&text, "count-pairs", &pack()), ["KNG-03"]);
    }

    #[test]
    fn explanation_with_forbidden_word_is_kng_04() {
        let text = ghost("same-index").replace(
            "A value cannot pair with itself.",
            "This is the smallest failing input.",
        );

        assert_eq!(codes_for(&text, "count-pairs", &pack()), ["KNG-04"]);
    }

    #[test]
    fn explanation_over_400_bytes_is_kng_04() {
        let text =
            ghost("same-index").replace("A value cannot pair with itself.", &"a".repeat(401));

        assert_eq!(codes_for(&text, "count-pairs", &pack()), ["KNG-04"]);
    }

    #[test]
    fn concept_outside_the_pack_is_kng_05() {
        let text = ghost("same-index").replace("concepts = [\"pairs\"]", "concepts = [\"graphs\"]");

        assert_eq!(codes_for(&text, "count-pairs", &pack()), ["KNG-05"]);
    }

    #[test]
    fn dogfood_in_a_catalogue_pack_is_kng_06() {
        let text =
            ghost("same-index").replace("kind = \"wrong\"\n", "kind = \"wrong\"\ndogfood = true\n");

        assert_eq!(codes_for(&text, "count-pairs", &pack()), ["KNG-06"]);
    }

    #[test]
    fn dogfood_in_an_underscore_pack_is_allowed() {
        let text =
            ghost("same-index").replace("kind = \"wrong\"\n", "kind = \"wrong\"\ndogfood = true\n");

        assert_eq!(codes_for(&text, "_demo", &pack()), Vec::<String>::new());
    }

    #[test]
    fn forbidden_phrases_are_the_engine_list() {
        // Drift guard: the engine keeps its list in a test constant (open question 6).
        let engine_source = include_str!("../../engine/tests/properties.rs");
        for phrase in FORBIDDEN_PHRASES {
            assert!(
                engine_source.contains(&format!("\"{phrase}\"")),
                "missing {phrase}"
            );
        }
    }
}
