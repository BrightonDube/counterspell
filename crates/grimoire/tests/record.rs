//! The `ProblemRecord` handoff is built only from a pack with zero errors (GRM-54, T14).

use counterspell_engine::Comparator;
use counterspell_grimoire::record::{build, Difficulty, RecordError};
use std::fs;
use std::path::PathBuf;

/// A temporary `count-pairs` pack, removed when the test ends.
struct TempPack {
    parent: PathBuf,
    root: PathBuf,
}

impl TempPack {
    fn new(label: &str) -> Self {
        let parent =
            std::env::temp_dir().join(format!("grimoire-record-{label}-{}", std::process::id()));
        let root = parent.join("count-pairs");
        let _ = fs::remove_dir_all(&parent);
        fs::create_dir_all(root.join("reference")).expect("create reference directory");
        let pack = Self { parent, root };
        pack.write("problem.md", PROBLEM);
        pack.write("meta.toml", META);
        pack.write("hints.toml", HINTS);
        pack.write("reference/solution.py", REFERENCE);
        pack
    }

    fn write(&self, relative: &str, text: &str) {
        fs::write(self.root.join(relative), text).expect("write fixture file");
    }
}

impl Drop for TempPack {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.parent);
    }
}

const PROBLEM: &str = "# Count the Pairs\n\n## Task\nCount pairs.\n\n## Input\nA list and a target.\n\n## Output\nA count.\n\n## Examples\nSee samples.\n\n## Constraints\nSmall.\n";

const META: &str = r#"slug = "count-pairs"
title = "Count the Pairs"
status = "draft"
difficulty = "easy"
track = "arrays"
concepts = ["arrays", "pairs", "nested-loops"]
origin = "original"
author = "Brighton"
licence = "MIT"
pack_version = 3
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
"#;

const HINTS: &str = r#"[[hint]]
rung = 1
text = "Ask what a pair needs."

[[hint]]
rung = 2
text = "Loop over each first item."

[[hint]]
rung = 3
text = "Start the second loop after the first."

[[hint]]
rung = 4
text = "Change where the inner loop starts."
"#;

const REFERENCE: &str = "def count_pairs(values, target):\n    count = 0\n    for i in range(len(values)):\n        for j in range(i + 1, len(values)):\n            if values[i] + values[j] == target:\n                count += 1\n    return count\n";

#[test]
fn valid_pack_builds_a_record_with_its_fields() {
    let pack = TempPack::new("valid");

    let record = build(&pack.root).expect("a clean pack builds a record");

    assert_eq!(record.slug, "count-pairs");
    assert_eq!(record.title, "Count the Pairs");
    assert_eq!(record.version, 3);
    assert_eq!(record.difficulty, Difficulty::Easy);
    assert_eq!(record.comparator, Comparator::Exact);
    assert_eq!(record.concepts, ["arrays", "pairs", "nested-loops"]);
    assert_eq!(record.limits.time_ms, 2000);
    assert!(record.statement_md.starts_with("# Count the Pairs"));
}

#[test]
fn pack_with_an_error_is_refused_with_its_report() {
    let pack = TempPack::new("invalid");
    pack.write("meta.toml", &META.replace("licence = \"MIT\"\n", ""));

    match build(&pack.root) {
        Err(RecordError::Invalid(report)) => {
            assert_eq!(report.error_count(), 1);
            assert_eq!(report.diagnostics()[0].code, "MET-02");
        }
        other => panic!("expected an invalid pack, got {other:?}"),
    }
}
