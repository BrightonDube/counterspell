//! Intake for browser Content-Security-Policy violation reports (PF08 stage 0).
//!
//! `POST /api/csp-report` is a pure data sink: it parses each report, writes one bounded log line,
//! and answers `204`. The origin guard exempts this exact path (see `auth::csrf`) because browsers
//! send reports without `Origin`/`Sec-Fetch-Site` and may attach the session cookie — a bounded,
//! rate-limited sink has nothing a forged POST could abuse.

use axum::{
    body::to_bytes,
    extract::State,
    http::{Request, StatusCode},
    response::{IntoResponse, Response},
};
use serde_json::{Map, Value};

use crate::{
    error::ApiError,
    ratelimit::{Decision, RateLimiter},
    AppState,
};

/// The route path, shared with the origin-guard exemption in `auth::csrf`.
pub const PATH: &str = "/api/csp-report";

/// The largest accepted report body, in bytes (8 KiB — a real report is about 1 KiB).
const MAX_REPORT_BYTES: usize = 8 * 1024;

/// One shared bucket for all reporters — a public write path must bound its log volume.
const REPORTS_PER_MINUTE: u64 = 60;

/// Fields are clipped before logging so a padded report stays one readable line.
const MAX_FIELD_CHARS: usize = 200;

/// `POST /api/csp-report`: parse the report, log it, answer `204`.
pub async fn receive(State(state): State<AppState>, request: Request) -> Response {
    if let Decision::Limited {
        retry_after_seconds,
    } = state
        .limiter
        .hit("rl:csp-report", REPORTS_PER_MINUTE, 60)
        .await
    {
        return ApiError::rate_limited(retry_after_seconds).into_response();
    }
    let (_, body) = request.into_parts();
    let Ok(bytes) = to_bytes(body, MAX_REPORT_BYTES).await else {
        return ApiError::new(
            StatusCode::PAYLOAD_TOO_LARGE,
            "report_too_large",
            "The report is too large.",
        )
        .into_response();
    };
    let Ok(value) = serde_json::from_slice::<Value>(&bytes) else {
        return ApiError::bad_request("invalid_report", "The report body is not JSON.")
            .into_response();
    };
    for report in reports(&value) {
        log_report(report);
    }
    StatusCode::NO_CONTENT.into_response()
}

/// The report objects inside a body: legacy `{"csp-report": {…}}`, a `report-to` batch array, or a
/// bare object.
fn reports(value: &Value) -> Vec<&Map<String, Value>> {
    if let Some(report) = value.get("csp-report").and_then(Value::as_object) {
        return vec![report];
    }
    if let Some(batch) = value.as_array() {
        return batch
            .iter()
            .filter_map(|item| {
                item.get("body")
                    .and_then(Value::as_object)
                    .or_else(|| item.as_object())
            })
            .collect();
    }
    value.as_object().into_iter().collect()
}

fn field<'a>(report: &'a Map<String, Value>, names: &[&str]) -> &'a str {
    names
        .iter()
        .find_map(|name| report.get(*name).and_then(Value::as_str))
        .unwrap_or("")
}

fn clip(text: &str) -> &str {
    let mut end = MAX_FIELD_CHARS.min(text.len());
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    &text[..end]
}

fn log_report(report: &Map<String, Value>) {
    tracing::info!(
        target: "csp-report",
        document = %clip(field(report, &["document-uri", "documentURI", "url"])),
        directive = %clip(field(
            report,
            &[
                "effective-directive",
                "effectiveDirective",
                "violated-directive",
                "violatedDirective",
            ],
        )),
        blocked = %clip(field(report, &["blocked-uri", "blockedURI", "blockedURL"])),
        disposition = %clip(field(report, &["disposition"])),
        source = %clip(field(report, &["source-file", "sourceFile", "sourceURL"])),
        "csp violation report"
    );
}
