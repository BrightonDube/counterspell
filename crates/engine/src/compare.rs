//! Trusted comparators for judging JSON outputs.

use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Comparison strategies supported by the ghost engine.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum Comparator {
    /// Compares JSON values with strict [`Value`] equality.
    #[serde(rename = "exact")]
    Exact,
    /// Compares top-level arrays as multisets and other values strictly.
    #[serde(rename = "unordered")]
    Unordered,
}

impl Comparator {
    /// Returns whether `expected` and `got` match under this comparator.
    #[must_use]
    pub fn equal(&self, expected: &Value, got: &Value) -> bool {
        match self {
            Self::Exact => expected == got,
            Self::Unordered => match (expected, got) {
                (Value::Array(expected), Value::Array(got)) => {
                    let mut expected = expected.iter().map(ToString::to_string).collect::<Vec<_>>();
                    let mut got = got.iter().map(ToString::to_string).collect::<Vec<_>>();
                    expected.sort_unstable();
                    got.sort_unstable();
                    expected == got
                }
                _ => expected == got,
            },
        }
    }
}
