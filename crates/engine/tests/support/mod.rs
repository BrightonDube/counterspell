//! Shared fixtures for the integration tests.
//!
//! No test in this crate runs the Python files in `packs/_test/`. Each file
//! is mirrored by a Rust model function here, and the hand-traced vectors in
//! `ghost.json` and `golden.json` tie those models to the Python source. The
//! test binaries use different parts of this module, so unused items are
//! allowed.

#![allow(dead_code)]

use counterspell_engine::{
    Comparator, FailureKind, KnownGhost, ManualClock, Observation, Oracle, OracleError,
    ProgramResult, Schema,
};
use serde_json::Value;
use std::path::PathBuf;
use std::time::Duration;

/// The three launch fixture packs, in a fixed order.
pub const PACKS: [&str; 3] = ["pair-count", "longest-run", "diagonal-sum"];

/// A Rust model of one program. It receives the argument list (the input) and
/// returns the raw result the program would produce.
pub type Model = fn(&Value) -> ProgramResult;

/// What a vector expects: a returned JSON value, or a crash with an exception class.
#[derive(Clone, Debug, PartialEq)]
pub enum Expected {
    /// The program returns this value.
    Returned(Value),
    /// The program raises this exception class.
    Crashed(String),
}

/// One (input, expectation) pair from a pack file.
#[derive(Clone, Debug, PartialEq)]
pub struct Vector {
    /// The argument list.
    pub input: Value,
    /// The expected result.
    pub expected: Expected,
}

/// One incorrect program listed in a pack's `ghost.json`.
#[derive(Clone, Debug)]
pub struct Incorrect {
    /// The id used by [`incorrect`].
    pub id: String,
    /// The Python file, relative to the pack directory.
    pub file: String,
    /// The failure kind the program shows.
    pub kind: FailureKind,
    /// The exception class, for a crash.
    pub exception_class: Option<String>,
    /// The smallest failing input, as a regression expectation for the shrinker.
    pub smallest_input: Value,
    /// The node count of `smallest_input` under the engine's size metric.
    pub smallest_nodes: u64,
    /// The magnitude of `smallest_input` under the engine's size metric.
    pub smallest_magnitude: u128,
    /// Whether the program is the dogfooding demo candidate.
    pub dogfood: bool,
    /// Hand-traced vectors for the program.
    pub vectors: Vec<Vector>,
}

/// Everything the tests need from one pack directory.
#[derive(Clone, Debug)]
pub struct Pack {
    /// The pack slug, which is also its directory name.
    pub slug: String,
    /// The problem version recorded in `ghost.json`.
    pub problem_version: u32,
    /// The input schema.
    pub schema: Schema,
    /// The output comparator.
    pub comparator: Comparator,
    /// Concept tags copied into autopsies.
    pub concept_tags: Vec<String>,
    /// Public samples, with the reference's expected values.
    pub samples: Vec<Vector>,
    /// Authored known ghosts.
    pub known_ghosts: Vec<KnownGhost>,
    /// The incorrect programs.
    pub incorrect: Vec<Incorrect>,
    /// Hand-verified reference vectors from `golden.json`.
    pub golden: Vec<Vector>,
}

/// The `packs/_test` directory, found from this crate's manifest.
pub fn packs_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../packs/_test")
}

fn read_json(path: &std::path::Path) -> Value {
    let text = std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
    serde_json::from_str(&text)
        .unwrap_or_else(|error| panic!("invalid JSON in {}: {error}", path.display()))
}

fn parse_vector(entry: &Value) -> Vector {
    let input = entry["input"].clone();
    let expected = match entry.get("expected").or_else(|| entry.get("result")) {
        Some(value) => Expected::Returned(value.clone()),
        None => Expected::Crashed(
            entry["crash"]
                .as_str()
                .expect("a vector has either a result or a crash class")
                .to_owned(),
        ),
    };
    Vector { input, expected }
}

fn parse_vectors(list: &Value) -> Vec<Vector> {
    list.as_array()
        .expect("vector list is an array")
        .iter()
        .map(parse_vector)
        .collect()
}

