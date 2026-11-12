//! Strict JSON schemas and their definition checks.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use std::fmt::{self, Display, Formatter};

const JS_SAFE_INTEGER: i64 = (1_i64 << 53) - 1;
const MAX_LENGTH: usize = 1_000_000;

/// A bounded input shape accepted by the ghost engine.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Schema {
    /// A JavaScript-safe signed integer range.
    Int {
        /// The smallest accepted integer.
        min: i64,
        /// The largest accepted integer.
        max: i64,
    },
    /// A printable-ASCII string with an explicit alphabet and length range.
    #[serde(rename = "string")]
    Str {
        /// Characters that may appear in generated strings.
        alphabet: String,
        /// The shortest accepted string length.
        min_len: usize,
        /// The longest accepted string length.
        max_len: usize,
    },
    /// An integer array with optional sortedness and uniqueness constraints.
    IntArray {
        /// The shortest accepted array length.
        min_len: usize,
        /// The longest accepted array length.
        max_len: usize,
        /// The smallest accepted element.
        min: i64,
        /// The largest accepted element.
        max: i64,
        /// Whether elements must be non-decreasing.
        #[serde(default)]
        sorted: bool,
        /// Whether elements must be pairwise distinct.
        #[serde(default)]
        unique: bool,
    },
    /// A rectangular two-dimensional integer array.
    Matrix {
        /// The smallest accepted row count.
        min_rows: usize,
        /// The largest accepted row count.
        max_rows: usize,
        /// The smallest accepted column count.
        min_cols: usize,
        /// The largest accepted column count.
        max_cols: usize,
        /// The smallest accepted cell value.
        min: i64,
        /// The largest accepted cell value.
        max: i64,
    },
    /// A fixed-arity list of non-tuple input shapes.
    Tuple {
        /// The argument schemas in their driver order.
        items: Vec<Schema>,
    },
}

/// A human-readable schema-definition failure.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SchemaError {
    message: String,
}

impl SchemaError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl Display for SchemaError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for SchemaError {}

/// A value-validation failure with a JSON Pointer path.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Violation {
    /// The JSON Pointer path where validation stopped. The root is an empty path.
    pub path: String,
    /// A short, plain-English reason for the violation.
    pub reason: String,
}

impl Violation {
    /// Creates a validation failure at a JSON Pointer path.
    pub fn new(path: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            reason: reason.into(),
        }
    }
}

impl Schema {
    /// Checks that this schema is internally consistent and supported.
    ///
    /// # Errors
    ///
    /// Returns a [`SchemaError`] when bounds, alphabets, lengths, or tuple
    /// structure violate the engine's supported schema rules.
    pub fn check_definition(&self) -> Result<(), SchemaError> {
        match self {
            Self::Int { min, max } => {
                check_bounds(*min, *max, "integer")?;
                check_safe_integer(*min, "integer minimum")?;
                check_safe_integer(*max, "integer maximum")
            }
            Self::Str {
                alphabet,
                min_len,
                max_len,
            } => {
                check_lengths(*min_len, *max_len, "string")?;
                if alphabet.is_empty() {
                    return Err(SchemaError::new("string alphabet must not be empty"));
                }
                let characters: Vec<char> = alphabet.chars().collect();
                if characters.iter().any(|character| {
                    let code_point = u32::from(*character);
                    !(0x20..=0x7e).contains(&code_point)
                }) {
                    return Err(SchemaError::new(
                        "string alphabet must contain printable ASCII only",
                    ));
                }
                for (index, character) in characters.iter().enumerate() {
                    if characters[..index].contains(character) {
                        return Err(SchemaError::new(
                            "string alphabet characters must be distinct",
                        ));
                    }
                }
                Ok(())
            }
            Self::IntArray {
                min_len,
                max_len,
                min,
                max,
                unique,
                ..
            } => {
                check_lengths(*min_len, *max_len, "integer array")?;
                check_bounds(*min, *max, "integer array element")?;
                check_safe_integer(*min, "integer array minimum")?;
                check_safe_integer(*max, "integer array maximum")?;
                if *unique {
                    let capacity = u128::try_from(i128::from(*max) - i128::from(*min) + 1)
                        .map_err(|_| SchemaError::new("unique array range is invalid"))?;
                    if capacity < *min_len as u128 {
                        return Err(SchemaError::new(
                            "unique array range cannot reach its minimum length",
                        ));
                    }
                }
                Ok(())
            }
            Self::Matrix {
                min_rows,
                max_rows,
                min_cols,
                max_cols,
                min,
                max,
            } => {
                check_lengths(*min_rows, *max_rows, "matrix rows")?;
                check_lengths(*min_cols, *max_cols, "matrix columns")?;
                check_bounds(*min, *max, "matrix cell")?;
                check_safe_integer(*min, "matrix minimum")?;
                check_safe_integer(*max, "matrix maximum")
            }
            Self::Tuple { items } => {
                if items.is_empty() {
                    return Err(SchemaError::new("tuple must contain at least one item"));
                }
                for item in items {
                    if matches!(item, Self::Tuple { .. }) {
                        return Err(SchemaError::new("nested tuples are not supported"));
                    }
                    item.check_definition()?;
                }
                Ok(())
            }
        }
    }

