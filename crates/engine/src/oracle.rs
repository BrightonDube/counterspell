//! The Oracle seam between the pure engine and the sandboxed judge, plus clocks.
//!
//! The engine never runs a program. It asks an [`Oracle`] for the user's
//! result and the trusted reference's result on one input, and it reads time
//! only through a [`Clock`].

use crate::classify::Observation;
use serde_json::Value;
use std::cell::Cell;
use std::fmt::{self, Display, Formatter};
use std::time::{Duration, Instant};

/// An infrastructure failure reported by an Oracle.
///
/// It means the sandbox or image failed, never that the user was wrong. The
/// hunt turns it into an `Aborted` outcome.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OracleError {
    message: String,
}

impl OracleError {
    /// Creates an infrastructure error with a short, plain-English message.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }

    /// Returns the message the Oracle gave.
    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl Display for OracleError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.message)
    }
}

impl std::error::Error for OracleError {}

/// The only way the engine reaches a program.
///
/// Contract for implementers (the judge's sandbox Oracle):
///
/// - Run the user's program and the trusted reference on `input`, each with
///   at most `limit`, and return both raw results in one [`Observation`].
/// - Kill a program that runs past `limit` from outside, and report it as
///   `TimedOut`. Return within `limit` plus a small slack.
/// - Treat everything the user's program prints as untrusted text.
/// - Return `Err` for any infrastructure fault. Never turn one into a user
///   failure.
/// - Receive public inputs only: candidates, samples and authored known ghosts.
pub trait Oracle {
    /// Evaluates one input and returns both results, or an infrastructure error.
    ///
    /// # Errors
    ///
    /// Returns [`OracleError`] when the sandbox or reference infrastructure fails.
    fn evaluate(&mut self, input: &Value, limit: Duration) -> Result<Observation, OracleError>;
}

/// A monotonic time source. Only the difference between two readings matters.
pub trait Clock {
    /// Returns the time elapsed since this clock started.
    fn now(&self) -> Duration;
}

/// The real clock, backed by `std::time::Instant`.
#[derive(Debug)]
pub struct SystemClock {
    start: Instant,
}

impl SystemClock {
    /// Starts a clock at the current instant.
    #[must_use]
    pub fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl Default for SystemClock {
    fn default() -> Self {
        Self::new()
    }
}

impl Clock for SystemClock {
    fn now(&self) -> Duration {
        self.start.elapsed()
    }
}

/// A clock that moves only when a test or a replay says so.
#[derive(Debug, Default)]
pub struct ManualClock {
    now: Cell<Duration>,
}

impl ManualClock {
    /// Starts a manual clock at zero.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Moves the clock forward by `by`.
    pub fn advance(&self, by: Duration) {
        self.now.set(self.now.get().saturating_add(by));
    }
}

impl Clock for ManualClock {
    fn now(&self) -> Duration {
        self.now.get()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify::{classify, Classification, ProgramResult};
    use crate::compare::Comparator;
    use serde_json::json;

    struct EchoOracle;

    impl Oracle for EchoOracle {
        fn evaluate(
            &mut self,
            input: &Value,
            _limit: Duration,
        ) -> Result<Observation, OracleError> {
            Ok(Observation {
                user: ProgramResult::Returned(input.clone()),
                reference: ProgramResult::Returned(input.clone()),
            })
        }
    }

    #[test]
    fn oracle_is_usable_through_a_trait_object() {
        let mut oracle = EchoOracle;
        let dynamic: &mut dyn Oracle = &mut oracle;
        let observation = dynamic
            .evaluate(&json!([1, 2]), Duration::from_millis(5))
            .expect("echo oracle never fails");
        assert_eq!(
            classify(&observation, &Comparator::Exact),
            Classification::Pass
        );
    }

    #[test]
    fn oracle_error_keeps_its_message() {
        let error = OracleError::new("sandbox daemon unavailable");
        assert_eq!(error.message(), "sandbox daemon unavailable");
        assert_eq!(error.to_string(), "sandbox daemon unavailable");
    }

    #[test]
    fn clock_manual_clock_moves_only_when_advanced() {
        let clock = ManualClock::new();
        assert_eq!(clock.now(), Duration::ZERO);
        clock.advance(Duration::from_millis(250));
        assert_eq!(clock.now(), Duration::from_millis(250));
        assert_eq!(clock.now(), Duration::from_millis(250));
    }

    #[test]
    fn clock_system_clock_never_goes_backwards() {
        let clock = SystemClock::new();
        let first = clock.now();
        let second = clock.now();
        assert!(second >= first);
    }
}
