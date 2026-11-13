//! Bug Autopsy data: the expected-versus-actual diff, the one-line explanation,
//! and matching against authored known ghosts.
//!
//! Nothing here reads hidden test data. Known ghosts are authored for display.

use crate::classify::{FailureKind, FailurePredicate, ProgramResult};
use crate::size::size_of;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::time::Duration;

/// The most differences an autopsy keeps.
pub const MAX_DIFFERENCES: usize = 20;
/// The longest explanation line, in bytes.
pub const MAX_EXPLANATION_BYTES: usize = 240;
/// The longest rendered input or value shown in a line, in characters.
const MAX_RENDERED_CHARS: usize = 120;

/// One place where the actual value differs from the expected value.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Difference {
    /// The JSON Pointer to the difference. The root is an empty string.
    pub path: String,
    /// The expected value at this path, if there is one.
    pub expected: Option<Value>,
    /// The actual value at this path, if there is one.
    pub got: Option<Value>,
}

/// An authored explanation for a known ghost that matched this failure.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct AuthoredMistake {
    /// The id of the known ghost that matched.
    pub known_ghost_id: String,
    /// The author's explanation of the mistake.
    pub text: String,
}

/// An authored known ghost from a problem pack.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct KnownGhost {
    /// A stable id, unique within the pack.
    pub id: String,
    /// The input the author chose. It should already be minimal.
    pub input: Value,
    /// The failure kind this known ghost shows.
    pub kind: FailureKind,
    /// The exception class, for a crash known ghost.
    pub exception_class: Option<String>,
    /// The wrong value the user returned, if the author recorded one.
    pub got: Option<Value>,
    /// The author's one or two sentence explanation.
    pub explanation: String,
    /// Concept tags for the mistake.
    pub concept_tags: Vec<String>,
}

impl KnownGhost {
    /// Reports whether this known ghost describes the same failure.
    ///
    /// The input and kind must match. The exception class and the wrong value
    /// are compared only when the known ghost records them.
    #[must_use]
    pub fn matches(
        &self,
        input: &Value,
        predicate: &FailurePredicate,
        got: &ProgramResult,
    ) -> bool {
        let class_matches = match &self.exception_class {
            None => true,
            Some(class) => predicate.exception_class.as_deref() == Some(class.as_str()),
        };
        let got_matches = match &self.got {
            None => true,
            Some(expected_got) => {
                matches!(got, ProgramResult::Returned(value) if value == expected_got)
            }
        };
        self.input == *input && self.kind == predicate.kind && class_matches && got_matches
    }
}

/// The Bug Autopsy shown beside a ghost.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Autopsy {
    /// One plain-English line, at most 240 bytes.
    pub explanation: String,
    /// The expected-versus-actual differences, at most 20.
    pub differences: Vec<Difference>,
    /// Whether more differences existed than were kept.
    pub differences_truncated: bool,
    /// Concept tags from the problem.
    pub concept_tags: Vec<String>,
    /// The authored explanation, when a known ghost matched.
    pub authored_mistake: Option<AuthoredMistake>,
}

/// Lists where `got` differs from `expected`, walking arrays index by index.
///
/// Returns at most 20 differences and a flag that says whether more existed.
#[must_use]
pub fn diff_values(expected: &Value, got: &Value) -> (Vec<Difference>, bool) {
    let mut walker = DiffWalker::default();
    walker.walk("", Some(expected), Some(got));
    (walker.items, walker.truncated)
}

