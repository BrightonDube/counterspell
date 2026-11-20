//! HTTP handlers for the three break-room routes.

use axum::{
    body::Bytes,
    extract::{rejection::BytesRejection, State},
    http::{HeaderMap, StatusCode},
    Json,
};
use serde::Serialize;
use serde_json::{json, Value};
use sqlx::Row;
use time::OffsetDateTime;

use crate::{
    breakroom::{
        daily::{daily_index, days_since_epoch, sast_date},
        scores::{check_meta_size, game_tag, score_for, ScoreRequest},
    },
    error::ApiError,
    ratelimit::{Decision, RateLimiter},
    AppState,
};

const SCORES_PER_HOUR: u64 = 30;

#[derive(Serialize)]
pub struct PuzzleOut {
    id: String,
    fen: String,
    moves: String,
    rating: i32,
    themes: Vec<String>,
}

/// `GET /api/breakroom/puzzle/daily` - today's puzzle, public.
pub async fn daily_puzzle(State(state): State<AppState>) -> Result<Json<Value>, ApiError> {
    let count: i64 = sqlx::query_scalar("SELECT count(*) FROM puzzles")
        .fetch_one(&state.pool)
        .await
        .map_err(|e| internal(&e))?;
    let now = state
        .auth
        .as_ref()
        .map_or_else(OffsetDateTime::now_utc, |auth| auth.clock.now());
    let today = sast_date(now);
    let index = daily_index(days_since_epoch(today), count).ok_or_else(|| {
        ApiError::new(
            StatusCode::SERVICE_UNAVAILABLE,
            "not_imported",
            "No puzzles are imported yet.",
        )
    })?;
    let row = sqlx::query(
        "SELECT id, fen, moves, rating, themes FROM puzzles ORDER BY id OFFSET $1 LIMIT 1",
    )
    .bind(index)
    .fetch_one(&state.pool)
    .await
    .map_err(|e| internal(&e))?;
    let themes: String = row.get("themes");
    Ok(Json(json!({
        "date": today.to_string(),
        "puzzle": PuzzleOut {
            id: row.get("id"),
            fen: row.get("fen"),
            moves: row.get("moves"),
            rating: row.get("rating"),
            themes: themes.split_whitespace().map(str::to_owned).collect(),
        },
    })))
}

/// `POST /api/breakroom/scores` - store one score for the signed-in user.
pub async fn save_score(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Result<Bytes, BytesRejection>,
) -> Result<(StatusCode, Json<Value>), ApiError> {
    let user = state
        .user_resolver
        .resolve(&headers)
        .await
        .ok_or_else(ApiError::unauthenticated)?;
    let body =
        body.map_err(|_| ApiError::bad_request("invalid_body", "The body is unreadable."))?;
    let request: ScoreRequest = serde_json::from_slice(&body)
        .map_err(|e| ApiError::bad_request("invalid_body", format!("Invalid score body: {e}")))?;

    let key = format!("rl:breakroom:scores:{}", user.as_uuid());
    if let Decision::Limited {
        retry_after_seconds,
    } = state.limiter.hit(&key, SCORES_PER_HOUR, 3600).await
    {
        return Err(ApiError::rate_limited(retry_after_seconds));
    }

    let now = state
        .auth
        .as_ref()
        .map_or_else(OffsetDateTime::now_utc, |auth| auth.clock.now());
    let today = sast_date(now);
    let (score, meta) = score_for(&request, today)?;
    check_meta_size(&meta)?;

    let inserted: Option<i64> = sqlx::query_scalar(
        "INSERT INTO game_scores (user_id, game, score, meta)          VALUES ($1, $2, $3, $4) ON CONFLICT DO NOTHING RETURNING id",
    )
    .bind(user.as_uuid())
    .bind(game_tag(&request))
    .bind(score)
    .bind(&meta)
    .fetch_optional(&state.pool)
    .await
    .map_err(|e| internal(&e))?;

    match inserted {
        Some(id) => Ok((
            StatusCode::CREATED,
            Json(json!({ "id": id, "score": score })),
        )),
        None => Ok((
            StatusCode::OK,
            Json(json!({ "status": "already_recorded" })),
        )),
    }
}

/// `GET /api/breakroom/scores` - the signed-in user's latest ten rows.
pub async fn list_scores(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Json<Value>, ApiError> {
    let user = state
        .user_resolver
        .resolve(&headers)
        .await
        .ok_or_else(ApiError::unauthenticated)?;
    let rows = sqlx::query(
        "SELECT id, game, score, meta, created_at FROM game_scores          WHERE user_id = $1 ORDER BY created_at DESC LIMIT 10",
    )
    .bind(user.as_uuid())
    .fetch_all(&state.pool)
    .await
    .map_err(|e| internal(&e))?;
    let scores: Vec<Value> = rows
        .iter()
        .map(|r| {
            let created: OffsetDateTime = r.get("created_at");
            json!({
                "id": r.get::<i64, _>("id"),
                "game": r.get::<String, _>("game"),
                "score": r.get::<i32, _>("score"),
                "meta": r.get::<Value, _>("meta"),
                "created_at": created.to_string(),
            })
        })
        .collect();
    Ok(Json(json!({ "scores": scores })))
}

/// Any other `/api/breakroom/*` path is an API error, never the SPA shell.
pub async fn breakroom_not_found() -> ApiError {
    ApiError::not_found("API route not found")
}

fn internal(e: &sqlx::Error) -> ApiError {
    ApiError::new(
        StatusCode::INTERNAL_SERVER_ERROR,
        "internal",
        format!("Break-room store failed: {e}"),
    )
}
