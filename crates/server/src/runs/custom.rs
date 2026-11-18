//! `POST /api/runs/ghost`: queues a custom ghost run (ide-workspace T02).
//!
//! A ghost run replays the user's current source on one stored ghost input. The server
//! validates the request and enqueues it; the trusted judge owns execution and the verdict.
//! The checks run in the same fixed order as `POST /api/runs` — cheap and stateless first,
//! then the single ownership lookup, replay, limits and the atomic insert.

use std::net::SocketAddr;

use axum::{
    body::Bytes,
    extract::{rejection::BytesRejection, ConnectInfo, State},
    http::{header, HeaderMap, HeaderName, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use counterspell_common::{ids::RunId, status::RunMode};
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use uuid::Uuid;

use crate::{
    client_ip::client_ip,
    config::RUN_WINDOW_SECS,
    error::ApiError,
    ratelimit::RateLimiter,
    ratelimit::{ip_key, user_key},
    runs::{
        create::{body_error, enforce, idempotency_key, require_json, reuse_error},
        enqueue::{enqueue, existing_replay, EnqueueOutcome, GhostRef, NewRun},
    },
    validate::validate_source,
    AppState,
};

/// The largest a ghost input may be once serialised, matching the `submissions_ghost_ck`
/// bound (`pg_column_size(ghost_input) <= 65536`). Larger inputs are refused before any
/// database work.
const MAX_GHOST_INPUT_BYTES: usize = 65_536;

/// The body of `POST /api/runs/ghost`. Unknown fields are refused at both levels.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GhostRunRequest {
    /// The problem slug, for example `two-sum`.
    problem: String,
    /// The language name, for example `python`.
    lang: String,
    /// The source code, as plain text.
    code: String,
    /// The stored ghost to replay the source on.
    ghost: GhostField,
}

/// The `ghost` member: the source submission and the input it produced.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GhostField {
    /// The earlier submission whose ghost this is. It must belong to the caller.
    submission_id: String,
    /// The ghost input, an arbitrary JSON value.
    input: serde_json::Value,
}

/// The `202` body: the new run's id and the shape the workspace expects.
#[derive(Debug, Serialize)]
struct Accepted {
    run_id: String,
    status: String,
    mode: String,
    custom: bool,
}

/// Accepts one ghost run from a signed-in user, checks it and queues it.
///
/// # Errors
///
/// Returns an [`ApiError`] for each failed check, in the same order as `POST /api/runs`.
/// A `submission_id` that is missing, malformed, or owned by someone else produces
/// `unknown_submission` — whether the row exists is never revealed.
pub async fn post_ghost_run(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Result<Response, ApiError> {
    let body = body.map_err(body_error)?;
    require_json(&headers)?;
    let request: GhostRunRequest = serde_json::from_slice(&body).map_err(|_| {
        ApiError::bad_request(
            "invalid_request",
            "The request must be one JSON object with problem, lang, code and ghost.",
        )
    })?;
    let user = state
        .user_resolver
        .resolve(&headers)
        .await
        .ok_or_else(ApiError::unauthenticated)?;
    let key = idempotency_key(&headers)?;
    let language = state
        .languages
        .find(&request.lang)
        .ok_or_else(|| ApiError::bad_request("invalid_request", "lang is not supported."))?;
    validate_source(&language.name, &request.code, language.max_source_bytes)?;

    if state.problems.published(&request.problem).await.is_none() {
        return Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "unknown_problem",
            "No published problem has that name.",
        ));
    }

    let ghost = check_ghost(&state, user, &request.ghost).await?;

    let mut run = NewRun::new(
        RunId::new(),
        user,
        &request.problem,
        &language.name,
        RunMode::Run,
        &request.code,
        key,
    );
    run.ghost = Some(ghost);
    if let Some(earlier) = existing_replay(&state.pool, &run)
        .await
        .map_err(ApiError::internal)?
    {
        return match earlier {
            EnqueueOutcome::Replayed { run_id } => replayed(&state.pool, run_id).await,
            _ => Err(reuse_error()),
        };
    }

    let limits = &state.config.limits;
    enforce(
        &state
            .limiter
            .hit(&user_key(user), limits.run_rate_user, RUN_WINDOW_SECS)
            .await,
    )?;
    let address = client_ip(peer.ip(), &headers, &state.config.trusted_proxies);
    enforce(
        &state
            .limiter
            .hit(&ip_key(address), limits.run_rate_ip, RUN_WINDOW_SECS)
            .await,
    )?;

    match enqueue(&state.pool, &run, limits)
        .await
        .map_err(ApiError::internal)?
    {
        EnqueueOutcome::Created { run_id } => Ok(accepted(run_id, "queued".to_owned(), false)),
        EnqueueOutcome::Replayed { run_id } => replayed(&state.pool, run_id).await,
        EnqueueOutcome::Reuse => Err(reuse_error()),
        EnqueueOutcome::NotPublished => Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "unknown_problem",
            "No published problem has that name.",
        )),
        EnqueueOutcome::CapReached => Err(ApiError::throttled(
            "queue_cap",
            "You already have the maximum number of runs waiting or running. Wait for one to finish.",
            1,
        )),
        EnqueueOutcome::QueueFull => Err(ApiError::throttled(
            "queue_full",
            "The judge queue is full. Try again shortly.",
            1,
        )),
    }
}

