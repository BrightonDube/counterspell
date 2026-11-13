//! Signature and schema mapping (MET-04, MET-05, MET-06, MET-07).

use crate::codes;
use crate::meta::Meta;
use crate::report::{Diagnostic, Report};
use counterspell_engine::Schema;

/// A launch parameter or return type, as written in `meta.toml`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum LaunchType {
    /// A whole number.
    Int,
    /// A string.
    Str,
    /// A list of whole numbers.
    IntArray,
    /// A list of lists of whole numbers.
    Matrix,
}

impl LaunchType {
    /// Reads a launch type name. Returns `None` for anything outside the launch set.
    #[must_use]
    pub fn parse(name: &str) -> Option<Self> {
        match name {
            "int" => Some(Self::Int),
            "str" => Some(Self::Str),
            "list[int]" => Some(Self::IntArray),
            "list[list[int]]" => Some(Self::Matrix),
            _ => None,
        }
    }

    /// Whether `schema` is the engine schema kind this type maps to (GRM-12).
    fn matches(self, schema: &Schema) -> bool {
        matches!(
            (self, schema),
            (Self::Int, Schema::Int { .. })
                | (Self::Str, Schema::Str { .. })
                | (Self::IntArray, Schema::IntArray { .. })
                | (Self::Matrix, Schema::Matrix { .. })
        )
    }

    /// Whether this type is a list, so the `unordered` comparator may apply.
    const fn is_list(self) -> bool {
        matches!(self, Self::IntArray | Self::Matrix)
    }
}

/// Checks the signature, the schema mapping, the schema definition and the comparator.
pub fn check(meta: &Meta, report: &mut Report) {
    for (index, param) in meta.params.iter().enumerate() {
        if LaunchType::parse(&param.kind).is_none() {
            report.push(Diagnostic::new(
                codes::MET_04,
                "meta.toml",
                Some(&format!("/params/{index}/type")),
                format!("parameter type {:?} is not in the launch set", param.kind),
            ));
        }
    }
    if LaunchType::parse(&meta.returns).is_none() {
        report.push(Diagnostic::new(
            codes::MET_04,
            "meta.toml",
            Some("/returns"),
            format!("return type {:?} is not in the launch set", meta.returns),
        ));
    }
    check_schema_mapping(meta, report);
    if let Err(error) = meta.input_schema.check_definition() {
        report.push(Diagnostic::new(
            codes::MET_06,
            "meta.toml",
            Some("/input_schema"),
            format!("the input schema is not supported: {error}"),
        ));
    }
    check_comparator(meta, report);
}

/// Checks that `input_schema` is a tuple with one item per parameter, of the matching kind.
fn check_schema_mapping(meta: &Meta, report: &mut Report) {
    let Schema::Tuple { items } = &meta.input_schema else {
        report.push(Diagnostic::new(
            codes::MET_05,
            "meta.toml",
            Some("/input_schema"),
            "input_schema must be a tuple, one item per parameter",
        ));
        return;
    };
    if items.len() != meta.params.len() {
        report.push(Diagnostic::new(
            codes::MET_05,
            "meta.toml",
            Some("/input_schema/items"),
            format!(
                "input_schema has {} items but there are {} parameters",
                items.len(),
                meta.params.len()
            ),
        ));
        return;
    }
    for (index, (param, item)) in meta.params.iter().zip(items).enumerate() {
        let Some(kind) = LaunchType::parse(&param.kind) else {
            continue;
        };
        if !kind.matches(item) {
            report.push(Diagnostic::new(
                codes::MET_05,
                "meta.toml",
                Some(&format!("/input_schema/items/{index}")),
                format!(
                    "schema item {index} does not match parameter {:?}",
                    param.name
                ),
            ));
        }
    }
}

/// Checks the comparator name, and that `unordered` is used only for list returns.
fn check_comparator(meta: &Meta, report: &mut Report) {
    match meta.comparator.as_str() {
        "exact" => {}
        "unordered" => {
            let returns_list = LaunchType::parse(&meta.returns).is_some_and(LaunchType::is_list);
            if !returns_list {
                report.push(Diagnostic::new(
                    codes::MET_07,
                    "meta.toml",
                    Some("/comparator"),
                    "unordered comparison is allowed only for list returns",
                ));
            }
        }
        _ => report.push(Diagnostic::new(
            codes::MET_07,
            "meta.toml",
            Some("/comparator"),
            "comparator must be exact or unordered",
        )),
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
    fn valid_signature_has_no_codes() {
        assert_eq!(codes_for(&fixtures::valid_text()), Vec::<String>::new());
    }

    #[test]
    fn unknown_parameter_type_is_met_04() {
        let text = fixtures::valid_text().replace("type = \"int\"\n", "type = \"float\"\n");

        assert_eq!(codes_for(&text), ["MET-04"]);
    }

    #[test]
    fn unknown_return_type_is_met_04() {
        let text = fixtures::valid_text().replace("returns = \"int\"", "returns = \"float\"");

        assert_eq!(codes_for(&text), ["MET-04"]);
    }

    #[test]
    fn schema_with_one_item_for_two_params_is_met_05() {
        let text =
            fixtures::valid_text().replace("  { type = \"int\", min = -100, max = 100 },\n", "");

        assert_eq!(codes_for(&text), ["MET-05"]);
    }

    #[test]
    fn schema_item_of_the_wrong_kind_is_met_05() {
        let text = fixtures::valid_text().replace(
            "{ type = \"int\", min = -100, max = 100 }",
            "{ type = \"string\", alphabet = \"ab\", min_len = 0, max_len = 3 }",
        );

        assert_eq!(codes_for(&text), ["MET-05"]);
    }

    #[test]
    fn schema_that_is_not_a_tuple_is_met_05() {
        let text = fixtures::valid_text().replace(
            "type = \"tuple\"\nitems = [\n  { type = \"int_array\", min_len = 0, max_len = 30, min = -50, max = 50 },\n  { type = \"int\", min = -100, max = 100 },\n]",
            "type = \"int\"\nmin = -5\nmax = 5",
        );

        assert_eq!(codes_for(&text), ["MET-05"]);
    }

    #[test]
    fn schema_the_engine_rejects_is_met_06() {
        let text = fixtures::valid_text().replace(
            "{ type = \"int\", min = -100, max = 100 }",
            "{ type = \"int\", min = 10, max = -10 }",
        );

        assert_eq!(codes_for(&text), ["MET-06"]);
    }

    #[test]
    fn unordered_comparator_on_an_int_return_is_met_07() {
        let text =
            fixtures::valid_text().replace("comparator = \"exact\"", "comparator = \"unordered\"");

        assert_eq!(codes_for(&text), ["MET-07"]);
    }

    #[test]
    fn unknown_comparator_is_met_07() {
        let text =
            fixtures::valid_text().replace("comparator = \"exact\"", "comparator = \"float\"");

        assert_eq!(codes_for(&text), ["MET-07"]);
    }

    #[test]
    fn unordered_comparator_on_a_list_return_is_allowed() {
        let text = fixtures::valid_text()
            .replace("returns = \"int\"", "returns = \"list[int]\"")
            .replace("comparator = \"exact\"", "comparator = \"unordered\"");

        assert_eq!(codes_for(&text), Vec::<String>::new());
    }
}
