//! `POST /api/runs/{id}/familiar-hint`: the Familiar's speech bubble for one finished run.
//!
//! The route reads the caller's own run, then hands every decision to [`speak`]. The response has no
//! mood field: the verdict and the mood were stored before this route could be called, and nothing on
//! this path writes them.

use std::str::FromStr;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};
use counterspell_common::ids::{RunId, UserId};
use serde::Serialize;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};

use crate::{
    error::ApiError,
    familiar::{
        prefs::{from_stored, Speech as SpeechPref},
        speech::{speak, Bubble, Source, Speech, TemplateReason},
        store::load_run_facts,
    },
    hints::{
        ai::Deps,
        quota::{next_utc_midnight, QuotaStore},
    },
    ratelimit::{user_key, RateLimiter},
    runs::create::enforce,
    seams::CurrentUser,
    AppState,
};

/// Speech requests allowed per user in one window.
const RATE_LIMIT: u64 = 30;
/// The window for [`RATE_LIMIT`], in seconds.
const RATE_WINDOW_SECS: u64 = 600;

/// The label shown with AI text.
const AI_LABEL: &str = "AI hint";

/// The learner's AI quota, as the bubble response reports it.
#[derive(Debug, Serialize)]
pub struct QuotaBody {
    used: u32,
    limit: u32,
    resets_at: String,
}

/// The bubble response. It has no mood field on purpose.
#[derive(Debug, Serialize)]
pub struct BubbleBody {
    run_id: String,
    source: &'static str,
    label: Option<&'static str>,
    body: String,
    provider: Option<String>,
    model: Option<String>,
    sent_to_provider: bool,
    reason: Option<&'static str>,
    quota: Option<QuotaBody>,
}

/// Returns the speech bubble for the caller's finished run.
///
/// # Errors
///
/// `401 unauthenticated` without a session. `429` when the caller used the window's allowance.
/// `404 not_found` for an unknown, malformed or foreign run, with one identical body. `409
/// run_not_finished` while the caller's run is still going. `422 familiar_not_applicable` when the run
/// has no stored mood. `500` for a database failure, with no detail shown.
pub async fn post_familiar_hint(
    State(state): State<AppState>,
    CurrentUser(user): CurrentUser,
    Path(id): Path<String>,
) -> Result<Json<BubbleBody>, ApiError> {
    enforce(
        &state
            .limiter
            .hit(
                &format!("familiar-hint:{}", user_key(user)),
                RATE_LIMIT,
                RATE_WINDOW_SECS,
            )
            .await,
    )?;
    let not_found = || ApiError::not_found("Run not found.");
    let run = RunId::from_str(&id).map_err(|_| not_found())?;
    let facts = load_run_facts(&state.pool, run.as_uuid(), user.as_uuid())
        .await
        .map_err(ApiError::internal)?
        .ok_or_else(not_found)?;
    if !facts.finished {
        return Err(ApiError::new(
            StatusCode::CONFLICT,
            "run_not_finished",
            "This run has not finished yet.",
        ));
    }
    if facts.mood.is_none() {
        return Err(ApiError::new(
            StatusCode::UNPROCESSABLE_ENTITY,
            "familiar_not_applicable",
            "This run has no Familiar result to explain.",
        ));
    }

    let speech_off = speech_is_off(&state, user).await?;
    let now = OffsetDateTime::now_utc();
    let hints = state.hints.clone();
    let deps = hints.quota.as_ref().map(|quota| Deps {
        config: &hints.config,
        quota,
        transport: &hints.transport,
    });
    let Some(marker) = state.familiar_attempt.as_ref() else {
        return Err(ApiError::internal(
            "the speech attempt marker is not set up",
        ));
    };
    let speech = Speech {
        pool: &state.pool,
        ai: deps.as_ref(),
        marker,
    };
    let bubble = speak(&speech, user, &facts, speech_off, now)
        .await
        .map_err(ApiError::internal)?;
    let quota = quota_body(
        hints.quota.as_ref(),
        hints.config.daily_limit,
        user,
        &bubble,
        now,
    )
    .await;
    Ok(Json(body_of(run, &bubble, quota)))
}

/// Reads the caller's `speech` preference. A missing row means the session is stale.
async fn speech_is_off(state: &AppState, user: UserId) -> Result<bool, ApiError> {
    let stored: serde_json::Value =
        sqlx::query_scalar("SELECT familiar_prefs FROM users WHERE id = $1")
            .bind(user.as_uuid())
            .fetch_optional(&state.pool)
            .await
            .map_err(ApiError::internal)?
            .ok_or_else(ApiError::unauthenticated)?;
    Ok(from_stored(&stored).speech == SpeechPref::Off)
}

/// The quota to report: the count the call knows, or today's count from Valkey, or nothing.
async fn quota_body(
    store: Option<&QuotaStore>,
    limit: u32,
    user: UserId,
    bubble: &Bubble,
    now: OffsetDateTime,
) -> Option<QuotaBody> {
    let count = match bubble.quota_used {
        Some(count) => count,
        None => store?.used_today(user, now).await.ok()?,
    };
    Some(QuotaBody {
        used: count,
        limit,
        resets_at: next_utc_midnight(now).format(&Rfc3339).unwrap_or_default(),
    })
}

fn body_of(run: RunId, bubble: &Bubble, quota: Option<QuotaBody>) -> BubbleBody {
    let is_ai = bubble.source == Source::Ai;
    BubbleBody {
        run_id: run.to_string(),
        source: if is_ai { "ai" } else { "template" },
        label: is_ai.then_some(AI_LABEL),
        body: bubble.body.clone(),
        provider: bubble.provider.clone(),
        model: bubble.model.clone(),
        sent_to_provider: bubble.sent_to_provider,
        reason: bubble.reason.map(TemplateReason::code),
        quota,
    }
}