/// Validates the `ghost` member and loads the reference the run carries.
///
/// The submission id must parse, the input must fit the stored bound, and the source
/// submission must exist and belong to the caller — the only database lookup before
/// enqueue. Missing and foreign submissions answer the same `unknown_submission`, so
/// whether another user's run exists is never revealed.
async fn check_ghost(
    state: &AppState,
    user: counterspell_common::ids::UserId,
    ghost: &GhostField,
) -> Result<GhostRef, ApiError> {
    let source_uuid = Uuid::parse_str(&ghost.submission_id).map_err(|_| {
        ApiError::bad_request("invalid_request", "ghost.submission_id must be a run id.")
    })?;
    let input_bytes = serde_json::to_vec(&ghost.input)
        .map_err(|_| ApiError::bad_request("invalid_request", "ghost.input is not JSON."))?;
    if input_bytes.len() > MAX_GHOST_INPUT_BYTES {
        return Err(ApiError::bad_request(
            "ghost_input_too_large",
            "ghost.input is larger than the allowed size.",
        ));
    }

    let owned: Option<Uuid> = sqlx::query_scalar("SELECT user_id FROM submissions WHERE id = $1")
        .bind(source_uuid)
        .fetch_optional(&state.pool)
        .await
        .map_err(ApiError::internal)?;
    if owned != Some(user.as_uuid()) {
        return Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "unknown_submission",
            "No submission has that id.",
        ));
    }
    Ok(GhostRef {
        input: ghost.input.clone(),
        source_id: RunId::from_uuid(source_uuid),
    })
}

async fn replayed(pool: &PgPool, run_id: RunId) -> Result<Response, ApiError> {
    let status: String = sqlx::query_scalar("SELECT status FROM submissions WHERE id = $1")
        .bind(run_id.as_uuid())
        .fetch_one(pool)
        .await
        .map_err(ApiError::internal)?;
    Ok(accepted(run_id, status, true))
}

fn accepted(run_id: RunId, status: String, replay: bool) -> Response {
    let body = Accepted {
        run_id: run_id.to_string(),
        status,
        mode: "run".to_owned(),
        custom: true,
    };
    let mut response = (StatusCode::ACCEPTED, Json(body)).into_response();
    let headers = response.headers_mut();
    if let Ok(location) = HeaderValue::from_str(&format!("/api/runs/{run_id}")) {
        headers.insert(header::LOCATION, location);
    }
    if replay {
        headers.insert(
            HeaderName::from_static("idempotent-replayed"),
            HeaderValue::from_static("true"),
        );
    }
    response
}
