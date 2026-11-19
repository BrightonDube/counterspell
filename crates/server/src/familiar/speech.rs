//! The speech bubble orchestration: one place that decides between a stored bubble, an AI sentence and
//! the fixed template.
//!
//! The order is fixed. A stored bubble comes first and costs nothing. Then come the cheap refusals
//! (speech off, code too large), then the one-attempt marker, then the shared AI pipeline, then the
//! claim guard, and last the single write. Every failure ends in the template built from the stored
//! facts, so the learner always gets a sentence.
//!
//! This module never reads or writes the mood. The mood and the verdict are set before the bubble is
//! requested, and the bubble text cannot change them.

use counterspell_common::{
    familiar::{payload::ComplexitySection, ClassId},
    ids::UserId,
};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use time::OffsetDateTime;

use crate::{
    familiar::{
        attempt::AttemptMarker,
        claims::{check_claims, Verdict},
        prompt::{build_prompt, GhostFacts, Measured, PromptError, PromptInput, CODE_LIMIT_BYTES},
        sections::{sections, SectionError, StoredMeasure},
        store::{self, NewBubble, RunFacts},
    },
    hints::{
        ai::{generate_from_prompt, AiError, Deps},
        guard::Prompt as HintPrompt,
        provider::ChatTransport,
        store::{ghost_for, GhostRow},
    },
};

/// Why a template is shown instead of AI text. The codes are the API's `reason` values.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TemplateReason {
    /// The learner turned speech off.
    SpeechOff,
    /// No AI provider is configured.
    AiDisabled,
    /// The daily quota or the global spend cap is used up.
    AiCapReached,
    /// Every provider failed, or the quota store could not be reached.
    AiUnavailable,
    /// The code is larger than the prompt accepts.
    CodeTooLarge,
    /// The AI text named a class that was not measured, or used a forbidden phrase.
    ClaimMismatch,
    /// This run already used its one attempt.
    AlreadyAttempted,
}

impl TemplateReason {
    /// The stable code the API returns.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::SpeechOff => "speech_off",
            Self::AiDisabled => "ai_disabled",
            Self::AiCapReached => "ai_cap_reached",
            Self::AiUnavailable => "ai_unavailable",
            Self::CodeTooLarge => "code_too_large",
            Self::ClaimMismatch => "claim_mismatch",
            Self::AlreadyAttempted => "already_attempted",
        }
    }
}

/// Where the bubble text came from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Source {
    /// AI text, stored or fresh. The UI labels it "AI hint".
    Ai,
    /// The fixed sentence built from stored facts.
    Template,
}

/// One speech bubble. It has no mood field on purpose.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Bubble {
    /// AI text or the template.
    pub source: Source,
    /// Plain text.
    pub body: String,
    /// The provider, for AI text.
    pub provider: Option<String>,
    /// The model, for AI text.
    pub model: Option<String>,
    /// Why the template was used. `None` for AI text.
    pub reason: Option<TemplateReason>,
    /// `true` when the learner's code was sent to a provider for this bubble.
    pub sent_to_provider: bool,
    /// The learner's AI generations used today, when this call knows it.
    pub quota_used: Option<u32>,
}

/// Why no bubble could be made at all. Each case is a server-side problem, not a learner-facing one.
#[derive(Debug)]
pub enum SpeechError {
    /// The database failed.
    Database(sqlx::Error),
    /// The run has no stored mood, so there are no facts to explain.
    NoMeasuredMood,
    /// A stored value does not have the shape the judge writes.
    BadStoredFacts(SectionError),
}

impl std::fmt::Display for SpeechError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Database(error) => write!(f, "database error: {error}"),
            Self::NoMeasuredMood => f.write_str("the run has no stored mood"),
            Self::BadStoredFacts(error) => write!(f, "stored facts are malformed: {error}"),
        }
    }
}

impl std::error::Error for SpeechError {}

impl From<sqlx::Error> for SpeechError {
    fn from(error: sqlx::Error) -> Self {
        Self::Database(error)
    }
}

/// What [`speak`] needs. Build it per request from the shared state.
pub struct Speech<'a, T: ChatTransport> {
    /// Postgres, for the stored bubble and the run's ghost.
    pub pool: &'a PgPool,
    /// The AI pipeline's settings, quota and transport.
    pub ai: &'a Deps<'a, T>,
    /// The one-attempt marker.
    pub marker: &'a AttemptMarker,
}

