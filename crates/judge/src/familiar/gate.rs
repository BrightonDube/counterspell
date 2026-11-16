//! Serialises benches inside one judge process, and reads the bench settings from their text values.
//!
//! The gate is a one-permit semaphore. Two sandboxes can run tests together, but two benches never
//! run at once, so they do not compete for the CPU. A waiting run is still `running` and keeps renewing
//! its lease; the gate does not hold a database connection.

use std::sync::Arc;

use thiserror::Error;
use tokio::sync::{AcquireError, OwnedSemaphorePermit, Semaphore};

use super::load::DEFAULT_MAX_LOAD;

/// Hands out bench permits. Clone it to share it between tasks.
#[derive(Debug, Clone)]
pub struct BenchGate {
    semaphore: Arc<Semaphore>,
}

impl BenchGate {
    /// A gate that allows one bench at a time when `serialise` is true, and no limit otherwise.
    #[must_use]
    pub fn new(serialise: bool) -> Self {
        let permits = if serialise { 1 } else { Semaphore::MAX_PERMITS };
        Self {
            semaphore: Arc::new(Semaphore::new(permits)),
        }
    }

    /// Waits for a permit. The permit is released when it is dropped.
    ///
    /// # Errors
    ///
    /// Returns an error only if the semaphore has been closed, which this module never does.
    pub async fn acquire(&self) -> Result<OwnedSemaphorePermit, AcquireError> {
        Arc::clone(&self.semaphore).acquire_owned().await
    }

    /// Takes a permit now if one is free, without waiting.
    #[must_use]
    pub fn try_acquire(&self) -> Option<OwnedSemaphorePermit> {
        Arc::clone(&self.semaphore).try_acquire_owned().ok()
    }
}

/// The bench settings read from `BENCH_SERIALISE` and `BENCH_MAX_LOAD`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BenchSettings {
    /// Whether benches run one at a time.
    pub serialise: bool,
    /// The highest 1-minute load a bench may run under.
    pub max_load: f64,
}

impl Default for BenchSettings {
    fn default() -> Self {
        Self {
            serialise: true,
            max_load: DEFAULT_MAX_LOAD,
        }
    }
}

/// A bench setting that is not a valid value.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SettingsError {
    /// `BENCH_SERIALISE` was not `true`, `false`, `1` or `0`.
    #[error("BENCH_SERIALISE must be true, false, 1 or 0, got {0:?}")]
    Serialise(String),
    /// `BENCH_MAX_LOAD` was not a finite number above zero.
    #[error("BENCH_MAX_LOAD must be a number above zero, got {0:?}")]
    MaxLoad(String),
}

/// Reads the two settings from their text values. A missing value takes the default.
///
/// # Errors
///
/// Returns [`SettingsError`] for a value that is not a valid setting. It never guesses.
pub fn parse_settings(
    serialise: Option<&str>,
    max_load: Option<&str>,
) -> Result<BenchSettings, SettingsError> {
    let defaults = BenchSettings::default();
    let serialise = match serialise {
        None => defaults.serialise,
        Some("true" | "1") => true,
        Some("false" | "0") => false,
        Some(other) => return Err(SettingsError::Serialise(other.to_owned())),
    };
    let max_load = match max_load {
        None => defaults.max_load,
        Some(text) => match text.parse::<f64>() {
            Ok(value) if value.is_finite() && value > 0.0 => value,
            _ => return Err(SettingsError::MaxLoad(text.to_owned())),
        },
    };
    Ok(BenchSettings {
        serialise,
        max_load,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_serialised_gate_lets_only_one_bench_in() {
        let gate = BenchGate::new(true);
        let first = gate.try_acquire();
        assert!(first.is_some());
        assert!(gate.try_acquire().is_none());
    }

    #[test]
    fn the_next_bench_can_start_once_the_holder_releases() {
        let gate = BenchGate::new(true);
        let first = gate.try_acquire().expect("the first permit is free");
        drop(first);
        assert!(gate.try_acquire().is_some());
    }

    #[test]
    fn an_unserialised_gate_lets_two_benches_in() {
        let gate = BenchGate::new(false);
        assert!(gate.try_acquire().is_some());
        assert!(gate.try_acquire().is_some());
    }

    #[tokio::test]
    async fn a_waiter_proceeds_once_the_holder_releases() {
        let gate = BenchGate::new(true);
        let first = gate.acquire().await.expect("the first permit is free");
        let waiter = {
            let gate = gate.clone();
            tokio::spawn(async move { gate.acquire().await.is_ok() })
        };
        drop(first);
        assert!(waiter.await.expect("the waiter task finishes"));
    }

    #[test]
    fn the_defaults_are_serialised_with_a_load_of_three() {
        assert_eq!(
            parse_settings(None, None),
            Ok(BenchSettings {
                serialise: true,
                max_load: 3.0,
            })
        );
    }

    #[test]
    fn serialise_accepts_true_false_one_and_zero_only() {
        assert_eq!(
            parse_settings(Some("false"), None).map(|s| s.serialise),
            Ok(false)
        );
        assert_eq!(
            parse_settings(Some("0"), None).map(|s| s.serialise),
            Ok(false)
        );
        assert_eq!(
            parse_settings(Some("1"), None).map(|s| s.serialise),
            Ok(true)
        );
        assert_eq!(
            parse_settings(Some("yes"), None),
            Err(SettingsError::Serialise("yes".to_owned()))
        );
    }

    #[test]
    fn max_load_must_be_a_finite_number_above_zero() {
        assert_eq!(
            parse_settings(None, Some("2.5")).map(|s| s.max_load),
            Ok(2.5)
        );
        assert_eq!(
            parse_settings(None, Some("0")),
            Err(SettingsError::MaxLoad("0".to_owned()))
        );
        assert_eq!(
            parse_settings(None, Some("NaN")),
            Err(SettingsError::MaxLoad("NaN".to_owned()))
        );
        assert_eq!(
            parse_settings(None, Some("abc")),
            Err(SettingsError::MaxLoad("abc".to_owned()))
        );
    }
}
