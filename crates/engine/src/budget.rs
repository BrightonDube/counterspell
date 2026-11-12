//! Time and work limits for one ghost hunt.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// The longest total time any hunt may take (R8.2).
pub const MAX_TOTAL_WALL: Duration = Duration::from_secs(8);

/// The work and time limits for one hunt.
///
/// The defaults are estimates to tune on the VPS. Times are stored as whole
/// milliseconds in JSON.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct Budget {
    /// The most candidates the search stage may evaluate.
    pub max_candidates: u32,
    /// The time the search stage may use, from the start of the hunt.
    #[serde(with = "duration_millis")]
    pub search_wall: Duration,
    /// The most evaluations the shrink stage may spend.
    pub max_shrink_evals: u32,
    /// The time the shrink stage may use after it starts.
    #[serde(with = "duration_millis")]
    pub shrink_wall: Duration,
    /// The time the whole hunt may use, including verification.
    #[serde(with = "duration_millis")]
    pub total_wall: Duration,
    /// The longest time one Oracle call may run.
    #[serde(with = "duration_millis")]
    pub eval_limit: Duration,
    /// How many times the final input is re-run to confirm it.
    pub verify_runs: u32,
    /// Time kept back at the end of the hunt for verification.
    #[serde(with = "duration_millis")]
    pub verify_reserve: Duration,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            max_candidates: 500,
            search_wall: Duration::from_secs(4),
            max_shrink_evals: 300,
            shrink_wall: Duration::from_secs(3),
            total_wall: MAX_TOTAL_WALL,
            eval_limit: Duration::from_secs(1),
            verify_runs: 2,
            verify_reserve: Duration::from_secs(1),
        }
    }
}

impl Budget {
    /// Checks that the budget is safe to run.
    ///
    /// # Errors
    ///
    /// Returns a short reason when the total time is over 8 s, when the phase
    /// budgets leave no room for verification, or when a limit is zero.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.total_wall > MAX_TOTAL_WALL {
            return Err("total time is over 8 seconds");
        }
        let phases = self
            .search_wall
            .saturating_add(self.shrink_wall)
            .saturating_add(self.verify_reserve);
        if phases > self.total_wall {
            return Err("search and shrink budgets leave no room for verification");
        }
        if self.verify_runs == 0 {
            return Err("at least one verification run is needed");
        }
        if self.eval_limit.is_zero() {
            return Err("the per-call time limit must be positive");
        }
        Ok(())
    }

    /// The time, from the hunt start, when the search stage must stop.
    #[must_use]
    pub fn search_deadline(&self) -> Duration {
        self.search_wall
            .min(self.total_wall.saturating_sub(self.verify_reserve))
    }

    /// The time, from the hunt start, when the shrink stage must stop, given
    /// the time it started.
    #[must_use]
    pub fn shrink_deadline(&self, shrink_start: Duration) -> Duration {
        shrink_start
            .saturating_add(self.shrink_wall)
            .min(self.total_wall.saturating_sub(self.verify_reserve))
    }
}

/// Stores durations as whole milliseconds in JSON.
mod duration_millis {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::Duration;

    /// Writes a duration as a whole number of milliseconds.
    pub fn serialize<S: Serializer>(value: &Duration, serializer: S) -> Result<S::Ok, S::Error> {
        u64::try_from(value.as_millis())
            .unwrap_or(u64::MAX)
            .serialize(serializer)
    }

    /// Reads a duration from whole milliseconds.
    pub fn deserialize<'de, D: Deserializer<'de>>(deserializer: D) -> Result<Duration, D::Error> {
        u64::deserialize(deserializer).map(Duration::from_millis)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn budget_default_fits_the_eight_second_cap() {
        let budget = Budget::default();
        assert_eq!(budget.validate(), Ok(()));
        assert_eq!(budget.search_deadline(), Duration::from_secs(4));
        assert_eq!(
            budget.shrink_deadline(Duration::from_secs(4)),
            Duration::from_secs(7)
        );
    }

    #[test]
    fn budget_over_eight_seconds_is_rejected() {
        let budget = Budget {
            total_wall: Duration::from_secs(9),
            ..Budget::default()
        };
        assert!(budget.validate().is_err());
    }

    #[test]
    fn budget_without_a_verification_reserve_is_rejected() {
        let budget = Budget {
            verify_reserve: Duration::from_secs(2),
            ..Budget::default()
        };
        assert!(budget.validate().is_err());
    }

    #[test]
    fn budget_zero_limits_are_rejected() {
        let no_runs = Budget {
            verify_runs: 0,
            ..Budget::default()
        };
        let no_time = Budget {
            eval_limit: Duration::ZERO,
            ..Budget::default()
        };
        assert!(no_runs.validate().is_err());
        assert!(no_time.validate().is_err());
    }

    #[test]
    fn budget_deadlines_leave_the_reserve_on_a_short_total() {
        let budget = Budget {
            total_wall: Duration::from_secs(5),
            ..Budget::default()
        };
        assert_eq!(budget.search_deadline(), Duration::from_secs(4));
        assert_eq!(
            budget.shrink_deadline(Duration::from_secs(4)),
            Duration::from_secs(4)
        );
    }

    #[test]
    fn budget_serialises_times_as_milliseconds() {
        let json = serde_json::to_value(Budget::default()).expect("serialise");
        assert_eq!(json["search_wall"], serde_json::json!(4000));
        assert_eq!(json["verify_runs"], serde_json::json!(2));
    }
}
