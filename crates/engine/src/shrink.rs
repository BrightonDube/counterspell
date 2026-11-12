//! Shrink proposals and the shrink loop.
//!
//! This module imports only `schema` and `size`. It never sees a program,
//! an Oracle or a clock: the caller answers yes or no for each proposal.

use crate::schema::Schema;
use serde_json::{json, Value};

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
