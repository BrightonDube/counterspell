//! Shrink proposals and the shrink loop.
//!
//! This module imports only `schema` and `size`. It never sees a program,
//! an Oracle or a clock: the caller answers yes or no for each proposal.

use crate::schema::Schema;
use crate::size::{compare_size, size_of, SizeMetric};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashSet, VecDeque};

/// Whole-value moves: every integer goes to its target, then halfway to it.
#[must_use]
pub fn whole_value_moves(schema: &Schema, value: &Value) -> Vec<Value> {
    vec![
        map_ints(schema, value, &|_, min, max| target(min, max)),
        map_ints(schema, value, &|current, min, max| {
            halfway(current, min, max)
        }),
    ]
}

/// Scalar moves, in document order: each integer jumps to its target, moves
/// halfway, then steps by one. Each string character becomes the first
/// alphabet character, then the one before it.
#[must_use]
pub fn scalar_moves(schema: &Schema, value: &Value) -> Vec<Value> {
    let mut found = Vec::new();
    collect_slots(schema, value, "", &mut found);

    let mut moves = Vec::new();
    for slot in &found {
        match slot {
            Slot::Int { pointer, min, max } => {
                let Some(current) = value.pointer(pointer).and_then(Value::as_i64) else {
                    continue;
                };
                let aim = target(*min, *max);
                for candidate in [aim, halfway(current, *min, *max), step_toward(current, aim)] {
                    moves.push(replace_at(value, pointer, json!(candidate)));
                }
            }
            Slot::Text { pointer, alphabet } => {
                let Some(text) = value.pointer(pointer).and_then(Value::as_str) else {
                    continue;
                };
                let chars: Vec<char> = text.chars().collect();
                for (index, letter) in chars.iter().enumerate() {
                    let Some(position) = alphabet.iter().position(|item| item == letter) else {
                        continue;
                    };
                    let mut options = vec![alphabet[0]];
                    if position > 0 {
                        options.push(alphabet[position - 1]);
                    }
                    for option in options {
                        let mut edited = chars.clone();
                        edited[index] = option;
                        let text: String = edited.into_iter().collect();
                        moves.push(replace_at(value, pointer, json!(text)));
                    }
                }
            }
        }
    }
    moves
}

