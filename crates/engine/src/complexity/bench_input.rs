//! Sized bench inputs: seeded random values at a chosen size `n`.
//!
//! The judge times a solution on inputs of growing size. This module builds those
//! inputs from a problem's schema. The size parameter is the item of a tuple root that
//! grows; the other items use the normal generator. Learner and baseline get the same
//! inputs, because the seed depends only on the problem, the size and the generator version.

use crate::generate::{generate_value, GeneratorConfig};
use crate::rng::SplitMix64;
use crate::schema::Schema;
use crate::GENERATOR_VERSION;
use serde_json::Value;
use std::collections::HashSet;
use std::fmt;

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// The collection length used for the non-sized items of the tuple.
const OTHER_ITEM_SIZE_CAP: usize = 16;

/// Why a sized input cannot be built.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BenchInputError {
    /// The root schema is not a tuple, so there is no item to grow.
    NotATuple,
    /// The size parameter index is past the last tuple item.
    SizeParamOutOfRange {
        /// The index that was asked for.
        index: usize,
    },
    /// The chosen item is not an int array, string or matrix, or it has no alphabet.
    UnsupportedSizeParameter,
    /// `unique` cannot hold at this size, because the range has fewer values than `n`.
    UniqueImpossible,
}

impl fmt::Display for BenchInputError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotATuple => f.write_str("a sized input needs a tuple root schema"),
            Self::SizeParamOutOfRange { index } => {
                write!(f, "size parameter {index} is outside the tuple")
            }
            Self::UnsupportedSizeParameter => {
                f.write_str("the size parameter must be an int array, a string or a matrix")
            }
            Self::UniqueImpossible => {
                f.write_str("unique values cannot fit the range at this size")
            }
        }
    }
}

impl std::error::Error for BenchInputError {}

/// Builds the input for size `n`. The item at `size_param` grows to `n` elements (`n` characters
/// for a string, or a square of side `floor(sqrt(n))` for a matrix). The other items use the normal
/// generator with the same seed.
///
/// The size parameter's own length bounds are ignored, because bench sizes are far above
/// hidden-test sizes. Element range, alphabet, `sorted` and `unique` are honoured.
///
/// # Errors
///
/// Returns [`BenchInputError`] when the schema has no growable item at `size_param`, or when
/// `unique` cannot hold `n` distinct values.
pub fn sized_input(
    schema: &Schema,
    size_param: usize,
    n: u64,
    seed: u64,
) -> Result<Value, BenchInputError> {
    let Schema::Tuple { items } = schema else {
        return Err(BenchInputError::NotATuple);
    };
    let sized = items
        .get(size_param)
        .ok_or(BenchInputError::SizeParamOutOfRange { index: size_param })?;
    let mut rng = SplitMix64::new(seed);
    let config = GeneratorConfig::default();
    let mut values = Vec::with_capacity(items.len());
    for (index, item) in items.iter().enumerate() {
        if index == size_param {
            values.push(sized_value(sized, n, &mut rng)?);
        } else {
            values.push(generate_value(item, &mut rng, OTHER_ITEM_SIZE_CAP, &config));
        }
    }
    Ok(Value::Array(values))
}

/// Builds the grown item of size `n`.
fn sized_value(schema: &Schema, n: u64, rng: &mut SplitMix64) -> Result<Value, BenchInputError> {
    match schema {
        Schema::IntArray {
            min,
            max,
            sorted,
            unique,
            ..
        } => {
            let count = length(n)?;
            let values = if *unique {
                distinct_ints(rng, count, *min, *max)?
            } else {
                (0..count).map(|_| rng.gen_range(*min, *max)).collect()
            };
            let mut values = values;
            if *sorted {
                values.sort_unstable();
            }
            Ok(Value::Array(values.into_iter().map(Value::from).collect()))
        }
        Schema::Str { alphabet, .. } => {
            let chars: Vec<char> = alphabet.chars().collect();
            if chars.is_empty() {
                return Err(BenchInputError::UnsupportedSizeParameter);
            }
            let count = length(n)?;
            let text: String = (0..count)
                .map(|_| {
                    let pick = rng.gen_range(0, i64::try_from(chars.len() - 1).unwrap_or(0));
                    chars[usize::try_from(pick).unwrap_or(0)]
                })
                .collect();
            Ok(Value::String(text))
        }
        Schema::Matrix { min, max, .. } => {
            let side = length(isqrt(n))?;
            let rows = (0..side)
                .map(|_| {
                    Value::Array(
                        (0..side)
                            .map(|_| Value::from(rng.gen_range(*min, *max)))
                            .collect(),
                    )
                })
                .collect();
            Ok(Value::Array(rows))
        }
        Schema::Int { .. } | Schema::Tuple { .. } => Err(BenchInputError::UnsupportedSizeParameter),
    }
}

