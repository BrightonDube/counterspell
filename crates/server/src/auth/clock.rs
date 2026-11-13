//! One injectable source of "now" for every expiry decision in the auth module.
//!
//! Production uses [`SystemClock`]. Tests use [`ManualClock`], which they can move forward, so
//! expiry tests are deterministic and never sleep.

use std::sync::Mutex;

use time::{Duration, OffsetDateTime};

/// Gives the current time. Issuing and checking tokens, sessions and flows all read it here.
pub trait Clock: Send + Sync {
    /// The current UTC time.
    fn now(&self) -> OffsetDateTime;
}

/// The real clock, read from the operating system.
#[derive(Debug, Clone, Copy, Default)]
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> OffsetDateTime {
        OffsetDateTime::now_utc()
    }
}

/// A clock that stands still until a test moves it.
#[cfg(any(test, feature = "testkit"))]
#[derive(Debug)]
pub struct ManualClock {
    now: Mutex<OffsetDateTime>,
}

#[cfg(any(test, feature = "testkit"))]
impl ManualClock {
    /// Starts the clock at `start`.
    #[must_use]
    pub fn new(start: OffsetDateTime) -> Self {
        Self {
            now: Mutex::new(start),
        }
    }

    /// Moves the clock forward by `by`.
    pub fn advance(&self, by: Duration) {
        let mut now = self
            .now
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *now += by;
    }

    /// Sets the clock to an exact instant.
    pub fn set(&self, at: OffsetDateTime) {
        let mut now = self
            .now
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        *now = at;
    }
}

#[cfg(any(test, feature = "testkit"))]
impl Clock for ManualClock {
    fn now(&self) -> OffsetDateTime {
        *self
            .now
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::macros::datetime;

    #[test]
    fn a_manual_clock_moves_only_when_told() {
        let clock = ManualClock::new(datetime!(2026-11-13 10:00 UTC));
        assert_eq!(clock.now(), datetime!(2026-11-13 10:00 UTC));
        clock.advance(Duration::minutes(15));
        assert_eq!(clock.now(), datetime!(2026-11-13 10:15 UTC));
        clock.set(datetime!(2026-11-14 00:00 UTC));
        assert_eq!(clock.now(), datetime!(2026-11-14 00:00 UTC));
    }

    #[test]
    fn the_system_clock_is_utc() {
        assert_eq!(SystemClock.now().offset(), time::UtcOffset::UTC);
    }
}
