//! The claim guard: checks AI speech before it is shown. Pure: no I/O and no provider call.
//!
//! The speech bubble may mention only the complexity classes that were measured or are plausible, and
//! it may not use the forbidden praise words. Any other class mention, or any forbidden phrase, rejects
//! the text, and the caller shows the fixed template instead. The check is strict on purpose: a false
//! positive costs one template, while a wrong complexity claim costs trust.

use counterspell_common::familiar::ClassId;

/// The outcome of checking one piece of speech.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Every class mention is allowed and no forbidden phrase appears.
    Ok,
    /// The text names a class outside the allowed set, or an `O(...)` form this guard does not know.
    UnmeasuredClass,
    /// The text contains a forbidden phrase. The phrase is the fixed word from the list, never the input.
    Forbidden(&'static str),
}

/// Phrases that praise a solution as the best possible. Matched whole-word, case-insensitively.
const FORBIDDEN: [&str; 5] = [
    "optimal",
    "optimum",
    "best possible",
    "fastest possible",
    "cannot be improved",
];

/// Written forms that name each class. Matched on lowercased text, with word boundaries at alphanumeric edges.
const PATTERNS: [(ClassId, &[&str]); 7] = [
    (ClassId::Constant, &["o(1)", "constant"]),
    (ClassId::Logarithmic, &["o(log n)", "logarithmic"]),
    (ClassId::Linear, &["o(n)", "linear"]),
    (
        ClassId::Linearithmic,
        &["o(n log n)", "n log n", "linearithmic"],
    ),
    (
        ClassId::Quadratic,
        &["o(n^2)", "o(n²)", "n^2", "n²", "quadratic", "n squared"],
    ),
    (ClassId::Cubic, &["o(n^3)", "o(n³)", "n^3", "n³", "cubic"]),
    (
        ClassId::Exponential,
        &["o(2^n)", "o(2ⁿ)", "2^n", "2ⁿ", "exponential"],
    ),
];

/// The inside of an `O(...)` form that matches a known class, after lowercasing and removing spaces.
const KNOWN_BIG_O: [&str; 10] = [
    "1", "logn", "n", "nlogn", "n^2", "n²", "n^3", "n³", "2^n", "2ⁿ",
];

/// Checks `text` against the allowed classes and the forbidden phrases.
///
/// The forbidden phrases are checked first. Class mentions are checked next, and any `O(...)` whose
/// inside is not a known class counts as an unmeasured class.
#[must_use]
pub fn check_claims(text: &str, allowed: &[ClassId]) -> Verdict {
    let lower = text.to_lowercase();
    for phrase in FORBIDDEN {
        if contains_word(&lower, phrase) {
            return Verdict::Forbidden(phrase);
        }
    }
    for (class, forms) in PATTERNS {
        let mentioned = forms.iter().any(|form| contains_word(&lower, form));
        if mentioned && !allowed.contains(&class) {
            return Verdict::UnmeasuredClass;
        }
    }
    if has_unknown_big_o(&lower) {
        return Verdict::UnmeasuredClass;
    }
    Verdict::Ok
}

/// True when `needle` appears in `haystack` with no letter or digit directly before or after it.
/// Only an alphanumeric edge of the needle is checked, so `o(n)` still matches inside `(o(n))`.
fn contains_word(haystack: &str, needle: &str) -> bool {
    let starts_alnum = needle.chars().next().is_some_and(char::is_alphanumeric);
    let ends_alnum = needle.chars().last().is_some_and(char::is_alphanumeric);
    haystack.match_indices(needle).any(|(index, _)| {
        let before_ok = !starts_alnum
            || haystack[..index]
                .chars()
                .next_back()
                .is_none_or(|c| !c.is_alphanumeric());
        let after_ok = !ends_alnum
            || haystack[index + needle.len()..]
                .chars()
                .next()
                .is_none_or(|c| !c.is_alphanumeric());
        before_ok && after_ok
    })
}