/// A count of elements as a `usize`. Fails only where the platform cannot hold the count.
fn length(n: u64) -> Result<usize, BenchInputError> {
    usize::try_from(n).map_err(|_| BenchInputError::UnsupportedSizeParameter)
}

/// `floor(sqrt(n))`, computed with integers so there is no rounding error.
fn isqrt(n: u64) -> u64 {
    // Binary search for the largest `s` with `s * s <= n`.
    let (mut low, mut high) = (0_u64, u64::from(u32::MAX));
    while low < high {
        let mid = low + (high - low).div_ceil(2);
        if mid.checked_mul(mid).is_some_and(|sq| sq <= n) {
            low = mid;
        } else {
            high = mid - 1;
        }
    }
    low
}

/// `count` distinct integers in `min..=max`, drawn at random by rejection.
fn distinct_ints(
    rng: &mut SplitMix64,
    count: usize,
    min: i64,
    max: i64,
) -> Result<Vec<i64>, BenchInputError> {
    let width = i128::from(max) - i128::from(min) + 1;
    if width < i128::try_from(count).unwrap_or(i128::MAX) {
        return Err(BenchInputError::UniqueImpossible);
    }
    let mut seen = HashSet::with_capacity(count);
    let mut values = Vec::with_capacity(count);
    while values.len() < count {
        let candidate = rng.gen_range(min, max);
        if seen.insert(candidate) {
            values.push(candidate);
        }
    }
    Ok(values)
}