/// Loads one pack and checks that its schema is a valid definition.
pub fn load_pack(slug: &str) -> Pack {
    let dir = packs_root().join(slug);
    let ghost = read_json(&dir.join("ghost.json"));
    let golden = read_json(&dir.join("golden.json"));

    let schema: Schema = serde_json::from_value(ghost["schema"].clone())
        .unwrap_or_else(|error| panic!("{slug}: bad schema: {error}"));
    schema
        .check_definition()
        .unwrap_or_else(|error| panic!("{slug}: schema is not a valid definition: {error}"));
    let comparator: Comparator = serde_json::from_value(ghost["comparator"].clone())
        .unwrap_or_else(|error| panic!("{slug}: bad comparator: {error}"));
    let known_ghosts: Vec<KnownGhost> = serde_json::from_value(ghost["known_ghosts"].clone())
        .unwrap_or_else(|error| panic!("{slug}: bad known ghosts: {error}"));

    let incorrect = ghost["incorrect"]
        .as_array()
        .expect("incorrect is an array")
        .iter()
        .map(|entry| Incorrect {
            id: entry["id"].as_str().expect("incorrect id").to_owned(),
            file: entry["file"].as_str().expect("incorrect file").to_owned(),
            kind: serde_json::from_value(entry["kind"].clone()).expect("incorrect kind"),
            exception_class: entry["exception_class"].as_str().map(str::to_owned),
            smallest_input: entry["smallest_input"].clone(),
            smallest_nodes: entry["smallest_size"]["nodes"]
                .as_u64()
                .expect("smallest_size.nodes"),
            smallest_magnitude: entry["smallest_size"]["magnitude"]
                .as_str()
                .expect("smallest_size.magnitude is a string")
                .parse()
                .expect("smallest_size.magnitude is a decimal number"),
            dogfood: entry["dogfood"].as_bool().unwrap_or(false),
            vectors: parse_vectors(&entry["vectors"]),
        })
        .collect();

    Pack {
        slug: slug.to_owned(),
        problem_version: u32::try_from(ghost["problem_version"].as_u64().unwrap_or(0))
            .expect("problem version fits in u32"),
        schema,
        comparator,
        concept_tags: ghost["concept_tags"]
            .as_array()
            .expect("concept tags")
            .iter()
            .filter_map(|tag| tag.as_str().map(str::to_owned))
            .collect(),
        samples: parse_vectors(&ghost["samples"]),
        known_ghosts,
        incorrect,
        golden: parse_vectors(&golden["vectors"]),
    }
}

fn returned(value: i64) -> ProgramResult {
    ProgramResult::Returned(Value::from(value))
}

fn crashed(exception_class: &str, message: &str) -> ProgramResult {
    ProgramResult::Crashed {
        exception_class: exception_class.to_owned(),
        message: message.to_owned(),
    }
}

fn ints(value: &Value) -> Vec<i64> {
    value
        .as_array()
        .expect("an array of integers")
        .iter()
        .map(|item| item.as_i64().expect("an integer"))
        .collect()
}

fn grid(value: &Value) -> Vec<Vec<i64>> {
    value
        .as_array()
        .expect("a grid is an array of rows")
        .iter()
        .map(ints)
        .collect()
}

/// Counts pairs `i < j` (when `first_offset` is 1) or `i <= j` (when it is 0)
/// whose values add up to `target`, mirroring the Python loops.
fn count_pairs(bones: &[i64], target: i64, first_offset: usize) -> i64 {
    let mut count = 0;
    for i in 0..bones.len() {
        for j in (i + first_offset)..bones.len() {
            if i128::from(bones[i]) + i128::from(bones[j]) == i128::from(target) {
                count += 1;
            }
        }
    }
    count
}

fn pair_args(args: &Value) -> (Vec<i64>, i64) {
    (
        ints(&args[0]),
        args[1].as_i64().expect("target is an integer"),
    )
}

fn pair_count_reference(args: &Value) -> ProgramResult {
    let (bones, target) = pair_args(args);
    returned(count_pairs(&bones, target, 1))
}

fn pair_count_same_index(args: &Value) -> ProgramResult {
    let (bones, target) = pair_args(args);
    returned(count_pairs(&bones, target, 0))
}

fn pair_count_empty_min(args: &Value) -> ProgramResult {
    let (bones, target) = pair_args(args);
    // Mirrors `target < 2 * min(bones)`: min() of an empty list raises ValueError.
    let Some(&smallest) = bones.iter().min() else {
        return crashed("ValueError", "min() arg is an empty sequence");
    };
    if i128::from(target) < 2 * i128::from(smallest) {
        return returned(0);
    }
    returned(count_pairs(&bones, target, 1))
}

fn longest_run_chars(args: &Value) -> Vec<char> {
    args[0]
        .as_str()
        .expect("the input is a string")
        .chars()
        .collect()
}