/// Builds the one-line explanation for a failure, capped at 240 bytes.
///
/// `expected` is the reference value for a wrong answer and is ignored for
/// other kinds. `limit` is the per-call time limit shown for timeouts.
#[must_use]
pub fn explain(
    predicate: &FailurePredicate,
    input: &Value,
    expected: Option<&Value>,
    got: &ProgramResult,
    limit: Duration,
) -> String {
    let input_text = render(input);
    let line = match predicate.kind {
        FailureKind::WrongAnswer => {
            let got_text = match got {
                ProgramResult::Returned(value) => render(value),
                _ => "unreadable output".to_owned(),
            };
            let expected_text = expected.map_or_else(|| "no value".to_owned(), render);
            format!("Your solution returned {got_text}; expected {expected_text} for input {input_text}.")
        }
        FailureKind::RuntimeError => {
            let class = predicate
                .exception_class
                .as_deref()
                .unwrap_or("UnknownError");
            format!("Your solution crashed with {class} for input {input_text}.")
        }
        FailureKind::Timeout => format!(
            "Your solution did not finish within {} ms for an input of {} nodes.",
            limit.as_millis(),
            size_of(input).nodes
        ),
    };
    cut_bytes(&line, MAX_EXPLANATION_BYTES)
}

/// Assembles the full Bug Autopsy for one failure.
///
/// The diff is shown only for wrong answers with a returned value. The
/// authored mistake comes from the first known ghost that matches.
#[must_use]
pub fn build_autopsy(
    input: &Value,
    predicate: &FailurePredicate,
    expected: Option<&Value>,
    got: &ProgramResult,
    limit: Duration,
    known: &[KnownGhost],
    concept_tags: &[String],
) -> Autopsy {
    let explanation = explain(predicate, input, expected, got, limit);
    let (differences, differences_truncated) = match (predicate.kind, expected, got) {
        (FailureKind::WrongAnswer, Some(expected), ProgramResult::Returned(value)) => {
            diff_values(expected, value)
        }
        _ => (Vec::new(), false),
    };
    let authored_mistake = known
        .iter()
        .find(|ghost| ghost.matches(input, predicate, got))
        .map(|ghost| AuthoredMistake {
            known_ghost_id: ghost.id.clone(),
            text: ghost.explanation.clone(),
        });
    Autopsy {
        explanation,
        differences,
        differences_truncated,
        concept_tags: concept_tags.to_vec(),
        authored_mistake,
    }
}

/// Collects differences while walking two values together.
#[derive(Default)]
struct DiffWalker {
    items: Vec<Difference>,
    truncated: bool,
}

impl DiffWalker {
    fn walk(&mut self, path: &str, expected: Option<&Value>, got: Option<&Value>) {
        if self.truncated || expected == got {
            return;
        }
        if let (Some(Value::Array(expected_items)), Some(Value::Array(got_items))) = (expected, got)
        {
            for index in 0..expected_items.len().max(got_items.len()) {
                let child_path = format!("{path}/{index}");
                self.walk(&child_path, expected_items.get(index), got_items.get(index));
            }
            return;
        }
        if self.items.len() == MAX_DIFFERENCES {
            self.truncated = true;
            return;
        }
        self.items.push(Difference {
            path: path.to_owned(),
            expected: expected.cloned(),
            got: got.cloned(),
        });
    }
}

/// Renders a value compactly, cut to a readable length.
fn render(value: &Value) -> String {
    cut_chars(&value.to_string(), MAX_RENDERED_CHARS)
}

/// Keeps at most `max_chars` characters, ending with an ellipsis when cut.
fn cut_chars(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        return text.to_owned();
    }
    let kept: String = text.chars().take(max_chars.saturating_sub(1)).collect();
    format!("{kept}…")
}

