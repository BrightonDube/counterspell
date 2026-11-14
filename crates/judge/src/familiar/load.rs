//! Host load guard for the bench. A bench is discarded when the host is busy, so a noisy
//! neighbour cannot make a learner look slow.
//!
//! The only file this module reads is `/proc/loadavg`, through [`ProcLoadAvg`]. Tests use a
//! scripted [`LoadSource`] instead.

use std::io;

use tracing::warn;

/// The default highest 1-minute load a bench may run under (`BENCH_MAX_LOAD`).
pub const DEFAULT_MAX_LOAD: f64 = 3.0;

/// Where a 1-minute load average comes from. A trait, so tests can script the readings.
pub trait LoadSource {
    /// Returns the 1-minute load average.
    ///
    /// # Errors
    ///
    /// Returns an error when the load cannot be read or parsed.
    fn one_minute(&self) -> io::Result<f64>;
}

/// Reads the real host load from `/proc/loadavg`.
pub struct ProcLoadAvg;

impl LoadSource for ProcLoadAvg {
    fn one_minute(&self) -> io::Result<f64> {
        let text = std::fs::read_to_string("/proc/loadavg")?;
        parse_loadavg(&text)
            .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "unreadable load average"))
    }
}

/// Parses the first field of `/proc/loadavg`. Returns `None` unless the field is a finite, non-negative number.
#[must_use]
pub fn parse_loadavg(text: &str) -> Option<f64> {
    let value: f64 = text.split_whitespace().next()?.parse().ok()?;
    (value.is_finite() && value >= 0.0).then_some(value)
}

/// Takes one load sample. An unreadable load is logged and returned as unknown (`None`).
#[must_use]
pub fn sample(source: &impl LoadSource) -> Option<f64> {
    match source.one_minute() {
        Ok(load) => Some(load),
        Err(error) => {
            warn!(%error, "host load unreadable; the bench is not discarded for it");
            None
        }
    }
}

/// True when neither reading is above `limit`. A reading equal to the limit is quiet. An unknown
/// reading (`None`) never discards the bench, because the load could not be checked either way.
#[must_use]
pub fn quiet(before: Option<f64>, after: Option<f64>, limit: f64) -> bool {
    [before, after]
        .into_iter()
        .all(|reading| reading.is_none_or(|load| load <= limit))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A load source that returns a fixed reading.
    struct Fixed(f64);

    impl LoadSource for Fixed {
        fn one_minute(&self) -> io::Result<f64> {
            Ok(self.0)
        }
    }

    /// A load source that cannot be read.
    struct Unreadable;

    impl LoadSource for Unreadable {
        fn one_minute(&self) -> io::Result<f64> {
            Err(io::Error::other("no /proc"))
        }
    }

    #[test]
    fn the_first_field_of_proc_loadavg_is_the_one_minute_load() {
        assert_eq!(parse_loadavg("0.52 0.58 0.59 1/345 12345\n"), Some(0.52));
    }

    #[test]
    fn text_that_is_not_a_load_average_is_unknown() {
        assert_eq!(parse_loadavg(""), None);
        assert_eq!(parse_loadavg("abc 0.1 0.1"), None);
        assert_eq!(parse_loadavg("-1.0 0 0"), None);
        assert_eq!(parse_loadavg("NaN 0 0"), None);
    }

    #[test]
    fn a_reading_equal_to_the_limit_is_quiet_and_one_above_it_is_busy() {
        assert!(quiet(Some(2.9), Some(3.0), 3.0));
        assert!(!quiet(Some(3.1), Some(1.0), 3.0));
        assert!(!quiet(Some(1.0), Some(3.5), 3.0));
    }

    #[test]
    fn an_unknown_reading_does_not_discard_the_bench() {
        assert!(quiet(None, Some(1.0), 3.0));
        assert!(quiet(None, None, 3.0));
    }

    #[test]
    fn a_readable_source_gives_a_sample() {
        assert_eq!(sample(&Fixed(0.75)), Some(0.75));
    }

    #[test]
    fn an_unreadable_source_gives_an_unknown_sample() {
        assert_eq!(sample(&Unreadable), None);
    }
}
