//! Classification of raw Oracle observations.

use crate::compare::Comparator;
use serde::{Deserialize, Serialize};
use serde_json::Value;

const MAX_TEXT_BYTES: usize = 200;
const MAX_CLASS_BYTES: usize = 64;

/// The raw result returned by one program evaluation.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum ProgramResult {
    /// The program returned a JSON value.
    Returned(Value),
    /// The program raised an exception or was killed for a resource limit.
    Crashed {
        /// The untrusted exception class extracted by the judge.
        exception_class: String,
        /// The untrusted diagnostic message.
        message: String,
    },
    /// The judge killed the program after its time limit.
    TimedOut,
    /// The program did not produce a valid driver answer.
    Garbled {
        /// A bounded snippet of the invalid output.
        snippet: String,
    },
}

impl ProgramResult {
    /// Sanitises untrusted strings before a result is stored in a ghost.
    #[must_use]
    pub fn sanitised(self) -> Self {
        match self {
            Self::Returned(value) => Self::Returned(value),
            Self::Crashed {
                exception_class,
                message,
            } => Self::Crashed {
                exception_class: sanitise_class(&exception_class),
                message: sanitise_text(&message),
            },
            Self::TimedOut => Self::TimedOut,
            Self::Garbled { snippet } => Self::Garbled {
                snippet: sanitise_text(&snippet),
            },
        }
    }
}

/// The user and trusted reference results for one input.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    /// The user's raw result.
    pub user: ProgramResult,
    /// The reference program's raw result.
    pub reference: ProgramResult,
}

/// The coarse failure category used by the hunt and shrinker.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FailureKind {
    /// The returned value differed from the reference value.
    #[serde(rename = "wrong")]
    WrongAnswer,
    /// The user program crashed.
    #[serde(rename = "crash")]
    RuntimeError,
    /// The user program exceeded its time limit.
    #[serde(rename = "timeout")]
    Timeout,
}

/// The failure behavior a shrink candidate must reproduce.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct FailurePredicate {
    /// The failure category.
    pub kind: FailureKind,
    /// The sanitised exception class for a crash, if applicable.
    pub exception_class: Option<String>,
}

/// The trusted classification of one Oracle observation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Classification {
    /// The user returned the expected value.
    Pass,
    /// The user produced a reproducible failure predicate.
    Fail(FailurePredicate),
    /// The reference did not return a usable expected value.
    ReferenceFailed,
}

/// Classifies an observation without allowing user output to declare a verdict.
#[must_use]
pub fn classify(observation: &Observation, comparator: &Comparator) -> Classification {
    let ProgramResult::Returned(expected) = &observation.reference else {
        return Classification::ReferenceFailed;
    };

    match &observation.user {
        ProgramResult::Returned(got) if comparator.equal(expected, got) => Classification::Pass,
        ProgramResult::Returned(_) | ProgramResult::Garbled { .. } => {
            Classification::Fail(FailurePredicate {
                kind: FailureKind::WrongAnswer,
                exception_class: None,
            })
        }
        ProgramResult::Crashed {
            exception_class, ..
        } => Classification::Fail(FailurePredicate {
            kind: FailureKind::RuntimeError,
            exception_class: Some(sanitise_class(exception_class)),
        }),
        ProgramResult::TimedOut => Classification::Fail(FailurePredicate {
            kind: FailureKind::Timeout,
            exception_class: None,
        }),
    }
}

/// Replaces an invalid exception class with the safe reserved class.
#[must_use]
pub(crate) fn sanitise_class(class: &str) -> String {
    let bytes = class.as_bytes();
    if bytes.is_empty() || bytes.len() > MAX_CLASS_BYTES {
        return "UnknownError".to_owned();
    }
    let first_valid = bytes[0].is_ascii_alphabetic() || bytes[0] == b'_';
    let rest_valid = bytes[1..]
        .iter()
        .all(|byte| byte.is_ascii_alphanumeric() || *byte == b'_' || *byte == b'.');
    if first_valid && rest_valid {
        class.to_owned()
    } else {
        "UnknownError".to_owned()
    }
}

