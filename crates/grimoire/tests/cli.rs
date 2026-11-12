//! The `validate` command: exit codes, text and JSON output, `--all` and `--sandbox` (T13).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

/// A temporary directory that holds packs and is removed when the test ends.
struct TempRoot(PathBuf);

impl TempRoot {
    fn new(label: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("grimoire-cli-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create temp root");
        Self(path)
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempRoot {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Writes a complete `count-pairs`-style pack under `root/<name>`. The meta file
/// is valid unless `meta` is given.
fn write_pack(root: &Path, name: &str, meta: Option<&str>) {
    let dir = root.join(name);
    fs::create_dir_all(dir.join("reference")).expect("create pack directory");
    fs::write(dir.join("problem.md"), "# Count the Pairs\n\n## Task\nCount.\n\n## Input\nA.\n\n## Output\nB.\n\n## Examples\nC.\n\n## Constraints\nD.\n").expect("write problem");
    let valid_meta = META.replace("count-pairs", name);
    fs::write(dir.join("meta.toml"), meta.unwrap_or(&valid_meta)).expect("write meta");
    fs::write(dir.join("hints.toml"), HINTS).expect("write hints");
    fs::write(
        dir.join("reference/solution.py"),
        "def count_pairs(values, target):\n    return 0\n",
    )
    .expect("write reference");
}

const META: &str = r#"slug = "count-pairs"
title = "Count the Pairs"
status = "draft"
difficulty = "easy"
track = "arrays"
concepts = ["arrays", "pairs"]
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
text = "One."

[[hint]]
rung = 2
text = "Two."

[[hint]]
rung = 3
text = "Three."

[[hint]]
rung = 4
text = "Four."
"#;

/// Runs the `validate` binary and returns its exit code and stdout.
fn run(args: &[&str]) -> (i32, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_validate"))
        .args(args)
        .output()
        .expect("run validate binary");
    (
        output.status.code().unwrap_or(-1),
        String::from_utf8_lossy(&output.stdout).into_owned(),
    )
}

fn path_str(path: &Path) -> &str {
    path.to_str().expect("temp path is UTF-8")
}

#[test]
fn clean_pack_exits_zero() {
    let root = TempRoot::new("clean");
    write_pack(root.path(), "count-pairs", None);

    let (code, _) = run(&["validate", path_str(&root.path().join("count-pairs"))]);

    assert_eq!(code, 0);
}

#[test]
fn broken_pack_exits_one_and_prints_the_code() {
    let root = TempRoot::new("broken");
    let broken = META.replace("licence = \"MIT\"\n", "");
    write_pack(root.path(), "count-pairs", Some(&broken));

    let (code, stdout) = run(&["validate", path_str(&root.path().join("count-pairs"))]);

    assert_eq!(code, 1);
    assert!(stdout.contains("ERROR MET-02 meta.toml"), "{stdout}");
}

#[test]
fn json_output_is_one_report_with_a_summary() {
    let root = TempRoot::new("json");
    write_pack(root.path(), "count-pairs", None);

    let (code, stdout) = run(&[
        "validate",
        "--json",
        path_str(&root.path().join("count-pairs")),
    ]);

    assert_eq!(code, 0);
    let value: serde_json::Value = serde_json::from_str(&stdout).expect("stdout is JSON");
    assert_eq!(value["summary"]["errors"], 0);
    assert_eq!(value["pack"], "count-pairs");
}

#[test]
fn missing_arguments_exit_two() {
    let (code, _) = run(&["validate"]);

    assert_eq!(code, 2);
}

#[test]
fn unreadable_pack_path_exits_two() {
    let root = TempRoot::new("missing");

    let (code, _) = run(&["validate", path_str(&root.path().join("no-such-pack"))]);

    assert_eq!(code, 2);
}

#[test]
fn all_skips_underscore_packs_unless_asked() {
    let root = TempRoot::new("all");
    write_pack(root.path(), "count-pairs", None);
    write_pack(
        root.path(),
        "_scratch",
        Some(&META.replace("licence = \"MIT\"\n", "")),
    );

    let (skipped, _) = run(&["validate", "--all", path_str(root.path())]);
    let (included, stdout) = run(&[
        "validate",
        "--all",
        "--include-underscore",
        path_str(root.path()),
    ]);

    assert_eq!(skipped, 0);
    assert_eq!(included, 1);
    assert!(stdout.contains("MET-02"), "{stdout}");
}

#[test]
fn sandbox_without_a_runner_exits_two_with_sbx_07() {
    let root = TempRoot::new("sandbox");
    write_pack(root.path(), "count-pairs", None);

    let (code, stdout) = run(&[
        "validate",
        "--sandbox",
        path_str(&root.path().join("count-pairs")),
    ]);

    assert_eq!(code, 2);
    assert!(stdout.contains("SBX-07"), "{stdout}");
}
