//! Runtime read path for pinned catalogue versions (problem-catalogue C24, CAT-L4, CAT-R4).
//!
//! The judge loads the version a submission was pinned to, never the latest one (P9). A `Run`
//! loads only the public examples. A `Submit` also loads the hidden tests. Every read uses the
//! judge role's grants, and nothing here writes.

use serde_json::Value;
use sqlx::{PgConnection, PgPool};
use std::future::Future;
use std::str::FromStr;

use counterspell_common::familiar::class::ClassId;
use counterspell_common::status::RunMode;
use counterspell_engine::{Comparator, Schema};

use crate::sandbox::problem::{BenchDecl, ProblemError, ProblemSpec, ProblemTests, TestCase};

/// Which rows a load needs. `Run` never touches hidden data, so it cannot leak it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoadMode {
    /// Public samples only.
    Run,
    /// Public samples and every hidden test.
    Submit,
}

/// A public example input and its expected output.
#[derive(Debug, Clone, PartialEq)]
pub struct Example {
    /// One-based position in the statement.
    pub ordinal: i32,
    /// The input value.
    pub input: Value,
    /// The expected output value.
    pub expected: Value,
}

/// A hidden test case. Loaded only for `LoadMode::Submit`.
#[derive(Debug, Clone, PartialEq)]
pub struct HiddenCase {
    /// One-based position in the hidden set.
    pub ordinal: i32,
    /// The input value.
    pub input: Value,
    /// The expected output value.
    pub expected: Value,
}

/// One pinned problem version, as the judge needs it to run tests.
#[derive(Debug, Clone, PartialEq)]
pub struct PinnedProblem {
    /// The problem slug.
    pub slug: String,
    /// The version that was loaded.
    pub version: i32,
    /// The driver-protocol signature, written to `signature.json` inside the sandbox.
    pub signature: Value,
    /// The engine input schema the bench sizes inputs with.
    pub input_schema: Value,
    /// `exact` or `unordered`, matching the version row.
    pub comparator: String,
    /// The version's limits object; `time_ms` inside it can override the language default.
    pub limits: Value,
    /// The author's declared complexity class, when one was declared.
    pub reference_complexity: Option<String>,
    /// The version's bench declaration (`size_param` names a signature parameter), when set.
    pub bench: Option<Value>,
    /// Public examples, in order.
    pub examples: Vec<Example>,
    /// Hidden tests. Empty unless the load mode was `Submit`.
    pub hidden: Vec<HiddenCase>,
}

/// A failed read. This is an infrastructure fault, not a missing problem.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// A database read failed, for example a missing grant or a lost connection.
    #[error("catalogue read failed")]
    Database(#[from] sqlx::Error),
}

/// Loads `slug` at `version`. Returns `Ok(None)` when no such version exists.
///
/// The publication state is not checked here. A queued run must still find the version it was
/// pinned to, even after the problem was unpublished (P9).
///
/// # Errors
///
/// Returns [`StoreError::Database`] when a read fails, for example when the connection lacks
/// the judge grants.
pub async fn load(
    conn: &mut PgConnection,
    slug: &str,
    version: i32,
    mode: LoadMode,
) -> Result<Option<PinnedProblem>, StoreError> {
    let row: Option<VersionRow> = sqlx::query_as(
        "SELECT signature, input_schema, comparator, limits, reference_complexity, bench \
             FROM problem_versions WHERE problem_slug = $1 AND version = $2",
    )
    .bind(slug)
    .bind(version)
    .fetch_optional(&mut *conn)
    .await?;
    let Some((signature, input_schema, comparator, limits, reference_complexity, bench)) = row
    else {
        return Ok(None);
    };

    let example_rows: Vec<(i32, Value, Value)> = sqlx::query_as(
        "SELECT ordinal, input, expected FROM problem_examples \
         WHERE problem_slug = $1 AND version = $2 ORDER BY ordinal",
    )
    .bind(slug)
    .bind(version)
    .fetch_all(&mut *conn)
    .await?;

    let hidden = match mode {
        LoadMode::Run => Vec::new(),
        LoadMode::Submit => {
            let rows: Vec<(i32, Value, Value)> = sqlx::query_as(
                "SELECT ordinal, input, expected FROM hidden_tests \
                 WHERE problem_slug = $1 AND version = $2 ORDER BY ordinal",
            )
            .bind(slug)
            .bind(version)
            .fetch_all(&mut *conn)
            .await?;
            rows.into_iter()
                .map(|(ordinal, input, expected)| HiddenCase {
                    ordinal,
                    input,
                    expected,
                })
                .collect()
        }
    };

    Ok(Some(PinnedProblem {
        slug: slug.to_string(),
        version,
        signature,
        input_schema,
        comparator,
        limits,
        reference_complexity,
        bench,
        examples: example_rows
            .into_iter()
            .map(|(ordinal, input, expected)| Example {
                ordinal,
                input,
                expected,
            })
            .collect(),
        hidden,
    }))
}