/// Structural moves: removes chunks of items from arrays, strings and matrices.
///
/// Chunk sizes run from half the length down to one. A removal that would go
/// below the schema's minimum length is never proposed.
fn structural_moves(schema: &Schema, value: &Value) -> Vec<Value> {
    match (schema, value) {
        (Schema::IntArray { min_len, .. }, Value::Array(items)) => removals(items, *min_len)
            .into_iter()
            .map(Value::Array)
            .collect(),
        (Schema::Str { min_len, .. }, Value::String(text)) => {
            let chars: Vec<char> = text.chars().collect();
            removals(&chars, *min_len)
                .into_iter()
                .map(|kept| json!(kept.into_iter().collect::<String>()))
                .collect()
        }
        (
            Schema::Matrix {
                min_rows, min_cols, ..
            },
            Value::Array(rows),
        ) => {
            let cols = rows.first().and_then(Value::as_array).map_or(0, Vec::len);
            let column_indices: Vec<usize> = (0..cols).collect();
            let without_rows = removals(rows, *min_rows).into_iter().map(Value::Array);
            let without_columns = removals(&column_indices, *min_cols)
                .into_iter()
                .map(|kept| {
                    Value::Array(rows.iter().map(|row| keep_columns(row, &kept)).collect())
                });
            without_rows.chain(without_columns).collect()
        }
        (Schema::Tuple { items }, Value::Array(values)) => items
            .iter()
            .zip(values)
            .enumerate()
            .flat_map(|(index, (item, item_value))| {
                structural_moves(item, item_value)
                    .into_iter()
                    .map(move |replacement| {
                        let mut edited = values.clone();
                        if let Some(slot) = edited.get_mut(index) {
                            *slot = replacement;
                        }
                        Value::Array(edited)
                    })
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// Every chunk removal from `items` that keeps at least `min_len` items.
fn removals<T: Clone>(items: &[T], min_len: usize) -> Vec<Vec<T>> {
    let mut proposals = Vec::new();
    let mut size = items.len().div_ceil(2);
    while size >= 1 {
        for start in (0..items.len()).step_by(size) {
            let end = (start + size).min(items.len());
            if items.len() - (end - start) < min_len {
                continue;
            }
            let kept: Vec<T> = items[..start]
                .iter()
                .chain(&items[end..])
                .cloned()
                .collect();
            proposals.push(kept);
        }
        if size == 1 {
            break;
        }
        size = size.div_ceil(2);
    }
    proposals
}

/// Keeps only the cells of one matrix row at the listed column indices.
fn keep_columns(row: &Value, kept: &[usize]) -> Value {
    match row.as_array() {
        Some(cells) => Value::Array(
            kept.iter()
                .filter_map(|index| cells.get(*index).cloned())
                .collect(),
        ),
        None => row.clone(),
    }
}

/// Every proposal for `value`, in priority order: whole-value moves, then
/// structural removals, then scalar moves. Each pass is built when first asked.
///
/// Stub: yields nothing until the green step.
pub fn proposals<'a>(schema: &'a Schema, value: &'a Value) -> impl Iterator<Item = Value> + 'a {
    std::iter::once(()).flat_map(move |()| {
        whole_value_moves(schema, value)
            .into_iter()
            .chain(structural_moves(schema, value))
            .chain(scalar_moves(schema, value))
    })
}

/// The verdict a caller gives for one proposal.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Probe {
    /// The proposal still fails in the same way, so keep it.
    Keep,
    /// The proposal does not fail in the same way.
    Reject,
    /// Stop shrinking now (for example, the time budget ran out).
    Stop,
}

/// Why the shrink loop stopped.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ShrinkStop {
    /// A full pass over every proposal found nothing smaller that fails.
    LocalMinimum,
    /// The evaluation budget ran out.
    EvalBudget,
    /// The caller asked the loop to stop.
    WallTime,
    /// No shrinking was attempted (used for timeout ghosts).
    NotAttempted,
}

/// One recorded entry in the shrink trace.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShrinkStep {
    /// Position in the full sequence. Zero is the first failing input.
    pub index: u32,
    /// The input at this step.
    pub input: Value,
    /// The size metric of `input`.
    pub size: SizeMetric,
    /// Evaluations spent when this step was recorded.
    pub evaluations: u32,
}

/// The bounded trace of a shrink run.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct ShrinkReport {
    /// The first 10 and last 10 entries, in order.
    pub steps: Vec<ShrinkStep>,
    /// The number of accepted reductions. Entries recorded equal this plus one.
    pub total_steps: u32,
    /// Entries between the head and the tail that were not kept.
    pub omitted_steps: u32,
    /// Probes spent in total.
    pub evaluations: u32,
    /// Why the loop stopped.
    pub stop: ShrinkStop,
}

/// The shrunk input and its trace.
#[derive(Clone, Debug, PartialEq)]
pub struct ShrinkOutcome {
    /// The smallest input the loop reached.
    pub value: Value,
    /// The trace of accepted steps.
    pub report: ShrinkReport,
}

/// How many trace entries are kept at the start of a run.
const TRACE_HEAD: usize = 10;
/// How many trace entries are kept at the end of a run.
const TRACE_TAIL: usize = 10;

/// Shrinks `start` by repeatedly accepting smaller valid proposals that the
/// probe keeps.
///
/// The loop is first-improvement: after each accepted proposal it starts
/// again from the top. A value is asked about at most once, and only valid,
/// strictly smaller proposals are asked about.
pub fn shrink(
    schema: &Schema,
    start: Value,
    max_evals: u32,
    probe: &mut dyn FnMut(&Value) -> Probe,
) -> ShrinkOutcome {
    let mut recorder = TraceRecorder::default();
    let mut tried: HashSet<String> = HashSet::new();
    tried.insert(start.to_string());
    let mut evaluations: u32 = 0;
    let mut current = start;
    recorder.record(&current, evaluations);

    loop {
        match find_accepted(
            schema,
            &current,
            &mut tried,
            &mut evaluations,
            max_evals,
            probe,
        ) {
            Search::Accepted(next) => {
                current = next;
                recorder.record(&current, evaluations);
            }
            Search::Stopped(stop) => {
                return ShrinkOutcome {
                    value: current,
                    report: recorder.into_report(evaluations, stop),
                };
            }
        }
    }
}

/// The result of one pass over the proposals for the current value.
enum Search {
    /// The probe kept this proposal.
    Accepted(Value),
    /// The pass ended without a kept proposal, or was cut short.
    Stopped(ShrinkStop),
}

