//! The provider fallback rule: try the primary, then the fallback once, and stop at the first success.
//!
//! This module has no timers and no I/O. The caller wraps each provider call in its own timeout,
//! so a stalled provider counts as a failed attempt.

use std::future::Future;

/// What to do after one attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Next {
    /// The attempt succeeded. Use the provider at `provider_index`.
    Done {
        /// Position of the provider that answered, starting at 0 for the primary.
        provider_index: usize,
    },
    /// The attempt failed and another provider is left to try.
    TryFallback,
    /// The attempt failed and no provider is left.
    Exhausted,
}

/// Decides what happens after attempt `attempt_index` out of `providers_left` providers.
#[must_use]
pub fn next_step<T, E>(
    providers_left: usize,
    attempt_index: usize,
    outcome: &Result<T, E>,
) -> Next {
    match outcome {
        Ok(_) => Next::Done {
            provider_index: attempt_index,
        },
        Err(_) if attempt_index + 1 < providers_left => Next::TryFallback,
        Err(_) => Next::Exhausted,
    }
}

/// The provider that answered, and the value it gave.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Answered<T> {
    /// Position of the provider that answered, starting at 0 for the primary.
    pub provider_index: usize,
    /// The value the provider returned.
    pub value: T,
}

/// Every provider failed, or there were none to try.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Exhausted {
    /// How many provider calls were made.
    pub attempts: usize,
}

/// Calls each provider in order through `call`, and stops at the first success.
///
/// `call` receives the provider's position. With no providers, nothing is called.
///
/// # Errors
///
/// Returns [`Exhausted`] when every provider failed, or when there are no providers.
pub async fn run_chain<T, E, F, Fut>(
    providers: usize,
    mut call: F,
) -> Result<Answered<T>, Exhausted>
where
    F: FnMut(usize) -> Fut,
    Fut: Future<Output = Result<T, E>>,
{
    for index in 0..providers {
        let outcome = call(index).await;
        match (next_step(providers, index, &outcome), outcome) {
            (Next::Done { provider_index }, Ok(value)) => {
                return Ok(Answered {
                    provider_index,
                    value,
                });
            }
            (Next::TryFallback, Err(_)) => {}
            // Exhausted: no provider is left to try.
            _ => {
                return Err(Exhausted {
                    attempts: index + 1,
                })
            }
        }
    }
    Err(Exhausted {
        attempts: providers,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use futures_util::FutureExt;
    use proptest::prelude::*;

    /// Runs the chain over scripted outcomes (`true` = success) and counts the calls made.
    fn simulate(outcomes: &[bool]) -> (Result<Answered<usize>, Exhausted>, usize) {
        let mut calls = 0;
        let result = run_chain(outcomes.len(), |index| {
            calls += 1;
            let outcome = if outcomes[index] {
                Ok(index)
            } else {
                Err("failed")
            };
            futures_util::future::ready(outcome)
        })
        .now_or_never()
        .expect("scripted futures are ready at once");
        (result, calls)
    }

    #[test]
    fn with_no_providers_nothing_is_called() {
        let (result, calls) = simulate(&[]);

        assert_eq!(calls, 0);
        assert_eq!(result.err(), Some(Exhausted { attempts: 0 }));
    }

    #[test]
    fn when_the_primary_succeeds_the_fallback_is_not_called() {
        let (result, calls) = simulate(&[true, true]);

        assert_eq!(calls, 1);
        assert_eq!(result.map(|answer| answer.provider_index).ok(), Some(0));
    }

    #[test]
    fn when_the_primary_fails_the_fallback_answers() {
        let (result, calls) = simulate(&[false, true]);

        assert_eq!(calls, 2);
        assert_eq!(result.map(|answer| answer.provider_index).ok(), Some(1));
    }

    #[test]
    fn when_every_provider_fails_the_chain_is_exhausted() {
        let (result, calls) = simulate(&[false, false]);

        assert_eq!(calls, 2);
        assert_eq!(result.err(), Some(Exhausted { attempts: 2 }));
    }

    #[test]
    fn next_step_stops_at_the_last_provider() {
        let failed: Result<(), &str> = Err("failed");

        assert_eq!(next_step(2, 0, &failed), Next::TryFallback);
        assert_eq!(next_step(2, 1, &failed), Next::Exhausted);
    }

    proptest! {
        // P4: the number of calls depends only on where the first success is, and the chain is
        // exhausted only when every provider failed.
        #[test]
        fn calls_stop_at_the_first_success(outcomes in proptest::collection::vec(any::<bool>(), 0..=3)) {
            let (result, calls) = simulate(&outcomes);
            let expected_calls = match outcomes.iter().position(|ok| *ok) {
                Some(index) => index + 1,
                None => outcomes.len(),
            };

            prop_assert_eq!(calls, expected_calls);
            prop_assert_eq!(result.is_err(), !outcomes.iter().any(|ok| *ok));
        }
    }
}
