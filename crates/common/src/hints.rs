//! Types shared by the server and the web client for the hints API (`POST /api/hints`).
//!
//! A rung is a level of help, from 1 (a nudge) to 4 (a targeted fix). The types here reject bad
//! values at the edge, so the rest of the server never handles a rung outside 1 to 4.

use std::fmt;

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// A rung of help, always from 1 to 4. Build it with `TryFrom<u8>`; there is no other way in.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(try_from = "u8", into = "u8")]
pub struct HintRung(u8);

/// The number was not a rung. Holds the rejected value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidRung(pub u8);

impl fmt::Display for InvalidRung {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "rung {} is outside 1 to 4", self.0)
    }
}

impl TryFrom<u8> for HintRung {
    type Error = InvalidRung;

    /// Accepts 1 to 4 and rejects every other number.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if (1..=4).contains(&value) {
            Ok(Self(value))
        } else {
            Err(InvalidRung(value))
        }
    }
}

impl From<HintRung> for u8 {
    fn from(rung: HintRung) -> Self {
        rung.0
    }
}

impl HintRung {
    /// The rung number, from 1 to 4.
    #[must_use]
    pub fn get(self) -> u8 {
        self.0
    }
}

/// Where a hint came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum HintSource {
    /// The author wrote it. No AI was involved.
    Ladder,
    /// An AI provider wrote it after a click.
    Ai,
    /// The Efficiency Familiar's speech bubble.
    Familiar,
}

/// The body of `POST /api/hints`. Unknown fields are rejected, so a client cannot send a model,
/// provider or price (R11.4).
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HintRequest {
    /// The finished submission the hint is about.
    pub submission_id: Uuid,
    /// The rung to reveal or generate, from 1 to 4.
    pub rung: HintRung,
    /// `false` (the default) reveals the authored ladder. `true` asks for an AI hint.
    #[serde(default)]
    pub ai: bool,
}

/// The learner's AI quota for the current UTC day.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QuotaView {
    /// AI generations already used today.
    pub used: u32,
    /// The most AI generations allowed each day.
    pub limit: u32,
    /// When the quota resets, as an RFC 3339 time in UTC.
    pub resets_at: String,
}

/// One hint as the client sees it. `label` is `"AI hint"` for AI text and `None` for authored text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HintView {
    /// The rung this hint belongs to.
    pub rung: u8,
    /// Where the text came from.
    pub source: HintSource,
    /// The hint text. Plain text only; clients must never render it as HTML.
    pub body: String,
    /// A label shown next to AI text, such as `"AI hint"`.
    pub label: Option<String>,
    /// The provider name for AI text.
    pub provider: Option<String>,
    /// The model name for AI text.
    pub model: Option<String>,
    /// `true` when the learner's code and ghost were sent to a provider for this text.
    pub sent_to_provider: bool,
    /// The learner's quota, or `None` when the quota store cannot be read.
    pub quota: Option<QuotaView>,
    /// The next authored rung the learner may reveal, or `None` after rung 4.
    pub next_rung: Option<u8>,
}

/// The stable error codes for the hints API. The wire name comes from [`HintErrorCode::as_str`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HintErrorCode {
    /// The request is malformed (bad JSON, unknown field, bad UUID or rung).
    InvalidRequest,
    /// No valid session.
    Unauthenticated,
    /// The submission is unknown, not owned by the caller, or not finished. The body is the same
    /// in all three cases.
    NotFound,
    /// The rung before this one has not been revealed.
    PreviousRungRequired,
    /// The code is too large for an AI hint.
    CodeTooLargeForHint,
    /// The window rate limit was hit.
    RateLimited,
    /// The daily AI quota or the global spend cap was reached.
    AiDailyCapReached,
    /// AI is disabled, Valkey is down, or every provider failed.
    AiUnavailable,
}

impl HintErrorCode {
    /// The stable code the API sends, for example `previous_rung_required`.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::Unauthenticated => "unauthenticated",
            Self::NotFound => "not_found",
            Self::PreviousRungRequired => "previous_rung_required",
            Self::CodeTooLargeForHint => "code_too_large_for_hint",
            Self::RateLimited => "rate_limited",
            Self::AiDailyCapReached => "ai_daily_cap_reached",
            Self::AiUnavailable => "ai_unavailable",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rungs_one_to_four_are_accepted() {
        assert_eq!(HintRung::try_from(1).map(HintRung::get), Ok(1));
        assert_eq!(HintRung::try_from(4).map(HintRung::get), Ok(4));
    }

    #[test]
    fn rungs_zero_and_five_are_rejected() {
        assert_eq!(HintRung::try_from(0), Err(InvalidRung(0)));
        assert_eq!(HintRung::try_from(5), Err(InvalidRung(5)));
    }

    #[test]
    fn request_with_rung_five_is_rejected_when_parsed() {
        let body = r#"{"submission_id":"7d3c5a10-0000-4000-8000-000000000001","rung":5}"#;

        assert!(serde_json::from_str::<HintRequest>(body).is_err());
    }

    #[test]
    fn request_with_an_unknown_field_is_rejected() {
        let body =
            r#"{"submission_id":"7d3c5a10-0000-4000-8000-000000000001","rung":2,"model":"x"}"#;

        assert!(serde_json::from_str::<HintRequest>(body).is_err());
    }

    #[test]
    fn request_with_a_bad_uuid_is_rejected() {
        let body = r#"{"submission_id":"not-a-uuid","rung":2}"#;

        assert!(serde_json::from_str::<HintRequest>(body).is_err());
    }

    #[test]
    fn ai_defaults_to_false_for_a_ladder_request() {
        let body = r#"{"submission_id":"7d3c5a10-0000-4000-8000-000000000001","rung":1}"#;

        let request: HintRequest = serde_json::from_str(body).expect("a valid ladder request");

        assert!(!request.ai);
        assert_eq!(request.rung.get(), 1);
    }

    #[test]
    fn source_serialises_as_its_lowercase_name() {
        let as_text = |source| serde_json::to_string(&source).expect("a source serialises");

        assert_eq!(as_text(HintSource::Ladder), "\"ladder\"");
        assert_eq!(as_text(HintSource::Ai), "\"ai\"");
        assert_eq!(as_text(HintSource::Familiar), "\"familiar\"");
    }

    #[test]
    fn error_codes_have_the_stable_wire_names() {
        assert_eq!(
            HintErrorCode::PreviousRungRequired.as_str(),
            "previous_rung_required"
        );
        assert_eq!(
            HintErrorCode::CodeTooLargeForHint.as_str(),
            "code_too_large_for_hint"
        );
        assert_eq!(
            HintErrorCode::AiDailyCapReached.as_str(),
            "ai_daily_cap_reached"
        );
        assert_eq!(HintErrorCode::AiUnavailable.as_str(), "ai_unavailable");
    }
}
