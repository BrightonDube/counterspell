//! Strict JSON schemas and their definition checks.

use serde::{Deserialize, Serialize};
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
    use super::Schema;

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
}
