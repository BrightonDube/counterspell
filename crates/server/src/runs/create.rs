//! `POST /api/runs`: accepts one run, checks it and queues it (design Path 1, requirements R2 to R5).
//!
//! The checks run in a fixed order. Cheap, stateless checks come first. A request that fails any of them
//! creates nothing and uses no quota. A replay returns the original run before any rate limit is counted.

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

use crate::{
    client_ip::client_ip,
    config::RUN_WINDOW_SECS,
    error::ApiError,
    ratelimit::{ip_key, user_key, Decision, RateLimiter},
    runs::enqueue::{enqueue, existing_replay, EnqueueOutcome, NewRun},
    validate::validate_source,
    AppState,
};

/// The body of `POST /api/runs`. Unknown fields are refused.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RunRequest {
    /// The problem slug, for example `two-sum`.
    problem: String,
    /// The language name, for example `python`.
    lang: String,
    /// The source code, as plain text.
    code: String,
    /// `run` for samples or `submit` for hidden tests.
    mode: String,
}

/// The `202` body: the run id and its status.
#[derive(Debug, Serialize)]
struct Accepted {
    run_id: String,
    status: String,
}

/// Accepts one run from a signed-in user, checks it and queues it.
///
/// # Errors
///
/// Returns an [`ApiError`] for each failed check. The error never names a table, a query or a
/// connection detail.
pub async fn post_run(
    State(state): State<AppState>,
    ConnectInfo(peer): ConnectInfo<SocketAddr>,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Result<Response, ApiError> {
    let body = body.map_err(body_error)?;
    require_json(&headers)?;
    let request: RunRequest = serde_json::from_slice(&body).map_err(|_| {
        ApiError::bad_request(
            "invalid_request",
            "The request must be one JSON object with problem, lang, code and mode.",
        )
    })?;
    let user = state
        .user_resolver
        .resolve(&headers)
        .await
        .ok_or_else(ApiError::unauthorized)?;
    let key = idempotency_key(&headers)?;
    let mode = RunMode::parse(&request.mode)
        .ok_or_else(|| ApiError::bad_request("invalid_request", "mode must be run or submit."))?;
    let language = state
        .languages
        .find(&request.lang)
        .ok_or_else(|| ApiError::bad_request("invalid_request", "lang is not supported."))?;
    validate_source(&language.name, &request.code, language.max_source_bytes)?;

    if state.problems.published(&request.problem).is_none() {
        return Err(ApiError::new(
            StatusCode::NOT_FOUND,
            "unknown_problem",
            "No published problem has that name.",
        ));
    }

    let run = NewRun::new(
        RunId::new(),
        user,
        &request.problem,
        &language.name,
        mode,
        &request.code,
        key,
    );
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

fn body_error(rejection: BytesRejection) -> ApiError {
    if rejection.into_response().status() == StatusCode::PAYLOAD_TOO_LARGE {
        ApiError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "body_too_large",
            "The request is larger than the allowed size.",
        )
    } else {
        ApiError::bad_request("invalid_request", "The request body could not be read.")
    }
}

pub(crate) fn require_json(headers: &HeaderMap) -> Result<(), ApiError> {
    let is_json = headers
        .get(header::CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.split(';').next())
        .is_some_and(|media| media.trim().eq_ignore_ascii_case("application/json"));
    if is_json {
        Ok(())
    } else {
        Err(ApiError::new(
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "unsupported_media_type",
            "Send the request as application/json.",
        ))
    }
}

/// Reads the optional `Idempotency-Key`: 1 to 128 letters, digits, dots, underscores or hyphens.
fn idempotency_key(headers: &HeaderMap) -> Result<Option<String>, ApiError> {
    let Some(value) = headers.get("idempotency-key") else {
        return Ok(None);
    };
    let text = value
        .to_str()
        .ok()
        .filter(|text| valid_key(text))
        .ok_or_else(|| {
            ApiError::bad_request(
                "invalid_idempotency_key",
                "Idempotency-Key must be 1 to 128 letters, digits, dots, underscores or hyphens.",
            )
        })?;
    Ok(Some(text.to_owned()))
}

fn valid_key(text: &str) -> bool {
    (1..=128).contains(&text.len())
        && text
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'.' | b'_' | b'-'))
}

fn enforce(decision: &Decision) -> Result<(), ApiError> {
    match decision {
        Decision::Limited {
            retry_after_seconds,
        } => Err(ApiError::rate_limited(*retry_after_seconds)),
        Decision::Allowed { .. } | Decision::SkippedFailOpen => Ok(()),
    }
}

fn reuse_error() -> ApiError {
    ApiError::new(
        StatusCode::UNPROCESSABLE_ENTITY,
        "idempotency_key_reuse",
        "This Idempotency-Key was already used for a different program or mode.",
    )
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
