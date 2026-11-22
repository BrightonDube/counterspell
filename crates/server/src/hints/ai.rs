//! The one AI generation pipeline (hints spec H12). Every AI call in the server goes through
//! [`generate`], so every path reserves quota and spend, and settles or releases them the same way.
//!
//! The order is fixed: check that AI is on, build the prompt (which refuses oversized code), reserve
//! the quota and the worst-case spend, call the providers in order, guard the answer, and then
//! record the real cost. A call that fails in every provider gives its reservation back.

use std::time::Duration;

use counterspell_common::ids::UserId;
use thiserror::Error;
use time::OffsetDateTime;

use crate::hints::{
    chain::run_chain,
    config::HintConfig,
    cost::{cost_micros, estimate_tokens, worst_case_micros},
    guard::{build_prompt, sanitise_output, Prompt, PromptContext, PromptError, PromptKind},
    provider::{complete, ChatTransport},
    quota::{next_utc_midnight, CapReason, QuotaStore, Reservation, Reserve},
};

/// What the pipeline needs. Build it per request from the shared settings.
pub struct Deps<'a, T: ChatTransport> {
    /// The provider list, limits and prices.
    pub config: &'a HintConfig,
    /// The quota and spend counters.
    pub quota: &'a QuotaStore,
    /// Sends the provider calls. Production uses the HTTP transport; tests use a scripted one.
    pub transport: &'a T,
}

/// A generated hint and the record of how it was made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Generated {
    /// The guarded plain-text hint.
    pub body: String,
    /// The provider that answered.
    pub provider: &'static str,
    /// The model that answered.
    pub model: String,
    /// Input tokens, reported or estimated.
    pub tokens_in: u32,
    /// Output tokens, reported or estimated.
    pub tokens_out: u32,
    /// The cost in micro-dollars.
    pub cost_micros: i64,
    /// `true` when the token counts were estimated.
    pub cost_estimated: bool,
    /// `true` when the guard removed code from the answer.
    pub removed_code: bool,
    /// The learner's generations used today, including this one.
    pub quota_used: u32,
}

/// Why no hint was generated.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum AiError {
    /// No provider is configured, so AI is off.
    #[error("AI hints are turned off")]
    Disabled,
    /// The learner's daily quota or the global spend cap is used up.
    #[error("the daily AI limit is reached")]
    CapReached {
        /// Which limit stopped the call.
        reason: CapReason,
        /// The learner's generations used today.
        used: u32,
        /// When the limits reset.
        resets_at: OffsetDateTime,
    },
    /// Every configured provider failed, or gave an empty answer.
    #[error("no AI provider answered")]
    Unavailable,
    /// Valkey could not be reached, so AI stays off to protect the spend cap.
    #[error("the hint quota store is unavailable")]
    QuotaUnavailable,
    /// The prompt cannot be built, for example because the code is too large.
    #[error(transparent)]
    Prompt(#[from] PromptError),
}

/// Generates one guarded hint for the learner `user`, using the same quota for every kind.
///
/// # Errors
///
/// Returns an [`AiError`] describing why no hint was produced. The cases are listed on the type.
pub async fn generate<T: ChatTransport>(
    deps: &Deps<'_, T>,
    user: UserId,
    now: OffsetDateTime,
    kind: PromptKind,
    context: &PromptContext<'_>,
) -> Result<Generated, AiError> {
    if deps.config.providers.is_empty() {
        return Err(AiError::Disabled);
    }
    let prompt = build_prompt(kind, context)?;
    generate_from_prompt(deps, user, now, &prompt).await
}

