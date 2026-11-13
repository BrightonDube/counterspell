//! `POST /api/runs/{id}/cancel`: asks the judge to stop a run (design Path 8, requirement R17).
//!
//! The server only sets `jobs.cancel_requested_at` and sends a wake-up. The judge makes the state change,
//! so the server cannot write a result. A cancel uses no Run/Submit quota.

use std::str::FromStr;

use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use counterspell_common::ids::RunId;
use serde::Serialize;
use sqlx::PgPool;

use crate::{error::ApiError, runs::create::require_json, AppState};

/// The cancel reply: the run and its status when the request arrived.
#[derive(Debug, Serialize)]
struct CancelReply {
    run_id: String,
    status: String,
}

/// Sets the cancel flag on the caller's run, once, if its job is still queued or claimed. The
/// ownership check is part of the query. Returns the run status when a row changed.
const CANCEL_SQL: &str = r"
    UPDATE jobs j
       SET cancel_requested_at = COALESCE(j.cancel_requested_at, now()),
           updated_at = now()
      FROM submissions s
     WHERE j.submission_id = s.id
       AND s.id = $1 AND s.user_id = $2
       AND j.state IN ('queued', 'claimed')
    RETURNING s.status
";

/// Asks the judge to cancel one of the caller's runs.
///
/// A run that is still queued or claimed gets `202` with its status. A finished run gets `200` with its
/// final status and is not changed. Any other run, or a run that is not the caller's, gets the same `404`.
///
/// # Errors
///
/// `401` without a session, `415` for a non-JSON content type, `404 not_found` as above, and `500` for a
/// database failure with no detail shown.
pub async fn cancel_run(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
) -> Result<Response, ApiError> {
    let user = state
        .user_resolver
        .resolve(&headers)
        .await
        .ok_or_else(ApiError::unauthorized)?;
    require_json(&headers)?;
    let not_found = || ApiError::not_found("Run not found.");
    let run_id = RunId::from_str(&id).map_err(|_| not_found())?;

    let flagged: Option<String> = sqlx::query_scalar(CANCEL_SQL)
        .bind(run_id.as_uuid())
        .bind(user.as_uuid())
        .fetch_optional(&state.pool)
        .await
        .map_err(ApiError::internal)?;
    if let Some(status) = flagged {
        wake_judges(&state.pool, run_id).await?;
        return Ok(reply(StatusCode::ACCEPTED, run_id, status));
    }

    let status: String =
        sqlx::query_scalar("SELECT status FROM submissions WHERE id = $1 AND user_id = $2")
            .bind(run_id.as_uuid())
            .bind(user.as_uuid())
            .fetch_optional(&state.pool)
            .await
            .map_err(ApiError::internal)?
            .ok_or_else(not_found)?;
    Ok(reply(StatusCode::OK, run_id, status))
}

/// Wakes the judge. The notification carries only the run id, and it is a hint: the judge still reads
/// the flag from Postgres.
async fn wake_judges(pool: &PgPool, run_id: RunId) -> Result<(), ApiError> {
    sqlx::query("SELECT pg_notify('run_cancel', $1), pg_notify('run_jobs', '')")
        .bind(run_id.to_string())
        .execute(pool)
        .await
        .map_err(ApiError::internal)?;
    Ok(())
}

fn reply(status: StatusCode, run_id: RunId, run_status: String) -> Response {
    (
        status,
        Json(CancelReply {
            run_id: run_id.to_string(),
            status: run_status,
        }),
    )
        .into_response()
}
