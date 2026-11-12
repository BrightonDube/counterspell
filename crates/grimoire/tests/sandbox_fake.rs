//! Sandbox checks SBX-01 to SBX-04 driven by a MODEL RUNNER.
//!
//! This file proves the logic of the sandbox checks. It does not prove the
//! sandbox. The model runner is plain Rust that mirrors each pack program, so no
//! Python runs here. The judge spec must run the same vectors through the real
//! Python files (task T20). Expected values are literals written by hand. The
//! model is never used to derive an expectation.

use counterspell_engine::ProgramResult;
use counterspell_grimoire::report::{Report, Severity};
use counterspell_grimoire::runner::{PackRunner, RunnerError, Target};
use counterspell_grimoire::{validate_sandbox, SandboxFault};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

/// A model of one pack program: argument list in, raw result out.
type Model = fn(&Value) -> ProgramResult;

/// Answers every run from Rust models, and counts the calls it receives.
struct ModelRunner {
    reference: Model,
    programs: HashMap<&'static str, Model>,
    calls: usize,
}

impl ModelRunner {
    fn new(reference: Model, programs: &[(&'static str, Model)]) -> Self {
        Self {
            reference,
            programs: programs.iter().copied().collect(),
            calls: 0,
        }
    }
}

impl PackRunner for ModelRunner {
    fn run(
        &mut self,
        target: Target<'_>,
        input: &Value,
        _limit: Duration,
    ) -> Result<ProgramResult, RunnerError> {
        self.calls += 1;
        match target {
            Target::Reference => Ok((self.reference)(input)),
            Target::Program(path) => self
                .programs
                .get(path)
                .map(|model| model(input))
                .ok_or_else(|| RunnerError::Infrastructure(format!("no model for {path}"))),
        }
    }

    fn bench(
        &mut self,
        _sized_inputs: &[(u64, Value)],
        _limit: Duration,
    ) -> Result<Vec<(u64, u64)>, RunnerError> {
        Err(RunnerError::Infrastructure(
            "bench is not modelled".to_owned(),
        ))
    }
}

/// Reads the two arguments of `count_pairs(values, target)`.
fn arguments(input: &Value) -> (Vec<i64>, i64) {
    let values = input[0]
        .as_array()
        .expect("first argument is a list")
        .iter()
        .map(|item| item.as_i64().expect("list items are integers"))
        .collect();
    let target = input[1].as_i64().expect("second argument is an integer");
    (values, target)
}

/// The authoritative reference: pairs of positions i < j whose values add up to the target.
fn reference_model(input: &Value) -> ProgramResult {
    let (values, target) = arguments(input);
    let mut count = 0;
    for i in 0..values.len() {
        for j in i + 1..values.len() {
            if values[i] + values[j] == target {
                count += 1;
            }
        }
    }
    ProgramResult::Returned(json!(count))
}

/// Wrong answer: the inner loop starts at the outer index, so a value pairs with itself.
fn same_index_model(input: &Value) -> ProgramResult {
    let (values, target) = arguments(input);
    let mut count = 0;
    for i in 0..values.len() {
        for j in i..values.len() {
            if values[i] + values[j] == target {
                count += 1;
            }
        }
    }
    ProgramResult::Returned(json!(count))
}

/// Crash: the early exit calls `min()`, which raises on an empty list.
fn empty_min_model(input: &Value) -> ProgramResult {
    let (values, target) = arguments(input);
    let Some(smallest) = values.iter().min() else {
        return ProgramResult::Crashed {
            exception_class: "ValueError".to_owned(),
            message: "min() arg is an empty sequence".to_owned(),
        };
    };
    if target < 2 * smallest {
        return ProgramResult::Returned(json!(0));
    }
    reference_model(input)
}

/// A reference that fails on every input, to show the hidden-input check fires.
fn always_crashes_model(_input: &Value) -> ProgramResult {
    ProgramResult::Crashed {
        exception_class: "RuntimeError".to_owned(),
        message: "model failure".to_owned(),
    }
}

/// The valid `count-pairs` files. Each is the text a starter pack uses.
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

[[samples]]
input = [[-3, 3, 0], 0]
expected = 1
"#;

const PROBLEM: &str = "# Count the Pairs

## Task
Given a list of whole numbers and a target, count the pairs of positions i < j whose values add up to the target.

## Input
A list of whole numbers, then a whole-number target.

## Output
One whole number: how many pairs of positions add up to the target.

## Examples
The public examples are listed in the pack's samples.

## Constraints
The list has at most 30 values, each from -50 to 50. The target is from -100 to 100.
";

const HINTS: &str = r#"[[hint]]
rung = 1
text = "Ask what a pair of positions needs before you count it."

[[hint]]
rung = 2
text = "Think about the two positions as a loop over each first item, then a loop over the items after it."

[[hint]]
rung = 3
text = "Start the second loop one place after the first, so an item never pairs with itself."

[[hint]]
rung = 4
text = "Your inner loop may start at the same index as the outer loop. Change where it starts."
"#;

const KNOWN_GHOSTS: &str = r#"[[ghost]]
id = "same-index"
program = "incorrect/same-index.py"
kind = "wrong"
input = [[0], 0]
got = 1
concepts = ["pairs", "nested-loops"]
explanation = "A value cannot pair with itself. The inner loop starts at the outer index, so the single item pairs with itself and is counted."

[[ghost]]
id = "empty-min"
program = "incorrect/empty-min.py"
kind = "crash"
input = [[], 0]
exception_class = "ValueError"
concepts = ["arrays"]
explanation = "The early exit calls min() on an empty list, which raises an error. Check for an empty list before you use min()."
"#;

const REFERENCE: &str = "def count_pairs(values, target):
    count = 0
    for i in range(len(values)):
        for j in range(i + 1, len(values)):
            if values[i] + values[j] == target:
                count += 1
    return count
";

const SAME_INDEX: &str = "def count_pairs(values, target):
    count = 0
    for i in range(len(values)):
        for j in range(i, len(values)):
            if values[i] + values[j] == target:
                count += 1
    return count
";

const EMPTY_MIN: &str = "def count_pairs(values, target):
    # Early exit: no pair can add up to less than twice the smallest value.
    if target < 2 * min(values):
        return 0
    count = 0
    for i in range(len(values)):
        for j in range(i + 1, len(values)):
            if values[i] + values[j] == target:
                count += 1
    return count
";

/// The text of one `count-pairs` file, by its path inside the pack.
fn default_text(name: &str) -> String {
    match name {
        "meta.toml" => META.to_owned(),
        "problem.md" => PROBLEM.to_owned(),
        "hints.toml" => HINTS.to_owned(),
        "known_ghosts.toml" => KNOWN_GHOSTS.to_owned(),
        "reference/solution.py" => REFERENCE.to_owned(),
        "incorrect/same-index.py" => SAME_INDEX.to_owned(),
        "incorrect/empty-min.py" => EMPTY_MIN.to_owned(),
        other => panic!("no default text for {other}"),
    }
}

/// A `count-pairs` pack in a temporary directory, removed when the test ends.
struct TempPack {
    base: PathBuf,
}

impl TempPack {
    /// Writes the valid pack, replacing the files named in `overrides` with their new text.
    fn new(label: &str, overrides: &[(&str, String)]) -> Self {
        let base =
            std::env::temp_dir().join(format!("grimoire-sandbox-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&base);
        let pack = Self { base };
        for name in [
            "meta.toml",
            "problem.md",
            "hints.toml",
            "known_ghosts.toml",
            "reference/solution.py",
            "incorrect/same-index.py",
            "incorrect/empty-min.py",
        ] {
            let text = overrides
                .iter()
                .find(|(file, _)| *file == name)
                .map_or_else(|| default_text(name), |(_, text)| text.clone());
            pack.write(name, &text);
        }
        pack
    }

    /// The pack directory. Its name must equal the slug in `meta.toml` (LAY-06).
    fn root(&self) -> PathBuf {
        self.base.join("count-pairs")
    }

    fn write(&self, relative: &str, text: &str) {
        let path = self.root().join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create pack subdirectory");
        }
        fs::write(path, text).expect("write pack file");
    }
}

impl Drop for TempPack {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.base);
    }
}

/// Runs the full validator, with the sandbox stage, and expects it to finish.
fn run_sandbox(root: &Path, runner: &mut ModelRunner) -> Report {
    validate_sandbox(root, runner).expect("model runner and pack files are readable")
}

/// The diagnostic codes in the report, in the order they were emitted.
fn codes(report: &Report) -> Vec<&'static str> {
    report.diagnostics().iter().map(|d| d.code).collect()
}

