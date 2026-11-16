//! `GET` and `PATCH /api/me/familiar`: the signed-in user's mascot preferences.
//!
//! Every query is scoped to the caller. The only write is to the `familiar_prefs` column of the caller's
//! own row, and a patch is merged in the database in one statement, so two patches cannot lose each
//! other's fields.

use axum::{
    body::Bytes,
    extract::State,
    http::HeaderMap,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::Value;

use crate::{
    error::ApiError,
    familiar::prefs::{check_size, from_stored, to_json, validate_patch, PrefsError},
    ratelimit::{user_key, RateLimiter},
    runs::create::{enforce, require_json},
    AppState,
};

/// Preference changes allowed per user in one window.
const PREFS_RATE_LIMIT: u64 = 60;
/// The window for [`PREFS_RATE_LIMIT`], in seconds.
const PREFS_WINDOW_SECS: u64 = 600;

/// Returns the caller's preferences with every default filled in.
///
/// # Errors
///
/// `401` without a session, and `500` for a database failure with no detail shown.
pub async fn get_prefs(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    let user = state
        .user_resolver
        .resolve(&headers)
        .await
        .ok_or_else(ApiError::unauthorized)?;
    let stored: Value = sqlx::query_scalar("SELECT familiar_prefs FROM users WHERE id = $1")
        .bind(user.as_uuid())
        .fetch_optional(&state.pool)
        .await
        .map_err(ApiError::internal)?
        .ok_or_else(ApiError::unauthorized)?;
    Ok(Json(to_json(&from_stored(&stored))).into_response())
}

/// Sets the caller's preferences. Only the fields named in the body change.
///
/// # Errors
///
/// `401` without a session, `415` for a non-JSON body, `429` when the caller has used the window's
/// allowance, `400 invalid_request` for an over-size, malformed, unknown or wrongly typed body, and
/// `500` for a database failure with no detail shown.
pub async fn patch_prefs(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response, ApiError> {
    let user = state
        .user_resolver
        .resolve(&headers)
        .await
        .ok_or_else(ApiError::unauthorized)?;
    require_json(&headers)?;
    enforce(
        &state
            .limiter
            .hit(&user_key(user), PREFS_RATE_LIMIT, PREFS_WINDOW_SECS)
            .await,
    )?;
    check_size(body.len()).map_err(refusal)?;
    let value: Value = serde_json::from_slice(&body)
        .map_err(|_| ApiError::bad_request("invalid_request", "Preferences must be valid JSON."))?;
    validate_patch(&value).map_err(refusal)?;

    // `||` keeps every stored field the patch does not name. The patch holds only checked fields.
    let stored: Value = sqlx::query_scalar(
        r"UPDATE users
             SET familiar_prefs = familiar_prefs || $2::jsonb,
                 updated_at = now()
           WHERE id = $1
       RETURNING familiar_prefs",
    )
    .bind(user.as_uuid())
    .bind(value.to_string())
    .fetch_optional(&state.pool)
    .await
    .map_err(ApiError::internal)?
    .ok_or_else(ApiError::unauthorized)?;
    Ok(Json(to_json(&from_stored(&stored))).into_response())
}

fn refusal(error: PrefsError) -> ApiError {
    ApiError::bad_request("invalid_request", error.message())
}
