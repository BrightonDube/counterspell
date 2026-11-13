//! The static pipeline on a complete temporary pack (T12).

use counterspell_grimoire::validate_static;
use std::fs;
use std::path::PathBuf;

/// A temporary `count-pairs` pack directory, removed when the test ends.
struct TempPack {
    root: PathBuf,
}

impl TempPack {
    fn new(label: &str) -> Self {
        let parent =
            std::env::temp_dir().join(format!("grimoire-pipeline-{label}-{}", std::process::id()));
        let root = parent.join("count-pairs");
        let _ = fs::remove_dir_all(&parent);
        fs::create_dir_all(root.join("reference")).expect("create reference directory");
        let pack = Self { root };
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
        if let Some(parent) = self.root.parent() {
            let _ = fs::remove_dir_all(parent);
        }
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
fn complete_pack_has_no_errors() {
    let pack = TempPack::new("clean");

    let report = validate_static(&pack.root).expect("temp pack is readable");

    assert_eq!(report.error_count(), 0, "{:?}", report.diagnostics());
    assert!(report.checked().contains(&"reference".to_owned()));
}

#[test]
fn broken_meta_reports_its_code_and_skips_content_stages() {
    let pack = TempPack::new("broken");
    pack.write("meta.toml", &META.replace("licence = \"MIT\"\n", ""));

    let report = validate_static(&pack.root).expect("temp pack is readable");

    let codes: Vec<&str> = report.diagnostics().iter().map(|d| d.code).collect();
    assert_eq!(codes, ["MET-02"]);
    assert!(report.skipped().contains(&"statement".to_owned()));
    assert!(report.skipped().contains(&"samples".to_owned()));
    assert!(!report.checked().contains(&"statement".to_owned()));
}
