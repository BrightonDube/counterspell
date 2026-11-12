//! Shared `meta.toml` text for unit tests. Compiled only for tests.

use crate::meta::Meta;

/// A complete, valid `meta.toml` for the slug `count-pairs`.
pub(crate) fn valid_text() -> String {
    r#"slug = "count-pairs"
title = "Count the Pairs"
status = "draft"
difficulty = "easy"
track = "arrays"
concepts = ["arrays", "pairs", "nested-loops"]
origin = "original"
author = "Brighton"
licence = "MIT"
pack_version = 1
function = "count_pairs"
returns = "int"
comparator = "exact"

[[params]]
name = "values"
type = "list[int]"

[[params]]
name = "target"
type = "int"

[limits]
time_ms = 2000
mem_mb = 256

[input_schema]
type = "tuple"
items = [
  { type = "int_array", min_len = 0, max_len = 30, min = -50, max = 50 },
  { type = "int", min = -100, max = 100 },
]

[generator]
seed = 1729
hidden_count = 12

[[samples]]
input = [[1, 2, 3, 2], 4]
expected = 2

[[samples]]
input = [[5, 5], 10]
expected = 1
"#
    .to_owned()
}

/// Parses meta text without running the value checks, so a test can exercise one check.
pub(crate) fn parse(text: &str) -> Meta {
    toml::from_str::<Meta>(text).expect("fixture meta.toml must deserialise")
}