/// True when the text has an `O(...)` whose inside is not a known class. The `O` must start a word.
fn has_unknown_big_o(lower: &str) -> bool {
    lower.match_indices("o(").any(|(index, _)| {
        let starts_word = lower[..index]
            .chars()
            .next_back()
            .is_none_or(|c| !c.is_alphanumeric());
        if !starts_word {
            return false;
        }
        let rest = &lower[index + 2..];
        let inside: String = match rest.find(')') {
            Some(end) => rest[..end].chars().filter(|c| !c.is_whitespace()).collect(),
            None => return true,
        };
        !KNOWN_BIG_O.contains(&inside.as_str())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    const ONLY_LINEAR: &[ClassId] = &[ClassId::Linear];

    #[test]
    fn a_sentence_naming_only_an_allowed_class_passes() {
        assert_eq!(
            check_claims(
                "Looks linear, medium confidence. Reference linear.",
                ONLY_LINEAR
            ),
            Verdict::Ok
        );
        assert_eq!(check_claims("The loop is O(n).", ONLY_LINEAR), Verdict::Ok);
    }

    #[test]
    fn a_class_outside_the_allowed_set_is_rejected() {
        assert_eq!(
            check_claims("This is O(n^2) work.", ONLY_LINEAR),
            Verdict::UnmeasuredClass
        );
        assert_eq!(
            check_claims("The nested scan is quadratic.", ONLY_LINEAR),
            Verdict::UnmeasuredClass
        );
        assert_eq!(
            check_claims("It runs in O(log n) time.", ONLY_LINEAR),
            Verdict::UnmeasuredClass
        );
    }

    #[test]
    fn the_superscript_forms_are_recognised() {
        assert_eq!(
            check_claims("Costs n² steps.", ONLY_LINEAR),
            Verdict::UnmeasuredClass
        );
        assert_eq!(
            check_claims("Costs O(n³) steps.", ONLY_LINEAR),
            Verdict::UnmeasuredClass
        );
    }

    #[test]
    fn an_unknown_big_o_form_is_rejected() {
        assert_eq!(
            check_claims("About O(n!) steps.", ONLY_LINEAR),
            Verdict::UnmeasuredClass
        );
    }

    #[test]
    fn a_forbidden_phrase_is_rejected_whatever_the_class() {
        assert_eq!(
            check_claims("This is optimal.", &ClassId::ALL),
            Verdict::Forbidden("optimal")
        );
        assert_eq!(
            check_claims(
                "Nothing can be faster: the best possible answer.",
                &ClassId::ALL
            ),
            Verdict::Forbidden("best possible")
        );
        assert_eq!(
            check_claims("The code cannot be improved.", &ClassId::ALL),
            Verdict::Forbidden("cannot be improved")
        );
    }

    #[test]
    fn forbidden_phrases_match_regardless_of_case() {
        assert_eq!(
            check_claims("OPTIMAL.", &ClassId::ALL),
            Verdict::Forbidden("optimal")
        );
        assert_eq!(
            check_claims("Optimum speed.", &ClassId::ALL),
            Verdict::Forbidden("optimum")
        );
    }

    #[test]
    fn linear_search_is_rejected_when_only_a_different_class_is_allowed() {
        // Strict by design: the word "linear" counts as a class mention even in "linear search".
        assert_eq!(
            check_claims("A linear search over the list.", &[ClassId::Quadratic]),
            Verdict::UnmeasuredClass
        );
    }

    #[test]
    fn linearithmic_is_not_mistaken_for_linear() {
        assert_eq!(
            check_claims("A linearithmic sort.", ONLY_LINEAR),
            Verdict::UnmeasuredClass
        );
        assert_eq!(
            check_claims("A linearithmic sort.", &[ClassId::Linearithmic]),
            Verdict::Ok
        );
    }

    #[test]
    fn a_word_that_only_contains_a_forbidden_phrase_is_not_rejected() {
        // Whole-word matching: "suboptimal" is not the forbidden word "optimal".
        assert_eq!(
            check_claims("A suboptimal guess.", &ClassId::ALL),
            Verdict::Ok
        );
    }

    /// One representative written form for each class, chosen by hand from the design's list.
    const REPRESENTATIVE: [(ClassId, &str); 7] = [
        (ClassId::Constant, "O(1)"),
        (ClassId::Logarithmic, "O(log n)"),
        (ClassId::Linear, "O(n)"),
        (ClassId::Linearithmic, "O(n log n)"),
        (ClassId::Quadratic, "O(n^2)"),
        (ClassId::Cubic, "O(n^3)"),
        (ClassId::Exponential, "O(2^n)"),
    ];

    #[test]
    fn a_sentence_passes_exactly_when_its_class_is_the_allowed_one() {
        for (mentioned, form) in REPRESENTATIVE {
            let sentence = format!("The loop costs {form} time.");
            for (allowed, _) in REPRESENTATIVE {
                let expected = if allowed == mentioned {
                    Verdict::Ok
                } else {
                    Verdict::UnmeasuredClass
                };
                assert_eq!(
                    check_claims(&sentence, &[allowed]),
                    expected,
                    "{sentence} with {allowed:?}"
                );
            }
        }
    }

    #[test]
    fn text_with_no_class_or_forbidden_phrase_always_passes() {
        for sentence in ["The solution is correct.", "Try a different loop.", ""] {
            assert_eq!(check_claims(sentence, &[]), Verdict::Ok, "{sentence}");
        }
    }
}