/// Replaces control characters and truncates text at a UTF-8 boundary.
#[must_use]
pub(crate) fn sanitise_text(text: &str) -> String {
    let mut result = String::new();
    for character in text.chars() {
        let character = if character == '\n' || character == '\t' || !character.is_control() {
            character
        } else {
            ' '
        };
        if result.len() + character.len_utf8() > MAX_TEXT_BYTES {
            break;
        }
        result.push(character);
    }
    result
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use serde_json::json;

    use super::{
        classify, sanitise_class, sanitise_text, Classification, FailureKind, FailurePredicate,
        Observation, ProgramResult,
    };
    use crate::compare::Comparator;

    fn returned(value: serde_json::Value) -> ProgramResult {
        ProgramResult::Returned(value)
    }

    #[test]
    fn classification_table_covers_pass_wrong_crash_timeout_and_reference_failure() {
        let pass = Observation {
            user: returned(json!(2)),
            reference: returned(json!(2)),
        };
        assert_eq!(classify(&pass, &Comparator::Exact), Classification::Pass);

        let wrong = Observation {
            user: returned(json!(3)),
            reference: returned(json!(2)),
        };
        assert_eq!(
            classify(&wrong, &Comparator::Exact),
            Classification::Fail(FailurePredicate {
                kind: FailureKind::WrongAnswer,
                exception_class: None,
            })
        );

        let crash = Observation {
            user: ProgramResult::Crashed {
                exception_class: "ValueError".to_owned(),
                message: "bad".to_owned(),
            },
            reference: returned(json!(2)),
        };
        assert_eq!(
            classify(&crash, &Comparator::Exact),
            Classification::Fail(FailurePredicate {
                kind: FailureKind::RuntimeError,
                exception_class: Some("ValueError".to_owned()),
            })
        );

        let timeout = Observation {
            user: ProgramResult::TimedOut,
            reference: returned(json!(2)),
        };
        assert_eq!(
            classify(&timeout, &Comparator::Exact),
            Classification::Fail(FailurePredicate {
                kind: FailureKind::Timeout,
                exception_class: None,
            })
        );

        let reference_crash = Observation {
            user: returned(json!(2)),
            reference: ProgramResult::Crashed {
                exception_class: "ValueError".to_owned(),
                message: "bad".to_owned(),
            },
        };
        assert_eq!(
            classify(&reference_crash, &Comparator::Exact),
            Classification::ReferenceFailed
        );

        let reference_timeout = Observation {
            user: returned(json!(2)),
            reference: ProgramResult::TimedOut,
        };
        assert_eq!(
            classify(&reference_timeout, &Comparator::Exact),
            Classification::ReferenceFailed
        );

        let reference_garbled = Observation {
            user: returned(json!(2)),
            reference: ProgramResult::Garbled {
                snippet: "not json".to_owned(),
            },
        };
        assert_eq!(
            classify(&reference_garbled, &Comparator::Exact),
            Classification::ReferenceFailed
        );
    }

    #[test]
    fn garbled_user_output_is_a_wrong_answer() {
        let observation = Observation {
            user: ProgramResult::Garbled {
                snippet: "not json".to_owned(),
            },
            reference: returned(json!(2)),
        };
        assert_eq!(
            classify(&observation, &Comparator::Exact),
            Classification::Fail(FailurePredicate {
                kind: FailureKind::WrongAnswer,
                exception_class: None,
            })
        );
    }

    #[test]
    fn hostile_classes_become_unknown_and_text_is_bounded_on_char_boundaries() {
        assert_eq!(sanitise_class("ValueError"), "ValueError");
        assert_eq!(sanitise_class("bad class"), "UnknownError");
        assert_eq!(sanitise_class("é"), "UnknownError");
        assert_eq!(sanitise_class(&"A".repeat(10_000)), "UnknownError");

        let text = format!("{}é", "a".repeat(199));
        let sanitised = sanitise_text(&text);
        assert_eq!(sanitised.len(), 199);
        assert!(sanitised.is_char_boundary(sanitised.len()));
        assert_eq!(sanitise_text("a\0b\u{7f}c\n\td"), "a b c\n\td");
    }

    #[test]
    fn sanitised_program_results_bound_all_untrusted_text() {
        let result = ProgramResult::Crashed {
            exception_class: "bad class".to_owned(),
            message: "x".repeat(500),
        }
        .sanitised();
        assert_eq!(
            result,
            ProgramResult::Crashed {
                exception_class: "UnknownError".to_owned(),
                message: "x".repeat(200),
            }
        );

        let garbled = ProgramResult::Garbled {
            snippet: format!("\u{0}{}", "x".repeat(500)),
        }
        .sanitised();
        assert_eq!(
            garbled,
            ProgramResult::Garbled {
                snippet: format!(" {}", "x".repeat(199)),
            }
        );
        let ProgramResult::Garbled { snippet } = garbled else {
            panic!("sanitised garbled result changed variant");
        };
        assert_eq!(snippet.len(), 200);
        assert!(!snippet.chars().any(char::is_control));

        let unicode = ProgramResult::Garbled {
            snippet: format!("{}é", "x".repeat(199)),
        }
        .sanitised();
        let ProgramResult::Garbled { snippet } = unicode else {
            panic!("sanitised Unicode result changed variant");
        };
        assert_eq!(snippet, "x".repeat(199));
        assert!(snippet.is_char_boundary(snippet.len()));
    }

    proptest! {
        #[test]
        fn p13_classify_never_panics_on_arbitrary_text(class in "[ -~]{0,300}", message in prop::collection::vec(any::<char>(), 0..300)) {
            let observation = Observation {
                user: ProgramResult::Crashed { exception_class: class, message: message.into_iter().collect() },
                reference: returned(json!(0)),
            };
            let classification = classify(&observation, &Comparator::Exact);
            if let Classification::Fail(predicate) = classification {
                prop_assert!(predicate.exception_class.as_ref().is_none_or(|value| value.len() <= 64));
            }
        }
    }
}
