//! `GET /api/runs/{id}`: one run's current or final result, for reloads and history (design Path 10).
//!
//! Every query includes the owner. A missing run, a malformed id and another user's run all give the same
//! `404`. Sample details are copied through a fixed whitelist of three fields, so nothing else stored in
//! the JSON column can reach the client.

use std::str::FromStr;

use axum::{
    extract::{Path, State},
    http::HeaderMap,
    Json,
};
use counterspell_common::{ids::RunId, status::RunStatus};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::{error::ApiError, AppState};

/// The run read model. Absent sections are left out, never invented.
#[derive(Debug, Serialize)]
pub struct RunView {
    /// The run's id.
    pub run_id: String,
    /// The problem slug.
    pub problem: String,
    /// The language name.
    pub lang: String,
    /// `run` or `submit`.
    pub mode: String,
    /// The current lifecycle status.
    pub status: String,
    /// True once the run has a final result.
    pub terminal: bool,
    /// The verdict, set only for a completed run.
    pub verdict: Option<String>,
    /// Why the run ended without a verdict, if it did.
    pub failure_reason: Option<String>,
    /// The attempt that is running or ran last, starting at 1.
    pub attempt: i32,
    /// Hidden and sample tests that passed.
    pub passed: Option<i32>,
    /// Tests run in total.
    pub total: Option<i32>,
    /// The score out of 100, when the run is scored.
    pub score: Option<i32>,
    /// Runtime in milliseconds.
    pub runtime_ms: Option<i32>,
    /// Peak memory in kilobytes.
    pub memory_kb: Option<i32>,
    /// When the run was saved, in UTC.
    pub created_at: String,
    /// When a judge first claimed the run, in UTC.
    pub started_at: Option<String>,
    /// When the run reached its final result, in UTC.
    pub finished_at: Option<String>,
    /// The highest event sequence number stored for this run.
    pub last_seq: i32,
    /// One entry per test that has a result, in order.
    pub tests: Vec<TestView>,
}

/// One test's public result. Hidden tests have no `detail`.
#[derive(Debug, Serialize)]
pub struct TestView {
    /// The test's position in the run, starting at 1.
    pub ordinal: i32,
    /// Whether the test is a visible sample.
    pub sample: bool,
    /// The per-test verdict, such as `pass` or `wrong_answer`.
    pub verdict: String,
    /// Time the test took, in milliseconds.
    pub time_ms: i32,
    /// Input, expected and actual output. Samples only.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<DetailView>,
}

/// The only fields a sample's detail may carry to the client.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct DetailView {
    /// The test input, as text.
    pub input: String,
    /// The expected output, as text.
    pub expected: String,
    /// What the program printed, as text.
    pub got: String,
}

#[derive(Debug, sqlx::FromRow)]
struct RunRow {
    id: uuid::Uuid,
    problem_slug: String,
    lang: String,
    mode: String,
    status: String,
    verdict: Option<String>,
    failure_reason: Option<String>,
    attempt: i32,
    passed: Option<i32>,
    total: Option<i32>,
    score: Option<i32>,
    runtime_ms: Option<i32>,
    memory_kb: Option<i32>,
    created_at: String,
    started_at: Option<String>,
    finished_at: Option<String>,
    last_seq: i32,
}

#[derive(Debug, sqlx::FromRow)]
struct TestRow {
    ordinal: i32,
    is_sample: bool,
    verdict: String,
    time_ms: i32,
    public_detail: Option<serde_json::Value>,
}

/// The owner check is part of the query, so a run that is not the caller's is simply not found.
const RUN_SQL: &str = r#"
    SELECT s.id, s.problem_slug, s.lang, s.mode, s.status, s.verdict, s.failure_reason,
           COALESCE(j.attempts, 0) + 1 AS attempt, s.passed, s.total, s.score,
           s.runtime_ms, s.memory_kb,
           to_char(s.created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS created_at,
           to_char(s.started_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS started_at,
           to_char(s.finished_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS.MS"Z"') AS finished_at,
           s.last_event_seq AS last_seq
      FROM submissions s
      LEFT JOIN jobs j ON j.submission_id = s.id
     WHERE s.id = $1 AND s.user_id = $2
"#;

/// Returns the caller's run. Only the caller's own runs are found.
///
/// # Errors
///
/// `401` when no user is signed in. `404 not_found` for a missing, malformed or foreign run, with the same
/// body in every case. `500` for a database failure, with no detail shown.
pub async fn get_run(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Json<RunView>, ApiError> {
    let user = state
        .user_resolver
        .resolve(&headers)
        .ok_or_else(ApiError::unauthorized)?;
    let not_found = || ApiError::not_found("Run not found.");
    let run_id = RunId::from_str(&id).map_err(|_| not_found())?;

    let row: RunRow = sqlx::query_as(RUN_SQL)
        .bind(run_id.as_uuid())
        .bind(user.as_uuid())
        .fetch_optional(&state.pool)
        .await
        .map_err(ApiError::internal)?
        .ok_or_else(not_found)?;
    let tests = load_tests(&state.pool, row.id).await?;

    let terminal = RunStatus::parse(&row.status).is_some_and(RunStatus::is_terminal);
    Ok(Json(RunView {
        run_id: RunId::from_uuid(row.id).to_string(),
        problem: row.problem_slug,
        lang: row.lang,
        mode: row.mode,
        status: row.status,
        terminal,
        verdict: row.verdict,
        failure_reason: row.failure_reason,
        attempt: row.attempt,
        passed: row.passed,
        total: row.total,
        score: row.score,
        runtime_ms: row.runtime_ms,
        memory_kb: row.memory_kb,
        created_at: row.created_at,
        started_at: row.started_at,
        finished_at: row.finished_at,
        last_seq: row.last_seq,
        tests,
    }))
}

/// Loads the test results of a run the caller has already been shown to own.
async fn load_tests(pool: &PgPool, run: uuid::Uuid) -> Result<Vec<TestView>, ApiError> {
    let rows: Vec<TestRow> = sqlx::query_as(
        "SELECT ordinal, is_sample, verdict, time_ms, public_detail \
         FROM test_results WHERE submission_id = $1 ORDER BY ordinal",
    )
    .bind(run)
    .fetch_all(pool)
    .await
    .map_err(ApiError::internal)?;

    rows.into_iter().map(test_view).collect()
}

fn test_view(row: TestRow) -> Result<TestView, ApiError> {
    let detail = if row.is_sample {
        match row.public_detail {
            Some(value) => Some(serde_json::from_value(value).map_err(ApiError::internal)?),
            None => None,
        }
    } else {
        None
    };
    Ok(TestView {
        ordinal: row.ordinal,
        sample: row.is_sample,
        verdict: row.verdict,
        time_ms: row.time_ms,
        detail,
    })
}