fn longest_run_reference(args: &Value) -> ProgramResult {
    let (mut best, mut cur, mut prev) = (0_i64, 0_i64, None);
    for ch in longest_run_chars(args) {
        cur = if Some(ch) == prev { cur + 1 } else { 1 };
        best = best.max(cur);
        prev = Some(ch);
    }
    returned(best)
}

fn longest_run_forgot_last(args: &Value) -> ProgramResult {
    // Updates `best` only when a run ends, so a final run is never counted.
    let (mut best, mut cur, mut prev) = (0_i64, 0_i64, None);
    for ch in longest_run_chars(args) {
        if Some(ch) == prev {
            cur += 1;
        } else {
            best = best.max(cur);
            cur = 1;
        }
        prev = Some(ch);
    }
    returned(best)
}

fn longest_run_empty_first_char(args: &Value) -> ProgramResult {
    let text = longest_run_chars(args);
    let Some(&first) = text.first() else {
        return crashed("IndexError", "string index out of range");
    };
    let (mut best, mut cur, mut prev) = (1_i64, 1_i64, first);
    for &ch in &text[1..] {
        cur = if ch == prev { cur + 1 } else { 1 };
        best = best.max(cur);
        prev = ch;
    }
    returned(best)
}

/// Reads `grid[i][i]`, raising `IndexError` when the row is too short or missing.
fn diagonal_cell(cells: &[Vec<i64>], i: usize) -> Result<i64, ProgramResult> {
    cells
        .get(i)
        .and_then(|row| row.get(i))
        .copied()
        .ok_or_else(|| crashed("IndexError", "list index out of range"))
}

fn diagonal_args(args: &Value) -> Vec<Vec<i64>> {
    grid(&args[0])
}

fn diagonal_reference(args: &Value) -> ProgramResult {
    let cells = diagonal_args(args);
    let rows = cells.len();
    let cols = cells.first().map_or(0, Vec::len);
    let mut total = 0_i64;
    for i in 0..rows.min(cols) {
        match diagonal_cell(&cells, i) {
            Ok(value) => total += value,
            Err(failure) => return failure,
        }
    }
    returned(total)
}

fn diagonal_abs(args: &Value) -> ProgramResult {
    // Adds the absolute value of each diagonal cell instead of the cell.
    let cells = diagonal_args(args);
    let rows = cells.len();
    let cols = cells.first().map_or(0, Vec::len);
    let mut total = 0_i64;
    for i in 0..rows.min(cols) {
        match diagonal_cell(&cells, i) {
            Ok(value) => total += value.abs(),
            Err(failure) => return failure,
        }
    }
    returned(total)
}

fn diagonal_assumes_square(args: &Value) -> ProgramResult {
    // Loops over the row count only, so more rows than columns reads past a row.
    let cells = diagonal_args(args);
    let mut total = 0_i64;
    for i in 0..cells.len() {
        match diagonal_cell(&cells, i) {
            Ok(value) => total += value,
            Err(failure) => return failure,
        }
    }
    returned(total)
}

/// The Rust model of a pack's trusted reference (its Python `reference/solution.py`).
///
/// # Panics
///
/// Panics for an unknown slug, which is a bug in the test itself.
pub fn reference(slug: &str) -> Model {
    match slug {
        "pair-count" => pair_count_reference,
        "longest-run" => longest_run_reference,
        "diagonal-sum" => diagonal_reference,
        other => panic!("no reference model for pack {other}"),
    }
}

/// The Rust model of one incorrect program, named by pack slug and program id.
///
/// # Panics
///
/// Panics for an unknown pair, which is a bug in the test itself.
pub fn incorrect(slug: &str, id: &str) -> Model {
    match (slug, id) {
        ("pair-count", "same-index") => pair_count_same_index,
        ("pair-count", "empty-min") => pair_count_empty_min,
        ("longest-run", "forgot-last-run") => longest_run_forgot_last,
        ("longest-run", "empty-first-char") => longest_run_empty_first_char,
        ("diagonal-sum", "abs-diagonal") => diagonal_abs,
        ("diagonal-sum", "assumes-square") => diagonal_assumes_square,
        other => panic!("no incorrect model for {other:?}"),
    }
}

/// An Oracle that runs two Rust models and charges a fixed cost on a manual clock.
///
/// If the cost is above the caller's limit, the user's result is `TimedOut`.
pub struct ModelOracle<'a> {
    reference: Model,
    user: Model,
    clock: &'a ManualClock,
    cost: Duration,
    calls: u32,
}