/// Asks the probe about each new, valid, smaller proposal until one is kept.
fn find_accepted(
    schema: &Schema,
    current: &Value,
    tried: &mut HashSet<String>,
    evaluations: &mut u32,
    max_evals: u32,
    probe: &mut dyn FnMut(&Value) -> Probe,
) -> Search {
    for proposal in proposals(schema, current) {
        if schema.contains(&proposal).is_err() {
            continue;
        }
        if compare_size(&proposal, current) != std::cmp::Ordering::Less {
            continue;
        }
        if !tried.insert(proposal.to_string()) {
            continue;
        }
        if *evaluations >= max_evals {
            return Search::Stopped(ShrinkStop::EvalBudget);
        }
        *evaluations += 1;
        match probe(&proposal) {
            Probe::Keep => return Search::Accepted(proposal),
            Probe::Reject => {}
            Probe::Stop => return Search::Stopped(ShrinkStop::WallTime),
        }
    }
    Search::Stopped(ShrinkStop::LocalMinimum)
}

/// Keeps the first and last few trace entries and counts the rest.
#[derive(Default)]
struct TraceRecorder {
    head: Vec<ShrinkStep>,
    tail: VecDeque<ShrinkStep>,
    recorded: u32,
}

impl TraceRecorder {
    fn record(&mut self, value: &Value, evaluations: u32) {
        let step = ShrinkStep {
            index: self.recorded,
            input: value.clone(),
            size: size_of(value),
            evaluations,
        };
        self.recorded = self.recorded.saturating_add(1);
        if self.head.len() < TRACE_HEAD {
            self.head.push(step);
        } else {
            self.tail.push_back(step);
            if self.tail.len() > TRACE_TAIL {
                self.tail.pop_front();
            }
        }
    }

    fn into_report(self, evaluations: u32, stop: ShrinkStop) -> ShrinkReport {
        let kept = u32::try_from(self.head.len() + self.tail.len()).unwrap_or(u32::MAX);
        let omitted_steps = self.recorded.saturating_sub(kept);
        let total_steps = self.recorded.saturating_sub(1);
        let mut steps = self.head;
        steps.extend(self.tail);
        ShrinkReport {
            steps,
            total_steps,
            omitted_steps,
            evaluations,
            stop,
        }
    }
}

/// A position in a value that a scalar move may change.
enum Slot {
    /// An integer at a JSON Pointer, with the bounds from its schema.
    Int { pointer: String, min: i64, max: i64 },
    /// A string at a JSON Pointer, with its alphabet in order.
    Text {
        pointer: String,
        alphabet: Vec<char>,
    },
}

/// Finds every scalar position of `value` by walking it alongside `schema`.
fn collect_slots(schema: &Schema, value: &Value, pointer: &str, found: &mut Vec<Slot>) {
    match (schema, value) {
        (Schema::Int { min, max }, Value::Number(_)) => {
            found.push(int_slot(pointer, *min, *max));
        }
        (Schema::Str { alphabet, .. }, Value::String(_)) => found.push(Slot::Text {
            pointer: pointer.to_owned(),
            alphabet: alphabet.chars().collect(),
        }),
        (Schema::IntArray { min, max, .. }, Value::Array(items)) => {
            for index in 0..items.len() {
                found.push(int_slot(&child(pointer, index), *min, *max));
            }
        }
        (Schema::Matrix { min, max, .. }, Value::Array(rows)) => {
            for (row_index, row) in rows.iter().enumerate() {
                let Value::Array(cells) = row else {
                    continue;
                };
                for column_index in 0..cells.len() {
                    let cell_pointer = child(&child(pointer, row_index), column_index);
                    found.push(int_slot(&cell_pointer, *min, *max));
                }
            }
        }
        (Schema::Tuple { items }, Value::Array(values)) => {
            for (index, (item, item_value)) in items.iter().zip(values).enumerate() {
                collect_slots(item, item_value, &child(pointer, index), found);
            }
        }
        _ => {}
    }
}

/// Builds an integer slot. Non-integer items are recorded too, and moves skip them.
fn int_slot(pointer: &str, min: i64, max: i64) -> Slot {
    Slot::Int {
        pointer: pointer.to_owned(),
        min,
        max,
    }
}

/// Appends one array index to a JSON Pointer.
fn child(pointer: &str, index: usize) -> String {
    format!("{pointer}/{index}")
}

/// Returns a copy of `value` with the item at `pointer` replaced.
fn replace_at(value: &Value, pointer: &str, replacement: Value) -> Value {
    let mut edited = value.clone();
    if let Some(slot) = edited.pointer_mut(pointer) {
        *slot = replacement;
    }
    edited
}

