//! Seeded, edge-biased generation of candidate inputs.

use crate::rng::SplitMix64;
use crate::schema::Schema;
use serde_json::{json, Value};

/// Tunable generator defaults. Tests check properties, not these exact numbers.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GeneratorConfig {
    /// Chance, in percent, that an integer is drawn from the edge pool.
    pub edge_int_percent: u8,
    /// Chance, in percent, that a collection length is drawn from the edge lengths.
    pub edge_length_percent: u8,
    /// The largest number of JSON nodes a generated value should hold.
    pub max_units: usize,
}

impl Default for GeneratorConfig {
    fn default() -> Self {
        Self {
            edge_int_percent: 40,
            edge_length_percent: 35,
            max_units: 256,
        }
    }
}

/// Counts the JSON nodes in a value: scalars and containers each count as one.
#[must_use]
pub fn units(value: &Value) -> usize {
    match value {
        Value::Array(items) => 1 + items.iter().map(units).sum::<usize>(),
        Value::Object(fields) => 1 + fields.values().map(units).sum::<usize>(),
        _ => 1,
    }
}

/// Builds one value that follows `schema`, using only the seeded generator.
///
/// `size_cap` limits collection lengths when the schema allows it. The
/// result is not checked here; callers check it with `Schema::contains`.
pub fn generate_value(
    schema: &Schema,
    rng: &mut SplitMix64,
    size_cap: usize,
    config: &GeneratorConfig,
) -> Value {
    match schema {
        Schema::Int { min, max } => json!(int_value(rng, *min, *max, config)),
        Schema::Str {
            alphabet,
            min_len,
            max_len,
        } => {
            let chars: Vec<char> = alphabet.chars().collect();
            if chars.is_empty() {
                return json!("");
            }
            let length = pick_length(rng, *min_len, *max_len, size_cap, config);
            let text: String = (0..length)
                .map(|_| chars[index_below(rng, chars.len())])
                .collect();
            json!(text)
        }
        Schema::IntArray {
            min_len,
            max_len,
            min,
            max,
            sorted,
            unique,
        } => {
            let width = range_width(*min, *max);
            let max_len = if *unique {
                (*max_len).min(width)
            } else {
                *max_len
            };
            let length = pick_length(rng, *min_len, max_len, size_cap, config);
            let mut items: Vec<i64> = Vec::with_capacity(length);
            for index in 0..length {
                let item = if *unique {
                    fresh_int(rng, *min, *max, &items, config)
                } else if index > 0 && rng.gen_bool(20) {
                    items[index_below(rng, index)]
                } else {
                    int_value(rng, *min, *max, config)
                };
                items.push(item);
            }
            if *sorted {
                items.sort_unstable();
            }
            json!(items)
        }
        Schema::Matrix {
            min_rows,
            max_rows,
            min_cols,
            max_cols,
            min,
            max,
        } => {
            let rows = pick_length(rng, *min_rows, *max_rows, size_cap, config);
            let cols_cap = size_cap
                .checked_div(rows)
                .map_or(size_cap, |per_row| per_row.max(*min_cols));
            let cols = pick_length(rng, *min_cols, *max_cols, cols_cap, config);
            let grid: Vec<Vec<i64>> = (0..rows)
                .map(|_| {
                    (0..cols)
                        .map(|_| int_value(rng, *min, *max, config))
                        .collect()
                })
                .collect();
            json!(grid)
        }
        Schema::Tuple { items } => Value::Array(
            items
                .iter()
                .map(|item| generate_value(item, rng, size_cap, config))
                .collect(),
        ),
    }
}

/// Picks an integer in `min..=max`, favouring edge values when configured.
fn int_value(rng: &mut SplitMix64, min: i64, max: i64, config: &GeneratorConfig) -> i64 {
    if !rng.gen_bool(config.edge_int_percent) {
        return rng.gen_range(min, max);
    }
    let target = 0_i64.clamp(min, max);
    let candidates = [
        min,
        max,
        target,
        target.saturating_sub(1).clamp(min, max),
        target.saturating_add(1).clamp(min, max),
    ];
    candidates[index_below(rng, candidates.len())]
}

/// Picks an integer not already in `items`, falling back to a linear scan.
fn fresh_int(
    rng: &mut SplitMix64,
    min: i64,
    max: i64,
    items: &[i64],
    config: &GeneratorConfig,
) -> i64 {
    for _ in 0..50 {
        let candidate = int_value(rng, min, max, config);
        if !items.contains(&candidate) {
            return candidate;
        }
    }
    (min..=max)
        .find(|value| !items.contains(value))
        .unwrap_or(min)
}

