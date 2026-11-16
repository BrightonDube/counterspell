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

/// The four authored hints for one problem, held in rung order 1 to 4.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthoredLadder {
    bodies: [String; 4],
}

/// Why a `hints.toml` file does not describe a ladder.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LadderError {
    /// The file is not valid TOML or has an unknown key. The text is the parser's message.
    Parse(String),
    /// The file does not hold exactly four hints.
    Count {
        /// How many hints the file holds.
        found: usize,
    },
    /// The four hints do not carry rungs 1, 2, 3 and 4 in that order.
    RungOrder {
        /// The rung numbers in file order.
        found: Vec<i64>,
    },
}

impl AuthoredLadder {
    /// Reads the four hints from the text of `hints.toml`.
    ///
    /// The parser checks only the shape: four hints, in rung order. Text length and code fences
    /// are checked by [`check`], so one parser handles both the pack validator and any other reader.
    ///
    /// # Errors
    ///
    /// Returns a [`LadderError`] for invalid TOML, a count other than four, or rungs out of order.
    pub fn parse(text: &str) -> Result<Self, LadderError> {
        let file = toml::from_str::<HintFile>(text)
            .map_err(|error| LadderError::Parse(error.message().to_owned()))?;
        let rungs: Vec<i64> = file.hint.iter().map(|hint| hint.rung).collect();
        if rungs.len() != 4 {
            return Err(LadderError::Count { found: rungs.len() });
        }
        if rungs != [1, 2, 3, 4] {
            return Err(LadderError::RungOrder { found: rungs });
        }
        let bodies: Vec<String> = file.hint.into_iter().map(|hint| hint.text).collect();
        let bodies = <[String; 4]>::try_from(bodies).map_err(|rejected| LadderError::Count {
            found: rejected.len(),
        })?;
        Ok(Self { bodies })
    }

    /// The authored text for `rung` (1 to 4). Returns `None` for any other number.
    #[must_use]
    pub fn body(&self, rung: u8) -> Option<&str> {
        let index = usize::from(rung).checked_sub(1)?;
        self.bodies.get(index).map(String::as_str)
    }
}

/// Checks that there are four hints in rung order and that each is a short, safe text.
pub fn check(text: &str, report: &mut Report) {
    let ladder = match AuthoredLadder::parse(text) {
        Ok(ladder) => ladder,
        Err(LadderError::Parse(message)) => {
            report.push(Diagnostic::new(
                codes::HNT_01,
                "hints.toml",
                None,
                format!("hints.toml does not parse: {message}"),
            ));
            return;
        }
        Err(LadderError::Count { .. } | LadderError::RungOrder { .. }) => {
            report.push(Diagnostic::new(
                codes::HNT_01,
                "hints.toml",
                None,
                "there must be exactly four hints, with rungs 1 to 4 in order",
            ));
            return;
        }
    };
    for rung in 1..=4u8 {
        let body = ladder.body(rung).unwrap_or_default();
        let pointer = format!("/hint/{}/text", rung - 1);
        if !(1..=MAX_HINT_CHARACTERS).contains(&body.chars().count()) {
            report.push(Diagnostic::new(
                codes::HNT_02,
                "hints.toml",
                Some(&pointer),
                "each hint must be 1 to 600 characters",
            ));
        }
        if longest_fence(body) > MAX_FENCE_LINES {
            report.push(Diagnostic::new(
                codes::HNT_02,
                "hints.toml",
                Some(&pointer),
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
    fn parse_returns_the_four_bodies_in_rung_order() {
        let ladder = AuthoredLadder::parse(&ladder()).expect("a good ladder parses");

        assert_eq!(ladder.body(1), Some("Ask what a pair needs."));
        assert_eq!(ladder.body(4), Some("Change where the inner loop starts."));
        assert_eq!(ladder.body(5), None);
    }

    #[test]
    fn parse_rejects_a_missing_rung() {
        let text = ladder().replace(&hint(4, "Change where the inner loop starts."), "");

        assert_eq!(
            AuthoredLadder::parse(&text),
            Err(LadderError::Count { found: 3 })
        );
    }

    #[test]
    fn parse_rejects_an_extra_rung() {
        let text = ladder() + &hint(5, "One more.");

        assert_eq!(
            AuthoredLadder::parse(&text),
            Err(LadderError::Count { found: 5 })
        );
    }

    #[test]
    fn parse_rejects_a_duplicate_rung() {
        let text = ladder().replace("rung = 4", "rung = 1");

        assert_eq!(
            AuthoredLadder::parse(&text),
            Err(LadderError::RungOrder {
                found: vec![1, 2, 3, 1]
            })
        );
    }

    #[test]
    fn parse_reports_text_that_is_not_toml() {
        let result = AuthoredLadder::parse("[[hint]\nrung = ");

        assert!(matches!(result, Err(LadderError::Parse(_))));
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
