//! Runtime read path for pinned catalogue versions (problem-catalogue C24, CAT-L4, CAT-R4).
//!
//! The judge loads the version a submission was pinned to, never the latest one (P9). A `Run`
//! loads only the public examples. A `Submit` also loads the hidden tests. Every read uses the
//! judge role's grants, and nothing here writes.

use serde_json::Value;
use sqlx::PgConnection;

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
    let exists: Option<i32> = sqlx::query_scalar(
        "SELECT version FROM problem_versions WHERE problem_slug = $1 AND version = $2",
    )
    .bind(slug)
    .bind(version)
    .fetch_optional(&mut *conn)
    .await?;
    if exists.is_none() {
        return Ok(None);
    }

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