/// Picks a collection length within the bounds, favouring edge lengths.
fn pick_length(
    rng: &mut SplitMix64,
    min_len: usize,
    max_len: usize,
    size_cap: usize,
    config: &GeneratorConfig,
) -> usize {
    let high = max_len.min(size_cap).max(min_len);
    if rng.gen_bool(config.edge_length_percent) {
        let edges = [
            min_len,
            (min_len + 1).min(high),
            2_usize.clamp(min_len, high),
            high,
        ];
        edges[index_below(rng, edges.len())]
    } else {
        to_usize(rng.gen_range(to_i64(min_len), to_i64(high)))
    }
}

/// Picks an index in `0..len`. Returns 0 for an empty range.
fn index_below(rng: &mut SplitMix64, len: usize) -> usize {
    to_usize(rng.gen_range(0, to_i64(len.saturating_sub(1))))
}

/// The number of integers in `min..=max`, saturating at `usize::MAX`.
fn range_width(min: i64, max: i64) -> usize {
    let width = i128::from(max) - i128::from(min) + 1;
    usize::try_from(width).unwrap_or(usize::MAX)
}

/// Converts a length to `i64`, saturating for absurdly large values.
fn to_i64(value: usize) -> i64 {
    i64::try_from(value).unwrap_or(i64::MAX)
}

/// Converts a sampled value back to a length. Negative values become zero.
fn to_usize(value: i64) -> usize {
    usize::try_from(value).unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::tests::strategies::arb_schema;
    use proptest::prelude::*;
    use serde_json::json;

    fn int_array(min_len: usize, max_len: usize, min: i64, max: i64) -> Schema {
        Schema::IntArray {
            min_len,
            max_len,
            min,
            max,
            sorted: false,
            unique: false,
        }
    }

    #[test]
    fn gen_units_counts_every_node() {
        assert_eq!(units(&json!(0)), 1);
        assert_eq!(units(&json!([0, [1, 2]])), 5);
    }

    #[test]
    fn gen_equal_seeds_give_equal_values() {
        let schema = int_array(0, 10, -5, 5);
        let config = GeneratorConfig::default();
        let mut first = SplitMix64::new(42);
        let mut second = SplitMix64::new(42);
        for _ in 0..50 {
            assert_eq!(
                generate_value(&schema, &mut first, 256, &config),
                generate_value(&schema, &mut second, 256, &config),
            );
        }
    }

    #[test]
    fn gen_edge_values_appear_over_many_draws() {
        let schema = int_array(0, 3, -5, 5);
        let config = GeneratorConfig::default();
        let mut rng = SplitMix64::new(7);
        let (mut empty, mut low, mut high, mut zero, mut duplicate) =
            (false, false, false, false, false);

        for _ in 0..500 {
            let value = generate_value(&schema, &mut rng, 256, &config);
            let items: Vec<i64> = value
                .as_array()
                .expect("array")
                .iter()
                .map(|item| item.as_i64().expect("integer"))
                .collect();
            empty |= items.is_empty();
            low |= items.contains(&-5);
            high |= items.contains(&5);
            zero |= items.contains(&0);
            duplicate |= items
                .iter()
                .enumerate()
                .any(|(index, item)| items[..index].contains(item));
        }

        assert!(empty, "no empty array in 500 draws");
        assert!(low, "minimum value never drawn");
        assert!(high, "maximum value never drawn");
        assert!(zero, "zero never drawn");
        assert!(duplicate, "no duplicate element in 500 draws");
    }

    #[test]
    fn gen_unique_sorted_arrays_are_strictly_increasing() {
        let schema = Schema::IntArray {
            min_len: 0,
            max_len: 5,
            min: -3,
            max: 3,
            sorted: true,
            unique: true,
        };
        let config = GeneratorConfig::default();
        let mut rng = SplitMix64::new(11);
        for _ in 0..200 {
            let value = generate_value(&schema, &mut rng, 256, &config);
            assert!(schema.contains(&value).is_ok(), "invalid value {value}");
        }
    }

    proptest! {
        #[test]
        fn gen_p1_generated_values_are_valid(
            schema in arb_schema(),
            seed in any::<u64>(),
            cap in 0_usize..300,
        ) {
            let mut rng = SplitMix64::new(seed);
            let config = GeneratorConfig::default();
            let value = generate_value(&schema, &mut rng, cap, &config);
            prop_assert!(schema.contains(&value).is_ok(), "invalid value {}", value);
        }
    }
}