#[test]
fn clean_pack_passes_every_sandbox_check() {
    let pack = TempPack::new("clean", &[]);
    let mut runner = ModelRunner::new(
        reference_model,
        &[
            ("incorrect/same-index.py", same_index_model),
            ("incorrect/empty-min.py", empty_min_model),
        ],
    );

    let report = run_sandbox(&pack.root(), &mut runner);

    assert_eq!(codes(&report), Vec::<&str>::new());
    assert!(report.checked().iter().any(|stage| stage == "sandbox"));
}

#[test]
fn reference_that_disagrees_with_a_sample_is_sbx_01() {
    // Sample 1 is [[1, 2, 3, 2], 4]. Pairs summing to 4 are (1, 3) and (2, 2): two.
    let meta = META.replace("expected = 2\n", "expected = 3\n");
    let pack = TempPack::new("sample", &[("meta.toml", meta)]);
    let mut runner = ModelRunner::new(
        reference_model,
        &[
            ("incorrect/same-index.py", same_index_model),
            ("incorrect/empty-min.py", empty_min_model),
        ],
    );

    let report = run_sandbox(&pack.root(), &mut runner);

    assert_eq!(codes(&report), ["SBX-01"]);
}

#[test]
fn reference_that_fails_on_hidden_inputs_is_sbx_02() {
    let pack = TempPack::new("hidden", &[]);
    let mut runner = ModelRunner::new(
        always_crashes_model,
        &[
            ("incorrect/same-index.py", same_index_model),
            ("incorrect/empty-min.py", empty_min_model),
        ],
    );

    let report = run_sandbox(&pack.root(), &mut runner);

    assert!(codes(&report).contains(&"SBX-02"));
}