    /// Checks a JSON value against this schema without constructing a value.
    ///
    /// # Errors
    ///
    /// Returns the first violation with its JSON Pointer path and a short
    /// reason. The walk rejects wrong JSON types before reading their fields.
    pub fn contains(&self, value: &Value) -> Result<(), Violation> {
        self.contains_at(value, "")
    }

    // Keep this allowance narrow: the match exhaustively covers all five schema kinds.
    #[allow(clippy::too_many_lines)]
    fn contains_at(&self, value: &Value, path: &str) -> Result<(), Violation> {
        match self {
            Self::Int { min, max } => {
                let number = value
                    .as_i64()
                    .ok_or_else(|| Violation::new(path, "expected an integer"))?;
                if !(*min..=*max).contains(&number) {
                    return Err(Violation::new(path, "integer is outside its bounds"));
                }
                Ok(())
            }
            Self::Str {
                alphabet,
                min_len,
                max_len,
            } => {
                let string = value
                    .as_str()
                    .ok_or_else(|| Violation::new(path, "expected a string"))?;
                let length = string.chars().count();
                if !(*min_len..=*max_len).contains(&length) {
                    return Err(Violation::new(path, "string length is outside its bounds"));
                }
                for (index, character) in string.chars().enumerate() {
                    if !alphabet.contains(character) {
                        return Err(Violation::new(
                            child_path(path, index),
                            "character is not in the alphabet",
                        ));
                    }
                }
                Ok(())
            }
            Self::IntArray {
                min_len,
                max_len,
                min,
                max,
                sorted,
                unique,
            } => {
                let array = value
                    .as_array()
                    .ok_or_else(|| Violation::new(path, "expected an integer array"))?;
                if !(*min_len..=*max_len).contains(&array.len()) {
                    return Err(Violation::new(path, "array length is outside its bounds"));
                }
                let mut previous = None;
                let mut seen = HashSet::new();
                for (index, item) in array.iter().enumerate() {
                    let item_path = child_path(path, index);
                    let number = item
                        .as_i64()
                        .ok_or_else(|| Violation::new(&item_path, "expected an integer"))?;
                    if !(*min..=*max).contains(&number) {
                        return Err(Violation::new(item_path, "integer is outside its bounds"));
                    }
                    if let Some(previous_number) = previous {
                        if *sorted && number < previous_number {
                            return Err(Violation::new(item_path.clone(), "array is not sorted"));
                        }
                    }
                    if *unique && !seen.insert(number) {
                        return Err(Violation::new(
                            item_path.clone(),
                            "array contains a duplicate",
                        ));
                    }
                    previous = Some(number);
                }
                Ok(())
            }
            Self::Matrix {
                min_rows,
                max_rows,
                min_cols,
                max_cols,
                min,
                max,
            } => {
                let rows = value
                    .as_array()
                    .ok_or_else(|| Violation::new(path, "expected a matrix"))?;
                if !(*min_rows..=*max_rows).contains(&rows.len()) {
                    return Err(Violation::new(
                        path,
                        "matrix row count is outside its bounds",
                    ));
                }
                let mut expected_columns = None;
                for (row_index, row_value) in rows.iter().enumerate() {
                    let row_path = child_path(path, row_index);
                    let row = row_value
                        .as_array()
                        .ok_or_else(|| Violation::new(&row_path, "expected a matrix row"))?;
                    if !(*min_cols..=*max_cols).contains(&row.len()) {
                        return Err(Violation::new(
                            row_path.clone(),
                            "matrix column count is outside its bounds",
                        ));
                    }
                    if let Some(columns) = expected_columns {
                        if columns != row.len() {
                            return Err(Violation::new(
                                row_path,
                                "matrix rows must have equal lengths",
                            ));
                        }
                    } else {
                        expected_columns = Some(row.len());
                    }
                    for (column_index, item) in row.iter().enumerate() {
                        let item_path = child_path(&row_path, column_index);
                        let number = item
                            .as_i64()
                            .ok_or_else(|| Violation::new(&item_path, "expected an integer"))?;
                        if !(*min..=*max).contains(&number) {
                            return Err(Violation::new(item_path, "integer is outside its bounds"));
                        }
                    }
                }
                Ok(())
            }
            Self::Tuple { items } => {
                let tuple = value
                    .as_array()
                    .ok_or_else(|| Violation::new(path, "expected a tuple array"))?;
                if tuple.len() != items.len() {
                    return Err(Violation::new(path, "tuple has the wrong arity"));
                }
                for (index, (schema, item)) in items.iter().zip(tuple).enumerate() {
                    schema.contains_at(item, &child_path(path, index))?;
                }
                Ok(())
            }
        }
    }
}

