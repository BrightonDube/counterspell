//! Statement checks (STM-01, STM-02) for `problem.md`.

use crate::codes;
use crate::report::{Diagnostic, Report};

/// The level-2 headings every statement has, in this order.
const HEADINGS: [&str; 5] = [
    "## Task",
    "## Input",
    "## Output",
    "## Examples",
    "## Constraints",
];

/// The longest statement, in characters.
const MAX_CHARACTERS: usize = 8000;

/// Checks the title line, the heading order, the length and the absence of raw HTML.
pub fn check(problem: &str, title: &str, report: &mut Report) {
    let headings: Vec<&str> = problem
        .lines()
        .filter(|line| line.starts_with("## "))
        .collect();
    let expected_title = format!("# {title}");
    let first_line = problem.lines().next().unwrap_or_default();
    if first_line != expected_title {
        report.push(Diagnostic::new(
            codes::STM_01,
            "problem.md",
            Some("/title"),
            "the first line must be the title from meta.toml",
        ));
    }
    if headings != HEADINGS {
        report.push(Diagnostic::new(
            codes::STM_01,
            "problem.md",
            None,
            "the headings must be Task, Input, Output, Examples and Constraints, in that order",
        ));
    }
    if problem.chars().count() > MAX_CHARACTERS {
        report.push(Diagnostic::new(
            codes::STM_02,
            "problem.md",
            None,
            "the statement is longer than 8000 characters",
        ));
    }
    if has_raw_html(problem) {
        report.push(Diagnostic::new(
            codes::STM_02,
            "problem.md",
            None,
            "the statement contains raw HTML tags",
        ));
    }
}

/// Whether `text` has a `<` followed by a letter or a slash, which looks like an HTML tag.
fn has_raw_html(text: &str) -> bool {
    text.match_indices('<').any(|(index, _)| {
        text[index + 1..]
            .chars()
            .next()
            .is_some_and(|next| next.is_ascii_alphabetic() || next == '/')
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn statement(title: &str) -> String {
        format!(
            "# {title}\n\n## Task\nCount the pairs.\n\n## Input\nA list.\n\n## Output\nA number.\n\n## Examples\nSee samples.\n\n## Constraints\nSmall.\n"
        )
    }

    fn codes_for(problem: &str, title: &str) -> Vec<String> {
        let mut report = Report::new("count-pairs");
        check(problem, title, &mut report);
        report
            .diagnostics()
            .iter()
            .map(|d| d.code.to_owned())
            .collect()
    }

    #[test]
    fn complete_statement_has_no_codes() {
        assert_eq!(
            codes_for(&statement("Count the Pairs"), "Count the Pairs"),
            Vec::<String>::new()
        );
    }

    #[test]
    fn title_line_that_differs_from_meta_is_stm_01() {
        assert_eq!(
            codes_for(&statement("Count Pairs"), "Count the Pairs"),
            ["STM-01"]
        );
    }

    #[test]
    fn missing_heading_is_stm_01() {
        let text = statement("Count the Pairs").replace("## Output\nA number.\n\n", "");

        assert_eq!(codes_for(&text, "Count the Pairs"), ["STM-01"]);
    }

    #[test]
    fn heading_out_of_order_is_stm_01() {
        let text = statement("Count the Pairs")
            .replace("## Input\nA list.\n\n", "")
            .replace(
                "## Output\nA number.\n\n",
                "## Output\nA number.\n\n## Input\nA list.\n\n",
            );

        assert_eq!(codes_for(&text, "Count the Pairs"), ["STM-01"]);
    }

    #[test]
    fn raw_script_tag_is_stm_02() {
        let text = statement("Count the Pairs").replace("Small.", "<script>alert(1)</script>");

        assert_eq!(codes_for(&text, "Count the Pairs"), ["STM-02"]);
    }

    #[test]
    fn statement_of_exactly_8000_characters_is_allowed() {
        let base = statement("Count the Pairs");
        let text = format!(
            "{base}{}",
            "x".repeat(MAX_CHARACTERS - base.chars().count())
        );

        assert_eq!(codes_for(&text, "Count the Pairs"), Vec::<String>::new());
    }

    #[test]
    fn statement_over_8000_characters_is_stm_02() {
        let base = statement("Count the Pairs");
        let text = format!(
            "{base}{}",
            "x".repeat(MAX_CHARACTERS + 1 - base.chars().count())
        );

        assert_eq!(codes_for(&text, "Count the Pairs"), ["STM-02"]);
    }
}
