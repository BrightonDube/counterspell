//! Hint ladder checks (HNT-01, HNT-02) for `hints.toml`.

use crate::codes;
use crate::report::{Diagnostic, Report};
use serde::Deserialize;

/// The shape of `hints.toml`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HintFile {
    #[serde(default)]
    hint: Vec<Hint>,
}

/// One hint entry.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Hint {
    rung: i64,
    text: String,
}

/// The longest hint, in characters.
const MAX_HINT_CHARACTERS: usize = 600;

/// The most lines a code fence may hold inside a hint.
const MAX_FENCE_LINES: usize = 3;

/// Checks that there are four hints in rung order and that each is a short, safe text.
pub fn check(text: &str, report: &mut Report) {
    let file = match toml::from_str::<HintFile>(text) {
        Ok(file) => file,
        Err(error) => {
            report.push(Diagnostic::new(
                codes::HNT_01,
                "hints.toml",
                None,
                format!("hints.toml does not parse: {}", error.message()),
            ));
            return;
        }
    };
    let rungs: Vec<i64> = file.hint.iter().map(|hint| hint.rung).collect();
    if rungs != [1, 2, 3, 4] {
        report.push(Diagnostic::new(
            codes::HNT_01,
            "hints.toml",
            None,
            "there must be exactly four hints, with rungs 1 to 4 in order",
        ));
    }
    for (index, hint) in file.hint.iter().enumerate() {
        let length = hint.text.chars().count();
        if !(1..=MAX_HINT_CHARACTERS).contains(&length) {
            report.push(Diagnostic::new(
                codes::HNT_02,
                "hints.toml",
                Some(&format!("/hint/{index}/text")),
                "each hint must be 1 to 600 characters",
            ));
        }
        if longest_fence(&hint.text) > MAX_FENCE_LINES {
            report.push(Diagnostic::new(
                codes::HNT_02,
                "hints.toml",
                Some(&format!("/hint/{index}/text")),
                "a code fence in a hint may be at most 3 lines",
            ));
        }
    }
}

/// The number of lines in the longest fenced block in `text`.
fn longest_fence(text: &str) -> usize {
    let mut longest = 0;
    let mut current: Option<usize> = None;
    for line in text.lines() {
        if line.trim_start().starts_with("```") {
            match current.take() {
                Some(lines) => longest = longest.max(lines),
                None => current = Some(0),
            }
        } else if let Some(lines) = current.as_mut() {
            *lines += 1;
        }
    }
    longest
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hint(rung: u8, body: &str) -> String {
        format!("[[hint]]\nrung = {rung}\ntext = \"{body}\"\n\n")
    }

    fn ladder() -> String {
        hint(1, "Ask what a pair needs.")
            + &hint(2, "Loop over each first item.")
            + &hint(3, "Start the second loop after the first.")
            + &hint(4, "Change where the inner loop starts.")
    }

    fn codes_for(text: &str) -> Vec<String> {
        let mut report = Report::new("count-pairs");
        check(text, &mut report);
        report
            .diagnostics()
            .iter()
            .map(|d| d.code.to_owned())
            .collect()
    }

    #[test]
    fn four_hints_in_order_have_no_codes() {
        assert_eq!(codes_for(&ladder()), Vec::<String>::new());
    }

    #[test]
    fn three_hints_is_hnt_01() {
        let text = ladder().replace(&hint(4, "Change where the inner loop starts."), "");

        assert_eq!(codes_for(&text), ["HNT-01"]);
    }

    #[test]
    fn rung_out_of_order_is_hnt_01() {
        let text = ladder()
            .replace("rung = 2", "rung = 3")
            .replace("rung = 3\ntext = \"Start", "rung = 2\ntext = \"Start");

        assert_eq!(codes_for(&text), ["HNT-01"]);
    }

    #[test]
    fn empty_hint_text_is_hnt_02() {
        let text = ladder().replace("Ask what a pair needs.", "");

        assert_eq!(codes_for(&text), ["HNT-02"]);
    }

    #[test]
    fn hint_of_601_characters_is_hnt_02() {
        let text = ladder().replace("Ask what a pair needs.", &"a".repeat(601));

        assert_eq!(codes_for(&text), ["HNT-02"]);
    }

    #[test]
    fn code_fence_of_four_lines_is_hnt_02() {
        let text = ladder().replace(
            "Ask what a pair needs.",
            "Try this:\\n```\\nline one\\nline two\\nline three\\nline four\\n```",
        );

        assert_eq!(codes_for(&text), ["HNT-02"]);
    }

    #[test]
    fn code_fence_of_three_lines_is_allowed() {
        let text = ladder().replace(
            "Ask what a pair needs.",
            "Try this:\\n```\\nline one\\nline two\\nline three\\n```",
        );

        assert_eq!(codes_for(&text), Vec::<String>::new());
    }
}