/// Applies `change` to every integer in `value`, following `schema`.
fn map_ints(schema: &Schema, value: &Value, change: &dyn Fn(i64, i64, i64) -> i64) -> Value {
    match (schema, value) {
        (Schema::Int { min, max }, Value::Number(_)) => map_int(value, *min, *max, change),
        (Schema::IntArray { min, max, .. }, Value::Array(items)) => Value::Array(
            items
                .iter()
                .map(|item| map_int(item, *min, *max, change))
                .collect(),
        ),
        (Schema::Matrix { min, max, .. }, Value::Array(rows)) => Value::Array(
            rows.iter()
                .map(|row| match row {
                    Value::Array(cells) => Value::Array(
                        cells
                            .iter()
                            .map(|cell| map_int(cell, *min, *max, change))
                            .collect(),
                    ),
                    other => other.clone(),
                })
                .collect(),
        ),
        (Schema::Tuple { items }, Value::Array(values)) => Value::Array(
            items
                .iter()
                .zip(values)
                .map(|(item, item_value)| map_ints(item, item_value, change))
                .collect(),
        ),
        _ => value.clone(),
    }
}

fn map_int(value: &Value, min: i64, max: i64, change: &dyn Fn(i64, i64, i64) -> i64) -> Value {
    value
        .as_i64()
        .map_or_else(|| value.clone(), |current| json!(change(current, min, max)))
}

/// The simplest integer in range: zero, or the nearest bound when zero is outside.
fn target(min: i64, max: i64) -> i64 {
    0_i64.clamp(min, max)
}

/// The integer halfway between `current` and its target, rounding toward the target.
fn halfway(current: i64, min: i64, max: i64) -> i64 {
    let aim = target(min, max);
    aim.saturating_add(current.saturating_sub(aim) / 2)
}