/// A known ghost for one pinned version. Judge only, used to match an autopsy (CAT-L4).
#[derive(Debug, Clone, PartialEq)]
pub struct KnownGhostRow {
    /// Stable id within the pack.
    pub ghost_id: String,
    /// `wrong`, `crash` or `timeout`.
    pub kind: String,
    /// The failing input.
    pub input: Value,
    /// What the program returned, when there is a value.
    pub got: Option<Value>,
    /// The exception class for a crash.
    pub exception_class: Option<String>,
    /// The authored explanation, one line.
    pub explanation: String,
    /// Concept ids the ghost teaches.
    pub concepts: Vec<String>,
}

/// Returns the Python reference solution for `slug` at `version`, or `None` if there is none.
///
/// # Errors
///
/// Returns [`StoreError::Database`] when the read fails, for example without the judge grant.
pub async fn reference_for(
    conn: &mut PgConnection,
    slug: &str,
    version: i32,
) -> Result<Option<String>, StoreError> {
    let code: Option<String> = sqlx::query_scalar(
        "SELECT code FROM reference_solutions \
         WHERE problem_slug = $1 AND version = $2 AND lang = 'python'",
    )
    .bind(slug)
    .bind(version)
    .fetch_optional(&mut *conn)
    .await?;
    Ok(code)
}

/// One row of `known_ghosts` as the judge reads it, in column order.
type GhostRecord = (
    String,
    String,
    Value,
    Option<Value>,
    Option<String>,
    String,
    Vec<String>,
);

/// The spec columns `load` reads off a version row, in select order.
type VersionRow = (Value, Value, String, Value, Option<String>, Option<Value>);

/// Returns the known ghosts recorded for `slug` at `version`, in insertion order of their ids.
///
/// # Errors
///
/// Returns [`StoreError::Database`] when the read fails, for example without the judge grant.
pub async fn known_ghosts_for(
    conn: &mut PgConnection,
    slug: &str,
    version: i32,
) -> Result<Vec<KnownGhostRow>, StoreError> {
    let rows: Vec<GhostRecord> = sqlx::query_as(
        "SELECT ghost_id, kind, input, got, exception_class, explanation, concepts \
             FROM known_ghosts WHERE problem_slug = $1 AND version = $2 ORDER BY ghost_id",
    )
    .bind(slug)
    .bind(version)
    .fetch_all(&mut *conn)
    .await?;
    Ok(rows
        .into_iter()
        .map(
            |(ghost_id, kind, input, got, exception_class, explanation, concepts)| KnownGhostRow {
                ghost_id,
                kind,
                input,
                got,
                exception_class,
                explanation,
                concepts,
            },
        )
        .collect())
}

/// A bench declaration as the version row stores it: `size_param` names the signature
/// parameter whose size grows.
#[derive(serde::Deserialize)]
struct StoredBench {
    size_param: String,
    sizes: Vec<u64>,
}