impl<'a> ModelOracle<'a> {
    /// Creates an Oracle that charges `cost` on `clock` for each call.
    pub fn new(reference: Model, user: Model, clock: &'a ManualClock, cost: Duration) -> Self {
        Self {
            reference,
            user,
            clock,
            cost,
            calls: 0,
        }
    }

    /// The number of evaluations this Oracle has served.
    pub fn calls(&self) -> u32 {
        self.calls
    }
}

impl Oracle for ModelOracle<'_> {
    fn evaluate(&mut self, input: &Value, limit: Duration) -> Result<Observation, OracleError> {
        self.calls += 1;
        let reference = (self.reference)(input);
        if self.cost > limit {
            return Ok(Observation {
                user: ProgramResult::TimedOut,
                reference,
            });
        }
        self.clock.advance(self.cost);
        Ok(Observation {
            user: (self.user)(input),
            reference,
        })
    }
}

/// A bug family for the synthetic oracle. The reference counts JSON nodes, and
/// each family decides when the user's program is wrong or crashes.
#[derive(Clone, Debug, PartialEq)]
pub enum Bug {
    /// The user is always wrong.
    Always,
    /// The user is never wrong.
    Never,
    /// The user is wrong when the input has at least this many JSON nodes.
    WrongWhenLarge(usize),
    /// The user raises `ValueError` when the input has at least this many JSON nodes.
    CrashWhenLarge(usize),
    /// The user is wrong when the input contains this integer anywhere.
    WrongWhenContains(i64),
    /// The reference itself raises on every input.
    ReferenceCrashes,
}

/// Counts the JSON nodes in a value. Scalars and containers each count as one.
pub fn node_count(value: &Value) -> usize {
    match value {
        Value::Array(items) => 1 + items.iter().map(node_count).sum::<usize>(),
        Value::Object(entries) => 1 + entries.values().map(node_count).sum::<usize>(),
        _ => 1,
    }
}

/// Reports whether the integer `wanted` appears anywhere in a JSON value.
fn contains_integer(value: &Value, wanted: i64) -> bool {
    match value {
        Value::Number(number) => number.as_i64() == Some(wanted),
        Value::Array(items) => items.iter().any(|item| contains_integer(item, wanted)),
        Value::Object(entries) => entries.values().any(|item| contains_integer(item, wanted)),
        _ => false,
    }
}

/// An Oracle whose reference returns the node count and whose user program
/// follows a [`Bug`]. Each call charges a fixed cost on a manual clock, and a
/// cost above the caller's limit makes the user's result `TimedOut`.
pub struct SyntheticOracle<'a> {
    bug: Bug,
    clock: &'a ManualClock,
    cost: Duration,
    calls: u32,
}

impl<'a> SyntheticOracle<'a> {
    /// Creates an Oracle that follows `bug` and charges `cost` on `clock` per call.
    pub fn new(bug: Bug, clock: &'a ManualClock, cost: Duration) -> Self {
        Self {
            bug,
            clock,
            cost,
            calls: 0,
        }
    }

    /// The number of evaluations this Oracle has served.
    pub fn calls(&self) -> u32 {
        self.calls
    }
}

impl Oracle for SyntheticOracle<'_> {
    fn evaluate(&mut self, input: &Value, limit: Duration) -> Result<Observation, OracleError> {
        self.calls += 1;
        let nodes = node_count(input);
        let expected = i64::try_from(nodes).unwrap_or(i64::MAX);
        let reference = match self.bug {
            Bug::ReferenceCrashes => crashed("IndexError", "the reference raised"),
            _ => returned(expected),
        };
        if self.cost > limit {
            return Ok(Observation {
                user: ProgramResult::TimedOut,
                reference,
            });
        }
        self.clock.advance(self.cost);
        let user = match self.bug {
            Bug::Always => returned(expected + 1),
            Bug::WrongWhenLarge(threshold) if nodes >= threshold => returned(expected + 1),
            Bug::CrashWhenLarge(threshold) if nodes >= threshold => {
                crashed("ValueError", "synthetic crash")
            }
            Bug::WrongWhenContains(wanted) if contains_integer(input, wanted) => {
                returned(expected + 1)
            }
            Bug::ReferenceCrashes => returned(0),
            _ => returned(expected),
        };
        Ok(Observation { user, reference })
    }
}