/// Keeps whole characters while the total stays within `max_bytes`.
fn cut_bytes(text: &str, max_bytes: usize) -> String {
    let mut kept = String::new();
    for character in text.chars() {
        if kept.len() + character.len_utf8() > max_bytes {
            break;
        }
        kept.push(character);
    }
    kept
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::tests::strategies::arb_json;
    use proptest::prelude::*;
    use serde_json::json;

    fn wrong() -> FailurePredicate {
        FailurePredicate {
            kind: FailureKind::WrongAnswer,
            exception_class: None,
        }
    }

    fn crash(class: &str) -> FailurePredicate {
        FailurePredicate {
            kind: FailureKind::RuntimeError,
            exception_class: Some(class.to_owned()),
        }
    }

    fn known(
        input: Value,
        kind: FailureKind,
        class: Option<&str>,
        got: Option<Value>,
    ) -> KnownGhost {
        KnownGhost {
            id: "same-index".to_owned(),
            input,
            kind,
            exception_class: class.map(str::to_owned),
            got,
            explanation: "An item pairs with itself.".to_owned(),
            concept_tags: vec!["pairs".to_owned()],
        }
    }

    #[test]
    fn autopsy_scalar_difference_is_at_the_root() {
        assert_eq!(
            diff_values(&json!(0), &json!(1)),
            (
                vec![Difference {
                    path: String::new(),
                    expected: Some(json!(0)),
                    got: Some(json!(1)),
                }],
                false,
            ),
        );
    }

    #[test]
    fn autopsy_changed_array_index_is_named_by_pointer() {
        let (differences, _) = diff_values(&json!([1, 2, 3]), &json!([1, 9, 3]));
        assert_eq!(
            differences,
            vec![Difference {
                path: "/1".to_owned(),
                expected: Some(json!(2)),
                got: Some(json!(9)),
            }]
        );
    }

    #[test]
    fn autopsy_extra_and_missing_items_have_one_empty_side() {
        let (extra, _) = diff_values(&json!([1, 2]), &json!([1, 2, 3]));
        assert_eq!(
            extra,
            vec![Difference {
                path: "/2".to_owned(),
                expected: None,
                got: Some(json!(3)),
            }]
        );
        let (missing, _) = diff_values(&json!([1, 2, 3]), &json!([1, 2]));
        assert_eq!(
            missing,
            vec![Difference {
                path: "/2".to_owned(),
                expected: Some(json!(3)),
                got: None,
            }]
        );
    }

    #[test]
    fn autopsy_type_change_is_one_difference() {
        let (differences, _) = diff_values(&json!([1]), &json!(1));
        assert_eq!(
            differences,
            vec![Difference {
                path: String::new(),
                expected: Some(json!([1])),
                got: Some(json!(1)),
            }]
        );
    }

    #[test]
    fn autopsy_differences_stop_at_twenty() {
        let expected: Vec<i64> = (0..30).collect();
        let got: Vec<i64> = (100..130).collect();
        let (differences, truncated) = diff_values(&json!(expected), &json!(got));
        assert_eq!(differences.len(), 20);
        assert!(truncated);
    }

    #[test]
    fn autopsy_known_ghost_needs_the_same_input_and_kind() {
        let ghost = known(json!([[0], 0]), FailureKind::WrongAnswer, None, None);
        let got = ProgramResult::Returned(json!(1));
        assert!(ghost.matches(&json!([[0], 0]), &wrong(), &got));
        assert!(!ghost.matches(&json!([[1], 0]), &wrong(), &got));
        assert!(!ghost.matches(&json!([[0], 0]), &crash("IndexError"), &got));
    }

    #[test]
    fn autopsy_known_ghost_compares_class_when_recorded() {
        let ghost = known(
            json!([[]]),
            FailureKind::RuntimeError,
            Some("IndexError"),
            None,
        );
        let got = ProgramResult::Crashed {
            exception_class: "IndexError".to_owned(),
            message: String::new(),
        };
        assert!(ghost.matches(&json!([[]]), &crash("IndexError"), &got));
        assert!(!ghost.matches(&json!([[]]), &crash("ValueError"), &got));
    }

    #[test]
    fn autopsy_known_ghost_compares_the_wrong_value_when_recorded() {
        let ghost = known(
            json!([[0], 0]),
            FailureKind::WrongAnswer,
            None,
            Some(json!(1)),
        );
        assert!(ghost.matches(
            &json!([[0], 0]),
            &wrong(),
            &ProgramResult::Returned(json!(1)),
        ));
        assert!(!ghost.matches(
            &json!([[0], 0]),
            &wrong(),
            &ProgramResult::Returned(json!(2)),
        ));
    }

    #[test]
    fn autopsy_wrong_answer_line_uses_the_template() {
        let line = explain(
            &wrong(),
            &json!([[0], 0]),
            Some(&json!(0)),
            &ProgramResult::Returned(json!(1)),
            Duration::from_millis(2000),
        );
        assert_eq!(
            line,
            "Your solution returned 1; expected 0 for input [[0],0]."
        );
    }

    #[test]
    fn autopsy_crash_line_names_the_class() {
        let line = explain(
            &crash("IndexError"),
            &json!([[]]),
            None,
            &ProgramResult::Crashed {
                exception_class: "IndexError".to_owned(),
                message: String::new(),
            },
            Duration::from_millis(2000),
        );
        assert_eq!(
            line,
            "Your solution crashed with IndexError for input [[]]."
        );
    }

    #[test]
    fn autopsy_timeout_line_reports_the_limit_and_size() {
        let timeout = FailurePredicate {
            kind: FailureKind::Timeout,
            exception_class: None,
        };
        let line = explain(
            &timeout,
            &json!([1, 2]),
            None,
            &ProgramResult::TimedOut,
            Duration::from_millis(2000),
        );
        assert_eq!(
            line,
            "Your solution did not finish within 2000 ms for an input of 3 nodes."
        );
    }

    #[test]
    fn autopsy_long_input_is_cut_with_an_ellipsis() {
        let input = json!(vec![0; 100]);
        let line = explain(
            &crash("IndexError"),
            &input,
            None,
            &ProgramResult::Crashed {
                exception_class: "IndexError".to_owned(),
                message: String::new(),
            },
            Duration::from_millis(2000),
        );
        assert!(line.contains('…'), "no ellipsis in {line}");
        assert!(line.len() <= MAX_EXPLANATION_BYTES);
    }

    #[test]
    fn autopsy_explanation_is_capped_on_a_character_boundary() {
        let got = json!("é".repeat(500));
        let line = explain(
            &wrong(),
            &json!([1]),
            Some(&json!(0)),
            &ProgramResult::Returned(got),
            Duration::from_millis(2000),
        );
        assert!(line.len() <= MAX_EXPLANATION_BYTES);
        assert!(line.starts_with("Your solution returned "));
    }

    #[test]
    fn autopsy_built_record_names_the_authored_mistake() {
        let ghost = known(json!([[0], 0]), FailureKind::WrongAnswer, None, None);
        let autopsy = build_autopsy(
            &json!([[0], 0]),
            &wrong(),
            Some(&json!(0)),
            &ProgramResult::Returned(json!(1)),
            Duration::from_millis(2000),
            &[ghost],
            &["pairs".to_owned()],
        );
        assert_eq!(
            autopsy.authored_mistake,
            Some(AuthoredMistake {
                known_ghost_id: "same-index".to_owned(),
                text: "An item pairs with itself.".to_owned(),
            })
        );
        assert_eq!(autopsy.differences.len(), 1);
        assert_eq!(autopsy.concept_tags, vec!["pairs".to_owned()]);
        assert!(autopsy.explanation.starts_with("Your solution returned 1"));
    }

    proptest! {
        #[test]
        fn autopsy_p13_diff_never_panics_and_is_capped(
            expected in arb_json(),
            got in arb_json(),
        ) {
            let (differences, truncated) = diff_values(&expected, &got);
            prop_assert!(differences.len() <= MAX_DIFFERENCES);
            if expected == got {
                prop_assert!(differences.is_empty());
            }
            if !truncated {
                prop_assert!(differences.len() < MAX_DIFFERENCES || differences.is_empty());
            }
        }
    }
}
