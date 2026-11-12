//! Deterministic size metrics for JSON input values.

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use serde_json::Value;
use std::cmp::Ordering;

/// The deterministic metric used to decide whether one JSON value is smaller.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SizeMetric {
    /// The number of JSON nodes in the value.
    pub nodes: u64,
    /// The sum of integer magnitudes and string code points.
    #[serde(with = "u128_as_string")]
    pub magnitude: u128,
}

/// Computes the size metric for any JSON value.
#[must_use]
pub fn size_of(value: &Value) -> SizeMetric {
    match value {
        Value::Null | Value::Bool(_) => SizeMetric {
            nodes: 1,
            magnitude: 0,
        },
        Value::Number(number) => SizeMetric {
            nodes: 1,
            magnitude: number
                .as_i64()
                .map_or(0, |integer| u128::from(integer.unsigned_abs())),
        },
        Value::String(string) => SizeMetric {
            nodes: 1 + usize_to_u64(string.chars().count()),
            magnitude: string.chars().map(u128::from).fold(0, u128::saturating_add),
        },
        Value::Array(values) => combine_children(values.iter()),
        Value::Object(values) => combine_children(values.values()),
    }
}

/// Compares values by nodes, magnitude, and compact canonical JSON text.
#[must_use]
pub fn compare_size(a: &Value, b: &Value) -> Ordering {
    let a_metric = size_of(a);
    let b_metric = size_of(b);
    a_metric
        .nodes
        .cmp(&b_metric.nodes)
        .then_with(|| a_metric.magnitude.cmp(&b_metric.magnitude))
        .then_with(|| a.to_string().cmp(&b.to_string()))
}

fn combine_children<'a>(children: impl Iterator<Item = &'a Value>) -> SizeMetric {
    children.fold(
        SizeMetric {
            nodes: 1,
            magnitude: 0,
        },
        |metric, child| {
            let child_metric = size_of(child);
            SizeMetric {
                nodes: metric.nodes.saturating_add(child_metric.nodes),
                magnitude: metric.magnitude.saturating_add(child_metric.magnitude),
            }
        },
    )
}

fn usize_to_u64(value: usize) -> u64 {
    u64::try_from(value).unwrap_or(u64::MAX)
}

mod u128_as_string {
    use super::{Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(value: &u128, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&value.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<u128, D::Error>
    where
        D: Deserializer<'de>,
    {
        let text = String::deserialize(deserializer)?;
        text.parse::<u128>().map_err(serde::de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use proptest::prelude::*;
    use serde_json::{json, Value};

    use super::{compare_size, size_of, SizeMetric};

    #[test]
    fn size_metric_matches_the_hand_derived_examples() {
        assert_eq!(
            size_of(&json!(0)),
            SizeMetric {
                nodes: 1,
                magnitude: 0,
            }
        );
        assert_eq!(
            size_of(&json!([])),
            SizeMetric {
                nodes: 1,
                magnitude: 0,
            }
        );
        assert_eq!(
            size_of(&json!([0])),
            SizeMetric {
                nodes: 2,
                magnitude: 0,
            }
        );
        assert_eq!(
            size_of(&json!([[0], 0])),
            SizeMetric {
                nodes: 4,
                magnitude: 0,
            }
        );
        assert_eq!(
            size_of(&json!("ab")),
            SizeMetric {
                nodes: 3,
                magnitude: 195,
            }
        );
        assert_eq!(
            size_of(&json!([[]])),
            SizeMetric {
                nodes: 2,
                magnitude: 0,
            }
        );
    }

    #[test]
    fn magnitude_is_zero_for_floats_and_numbers_without_i64() {
        let float: Value = serde_json::from_str("1.5").unwrap();
        let large: Value = serde_json::from_str("9223372036854775808").unwrap();
        assert_eq!(size_of(&float).magnitude, 0);
        assert_eq!(size_of(&large).magnitude, 0);
    }

    #[test]
    fn size_metric_magnitude_serialises_as_a_decimal_string() {
        let metric = SizeMetric {
            nodes: 2,
            magnitude: 123_456_789_012_345_678_901_234_567_890u128,
        };
        let json = serde_json::to_string(&metric).unwrap();
        assert_eq!(
            json,
            r#"{"nodes":2,"magnitude":"123456789012345678901234567890"}"#
        );
        assert_eq!(serde_json::from_str::<SizeMetric>(&json).unwrap(), metric);
    }

    proptest! {
        #[test]
        fn p3_size_order_is_strict_total_and_transitive(
            a in strategies::arb_json(),
            b in strategies::arb_json(),
            c in strategies::arb_json(),
        ) {
            prop_assert_eq!(compare_size(&a, &a), Ordering::Equal);
            let ab = compare_size(&a, &b);
            let ba = compare_size(&b, &a);
            prop_assert_eq!(ab == Ordering::Less, ba == Ordering::Greater);
            prop_assert_eq!(ab == Ordering::Equal, a == b);
            if compare_size(&a, &b) == Ordering::Less && compare_size(&b, &c) == Ordering::Less {
                prop_assert_eq!(compare_size(&a, &c), Ordering::Less);
            }
        }
    }

    mod strategies {
        use super::*;

        pub fn arb_json() -> impl Strategy<Value = Value> {
            let leaves = prop_oneof![
                Just(Value::Null),
                any::<bool>().prop_map(Value::Bool),
                any::<i64>().prop_map(|value| json!(value)),
                "[a-z]{0,3}".prop_map(Value::String),
            ];
            leaves.prop_recursive(3, 32, 3, |inner| {
                prop_oneof![
                    prop::collection::vec(inner.clone(), 0..4).prop_map(Value::Array),
                    prop::collection::hash_map("[a-z]{0,2}", inner, 0..4)
                        .prop_map(|object| object.into_iter().collect()),
                ]
            })
        }
    }
}