/// Produces the bubble for one finished run that `user` owns.
///
/// # Errors
///
/// Returns [`SpeechError`] when the database fails, the run has no stored mood, or a stored value is
/// malformed. Every provider, quota, marker and guard failure is a template, not an error.
pub async fn speak<T: ChatTransport>(
    speech: &Speech<'_, T>,
    user: UserId,
    facts: &RunFacts,
    speech_off: bool,
    now: OffsetDateTime,
) -> Result<Bubble, SpeechError> {
    let measure = StoredMeasure {
        mode: facts.mode,
        mood: facts.mood,
        reason: facts.reason.clone(),
        fit: facts.fit.clone(),
    };
    let read = sections(&measure).map_err(SpeechError::BadStoredFacts)?;
    let familiar = read.familiar.ok_or(SpeechError::NoMeasuredMood)?;

    if let Some(stored) = store::find_bubble(speech.pool, facts.run, user.as_uuid()).await? {
        return Ok(Bubble {
            source: Source::Ai,
            body: stored.body,
            provider: stored.provider,
            model: stored.model,
            reason: None,
            sent_to_provider: true,
            quota_used: None,
        });
    }

    let template = |reason: TemplateReason, sent: bool, quota_used: Option<u32>| Bubble {
        source: Source::Template,
        body: familiar.headline.clone(),
        provider: None,
        model: None,
        reason: Some(reason),
        sent_to_provider: sent,
        quota_used,
    };

    if speech_off {
        return Ok(template(TemplateReason::SpeechOff, false, None));
    }
    if facts.code.len() > CODE_LIMIT_BYTES {
        return Ok(template(TemplateReason::CodeTooLarge, false, None));
    }
    match speech.marker.claim(facts.run).await {
        Ok(true) => {}
        Ok(false) => return Ok(template(TemplateReason::AlreadyAttempted, false, None)),
        Err(_) => return Ok(template(TemplateReason::AiUnavailable, false, None)),
    }

    let ghost = ghost_for(speech.pool, facts.run).await?;
    let Ok((prompt, allowed)) = grounded_prompt(facts, read.complexity.as_ref(), ghost.as_ref())
    else {
        return Ok(template(TemplateReason::CodeTooLarge, false, None));
    };

    let generated = match generate_from_prompt(speech.ai, user, now, &prompt).await {
        Ok(generated) => generated,
        Err(error) => return Ok(template_for_ai_error(&error, &template)),
    };

    if check_claims(&generated.body, &allowed) != Verdict::Ok {
        return Ok(template(
            TemplateReason::ClaimMismatch,
            true,
            Some(generated.quota_used),
        ));
    }

    let stored = store::insert_bubble(
        speech.pool,
        &NewBubble {
            user: user.as_uuid(),
            run: facts.run,
            provider: generated.provider,
            model: &generated.model,
            body: &generated.body,
            tokens_in: generated.tokens_in,
            tokens_out: generated.tokens_out,
            cost_micros: generated.cost_micros,
            cost_estimated: generated.cost_estimated,
            cache_key: bubble_key(facts.run),
        },
    )
    .await?;
    Ok(Bubble {
        source: Source::Ai,
        body: stored.body,
        provider: stored.provider,
        model: stored.model,
        reason: None,
        sent_to_provider: true,
        quota_used: Some(generated.quota_used),
    })
}

/// Builds the speech prompt from stored facts only, and returns it with the classes the AI text may
/// name: the observed class, the plausible ones and the reference.
fn grounded_prompt(
    facts: &RunFacts,
    complexity: Option<&ComplexitySection>,
    ghost: Option<&GhostRow>,
) -> Result<(HintPrompt, Vec<ClassId>), PromptError> {
    let observed = complexity.and_then(|section| section.class);
    let reference = complexity.map(|section| section.reference);
    let plausible: Vec<ClassId> = complexity.map(|s| s.plausible.clone()).unwrap_or_default();
    let summary = verdict_summary(facts);
    let input = PromptInput {
        code: &facts.code,
        verdict_summary: &summary,
        mode: facts.mode.as_str(),
        measured: Measured {
            observed,
            confidence: complexity.map(|section| section.confidence),
            plausible: &plausible,
            reference,
        },
        ghost: ghost.map(|row| GhostFacts {
            input: &row.input,
            expected: &row.expected,
            got: &row.got,
        }),
    };
    let prompt = build_prompt(&input)?;
    let mut allowed = plausible;
    allowed.extend(observed);
    allowed.extend(reference);
    Ok((
        HintPrompt {
            system: prompt.system,
            user: prompt.user,
        },
        allowed,
    ))
}