/// Runs the same pipeline as [`generate`] for a prompt the caller already built, such as the
/// Familiar's speech prompt. Quota, spend, provider order and the output guard are identical.
///
/// # Errors
///
/// Returns an [`AiError`] describing why no text was produced. [`AiError::Prompt`] is never returned
/// here, because the caller built the prompt.
pub async fn generate_from_prompt<T: ChatTransport>(
    deps: &Deps<'_, T>,
    user: UserId,
    now: OffsetDateTime,
    prompt: &Prompt,
) -> Result<Generated, AiError> {
    let Some(primary) = deps.config.providers.first() else {
        return Err(AiError::Disabled);
    };
    let worst = worst_case_micros(
        &primary.price,
        prompt.byte_len(),
        deps.config.max_output_tokens,
    );
    let reservation = match deps.quota.reserve(user, now, worst).await {
        Ok(Reserve::Granted(reservation)) => reservation,
        Ok(Reserve::CapReached { reason, used }) => {
            return Err(AiError::CapReached {
                reason,
                used,
                resets_at: next_utc_midnight(now),
            });
        }
        Err(_) => return Err(AiError::QuotaUnavailable),
    };

    let timeout = Duration::from_millis(deps.config.timeout_ms);
    let answered = run_chain(deps.config.providers.len(), |index| {
        complete(
            deps.transport,
            &deps.config.providers[index],
            prompt,
            deps.config.max_output_tokens,
            timeout,
        )
    })
    .await;
    let Ok(answered) = answered else {
        release_quietly(deps.quota, &reservation).await;
        return Err(AiError::Unavailable);
    };

    let spec = &deps.config.providers[answered.provider_index];
    let sanitised = sanitise_output(&answered.value.text);
    if sanitised.text.trim().is_empty() {
        release_quietly(deps.quota, &reservation).await;
        return Err(AiError::Unavailable);
    }
    let (tokens_in, tokens_out, cost_estimated) = match answered.value.usage {
        Some(usage) => (usage.prompt_tokens, usage.completion_tokens, false),
        None => (
            estimate_tokens(prompt.byte_len()),
            estimate_tokens(answered.value.text.len()),
            true,
        ),
    };
    let cost = cost_micros(&spec.price, tokens_in, tokens_out);
    if let Err(error) = deps.quota.settle(&reservation, cost).await {
        tracing::warn!(error = %error, "hint spend settle failed; the day's spend is over-counted");
    }
    Ok(Generated {
        body: sanitised.text,
        provider: spec.name.as_str(),
        model: spec.model.clone(),
        tokens_in,
        tokens_out,
        cost_micros: cost,
        cost_estimated,
        removed_code: sanitised.removed_code,
        quota_used: reservation.used_after(),
    })
}

