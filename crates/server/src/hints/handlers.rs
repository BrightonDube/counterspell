//! HTTP handlers for the hints API (hints spec H13): `POST /api/hints` and `GET /api/hints`.
//!
//! The order of checks is fixed and cheap first: the body, the learner, the window rate limit, the
//! submission's owner and finish state, the ladder gate, the cache, the code size, and only then
//! the quota and the provider. Every "not yours or not finished" case returns the same `404`.

use std::{collections::BTreeSet, fmt::Display};

use axum::{
    extract::{rejection::JsonRejection, rejection::QueryRejection, Query, State},
    http::{header, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use counterspell_common::{
    hints::{HintErrorCode, HintRequest, HintSource, HintView, QuotaView},
    ids::UserId,
};
use serde::Deserialize;
use serde_json::{json, Map, Value};
use sqlx::PgPool;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use uuid::Uuid;

use crate::{
    error::ApiError,
    hints::{
        ai::{self, AiError, Deps},
        cache::{ast_hash, cache_key},
        config::HintConfig,
        guard::{GhostView, PromptContext, PromptKind, MAX_CODE_BYTES},
        ladder::{gate, Gate},
        provider::ReqwestTransport,
        quota::{next_utc_midnight, QuotaStore},
        store::{self, AiHint, NewAiHint, Submission},
    },
    ratelimit::{Decision, RateLimiter},
    seams::CurrentUser,
    AppState,
};

/// The window the per-user hint request limit is counted over, in seconds (ten minutes).
const WINDOW_SECS: u64 = 600;

/// Everything the hints handlers need beyond the shared server state.
pub struct HintState {
    /// Postgres, for submissions, authored text and stored hints.
    pub pool: PgPool,
    /// Providers, limits and prices. Empty providers turn AI off.
    pub config: HintConfig,
    /// Quota and spend counters. `None` turns AI off, because the cost controls cannot be checked.
    pub quota: Option<QuotaStore>,
    /// The HTTP transport for provider calls.
    pub transport: ReqwestTransport,
}

impl HintState {
    /// A state with AI off. Ladder reveals still work.
    #[must_use]
    pub fn disabled(pool: PgPool) -> Self {
        Self {
            pool,
            config: HintConfig::disabled(),
            quota: None,
            transport: ReqwestTransport::new(),
        }
    }
}

/// The query string for `GET /api/hints`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SubmissionQuery {
    /// The submission whose hints to list.
    submission_id: Uuid,
}

/// `POST /api/hints`: reveals an authored rung, or returns an AI hint for a rung from 2 to 4.
pub async fn post_hint(
    State(app): State<AppState>,
    CurrentUser(learner): CurrentUser,
    body: Result<Json<HintRequest>, JsonRejection>,
) -> Response {
    let Ok(Json(request)) = body else {
        return invalid("Send JSON with submission_id and rung (1 to 4), and no other fields.");
    };
    let user = learner.as_uuid();
    if let Some(wait) = hit_window(&app, user).await {
        return rate_limited(wait);
    }
    let hints = app.hints.clone();
    let submission = match store::load_submission(&hints.pool, user, request.submission_id).await {
        Ok(Some(submission)) => submission,
        Ok(None) => return not_found(),
        Err(error) => return internal(error),
    };
    let revealed = match store::revealed_rungs(&hints.pool, submission.id).await {
        Ok(revealed) => revealed,
        Err(error) => return internal(error),
    };
    let rung = request.rung.get();
    if request.ai {
        ai_hint(&hints, user, &submission, rung, &revealed).await
    } else {
        ladder_hint(&hints, user, &submission, rung, &revealed).await
    }
}

/// `GET /api/hints?submission_id=`: the revealed ladder rungs and stored AI hints, so the panel can
/// be restored after a reload.
pub async fn get_hints(
    State(app): State<AppState>,
    CurrentUser(learner): CurrentUser,
    query: Result<Query<SubmissionQuery>, QueryRejection>,
) -> Response {
    let Ok(Query(query)) = query else {
        return invalid("Ask with a submission_id query parameter and no other parameters.");
    };
    let user = learner.as_uuid();
    let hints = app.hints.clone();
    let submission = match store::load_submission(&hints.pool, user, query.submission_id).await {
        Ok(Some(submission)) => submission,
        Ok(None) => return not_found(),
        Err(error) => return internal(error),
    };
    let ladder = match store::ladder_rows(&hints.pool, submission.id).await {
        Ok(rows) => rows,
        Err(error) => return internal(error),
    };
    let ai = match store::ai_rows(&hints.pool, submission.id).await {
        Ok(rows) => rows,
        Err(error) => return internal(error),
    };
    let revealed: BTreeSet<u8> = ladder.iter().map(|(rung, _)| *rung).collect();
    let body = json!({
        "ladder": ladder
            .iter()
            .map(|(rung, body)| json!({ "rung": rung, "body": body }))
            .collect::<Vec<_>>(),
        "ai": ai
            .iter()
            .map(|(rung, hint)| json!({
                "rung": rung,
                "body": hint.body,
                "label": "AI hint",
                "provider": hint.provider,
                "model": hint.model,
            }))
            .collect::<Vec<_>>(),
        "next_rung": next_rung(&revealed),
        "quota": quota_view(&hints, user).await,
    });
    (StatusCode::OK, Json(body)).into_response()
}

/// Reveals an authored rung. Returns the stored text when the rung was revealed before.
async fn ladder_hint(
    hints: &HintState,
    user: Uuid,
    submission: &Submission,
    rung: u8,
    revealed: &BTreeSet<u8>,
) -> Response {
    let body = match gate(revealed, rung) {
        Gate::Locked { missing } => return locked(missing),
        Gate::Revealed => match store::ladder_body(&hints.pool, submission.id, rung).await {
            Ok(Some(body)) => body,
            Ok(None) => return internal("a revealed rung has no stored text"),
            Err(error) => return internal(error),
        },
        Gate::Open => {
            let authored =
                match store::authored_body(&hints.pool, &submission.problem_slug, rung).await {
                    Ok(Some(text)) => text,
                    Ok(None) => return not_found(),
                    Err(error) => return internal(error),
                };
            match store::reveal_ladder(&hints.pool, user, submission.id, rung, &authored).await {
                Ok(stored) => stored.body,
                Err(error) => return internal(error),
            }
        }
    };
    let mut after = revealed.clone();
    after.insert(rung);
    let view = HintView {
        rung,
        source: HintSource::Ladder,
        body,
        label: None,
        provider: None,
        model: None,
        sent_to_provider: false,
        quota: quota_view(hints, user).await,
        next_rung: next_rung(&after),
    };
    (StatusCode::OK, Json(view)).into_response()
}

/// Returns the stored AI hint for a rung, or generates and stores one.
async fn ai_hint(
    hints: &HintState,
    user: Uuid,
    submission: &Submission,
    rung: u8,
    revealed: &BTreeSet<u8>,
) -> Response {
    if rung < 2 {
        return invalid("AI hints start at rung 2. Reveal rung 1 from the ladder first.");
    }
    if let Gate::Locked { missing } = gate(revealed, rung) {
        return locked(missing);
    }
    match store::find_ai(&hints.pool, submission.id, rung).await {
        Ok(Some(cached)) => {
            let quota = quota_view(hints, user).await;
            return ai_reply(&cached, rung, revealed, quota);
        }
        Ok(None) => {}
        Err(error) => return internal(error),
    }
    if submission.code.len() > MAX_CODE_BYTES {
        return reply(
            StatusCode::UNPROCESSABLE_ENTITY,
            HintErrorCode::CodeTooLargeForHint,
            "The code is too large for an AI hint. Reveal the next authored hint instead.",
            Map::new(),
        );
    }
    let Some(quota) = hints.quota.as_ref() else {
        return unavailable(revealed);
    };
    let Grounding {
        statement,
        verdict,
        ghost,
    } = match grounding(hints, submission).await {
        Ok(grounding) => grounding,
        Err(error) => return error.into_response(),
    };
    let ghost_view = ghost.as_ref().map(|ghost| GhostView {
        input: &ghost.input,
        expected: &ghost.expected,
        got: &ghost.got,
        failure: &ghost.kind,
    });
    let context = PromptContext {
        statement: &statement,
        code: &submission.code,
        ghost: ghost_view,
        verdict_summary: &verdict,
    };
    let deps = Deps {
        config: &hints.config,
        quota,
        transport: &hints.transport,
    };
    let generated = match ai::generate(
        &deps,
        UserId::from_uuid(user),
        OffsetDateTime::now_utc(),
        PromptKind::Ghost { rung },
        &context,
    )
    .await
    {
        Ok(generated) => generated,
        Err(error) => return ai_failure(hints, &error, revealed),
    };
    let ghost_input = ghost.as_ref().map_or("", |ghost| ghost.input.as_str());
    let key = cache_key(
        &submission.problem_slug,
        rung,
        &ast_hash(&submission.lang, &submission.code),
        ghost_input,
    );
    let stored = match store::insert_ai(
        &hints.pool,
        &NewAiHint {
            user,
            submission: submission.id,
            rung,
            provider: generated.provider,
            model: &generated.model,
            body: &generated.body,
            tokens_in: generated.tokens_in,
            tokens_out: generated.tokens_out,
            cost_micros: generated.cost_micros,
            cost_estimated: generated.cost_estimated,
            cache_key: key,
        },
    )
    .await
    {
        Ok(stored) => stored,
        Err(error) => return internal(error),
    };
    let quota = QuotaView {
        used: generated.quota_used,
        limit: hints.config.daily_limit,
        resets_at: resets_at(OffsetDateTime::now_utc()),
    };
    ai_reply(&stored, rung, revealed, Some(quota))
}

/// The text the prompt is built from: the statement, the verdict and the ghost. It never includes
/// hidden test data.
struct Grounding {
    statement: String,
    verdict: String,
    ghost: Option<store::GhostRow>,
}

/// Loads the grounding for a prompt. A database failure becomes the reply to send.
async fn grounding(hints: &HintState, submission: &Submission) -> Result<Grounding, ApiError> {
    let statement = store::statement_markdown(&hints.pool, &submission.problem_slug)
        .await
        .map_err(ApiError::internal)?
        .unwrap_or_default();
    let ghost = store::ghost_for(&hints.pool, submission.id)
        .await
        .map_err(ApiError::internal)?;
    let verdict = store::verdict_of(&hints.pool, submission.id)
        .await
        .map_err(ApiError::internal)?;
    Ok(Grounding {
        statement,
        verdict: format!(
            "verdict: {}",
            verdict.unwrap_or_else(|| "none yet".to_owned())
        ),
        ghost,
    })
}

/// The reply for an AI hint, stored or new. The label and the notice fields are always set.
fn ai_reply(
    hint: &AiHint,
    rung: u8,
    revealed: &BTreeSet<u8>,
    quota: Option<QuotaView>,
) -> Response {
    let view = HintView {
        rung,
        source: HintSource::Ai,
        body: hint.body.clone(),
        label: Some("AI hint".to_owned()),
        provider: hint.provider.clone(),
        model: hint.model.clone(),
        sent_to_provider: true,
        quota,
        next_rung: next_rung(revealed),
    };
    (StatusCode::OK, Json(view)).into_response()
}

/// Turns a failed AI generation into the matching reply.
fn ai_failure(hints: &HintState, error: &AiError, revealed: &BTreeSet<u8>) -> Response {
    match error {
        AiError::Disabled | AiError::Unavailable | AiError::QuotaUnavailable => {
            unavailable(revealed)
        }
        AiError::CapReached {
            used, resets_at, ..
        } => {
            let resets = resets_at_text(*resets_at);
            let mut extra = Map::new();
            extra.insert(
                "quota".to_owned(),
                json!({ "used": used, "limit": hints.config.daily_limit, "resets_at": resets }),
            );
            extra.insert("resets_at".to_owned(), json!(resets));
            extra.insert("next_rung".to_owned(), json!(next_rung(revealed)));
            reply(
                StatusCode::TOO_MANY_REQUESTS,
                HintErrorCode::AiDailyCapReached,
                "You have used today's AI hints. The next authored hint is still available.",
                extra,
            )
        }
        AiError::Prompt(_) => reply(
            StatusCode::UNPROCESSABLE_ENTITY,
            HintErrorCode::CodeTooLargeForHint,
            "The code is too large for an AI hint. Reveal the next authored hint instead.",
            Map::new(),
        ),
    }
}

/// The learner's quota for today, or `None` when Valkey cannot say.
async fn quota_view(hints: &HintState, user: Uuid) -> Option<QuotaView> {
    let counters = hints.quota.as_ref()?;
    let now = OffsetDateTime::now_utc();
    let count = counters
        .used_today(UserId::from_uuid(user), now)
        .await
        .ok()?;
    Some(QuotaView {
        used: count,
        limit: hints.config.daily_limit,
        resets_at: resets_at(now),
    })
}

/// Counts one hit on the learner's window. Returns the seconds to wait when the window is full.
async fn hit_window(app: &AppState, user: Uuid) -> Option<u64> {
    let key = format!("hints:window:{user}");
    match app
        .limiter
        .hit(&key, app.hints.config.window_limit, WINDOW_SECS)
        .await
    {
        Decision::Limited {
            retry_after_seconds,
        } => Some(retry_after_seconds),
        Decision::Allowed { .. } | Decision::SkippedFailOpen => None,
    }
}

/// The next authored rung the learner has not revealed, or `None` once all four are revealed.
fn next_rung(revealed: &BTreeSet<u8>) -> Option<u8> {
    (1..=4).find(|rung| !revealed.contains(rung))
}

fn resets_at(now: OffsetDateTime) -> String {
    resets_at_text(next_utc_midnight(now))
}

fn resets_at_text(at: OffsetDateTime) -> String {
    at.format(&Rfc3339).unwrap_or_default()
}

/// The error envelope for the hints API: `{code, message}` plus any extra fields.
fn reply(
    status: StatusCode,
    code: HintErrorCode,
    message: &str,
    extra: Map<String, Value>,
) -> Response {
    let mut body = Map::new();
    body.insert("code".to_owned(), json!(code.as_str()));
    body.insert("message".to_owned(), json!(message));
    body.extend(extra);
    (status, Json(Value::Object(body))).into_response()
}

fn invalid(message: &str) -> Response {
    reply(
        StatusCode::BAD_REQUEST,
        HintErrorCode::InvalidRequest,
        message,
        Map::new(),
    )
}

/// The same `404` for an unknown, unowned or unfinished submission, so none of them can be told apart.
fn not_found() -> Response {
    reply(
        StatusCode::NOT_FOUND,
        HintErrorCode::NotFound,
        "No submission was found for that id.",
        Map::new(),
    )
}

fn locked(missing: u8) -> Response {
    let mut extra = Map::new();
    extra.insert("missing".to_owned(), json!(missing));
    reply(
        StatusCode::CONFLICT,
        HintErrorCode::PreviousRungRequired,
        &format!("Reveal rung {missing} first."),
        extra,
    )
}

fn unavailable(revealed: &BTreeSet<u8>) -> Response {
    let mut extra = Map::new();
    extra.insert("next_rung".to_owned(), json!(next_rung(revealed)));
    reply(
        StatusCode::SERVICE_UNAVAILABLE,
        HintErrorCode::AiUnavailable,
        "AI hints are not available right now. The next authored hint is still there.",
        extra,
    )
}

fn rate_limited(wait_seconds: u64) -> Response {
    let mut response = reply(
        StatusCode::TOO_MANY_REQUESTS,
        HintErrorCode::RateLimited,
        "Too many hint requests. Wait a few minutes.",
        Map::new(),
    );
    if let Ok(value) = HeaderValue::from_str(&wait_seconds.to_string()) {
        response.headers_mut().insert(header::RETRY_AFTER, value);
    }
    response
}

/// A server error. The detail is logged by [`ApiError::internal`] and never sent to the learner.
fn internal(detail: impl Display) -> Response {
    ApiError::internal(detail).into_response()
}