/// The integer one step from `current` toward `aim`.
fn step_toward(current: i64, aim: i64) -> i64 {
    match current.cmp(&aim) {
        std::cmp::Ordering::Greater => current - 1,
        std::cmp::Ordering::Less => current + 1,
        std::cmp::Ordering::Equal => current,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::tests::strategies::arb_valid_value;
    use crate::size::size_of;
    use proptest::prelude::*;
    use serde_json::json;

    fn coupled_schema() -> Schema {
        Schema::Tuple {
            items: vec![
                Schema::IntArray {
                    min_len: 0,
                    max_len: 2,
                    min: -10,
                    max: 10,
                    sorted: false,
                    unique: false,
                },
                Schema::Int { min: -10, max: 10 },
            ],
        }
    }

    #[test]
    fn shrink_whole_moves_reach_coupled_zero() {
        let moves = whole_value_moves(&coupled_schema(), &json!([[5], 10]));
        assert_eq!(moves, vec![json!([[0], 0]), json!([[2], 5])]);
    }

    #[test]
    fn shrink_targets_respect_ranges_without_zero() {
        let schema = Schema::Int { min: 3, max: 9 };
        assert_eq!(
            scalar_moves(&schema, &json!(7)),
            vec![json!(3), json!(5), json!(6)],
        );
        assert_eq!(
            whole_value_moves(&schema, &json!(7)),
            vec![json!(3), json!(5)],
        );
    }

    #[test]
    fn shrink_string_moves_use_alphabet_order() {
        let schema = Schema::Str {
            alphabet: "abc".to_owned(),
            min_len: 0,
            max_len: 5,
        };
        assert_eq!(
            scalar_moves(&schema, &json!("cb")),
            vec![json!("ab"), json!("bb"), json!("ca"), json!("ca")],
        );
    }

    #[test]
    fn shrink_removal_chunks_run_from_half_down_to_one() {
        let schema = Schema::IntArray {
            min_len: 0,
            max_len: 8,
            min: 0,
            max: 9,
            sorted: false,
            unique: false,
        };
        let moves = structural_moves(&schema, &json!([0, 1, 2, 3, 4, 5, 6, 7]));
        assert_eq!(moves.len(), 14);
        assert_eq!(
            moves[..6],
            [
                json!([4, 5, 6, 7]),
                json!([0, 1, 2, 3]),
                json!([2, 3, 4, 5, 6, 7]),
                json!([0, 1, 4, 5, 6, 7]),
                json!([0, 1, 2, 3, 6, 7]),
                json!([0, 1, 2, 3, 4, 5]),
            ]
        );
    }

    #[test]
    fn shrink_removal_respects_minimum_length() {
        let schema = Schema::IntArray {
            min_len: 6,
            max_len: 8,
            min: 0,
            max: 9,
            sorted: false,
            unique: false,
        };
        let moves = structural_moves(&schema, &json!([0, 1, 2, 3, 4, 5, 6, 7]));
        assert_eq!(moves.len(), 12);
        assert!(moves
            .iter()
            .all(|value| value.as_array().map_or(0, Vec::len) >= 6));
    }

    #[test]
    fn shrink_matrix_column_removal_drops_the_same_index_everywhere() {
        let schema = Schema::Matrix {
            min_rows: 0,
            max_rows: 3,
            min_cols: 0,
            max_cols: 3,
            min: -9,
            max: 9,
        };
        let moves = structural_moves(&schema, &json!([[1, 2, 3], [4, 5, 6]]));
        assert_eq!(
            moves,
            vec![
                json!([[4, 5, 6]]),
                json!([[1, 2, 3]]),
                json!([[3], [6]]),
                json!([[1, 2], [4, 5]]),
                json!([[2, 3], [5, 6]]),
                json!([[1, 3], [4, 6]]),
                json!([[1, 2], [4, 5]]),
            ]
        );
    }

    #[test]
    fn shrink_tuple_keeps_its_arity() {
        let schema = Schema::Tuple {
            items: vec![
                Schema::IntArray {
                    min_len: 0,
                    max_len: 3,
                    min: 0,
                    max: 9,
                    sorted: false,
                    unique: false,
                },
                Schema::Int { min: 0, max: 9 },
            ],
        };
        assert_eq!(
            structural_moves(&schema, &json!([[1, 2], 5])),
            vec![json!([[2], 5]), json!([[1], 5])],
        );
    }

    #[test]
    fn shrink_proposals_run_whole_then_structural_then_scalar() {
        let schema = Schema::Int { min: 3, max: 9 };
        let found: Vec<Value> = proposals(&schema, &json!(7)).collect();
        assert_eq!(
            found,
            vec![json!(3), json!(5), json!(3), json!(5), json!(6)]
        );
    }

    #[test]
    fn shrink_loop_reaches_a_local_minimum_on_a_threshold() {
        let schema = Schema::Int { min: 0, max: 100 };
        let mut probe = |value: &Value| {
            if value.as_i64().is_some_and(|number| number >= 5) {
                Probe::Keep
            } else {
                Probe::Reject
            }
        };
        let outcome = shrink(&schema, json!(37), 300, &mut probe);
        assert_eq!(outcome.value, json!(5));
        assert_eq!(outcome.report.stop, ShrinkStop::LocalMinimum);
        assert_eq!(outcome.report.total_steps, 6);
        let trail: Vec<Value> = outcome
            .report
            .steps
            .iter()
            .map(|step| step.input.clone())
            .collect();
        assert_eq!(
            trail,
            vec![
                json!(37),
                json!(18),
                json!(9),
                json!(8),
                json!(7),
                json!(6),
                json!(5)
            ]
        );
    }

    #[test]
    fn shrink_stops_at_the_evaluation_budget() {
        let schema = Schema::Int { min: 0, max: 100 };
        let mut probe = |value: &Value| {
            if value.as_i64().is_some_and(|number| number >= 5) {
                Probe::Keep
            } else {
                Probe::Reject
            }
        };
        let outcome = shrink(&schema, json!(37), 2, &mut probe);
        assert_eq!(outcome.report.stop, ShrinkStop::EvalBudget);
        assert_eq!(outcome.report.evaluations, 2);
        assert_eq!(outcome.value, json!(18));
    }

    #[test]
    fn shrink_returns_when_the_probe_says_stop() {
        let schema = Schema::Int { min: 0, max: 100 };
        let mut probe = |_: &Value| Probe::Stop;
        let outcome = shrink(&schema, json!(37), 300, &mut probe);
        assert_eq!(outcome.report.stop, ShrinkStop::WallTime);
        assert_eq!(outcome.report.evaluations, 1);
        assert_eq!(outcome.value, json!(37));
    }

    #[test]
    fn shrink_never_asks_about_the_same_value_twice() {
        let schema = Schema::Int { min: 0, max: 100 };
        let mut asked: Vec<String> = Vec::new();
        let mut probe = |value: &Value| {
            asked.push(value.to_string());
            if value.as_i64().is_some_and(|number| number >= 5) {
                Probe::Keep
            } else {
                Probe::Reject
            }
        };
        let _ = shrink(&schema, json!(37), 300, &mut probe);
        let unique: HashSet<&String> = asked.iter().collect();
        assert_eq!(unique.len(), asked.len());
    }

    /// A pure, seeded predicate that rejects about a quarter of all values.
    fn seeded_probe(seed: u64) -> impl FnMut(&Value) -> Probe {
        move |value: &Value| {
            if mix(&value.to_string(), seed) % 4 == 0 {
                Probe::Reject
            } else {
                Probe::Keep
            }
        }
    }

    /// A small FNV-style hash used only to make test predicates deterministic.
    fn mix(text: &str, seed: u64) -> u64 {
        text.bytes()
            .fold(seed ^ 0xcbf2_9ce4_8422_2325, |hash, byte| {
                (hash ^ u64::from(byte)).wrapping_mul(0x0000_0100_0000_01b3)
            })
    }

    proptest! {
        #[test]
        fn shrink_p4_proposals_valid_and_smaller(
            pair in arb_valid_value(),
            seed in any::<u64>(),
        ) {
            let (schema, start) = pair;
            let mut probe_inner = seeded_probe(seed);
            let mut seen = Vec::new();
            let mut probe = |value: &Value| {
                seen.push(value.clone());
                probe_inner(value)
            };
            let _ = shrink(&schema, start.clone(), 300, &mut probe);
            for value in &seen {
                prop_assert!(schema.contains(value).is_ok(), "invalid probe {}", value);
                prop_assert_eq!(compare_size(value, &start), std::cmp::Ordering::Less);
            }
        }

        #[test]
        fn shrink_p5_accepted_steps_strictly_decrease(
            pair in arb_valid_value(),
            seed in any::<u64>(),
        ) {
            let (schema, start) = pair;
            let mut probe = seeded_probe(seed);
            let outcome = shrink(&schema, start.clone(), 300, &mut probe);
            let steps = &outcome.report.steps;
            prop_assert!(!steps.is_empty(), "no trace for {}", start);
            prop_assert_eq!(&steps[0].input, &start);
            for pair in steps.windows(2) {
                prop_assert_eq!(compare_size(&pair[1].input, &pair[0].input), std::cmp::Ordering::Less);
            }
        }

        #[test]
        fn shrink_p7_local_minimum_has_no_smaller_kept_proposal(
            pair in arb_valid_value(),
            seed in any::<u64>(),
        ) {
            let (schema, start) = pair;
            let mut probe = seeded_probe(seed);
            let outcome = shrink(&schema, start, 300, &mut probe);
            if outcome.report.stop == ShrinkStop::LocalMinimum {
                let mut check = seeded_probe(seed);
                for proposal in proposals(&schema, &outcome.value) {
                    if schema.contains(&proposal).is_ok()
                        && compare_size(&proposal, &outcome.value) == std::cmp::Ordering::Less
                    {
                        prop_assert_eq!(check(&proposal), Probe::Reject, "kept smaller {}", proposal);
                    }
                }
            }
        }

        #[test]
        fn shrink_p11_trace_is_capped_and_ends_at_the_ghost(
            pair in arb_valid_value(),
            seed in any::<u64>(),
        ) {
            let (schema, start) = pair;
            let mut probe = seeded_probe(seed);
            let outcome = shrink(&schema, start.clone(), 300, &mut probe);
            let report = &outcome.report;
            prop_assert!(report.steps.len() <= 20);
            prop_assert!(!report.steps.is_empty());
            prop_assert_eq!(&report.steps[0].input, &start);
            prop_assert_eq!(&report.steps[report.steps.len() - 1].input, &outcome.value);
            let entries = u32::try_from(report.steps.len()).unwrap_or(u32::MAX);
            prop_assert_eq!(report.total_steps + 1, entries + report.omitted_steps);
            prop_assert!(report.steps.windows(2).all(|pair| pair[0].index < pair[1].index));
        }
    }

    proptest! {
        #[test]
        fn shrink_proposals_keep_the_shape(pair in arb_valid_value()) {
            let (schema, value) = pair;
            let nodes = size_of(&value).nodes;
            let whole = whole_value_moves(&schema, &value);
            prop_assert!(!whole.is_empty(), "no whole-value moves for {}", value);
            for proposal in whole
                .into_iter()
                .chain(scalar_moves(&schema, &value))
            {
                prop_assert_eq!(size_of(&proposal).nodes, nodes, "shape changed to {}", proposal);
            }
        }
    }
}