/// Gives a reservation back. A failure is logged, not returned, because the caller is already
/// reporting why the hint failed.
async fn release_quietly(quota: &QuotaStore, reservation: &Reservation) {
    if let Err(error) = quota.release(reservation).await {
        tracing::warn!(error = %error, "hint quota release failed; the day's count is over-counted");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hints::config::HintConfig;
    use crate::hints::guard::{PromptContext, PromptKind};
    use crate::hints::provider::{ChatResponse, ProviderError, Usage};
    use crate::testkit::{ScriptedTransport, Step};
    use std::collections::HashMap;

    fn config(overrides: &[(&str, &str)]) -> HintConfig {
        let mut vars: HashMap<String, String> = HashMap::from([
            ("OPENAI_API_KEY".to_owned(), "sk-test-openai".to_owned()),
            ("GROQ_API_KEY".to_owned(), "gsk-test-groq".to_owned()),
        ]);
        for (name, value) in overrides {
            vars.insert((*name).to_owned(), (*value).to_owned());
        }
        HintConfig::from_map(&vars).expect("the fake test configuration is valid")
    }

    fn reply(text: &str) -> Step {
        Step::Reply(Ok(ChatResponse {
            text: text.to_owned(),
            usage: Some(Usage {
                prompt_tokens: 100,
                completion_tokens: 20,
            }),
        }))
    }

    fn failure() -> Step {
        Step::Reply(Err(ProviderError::Status(500)))
    }

    fn context(code: &str) -> PromptContext<'_> {
        PromptContext {
            statement: "Count the pairs.",
            code,
            ghost: None,
            verdict_summary: "1 of 4 hidden tests failed",
        }
    }

    fn unreachable_quota() -> QuotaStore {
        QuotaStore::open("redis://127.0.0.1:1/", 10, 5_000_000).expect("a fixed local URL parses")
    }

    fn valkey_quota(limit: u32, cap_micros: i64) -> QuotaStore {
        let url = std::env::var("VALKEY_URL").expect("VALKEY_URL is set for the ignored tests");
        QuotaStore::open(&url, limit, cap_micros).expect("VALKEY_URL is a valid URL")
    }

    fn now() -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }

    #[tokio::test]
    async fn with_no_providers_ai_is_disabled_and_nothing_is_called() {
        let config = config(&[("HINT_PROVIDERS", "")]);
        let quota = unreachable_quota();
        let transport = ScriptedTransport::new(vec![]);
        let deps = Deps {
            config: &config,
            quota: &quota,
            transport: &transport,
        };

        let result = generate(
            &deps,
            UserId::new(),
            now(),
            PromptKind::Ghost { rung: 2 },
            &context("print(1)"),
        )
        .await;

        assert!(matches!(result, Err(AiError::Disabled)));
        assert_eq!(transport.calls(), 0);
    }

    #[tokio::test]
    async fn code_over_the_limit_is_refused_before_any_quota_is_taken() {
        let config = config(&[]);
        let quota = unreachable_quota();
        let transport = ScriptedTransport::new(vec![]);
        let deps = Deps {
            config: &config,
            quota: &quota,
            transport: &transport,
        };
        let big = "x".repeat(crate::hints::guard::MAX_CODE_BYTES + 1);

        let result = generate(
            &deps,
            UserId::new(),
            now(),
            PromptKind::Ghost { rung: 2 },
            &context(&big),
        )
        .await;

        // An unreachable quota store would give QuotaUnavailable, so this proves no reservation ran.
        assert!(matches!(result, Err(AiError::Prompt(_))));
        assert_eq!(transport.calls(), 0);
    }

    #[tokio::test]
    async fn an_unreachable_quota_store_stops_the_call_before_the_provider() {
        let config = config(&[]);
        let quota = unreachable_quota();
        let transport = ScriptedTransport::new(vec![reply("A nudge.")]);
        let deps = Deps {
            config: &config,
            quota: &quota,
            transport: &transport,
        };

        let result = generate(
            &deps,
            UserId::new(),
            now(),
            PromptKind::Ghost { rung: 2 },
            &context("print(1)"),
        )
        .await;

        assert!(matches!(result, Err(AiError::QuotaUnavailable)));
        assert_eq!(
            transport.calls(),
            0,
            "AI fails closed, so no provider is called"
        );
    }

    #[tokio::test]
    #[ignore = "needs VALKEY_URL"]
    async fn valkey_a_successful_primary_is_called_once_and_settled() {
        let config = config(&[]);
        let quota = valkey_quota(10, 5_000_000);
        let transport = ScriptedTransport::new(vec![reply("Look at the second loop.")]);
        let deps = Deps {
            config: &config,
            quota: &quota,
            transport: &transport,
        };

        let generated = generate(
            &deps,
            UserId::new(),
            now(),
            PromptKind::Ghost { rung: 2 },
            &context("print(1)"),
        )
        .await
        .expect("the primary answers");

        assert_eq!(transport.calls(), 1);
        assert_eq!(generated.provider, "openai");
        assert_eq!(generated.body, "Look at the second loop.");
        assert_eq!(generated.tokens_in, 100);
        assert_eq!(generated.tokens_out, 20);
        assert!(!generated.cost_estimated);
        assert_eq!(generated.quota_used, 1);
    }

    #[tokio::test]
    #[ignore = "needs VALKEY_URL"]
    async fn valkey_a_failed_primary_uses_the_fallback_exactly_once() {
        let config = config(&[(
            "HINT_PROVIDERS",
            "openai:gpt-6-luna,groq:openai/gpt-oss-20b",
        )]);
        let quota = valkey_quota(10, 5_000_000);
        let transport = ScriptedTransport::new(vec![failure(), reply("Try a map.")]);
        let deps = Deps {
            config: &config,
            quota: &quota,
            transport: &transport,
        };

        let generated = generate(
            &deps,
            UserId::new(),
            now(),
            PromptKind::Ghost { rung: 3 },
            &context("print(1)"),
        )
        .await
        .expect("the fallback answers");

        assert_eq!(transport.calls(), 2);
        assert_eq!(generated.provider, "groq");
    }

    #[tokio::test]
    #[ignore = "needs VALKEY_URL"]
    async fn valkey_when_every_provider_fails_the_quota_is_released() {
        let config = config(&[(
            "HINT_PROVIDERS",
            "openai:gpt-6-luna,groq:openai/gpt-oss-20b",
        )]);
        let quota = valkey_quota(1, 5_000_000);
        let user = UserId::new();
        let transport = ScriptedTransport::new(vec![failure(), failure(), reply("Second try.")]);
        let deps = Deps {
            config: &config,
            quota: &quota,
            transport: &transport,
        };
        let first = generate(
            &deps,
            user,
            now(),
            PromptKind::Ghost { rung: 2 },
            &context("print(1)"),
        )
        .await;

        let second = generate(
            &deps,
            user,
            now(),
            PromptKind::Ghost { rung: 2 },
            &context("print(1)"),
        )
        .await;

        assert!(matches!(first, Err(AiError::Unavailable)));
        assert!(
            second.is_ok(),
            "the failed call gave its generation back, so the one allowed generation is still there"
        );
    }

    #[tokio::test]
    #[ignore = "needs VALKEY_URL"]
    async fn valkey_the_daily_limit_stops_the_call_before_the_provider() {
        let config = config(&[("AI_DAILY_LIMIT", "0")]);
        let quota = valkey_quota(0, 5_000_000);
        let transport = ScriptedTransport::new(vec![reply("Never sent.")]);
        let deps = Deps {
            config: &config,
            quota: &quota,
            transport: &transport,
        };

        let result = generate(
            &deps,
            UserId::new(),
            now(),
            PromptKind::Ghost { rung: 2 },
            &context("print(1)"),
        )
        .await;

        assert!(matches!(result, Err(AiError::CapReached { .. })));
        assert_eq!(transport.calls(), 0);
    }

    #[tokio::test]
    #[ignore = "needs VALKEY_URL"]
    async fn valkey_concurrent_calls_at_the_limit_stop_at_the_limit() {
        let config = config(&[]);
        let quota = valkey_quota(2, 5_000_000);
        let user = UserId::new();
        let transport = ScriptedTransport::new((0..5).map(|_| reply("A nudge.")).collect());
        let deps = Deps {
            config: &config,
            quota: &quota,
            transport: &transport,
        };

        let ctx = context("print(1)");
        let calls =
            (0..5).map(|_| generate(&deps, user, now(), PromptKind::Ghost { rung: 2 }, &ctx));
        let results = futures_util::future::join_all(calls).await;

        let granted = results.iter().filter(|result| result.is_ok()).count();
        assert_eq!(granted, 2, "exactly the daily limit succeeds");
        assert_eq!(transport.calls(), 2, "refused calls never reach a provider");
    }

    #[tokio::test]
    #[ignore = "needs VALKEY_URL"]
    async fn valkey_the_familiar_uses_the_same_quota_as_a_ghost_hint() {
        let config = config(&[]);
        let quota = valkey_quota(1, 5_000_000);
        let user = UserId::new();
        let transport = ScriptedTransport::new(vec![reply("A nudge."), reply("Never sent.")]);
        let deps = Deps {
            config: &config,
            quota: &quota,
            transport: &transport,
        };
        generate(
            &deps,
            user,
            now(),
            PromptKind::Ghost { rung: 2 },
            &context("print(1)"),
        )
        .await
        .expect("the ghost hint uses the one generation");

        let familiar = generate(
            &deps,
            user,
            now(),
            PromptKind::Familiar,
            &context("print(1)"),
        )
        .await;

        assert!(matches!(familiar, Err(AiError::CapReached { .. })));
        assert_eq!(transport.calls(), 1);
    }
}