/// The production problem loader: reads the pinned catalogue version through the judge
/// role's `SELECT`-only grants (problem-catalogue X1, CAT-L4). It never writes.
#[derive(Debug, Clone)]
pub struct PgProblemTests {
    pool: PgPool,
}

impl PgProblemTests {
    /// A loader reading through `pool`, which must connect as the judge role.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

/// The decode failures that turn a stored version into a [`ProblemError`]: every one is
/// broken catalogue data, which is an infrastructure fault, never an absent problem.
fn store(reason: impl std::fmt::Display) -> ProblemError {
    ProblemError::Store(reason.to_string())
}

/// Turns a stored input column into the signature-ordered call arguments. A pack writes
/// `input` as the argument array; anything else is broken catalogue data.
fn case_args(input: Value) -> Result<Vec<Value>, ProblemError> {
    serde_json::from_value(input).map_err(store)
}

/// Resolves a bench's `size_param` name to its index in the signature parameters.
fn bench_decl(signature: &Value, bench: Value) -> Result<BenchDecl, ProblemError> {
    let stored: StoredBench = serde_json::from_value(bench).map_err(store)?;
    let parameters = signature
        .get("parameters")
        .and_then(Value::as_array)
        .ok_or_else(|| store("signature has no parameters array"))?;
    let size_param = parameters
        .iter()
        .position(|parameter| {
            parameter.get("name").and_then(Value::as_str) == Some(stored.size_param.as_str())
        })
        .ok_or_else(|| store("bench size_param names no signature parameter"))?;
    Ok(BenchDecl {
        sizes: stored.sizes,
        size_param,
    })
}

/// Builds the executor's spec from a pinned version and its reference source.
fn pinned_spec(
    pinned: PinnedProblem,
    reference_python: Option<String>,
) -> Result<ProblemSpec, ProblemError> {
    // The grimoire validator guarantees 'exact' or 'unordered'; anything else defaults
    // to exact exactly as `Meta::engine_comparator` does for a checked pack.
    let comparator = match pinned.comparator.as_str() {
        "unordered" => Comparator::Unordered,
        _ => Comparator::Exact,
    };
    let time_limit_ms = pinned
        .limits
        .get("time_ms")
        .and_then(Value::as_u64)
        .and_then(|ms| u32::try_from(ms).ok());
    // The bench trio is mapped together so a partial declaration reaches the executor
    // as an incomplete bench, which it reports as `bench_declaration_incomplete`.
    let bench = pinned
        .bench
        .map(|bench| bench_decl(&pinned.signature, bench))
        .transpose()?;
    let generator_schema = if bench.is_some() {
        Some(serde_json::from_value::<Schema>(pinned.input_schema).map_err(store)?)
    } else {
        None
    };
    let reference_complexity = pinned
        .reference_complexity
        .map(|text| ClassId::from_str(&text).map_err(store))
        .transpose()?;
    let mut tests: Vec<TestCase> = Vec::with_capacity(pinned.examples.len() + pinned.hidden.len());
    for example in &pinned.examples {
        tests.push(TestCase {
            ordinal: u32::try_from(example.ordinal).map_err(store)?,
            is_sample: true,
            args: case_args(example.input.clone())?,
            expected: example.expected.clone(),
        });
    }
    for hidden in &pinned.hidden {
        tests.push(TestCase {
            ordinal: u32::try_from(hidden.ordinal).map_err(store)?,
            is_sample: false,
            args: case_args(hidden.input.clone())?,
            expected: hidden.expected.clone(),
        });
    }
    Ok(ProblemSpec {
        slug: pinned.slug,
        version: u32::try_from(pinned.version).map_err(store)?,
        signature: pinned.signature,
        comparator,
        time_limit_ms,
        tests,
        reference_python,
        bench,
        generator_schema,
        reference_complexity,
    })
}

impl ProblemTests for PgProblemTests {
    fn load(
        &self,
        slug: &str,
        version: u32,
        mode: RunMode,
    ) -> impl Future<Output = Result<Option<ProblemSpec>, ProblemError>> + Send {
        let pool = self.pool.clone();
        let slug = slug.to_owned();
        async move {
            let version =
                i32::try_from(version).map_err(|_| store("pinned version out of range"))?;
            let load_mode = match mode {
                RunMode::Run => LoadMode::Run,
                RunMode::Submit => LoadMode::Submit,
            };
            let mut conn = pool.acquire().await.map_err(store)?;
            let Some(pinned) = load(&mut conn, &slug, version, load_mode)
                .await
                .map_err(store)?
            else {
                return Ok(None);
            };
            // The design reads the reference for every run, not only ghost hunts.
            let reference = reference_for(&mut conn, &slug, version)
                .await
                .map_err(store)?;
            pinned_spec(pinned, reference).map(Some)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn pinned() -> PinnedProblem {
        PinnedProblem {
            slug: "count-pairs".to_owned(),
            version: 1,
            signature: json!({
                "protocol_version": 1,
                "function": "count_pairs",
                "parameters": [
                    {"name": "values", "type": {"kind": "list"}},
                    {"name": "target", "type": {"kind": "int"}}
                ],
                "returns": {"kind": "int"}
            }),
            input_schema: json!({
                "type": "tuple",
                "items": [
                    {"type": "int_array", "min_len": 0, "max_len": 30, "min": -50, "max": 50},
                    {"type": "int", "min": -50, "max": 50}
                ]
            }),
            comparator: "exact".to_owned(),
            limits: json!({"time_ms": 2000, "mem_mb": 256}),
            reference_complexity: Some("n^2".to_owned()),
            bench: Some(json!({"size_param": "values", "sizes": [16, 64]})),
            examples: vec![Example {
                ordinal: 1,
                input: json!([[1, 2, 3, 2], 4]),
                expected: json!(2),
            }],
            hidden: vec![HiddenCase {
                ordinal: 1,
                input: json!([[5, 5], 10]),
                expected: json!(1),
            }],
        }
    }

    #[test]
    fn pinned_spec_maps_every_column() {
        let spec = pinned_spec(pinned(), Some("def f():\n    return 0\n".to_owned()))
            .expect("a well-formed version maps");
        assert_eq!(spec.version, 1);
        assert_eq!(spec.comparator, Comparator::Exact);
        assert_eq!(spec.time_limit_ms, Some(2000));
        assert_eq!(spec.reference_complexity, Some(ClassId::Quadratic));
        let bench = spec.bench.expect("a declared bench maps");
        assert_eq!(bench.size_param, 0, "values is parameter 0");
        assert_eq!(bench.sizes, vec![16, 64]);
        assert!(spec.generator_schema.is_some());
        assert_eq!(spec.tests.len(), 2);
        assert_eq!(spec.tests[0].args, vec![json!([1, 2, 3, 2]), json!(4)]);
        assert!(spec.tests[0].is_sample);
        assert!(!spec.tests[1].is_sample);
        assert_eq!(
            spec.reference_python.as_deref(),
            Some("def f():\n    return 0\n")
        );
    }

    #[test]
    fn a_bench_param_outside_the_signature_is_a_store_error() {
        let mut pinned = pinned();
        pinned.bench = Some(json!({"size_param": "missing", "sizes": [16]}));
        assert!(
            pinned_spec(pinned, None).is_err(),
            "a bench that names no parameter is broken catalogue data"
        );
    }

    #[test]
    fn a_case_input_that_is_not_an_args_array_is_a_store_error() {
        let mut pinned = pinned();
        pinned.hidden[0].input = json!({"not": "an array"});
        assert!(pinned_spec(pinned, None).is_err());
    }
}