/// Maps a failed AI call to the template and the reason code shown with it.
fn template_for_ai_error(
    error: &AiError,
    template: &impl Fn(TemplateReason, bool, Option<u32>) -> Bubble,
) -> Bubble {
    match error {
        AiError::Disabled => template(TemplateReason::AiDisabled, false, None),
        AiError::CapReached { used, .. } => {
            template(TemplateReason::AiCapReached, false, Some(*used))
        }
        AiError::Unavailable => template(TemplateReason::AiUnavailable, true, None),
        AiError::QuotaUnavailable => template(TemplateReason::AiUnavailable, false, None),
        AiError::Prompt(_) => template(TemplateReason::CodeTooLarge, false, None),
    }
}

/// A short summary with counts only. It never carries a test input or an expected value.
fn verdict_summary(facts: &RunFacts) -> String {
    let verdict = facts.verdict.as_deref().unwrap_or("unknown");
    match (facts.passed, facts.total) {
        (Some(passed), Some(total)) => format!("{verdict}, {passed} of {total} tests passed"),
        _ => verdict.to_owned(),
    }
}

/// The 32-byte key the `hints` table requires for a familiar row. It is the hash of the run id, so it
/// reveals nothing and the same run always gives the same key.
fn bubble_key(run: uuid::Uuid) -> [u8; 32] {
    let mut hasher = Sha256::new();
    hasher.update(b"familiar-bubble:");
    hasher.update(run.as_bytes());
    hasher.finalize().into()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn facts(verdict: Option<&str>, passed: Option<i32>, total: Option<i32>) -> RunFacts {
        RunFacts {
            run: uuid::Uuid::from_u128(1),
            code: "x".to_owned(),
            mode: counterspell_common::status::RunMode::Submit,
            verdict: verdict.map(str::to_owned),
            passed,
            total,
            mood: Some(1),
            reason: Some("verdict_wrong_answer".to_owned()),
            fit: None,
            finished: true,
        }
    }

    // Breaks if the summary starts to carry anything beyond the verdict code and the two counts.
    #[test]
    fn the_summary_holds_the_verdict_and_counts_only() {
        let text = verdict_summary(&facts(Some("wrong_answer"), Some(3), Some(12)));

        assert_eq!(text, "wrong_answer, 3 of 12 tests passed");
    }

    // Breaks if a run without counts invents numbers.
    #[test]
    fn the_summary_without_counts_is_the_verdict_alone() {
        assert_eq!(
            verdict_summary(&facts(Some("time_limit"), None, None)),
            "time_limit"
        );
        assert_eq!(verdict_summary(&facts(None, None, None)), "unknown");
    }

    // Breaks if two runs share a key, or one run gets a different key each time.
    #[test]
    fn the_bubble_key_is_stable_per_run_and_differs_between_runs() {
        let one = uuid::Uuid::from_u128(1);
        let two = uuid::Uuid::from_u128(2);

        assert_eq!(bubble_key(one), bubble_key(one));
        assert_ne!(bubble_key(one), bubble_key(two));
    }

    // Breaks if a reason code the API documents changes spelling.
    #[test]
    fn the_reason_codes_are_the_documented_strings() {
        let codes: Vec<&str> = [
            TemplateReason::SpeechOff,
            TemplateReason::AiDisabled,
            TemplateReason::AiCapReached,
            TemplateReason::AiUnavailable,
            TemplateReason::CodeTooLarge,
            TemplateReason::ClaimMismatch,
            TemplateReason::AlreadyAttempted,
        ]
        .iter()
        .map(|reason| reason.code())
        .collect();

        assert_eq!(
            codes,
            [
                "speech_off",
                "ai_disabled",
                "ai_cap_reached",
                "ai_unavailable",
                "code_too_large",
                "claim_mismatch",
                "already_attempted"
            ]
        );
    }
}