#[test]
fn known_ghost_with_a_wrong_got_value_is_sbx_03() {
    // same-index returns 1 for [[0], 0]. An authored got of 2 does not reproduce.
    let ghosts = KNOWN_GHOSTS.replace("got = 1", "got = 2");
    let pack = TempPack::new("ghost-got", &[("known_ghosts.toml", ghosts)]);
    let mut runner = ModelRunner::new(
        reference_model,
        &[
            ("incorrect/same-index.py", same_index_model),
            ("incorrect/empty-min.py", empty_min_model),
        ],
    );

    let report = run_sandbox(&pack.root(), &mut runner);

    assert_eq!(codes(&report), ["SBX-03"]);
}

#[test]
fn known_ghost_that_a_smaller_input_also_breaks_is_an_sbx_04_warning() {
    // [[0, 0, 0], 0] fails the same way as [[0], 0]: the program gives 6, the reference 3.
    // The program returns 1 for [[0], 0] and the reference 0, so [[0], 0] is smaller.
    let ghosts = KNOWN_GHOSTS
        .replace("input = [[0], 0]", "input = [[0, 0, 0], 0]")
        .replace("got = 1", "got = 6");
    let pack = TempPack::new("minimal", &[("known_ghosts.toml", ghosts)]);
    let mut runner = ModelRunner::new(
        reference_model,
        &[
            ("incorrect/same-index.py", same_index_model),
            ("incorrect/empty-min.py", empty_min_model),
        ],
    );

    let report = run_sandbox(&pack.root(), &mut runner);

    assert_eq!(codes(&report), ["SBX-04"]);
    assert!(!report.has_errors());
}

#[test]
fn declared_bench_warns_sbx_08_without_calling_the_bench_runner() {
    // The model runner's bench always fails, so any call to it would surface as an error.
    let meta = format!(
        "reference_complexity = \"n^2\"\n{META}\n[bench]\nsize_param = \"values\"\nsizes = [8, 16, 32]\n"
    );
    let pack = TempPack::new("bench", &[("meta.toml", meta)]);
    let mut runner = ModelRunner::new(
        reference_model,
        &[
            ("incorrect/same-index.py", same_index_model),
            ("incorrect/empty-min.py", empty_min_model),
        ],
    );

    let report = run_sandbox(&pack.root(), &mut runner);

    assert_eq!(codes(&report), ["SBX-08"]);
    assert!(!report.has_errors());
}

/// A runner whose sandbox is down: every call is an infrastructure fault.
struct FaultyRunner;

impl PackRunner for FaultyRunner {
    fn run(
        &mut self,
        _target: Target<'_>,
        _input: &Value,
        _limit: Duration,
    ) -> Result<ProgramResult, RunnerError> {
        Err(RunnerError::Infrastructure(
            "sandbox daemon unreachable".to_owned(),
        ))
    }

    fn bench(
        &mut self,
        _sized_inputs: &[(u64, Value)],
        _limit: Duration,
    ) -> Result<Vec<(u64, u64)>, RunnerError> {
        Err(RunnerError::Infrastructure(
            "sandbox daemon unreachable".to_owned(),
        ))
    }
}

#[test]
fn runner_fault_maps_to_sbx_06() {
    let pack = TempPack::new("fault", &[]);

    let fault = match validate_sandbox(&pack.root(), &mut FaultyRunner) {
        Err(fault) => fault,
        Ok(report) => panic!("a runner fault must not return a report: {report:?}"),
    };

    assert!(matches!(
        fault,
        SandboxFault::Runner(RunnerError::Infrastructure(_))
    ));
    let diagnostic = fault.diagnostic();
    assert_eq!(diagnostic.code, "SBX-06");
    assert_eq!(diagnostic.severity, Severity::Error);
}

#[test]
fn static_error_stops_before_any_program_runs() {
    let meta = META.replace("licence = \"MIT\"", "licence = \"GPL\"");
    let pack = TempPack::new("static", &[("meta.toml", meta)]);
    let mut runner = ModelRunner::new(reference_model, &[]);

    let report = run_sandbox(&pack.root(), &mut runner);

    assert!(report.has_errors());
    assert!(report.skipped().iter().any(|stage| stage == "sandbox"));
    assert_eq!(runner.calls, 0);
}
