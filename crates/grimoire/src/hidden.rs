//! Curated hidden cases: the private `hidden.toml` file.
//!
//! Each `[[case]]` holds an authored input and its expected value. The sandbox checks
//! verify every case against the reference (SBX-09) before the import stores it.

use serde::Deserialize;
use serde_json::Value;

/// The private file that holds the curated hidden cases.
pub const HIDDEN_FILE: &str = "hidden.toml";

/// One `[[case]]` entry.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct CuratedCase {
    /// The authored input (the argument tuple).
    input: Value,
    /// The authored expected value.
    expected: Value,
    /// The author's note; parsed and never reported.
    #[allow(dead_code)]
    why: Option<String>,
}

/// The shape of `hidden.toml`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct HiddenFile {
    #[serde(default, rename = "case")]
    cases: Vec<CuratedCase>,
}

/// Parses `hidden.toml` into `(input, authored expected)` pairs, in file order.
///
/// # Errors
///
/// Returns the parser's message when the TOML is malformed or has unknown keys.
pub fn curated_cases(text: &str) -> Result<Vec<(Value, Value)>, String> {
    let file: HiddenFile = toml::from_str(text).map_err(|error| error.message().to_owned())?;
    Ok(file
        .cases
        .into_iter()
        .map(|case| (case.input, case.expected))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn cases_keep_file_order_and_drop_the_note() {
        let text = "[[case]]\ninput = [1]\nexpected = 2\nwhy = \"edge\"\n\n[[case]]\ninput = [3]\nexpected = 4\n";

        let cases = curated_cases(text).expect("the file parses");

        assert_eq!(cases, vec![(json!([1]), json!(2)), (json!([3]), json!(4))]);
    }

    #[test]
    fn an_unknown_key_is_refused() {
        assert!(curated_cases("[[case]]\ninput = [1]\nexpected = 2\nanswer = 2\n").is_err());
    }
}