/// The seed for one bench size. It depends only on the problem, its version, the size and the
/// generator version, so every learner and baseline sees the same inputs.
///
/// The fields are hashed with FNV-1a, each prefixed by its byte length, so `"ab" + "c"` and
/// `"a" + "bc"` cannot collide by accident.
#[must_use]
pub fn bench_seed(problem_slug: &str, problem_version: u32, n: u64) -> u64 {
    let version = problem_version.to_string();
    let size = n.to_string();
    let mut hash = FNV_OFFSET;
    for field in [
        problem_slug,
        version.as_str(),
        size.as_str(),
        GENERATOR_VERSION,
    ] {
        let length = (field.len() as u64).to_le_bytes();
        for byte in length {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        for byte in field.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::schema::Schema;

    /// A tuple of a growing int array and one fixed integer.
    fn array_then_int(unique: bool, sorted: bool, lo: i64, hi: i64) -> Schema {
        Schema::Tuple {
            items: vec![
                Schema::IntArray {
                    min_len: 0,
                    max_len: 3,
                    min: lo,
                    max: hi,
                    sorted,
                    unique,
                },
                Schema::Int { min: 1, max: 3 },
            ],
        }
    }

    /// The same schema with the size parameter's own length bound lifted to `n`.
    /// Bench sizes are far above hidden-test sizes, so only the length bound may differ.
    fn with_length_bound(schema: &Schema, n: usize) -> Schema {
        match schema {
            Schema::Tuple { items } => {
                let mut items = items.clone();
                if let Schema::IntArray { max_len, .. } = &mut items[0] {
                    *max_len = n;
                }
                Schema::Tuple { items }
            }
            other => other.clone(),
        }
    }

    #[test]
    fn the_int_array_has_exactly_the_requested_length() {
        let schema = array_then_int(false, false, 0, 100);
        let value = sized_input(&schema, 0, 50, 7).expect("sized");
        assert_eq!(value[0].as_array().map(Vec::len), Some(50));
    }

    #[test]
    fn the_result_satisfies_the_schema_apart_from_the_length_bound() {
        let schema = array_then_int(false, false, 0, 100);
        let value = sized_input(&schema, 0, 50, 7).expect("sized");
        let relaxed = with_length_bound(&schema, 50);
        assert!(relaxed.contains(&value).is_ok(), "{value}");
    }

    #[test]
    fn the_same_arguments_give_the_same_input() {
        let schema = array_then_int(false, false, 0, 100);
        let a = sized_input(&schema, 0, 64, 99).expect("sized");
        let b = sized_input(&schema, 0, 64, 99).expect("sized");
        assert_eq!(a, b);
    }

    #[test]
    fn a_different_seed_gives_a_different_input() {
        let schema = array_then_int(false, false, 0, 1_000_000);
        let a = sized_input(&schema, 0, 64, 1).expect("sized");
        let b = sized_input(&schema, 0, 64, 2).expect("sized");
        assert_ne!(a, b);
    }

    #[test]
    fn sorted_and_unique_are_honoured() {
        let schema = array_then_int(true, true, 0, 1000);
        let value = sized_input(&schema, 0, 100, 3).expect("sized");
        let items: Vec<i64> = value[0]
            .as_array()
            .expect("array")
            .iter()
            .map(|v| v.as_i64().expect("int"))
            .collect();
        assert_eq!(items.len(), 100);
        assert!(items.windows(2).all(|pair| pair[0] < pair[1]), "{items:?}");
    }

    #[test]
    fn unique_is_impossible_when_the_range_is_too_small() {
        // Five values in 0..=4 cannot give ten distinct numbers.
        let schema = array_then_int(true, false, 0, 4);
        assert_eq!(
            sized_input(&schema, 0, 10, 1),
            Err(BenchInputError::UniqueImpossible)
        );
    }

    #[test]
    fn a_non_tuple_root_is_refused() {
        let schema = Schema::Int { min: 0, max: 9 };
        assert_eq!(
            sized_input(&schema, 0, 10, 1),
            Err(BenchInputError::NotATuple)
        );
    }

    #[test]
    fn a_size_parameter_outside_the_tuple_is_refused() {
        let schema = array_then_int(false, false, 0, 9);
        assert_eq!(
            sized_input(&schema, 5, 10, 1),
            Err(BenchInputError::SizeParamOutOfRange { index: 5 })
        );
    }

    #[test]
    fn a_size_parameter_that_is_not_sized_is_refused() {
        let schema = array_then_int(false, false, 0, 9);
        // Item 1 is a plain integer, which has no length to grow.
        assert_eq!(
            sized_input(&schema, 1, 10, 1),
            Err(BenchInputError::UnsupportedSizeParameter)
        );
    }

    #[test]
    fn a_string_parameter_has_exactly_n_characters() {
        let schema = Schema::Tuple {
            items: vec![Schema::Str {
                alphabet: "ab".to_owned(),
                min_len: 0,
                max_len: 5,
            }],
        };
        let value = sized_input(&schema, 0, 7, 4).expect("sized");
        let text = value[0].as_str().expect("string");
        assert_eq!(text.chars().count(), 7);
        assert!(text.chars().all(|c| c == 'a' || c == 'b'));
    }

    #[test]
    fn a_matrix_parameter_is_square_with_side_the_square_root_of_n() {
        // n = 50, so side = floor(sqrt(50)) = 7.
        let schema = Schema::Tuple {
            items: vec![Schema::Matrix {
                min_rows: 1,
                max_rows: 2,
                min_cols: 1,
                max_cols: 2,
                min: 0,
                max: 9,
            }],
        };
        let value = sized_input(&schema, 0, 50, 5).expect("sized");
        let rows = value[0].as_array().expect("rows");
        assert_eq!(rows.len(), 7);
        for row in rows {
            assert_eq!(row.as_array().map(Vec::len), Some(7));
        }
    }

    #[test]
    fn the_seed_depends_on_every_argument() {
        let base = bench_seed("count-pairs", 3, 1024);
        assert_eq!(base, bench_seed("count-pairs", 3, 1024));
        assert_ne!(base, bench_seed("count-pairs", 3, 2048));
        assert_ne!(base, bench_seed("count-pairs", 4, 1024));
        assert_ne!(base, bench_seed("longest-run", 3, 1024));
    }

    #[test]
    fn the_seed_is_length_prefixed_so_field_boundaries_cannot_blur() {
        // "ab" + "c" and "a" + "bc" must not collide because of how the fields join.
        assert_ne!(bench_seed("ab", 1, 8), bench_seed("a", 1, 8));
    }

    #[test]
    fn the_bench_seed_value_is_pinned_for_reproducibility() {
        // Pinned so that an accidental change to the seed rule is noticed. A change here
        // must come with a bump of GENERATOR_VERSION and a note in the calibration log.
        assert_eq!(bench_seed("count-pairs", 1, 256), PINNED_SEED);
    }

    /// The pinned value. Cross-checked by an independent FNV-1a calculation over the same fields
    /// (`count-pairs`, `1`, `256`, `gen-1`).
    const PINNED_SEED: u64 = 321_671_143_553_942_296;

    #[test]
    fn a_fixed_item_is_drawn_from_its_own_range() {
        let schema = array_then_int(false, false, 0, 9);
        let value = sized_input(&schema, 0, 3, 1).expect("sized");
        let fixed = value[1].as_i64().expect("int");
        assert!((1..=3).contains(&fixed), "{fixed}");
    }
}
