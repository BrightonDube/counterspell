//! The hint leak check (PIP-14, HNT-03) and the fenced-block length rule (GRM-27).
//!
//! A hint may not quote a sample expected value, a hidden expected value or a ghost
//! `got` value. A value counts as quoted when it appears as a whole token, so `41`
//! inside `2041` is not a match. The check is deliberately strict: a short value
//! such as `3` may flag an ordinary rung, and a reviewer decides whether that is a
//! real leak.

use super::Diagnostic;

/// The longest fenced code block a rung may contain, in lines (GRM-27).
const MAX_FENCE_LINES: usize = 3;

/// The values a rung must not quote. They stay in memory and are never returned.
#[derive(Debug, Clone, Copy)]
pub struct HiddenValues<'a> {
    /// Expected outputs of the public samples.
    pub sample: &'a [String],
    /// Expected outputs of the curated hidden cases.
    pub hidden: &'a [String],
    /// The `got` values of the known ghosts.
    pub ghost_got: &'a [String],
}

/// Checks the rungs of a hint ladder, in order. Rung numbers start at 1.
#[must_use]
pub fn check_rungs(rungs: &[String], values: &HiddenValues<'_>) -> Vec<Diagnostic> {
    let mut diagnostics = Vec::new();
    for (index, rung) in rungs.iter().enumerate() {
        let number = index + 1;
        let sources = [
            ("sample", values.sample),
            ("hidden", values.hidden),
            ("ghost", values.ghost_got),
        ];
        for (source, list) in sources {
            for (value_index, value) in list.iter().enumerate() {
                if contains_token(rung, value) {
                    diagnostics.push(Diagnostic {
                        code: "HNT-03",
                        pointer: format!(
                            "hint rung {number} matches {source} value {}",
                            value_index + 1
                        ),
                    });
                }
            }
        }
        if longest_fence_lines(rung) > MAX_FENCE_LINES {
            diagnostics.push(Diagnostic {
                code: "GRM-27",
                pointer: format!("hint rung {number}"),
            });
        }
    }
    diagnostics
}

/// Returns `true` when `value` appears in `text` with no letter or digit directly before or after it.
fn contains_token(text: &str, value: &str) -> bool {
    if value.is_empty() {
        return false;
    }
    text.match_indices(value).any(|(start, matched)| {
        let before_is_word = text[..start]
            .chars()
            .next_back()
            .is_some_and(char::is_alphanumeric);
        let after_is_word = text[start + matched.len()..]
            .chars()
            .next()
            .is_some_and(char::is_alphanumeric);
        !before_is_word && !after_is_word
    })
}

/// The number of lines in the longest fenced block of `rung`. The fence lines themselves are not counted.
fn longest_fence_lines(rung: &str) -> usize {
    let mut in_fence = false;
    let mut current = 0;
    let mut longest = 0;
    for line in rung.lines() {
        if line.trim_start().starts_with("```") {
            in_fence = !in_fence;
            current = 0;
        } else if in_fence {
            current += 1;
            longest = longest.max(current);
        }
    }
    longest
}
