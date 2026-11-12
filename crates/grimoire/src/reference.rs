//! Reference presence and text check (REF-01). The text is read, never executed.

use crate::codes;
use crate::report::{Diagnostic, Report};

/// Checks that the reference source has a column-0 `def <function>(` line.
pub fn check(source: Option<&str>, function: &str, report: &mut Report) {
    if !source.is_some_and(|text| defines_function(text, function)) {
        report.push(Diagnostic::new(
            codes::REF_01,
            "reference/solution.py",
            None,
            format!("the reference must contain a column-0 line starting with def {function}("),
        ));
    }
}

/// Whether `source` has a line at column 0 that starts `def <function>(`.
pub(crate) fn defines_function(source: &str, function: &str) -> bool {
    source.lines().any(|line| {
        line.strip_prefix("def ")
            .and_then(|rest| rest.strip_prefix(function))
            .is_some_and(|rest| rest.starts_with('('))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn codes_for(source: Option<&str>, function: &str) -> Vec<String> {
        let mut report = Report::new("count-pairs");
        check(source, function, &mut report);
        report
            .diagnostics()
            .iter()
            .map(|d| d.code.to_owned())
            .collect()
    }

    #[test]
    fn reference_that_defines_the_function_has_no_codes() {
        let source = "def count_pairs(values, target):\n    return 0\n";

        assert_eq!(codes_for(Some(source), "count_pairs"), Vec::<String>::new());
    }

    #[test]
    fn missing_reference_is_ref_01() {
        assert_eq!(codes_for(None, "count_pairs"), ["REF-01"]);
    }

    #[test]
    fn misspelled_function_is_ref_01() {
        let source = "def count_pair(values, target):\n    return 0\n";

        assert_eq!(codes_for(Some(source), "count_pairs"), ["REF-01"]);
    }

    #[test]
    fn indented_definition_is_ref_01() {
        let source =
            "class Solution:\n    def count_pairs(self, values, target):\n        return 0\n";

        assert_eq!(codes_for(Some(source), "count_pairs"), ["REF-01"]);
    }
}
