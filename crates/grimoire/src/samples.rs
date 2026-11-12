//! Sample checks (SMP-01 to SMP-03) and the return-shape table.

use crate::codes;
use crate::meta::Meta;
use crate::report::{Diagnostic, Report};
use crate::signature::LaunchType;
use serde_json::Value;
use std::collections::BTreeSet;

/// Checks the sample count, each sample against the schema and return shape, and duplicates.
pub fn check(meta: &Meta, report: &mut Report) {
    let count = meta.samples.len();
    if !(2..=5).contains(&count) {
        report.push(Diagnostic::new(
            codes::SMP_01,
            "meta.toml",
            Some("/samples"),
            "there must be 2 to 5 samples",
        ));
    }
    let return_kind = LaunchType::parse(&meta.returns);
    let mut seen = BTreeSet::new();
    for (index, sample) in meta.samples.iter().enumerate() {
        if meta.input_schema.contains(&sample.input).is_err() {
            report.push(Diagnostic::new(
                codes::SMP_02,
                "meta.toml",
                Some(&format!("/samples/{index}/input")),
                "sample input does not fit the input schema",
            ));
        }
        if return_kind.is_some_and(|kind| !accepts_return(kind, &sample.expected)) {
            report.push(Diagnostic::new(
                codes::SMP_02,
                "meta.toml",
                Some(&format!("/samples/{index}/expected")),
                "sample expected value does not match the return type",
            ));
        }
        if !seen.insert(sample.input.to_string()) {
            report.push(Diagnostic::new(
                codes::SMP_03,
                "meta.toml",
                Some(&format!("/samples/{index}/input")),
                "two samples have the same input",
            ));
        }
    }
}

/// Whether `value` has the JSON shape of the return type `kind`.
#[must_use]
pub fn accepts_return(kind: LaunchType, value: &Value) -> bool {
    match kind {
        LaunchType::Int => value.is_i64(),
        LaunchType::Str => value.is_string(),
        LaunchType::IntArray => value
            .as_array()
            .is_some_and(|items| items.iter().all(Value::is_i64)),
        LaunchType::Matrix => value.as_array().is_some_and(|rows| {
            let width = rows.first().and_then(Value::as_array).map(Vec::len);
            rows.iter().all(|row| {
                row.as_array().is_some_and(|cells| {
                    Some(cells.len()) == width && cells.iter().all(Value::is_i64)
                })
            })
        }),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fixtures;

    fn codes_for(text: &str) -> Vec<String> {
        let meta = fixtures::parse(text);
        let mut report = Report::new("count-pairs");
        check(&meta, &mut report);
        report
            .diagnostics()
            .iter()
            .map(|d| d.code.to_owned())
            .collect()
    }

    #[test]
    fn valid_samples_have_no_codes() {
        assert_eq!(codes_for(&fixtures::valid_text()), Vec::<String>::new());
    }

    #[test]
    fn one_sample_is_smp_01() {
        let text = fixtures::valid_text()
            .replace("\n[[samples]]\ninput = [[5, 5], 10]\nexpected = 1\n", "");

        assert_eq!(codes_for(&text), ["SMP-01"]);
    }

    #[test]
    fn six_samples_is_smp_01() {
        let extra = (0..4)
            .map(|index| format!("\n[[samples]]\ninput = [[{index}], {index}]\nexpected = 0\n"))
            .collect::<Vec<_>>()
            .concat();
        let text = format!("{}{extra}", fixtures::valid_text());

        assert_eq!(codes_for(&text), ["SMP-01"]);
    }

    #[test]
    fn sample_input_outside_the_schema_is_smp_02() {
        let text = fixtures::valid_text().replace("input = [[5, 5], 10]", "input = [[5, 5], 500]");

        assert_eq!(codes_for(&text), ["SMP-02"]);
    }

    #[test]
    fn float_expected_for_an_int_return_is_smp_02() {
        let text = fixtures::valid_text().replace("expected = 1\n", "expected = 1.0\n");

        assert_eq!(codes_for(&text), ["SMP-02"]);
    }

    #[test]
    fn duplicate_sample_input_is_smp_03() {
        let text =
            fixtures::valid_text().replace("input = [[5, 5], 10]", "input = [[1, 2, 3, 2], 4]");

        assert_eq!(codes_for(&text), ["SMP-03"]);
    }

    #[test]
    fn return_shape_accepts_an_integer_and_rejects_a_float() {
        assert!(accepts_return(LaunchType::Int, &serde_json::json!(3)));
        assert!(!accepts_return(LaunchType::Int, &serde_json::json!(3.0)));
    }

    #[test]
    fn matrix_return_must_have_equal_row_lengths() {
        assert!(accepts_return(
            LaunchType::Matrix,
            &serde_json::json!([[1, 2], [3, 4]])
        ));
        assert!(!accepts_return(
            LaunchType::Matrix,
            &serde_json::json!([[1, 2], [3]])
        ));
    }
}