fn child_path(parent: &str, index: usize) -> String {
    format!("{parent}/{index}")
}

fn check_bounds(min: i64, max: i64, label: &str) -> Result<(), SchemaError> {
    if min > max {
        return Err(SchemaError::new(format!("{label} bounds must be ordered")));
    }
    Ok(())
}

fn check_safe_integer(value: i64, label: &str) -> Result<(), SchemaError> {
    if !(-JS_SAFE_INTEGER..=JS_SAFE_INTEGER).contains(&value) {
        return Err(SchemaError::new(format!(
            "{label} must fit within JavaScript-safe integer bounds"
        )));
    }
    Ok(())
}

fn check_lengths(min: usize, max: usize, label: &str) -> Result<(), SchemaError> {
    if min > max {
        return Err(SchemaError::new(format!("{label} bounds must be ordered")));
    }
    if max > MAX_LENGTH {
        return Err(SchemaError::new(format!(
            "{label} lengths must not exceed {MAX_LENGTH}"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use serde_json::{json, Value};

    use super::{Schema, Violation};

    #[allow(clippy::needless_pass_by_value)]
    fn rejects_definition(schema: Schema) {
        assert!(schema.check_definition().is_err());
    }

    #[test]
    fn three_pack_schema_examples_deserialise_and_validate() {
        let pair_count: Schema = serde_json::from_str(
            r#"{"type":"tuple","items":[{"type":"int_array","min_len":0,"max_len":30,"min":-50,"max":50},{"type":"int","min":-100,"max":100}]}"#,
        )
        .unwrap();
        let longest_run: Schema = serde_json::from_str(
            r#"{"type":"tuple","items":[{"type":"string","alphabet":"abc","min_len":0,"max_len":40}]}"#,
        )
        .unwrap();
        let diagonal_sum: Schema = serde_json::from_str(
            r#"{"type":"tuple","items":[{"type":"matrix","min_rows":0,"max_rows":6,"min_cols":0,"max_cols":6,"min":-9,"max":9}]}"#,
        )
        .unwrap();

        assert!(pair_count.check_definition().is_ok());
        assert!(longest_run.check_definition().is_ok());
        assert!(diagonal_sum.check_definition().is_ok());
    }

    #[test]
    fn rejects_reversed_bounds_for_every_bounded_kind() {
        rejects_definition(Schema::Int { min: 2, max: 1 });
        rejects_definition(Schema::Str {
            alphabet: "a".to_owned(),
            min_len: 2,
            max_len: 1,
        });
        rejects_definition(Schema::IntArray {
            min_len: 0,
            max_len: 1,
            min: 2,
            max: 1,
            sorted: false,
            unique: false,
        });
        rejects_definition(Schema::IntArray {
            min_len: 2,
            max_len: 1,
            min: 0,
            max: 1,
            sorted: false,
            unique: false,
        });
        rejects_definition(Schema::Matrix {
            min_rows: 2,
            max_rows: 1,
            min_cols: 0,
            max_cols: 1,
            min: 0,
            max: 1,
        });
        rejects_definition(Schema::Matrix {
            min_rows: 0,
            max_rows: 1,
            min_cols: 2,
            max_cols: 1,
            min: 0,
            max: 1,
        });
        rejects_definition(Schema::Matrix {
            min_rows: 0,
            max_rows: 1,
            min_cols: 0,
            max_cols: 1,
            min: 2,
            max: 1,
        });

        let error = Schema::Int { min: 2, max: 1 }
            .check_definition()
            .unwrap_err();
        assert!(error.to_string().contains("integer bounds must be ordered"));
    }

    #[test]
    fn rejects_invalid_alphabets_and_lengths() {
        rejects_definition(Schema::Str {
            alphabet: String::new(),
            min_len: 0,
            max_len: 1,
        });
        rejects_definition(Schema::Str {
            alphabet: "aa".to_owned(),
            min_len: 0,
            max_len: 1,
        });
        rejects_definition(Schema::Str {
            alphabet: "a\n".to_owned(),
            min_len: 0,
            max_len: 1,
        });
        rejects_definition(Schema::Str {
            alphabet: "a".to_owned(),
            min_len: 1_000_001,
            max_len: 1_000_001,
        });
        rejects_definition(Schema::IntArray {
            min_len: 0,
            max_len: 1_000_001,
            min: 0,
            max: 1,
            sorted: false,
            unique: false,
        });
        rejects_definition(Schema::Matrix {
            min_rows: 1_000_001,
            max_rows: 1_000_001,
            min_cols: 0,
            max_cols: 1,
            min: 0,
            max: 1,
        });
        rejects_definition(Schema::Matrix {
            min_rows: 0,
            max_rows: 1,
            min_cols: 1_000_001,
            max_cols: 1_000_001,
            min: 0,
            max: 1,
        });
    }

    #[test]
    fn rejects_unsafe_integer_bounds_and_impossible_unique_arrays() {
        rejects_definition(Schema::Int {
            min: -(1_i64 << 53),
            max: 1_i64 << 53,
        });
        rejects_definition(Schema::IntArray {
            min_len: 0,
            max_len: 1,
            min: -(1_i64 << 53),
            max: 1,
            sorted: false,
            unique: false,
        });
        rejects_definition(Schema::Matrix {
            min_rows: 0,
            max_rows: 1,
            min_cols: 0,
            max_cols: 1,
            min: -1,
            max: 1_i64 << 53,
        });
        assert!(Schema::Int {
            min: -((1_i64 << 53) - 1),
            max: (1_i64 << 53) - 1,
        }
        .check_definition()
        .is_ok());
        rejects_definition(Schema::IntArray {
            min_len: 3,
            max_len: 3,
            min: 0,
            max: 1,
            sorted: false,
            unique: true,
        });
    }

    #[test]
    fn rejects_empty_and_nested_tuples() {
        rejects_definition(Schema::Tuple { items: Vec::new() });
        rejects_definition(Schema::Tuple {
            items: vec![Schema::Tuple {
                items: vec![Schema::Int { min: 0, max: 1 }],
            }],
        });
    }

    #[test]
    fn serde_rejects_unknown_fields_unknown_types_and_float_bounds() {
        let unknown_field =
            serde_json::from_str::<Schema>(r#"{"type":"int","min":0,"max":1,"extra":true}"#);
        assert!(unknown_field.is_err());

        let unknown_type = serde_json::from_str::<Schema>(r#"{"type":"float","min":0,"max":1}"#);
        assert!(unknown_type.is_err());

        let float_bound = serde_json::from_str::<Schema>(r#"{"type":"int","min":0.5,"max":1}"#);
        assert!(float_bound.is_err());
    }

    #[test]
    fn contains_accepts_valid_values_for_each_schema_kind() {
        let cases = [
            (Schema::Int { min: -2, max: 2 }, json!(-2)),
            (
                Schema::Str {
                    alphabet: "ab".to_owned(),
                    min_len: 1,
                    max_len: 3,
                },
                json!("aba"),
            ),
            (
                Schema::IntArray {
                    min_len: 2,
                    max_len: 3,
                    min: 0,
                    max: 4,
                    sorted: true,
                    unique: true,
                },
                json!([1, 3]),
            ),
            (
                Schema::Matrix {
                    min_rows: 1,
                    max_rows: 2,
                    min_cols: 2,
                    max_cols: 2,
                    min: -1,
                    max: 1,
                },
                json!([[0, 1]]),
            ),
            (
                Schema::Tuple {
                    items: vec![
                        Schema::Int { min: 0, max: 1 },
                        Schema::Str {
                            alphabet: "x".to_owned(),
                            min_len: 0,
                            max_len: 1,
                        },
                    ],
                },
                json!([1, ""]),
            ),
        ];

        for (schema, value) in cases {
            assert_eq!(schema.contains(&value), Ok(()));
        }
    }

    #[test]
    fn contains_reports_integer_and_string_violations() {
        let integer = Schema::Int { min: 0, max: 2 };
        assert_eq!(
            integer.contains(&json!(3)),
            Err(Violation::new("", "integer is outside its bounds"))
        );
        assert_eq!(
            integer.contains(&json!(1.5)),
            Err(Violation::new("", "expected an integer"))
        );

        let string = Schema::Str {
            alphabet: "ab".to_owned(),
            min_len: 1,
            max_len: 2,
        };
        assert_eq!(
            string.contains(&json!("abc")),
            Err(Violation::new("", "string length is outside its bounds"))
        );
        assert_eq!(
            string.contains(&json!("ac")),
            Err(Violation::new("/1", "character is not in the alphabet"))
        );
    }

    #[test]
    fn contains_reports_array_constraints_and_uses_json_pointer_paths() {
        let schema = Schema::IntArray {
            min_len: 2,
            max_len: 3,
            min: 0,
            max: 5,
            sorted: true,
            unique: true,
        };
        assert_eq!(
            schema.contains(&json!([1])),
            Err(Violation::new("", "array length is outside its bounds"))
        );
        assert_eq!(
            schema.contains(&json!([2, 1])),
            Err(Violation::new("/1", "array is not sorted"))
        );
        assert_eq!(
            schema.contains(&json!([2, 2])),
            Err(Violation::new("/1", "array contains a duplicate"))
        );
        assert_eq!(
            schema.contains(&json!([1, 6])),
            Err(Violation::new("/1", "integer is outside its bounds"))
        );
        assert_eq!(
            schema.contains(&json!([1, 1.5])),
            Err(Violation::new("/1", "expected an integer"))
        );
    }

    #[test]
    fn contains_reports_matrix_tuple_and_type_violations() {
        let matrix = Schema::Matrix {
            min_rows: 1,
            max_rows: 2,
            min_cols: 1,
            max_cols: 2,
            min: 0,
            max: 5,
        };
        assert_eq!(
            matrix.contains(&json!([[1], [2, 3]])),
            Err(Violation::new("/1", "matrix rows must have equal lengths"))
        );
        assert_eq!(
            matrix.contains(&json!([[1, 6]])),
            Err(Violation::new("/0/1", "integer is outside its bounds"))
        );
        assert_eq!(
            matrix.contains(&json!([1])),
            Err(Violation::new("/0", "expected a matrix row"))
        );

        let tuple = Schema::Tuple {
            items: vec![Schema::Int { min: 0, max: 1 }],
        };
        assert_eq!(
            tuple.contains(&json!([])),
            Err(Violation::new("", "tuple has the wrong arity"))
        );
        assert_eq!(
            tuple.contains(&json!({"0": 1})),
            Err(Violation::new("", "expected a tuple array"))
        );
    }

    proptest! {
        #[test]
        fn p2_validator_rejects_a_controlled_break(value in strategies::arb_valid_value()) {
            let (schema, valid) = value;
            prop_assert!(schema.contains(&valid).is_ok());
            prop_assert!(schema.contains(&strategies::break_value(&schema, &valid)).is_err());
        }

        #[test]
        fn p13_contains_never_panics_on_arbitrary_json(value in strategies::arb_json()) {
            let schema = Schema::Tuple {
                items: vec![Schema::Int { min: -2, max: 2 }],
            };
            let _ = schema.contains(&value);
        }
    }

    mod strategies {
        use super::*;

        pub fn arb_schema() -> BoxedStrategy<Schema> {
            let int = (-5_i64..=5, -5_i64..=5).prop_map(|(left, right)| Schema::Int {
                min: left.min(right),
                max: left.max(right),
            });
            let string = (0_usize..=4, 0_usize..=4).prop_map(|(left, right)| Schema::Str {
                alphabet: "abc".to_owned(),
                min_len: left.min(right),
                max_len: left.max(right),
            });
            let int_array = (0_usize..=4, 0_usize..=4, any::<bool>(), any::<bool>()).prop_map(
                |(left, right, sorted, unique)| Schema::IntArray {
                    min_len: left.min(right),
                    max_len: left.max(right),
                    min: -5,
                    max: 5,
                    sorted,
                    unique,
                },
            );
            let matrix = (0_usize..=3, 0_usize..=3, 0_usize..=3, 0_usize..=3).prop_map(
                |(row_left, row_right, column_left, column_right)| Schema::Matrix {
                    min_rows: row_left.min(row_right),
                    max_rows: row_left.max(row_right),
                    min_cols: column_left.min(column_right),
                    max_cols: column_left.max(column_right),
                    min: -5,
                    max: 5,
                },
            );
            let tuple = prop::collection::vec(arb_leaf_schema(), 1..=3)
                .prop_map(|items| Schema::Tuple { items });

            prop_oneof![int, string, int_array, matrix, tuple].boxed()
        }

        fn arb_leaf_schema() -> BoxedStrategy<Schema> {
            let int = (-5_i64..=5, -5_i64..=5).prop_map(|(left, right)| Schema::Int {
                min: left.min(right),
                max: left.max(right),
            });
            let string = (0_usize..=4, 0_usize..=4).prop_map(|(left, right)| Schema::Str {
                alphabet: "abc".to_owned(),
                min_len: left.min(right),
                max_len: left.max(right),
            });
            let int_array = (0_usize..=4, 0_usize..=4, any::<bool>(), any::<bool>()).prop_map(
                |(left, right, sorted, unique)| Schema::IntArray {
                    min_len: left.min(right),
                    max_len: left.max(right),
                    min: -5,
                    max: 5,
                    sorted,
                    unique,
                },
            );
            let matrix = (0_usize..=3, 0_usize..=3, 0_usize..=3, 0_usize..=3).prop_map(
                |(row_left, row_right, column_left, column_right)| Schema::Matrix {
                    min_rows: row_left.min(row_right),
                    max_rows: row_left.max(row_right),
                    min_cols: column_left.min(column_right),
                    max_cols: column_left.max(column_right),
                    min: -5,
                    max: 5,
                },
            );

            prop_oneof![int, string, int_array, matrix].boxed()
        }

        pub fn arb_value(schema: &Schema) -> BoxedStrategy<Value> {
            match schema {
                Schema::Int { min, max } => {
                    let min = *min;
                    let max = *max;
                    (min..=max).prop_map(|value| json!(value)).boxed()
                }
                Schema::Str {
                    alphabet,
                    min_len,
                    max_len,
                } => {
                    let characters: Vec<char> = alphabet.chars().collect();
                    let min_len = *min_len;
                    let max_len = *max_len;
                    prop::collection::vec(prop::sample::select(characters), min_len..=max_len)
                        .prop_map(|characters| Value::String(characters.into_iter().collect()))
                        .boxed()
                }
                Schema::IntArray {
                    min_len,
                    max_len,
                    min,
                    max,
                    sorted,
                    unique,
                } => arb_int_array_value(*min_len, *max_len, *min, *max, *sorted, *unique),
                Schema::Matrix {
                    min_rows,
                    max_rows,
                    min_cols,
                    max_cols,
                    min,
                    max,
                } => arb_matrix_value(*min_rows, *max_rows, *min_cols, *max_cols, *min, *max),
                Schema::Tuple { items } => arb_tuple_value(items),
            }
        }

        fn arb_int_array_value(
            min_len: usize,
            max_len: usize,
            min: i64,
            max: i64,
            sorted: bool,
            unique: bool,
        ) -> BoxedStrategy<Value> {
            if unique {
                prop::collection::btree_set(min..=max, min_len..=max_len)
                    .prop_map(move |values| {
                        let mut values: Vec<i64> = values.into_iter().collect();
                        if !sorted {
                            values.reverse();
                        }
                        Value::Array(values.into_iter().map(|value| json!(value)).collect())
                    })
                    .boxed()
            } else {
                prop::collection::vec(min..=max, min_len..=max_len)
                    .prop_map(move |mut values| {
                        if sorted {
                            values.sort_unstable();
                        }
                        Value::Array(values.into_iter().map(|value| json!(value)).collect())
                    })
                    .boxed()
            }
        }

        fn arb_matrix_value(
            min_rows: usize,
            max_rows: usize,
            min_cols: usize,
            max_cols: usize,
            min: i64,
            max: i64,
        ) -> BoxedStrategy<Value> {
            (min_cols..=max_cols)
                .prop_flat_map(move |columns| {
                    let row_strategy = prop::collection::vec(min..=max, columns);
                    prop::collection::vec(row_strategy, min_rows..=max_rows)
                })
                .prop_map(|rows| {
                    Value::Array(
                        rows.into_iter()
                            .map(|row| {
                                Value::Array(row.into_iter().map(|value| json!(value)).collect())
                            })
                            .collect(),
                    )
                })
                .boxed()
        }

        fn arb_tuple_value(items: &[Schema]) -> BoxedStrategy<Value> {
            let mut values = Just(Vec::<Value>::new()).boxed();
            for item in items {
                let item_strategy = arb_value(item);
                values = values
                    .prop_flat_map(move |values| {
                        item_strategy.clone().prop_map(move |value| {
                            let mut next = values.clone();
                            next.push(value);
                            next
                        })
                    })
                    .boxed();
            }
            values.prop_map(Value::Array).boxed()
        }

        pub fn arb_valid_value() -> BoxedStrategy<(Schema, Value)> {
            arb_schema()
                .prop_flat_map(|schema| {
                    arb_value(&schema).prop_map(move |value| (schema.clone(), value))
                })
                .boxed()
        }

        pub fn break_value(schema: &Schema, _value: &Value) -> Value {
            match schema {
                Schema::Int { .. } => json!("not an integer"),
                Schema::Str { .. } => json!(0),
                Schema::IntArray { .. } => json!({}),
                Schema::Matrix { .. } => json!(true),
                Schema::Tuple { .. } => Value::Null,
            }
        }

        pub fn arb_json() -> impl Strategy<Value = Value> {
            let leaves = prop_oneof![
                Just(Value::Null),
                any::<bool>().prop_map(Value::Bool),
                any::<i64>().prop_map(|value| json!(value)),
                Just(json!(1.5)),
                Just(json!(u64::MAX)),
                "[a-z]{0,4}".prop_map(Value::String),
            ];
            leaves.prop_recursive(2, 16, 2, |inner| {
                prop_oneof![
                    prop::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
                    prop::collection::hash_map("[a-z]{0,2}", inner, 0..4)
                        .prop_map(|object| object.into_iter().collect()),
                ]
            })
        }
    }
}
