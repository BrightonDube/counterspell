//! Judge settings, read from environment variables with the defaults in `design.md`.

use std::time::Duration;

use thiserror::Error;

// Variable names. Keep them in step with the Configuration table in design.md.
const CONCURRENCY: &str = "JUDGE_CONCURRENCY";
const LEASE_SECS: &str = "LEASE_SECS";
const RENEW_SECS: &str = "RENEW_SECS";
const MAX_CLAIM_AGE_SECS: &str = "MAX_CLAIM_AGE_SECS";
const REAPER_INTERVAL_SECS: &str = "REAPER_INTERVAL_SECS";
const CLAIM_POLL_MS: &str = "CLAIM_POLL_MS";
const RETRY_BACKOFF_BASE_SECS: &str = "RETRY_BACKOFF_BASE_SECS";
const RETRY_BACKOFF_CAP_SECS: &str = "RETRY_BACKOFF_CAP_SECS";
const SHUTDOWN_GRACE_SECS: &str = "SHUTDOWN_GRACE_SECS";
const QUEUE_TTL_SECS: &str = "QUEUE_TTL_SECS";

/// Why a setting cannot be used. Messages name the variable but never echo its value.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConfigError {
    /// The variable is not a whole number.
    #[error("{name} must be a whole number")]
    NotANumber {
        /// The environment variable that is wrong.
        name: &'static str,
    },
    /// The variable is a whole number outside its allowed range.
    #[error("{name} must be between {min} and {max}")]
    OutOfRange {
        /// The environment variable that is wrong.
        name: &'static str,
        /// The smallest allowed value.
        min: u64,
        /// The largest allowed value.
        max: u64,
    },
    /// A lease must be renewed before it expires, so renewal must be shorter.
    #[error("RENEW_SECS must be shorter than LEASE_SECS")]
    RenewNotShorterThanLease,
}

/// Every setting the judge needs. Build it once at start-up and pass it down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JudgeConfig {
    /// How many runs one worker executes at the same time.
    pub concurrency: usize,
    /// How long a claim stays valid without renewal.
    pub lease: Duration,
    /// How often a worker renews its lease.
    pub renew: Duration,
    /// The longest a claim may last, whether or not it is renewed.
    pub max_claim_age: Duration,
    /// How often the reaper looks for expired leases.
    pub reaper_interval: Duration,
    /// How long an idle worker waits before asking for work again.
    pub claim_poll: Duration,
    /// The first retry delay. Each later retry doubles it.
    pub retry_backoff_base: Duration,
    /// The longest retry delay.
    pub retry_backoff_cap: Duration,
    /// How long a shutting-down worker lets running executors finish.
    pub shutdown_grace: Duration,
    /// How long a run may wait in the queue before it expires.
    pub queue_ttl: Duration,
}

impl JudgeConfig {
    /// Reads the settings from the process environment.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] when a variable is set to something unusable.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    /// Reads the settings through `lookup`, so tests never touch the real environment.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] when a value is not a number, is out of range, or
    /// makes renewal no shorter than the lease.
    pub fn from_lookup<F>(lookup: F) -> Result<Self, ConfigError>
    where
        F: Fn(&str) -> Option<String>,
    {
        let read = |name: &'static str, default: u64, min: u64, max: u64| {
            let value = match lookup(name) {
                None => default,
                Some(text) => text
                    .trim()
                    .parse::<u64>()
                    .map_err(|_| ConfigError::NotANumber { name })?,
            };
            if (min..=max).contains(&value) {
                Ok(value)
            } else {
                Err(ConfigError::OutOfRange { name, min, max })
            }
        };

        let lease = read(LEASE_SECS, 30, 5, 600)?;
        let renew = read(RENEW_SECS, 10, 1, 300)?;
        if renew >= lease {
            return Err(ConfigError::RenewNotShorterThanLease);
        }

        let concurrency = read(CONCURRENCY, 2, 1, 16)?;
        let config = Self {
            concurrency: usize::try_from(concurrency).unwrap_or(16),
            lease: Duration::from_secs(lease),
            renew: Duration::from_secs(renew),
            max_claim_age: Duration::from_secs(read(MAX_CLAIM_AGE_SECS, 120, 10, 3600)?),
            reaper_interval: Duration::from_secs(read(REAPER_INTERVAL_SECS, 5, 1, 300)?),
            claim_poll: Duration::from_millis(read(CLAIM_POLL_MS, 1000, 50, 60_000)?),
            retry_backoff_base: Duration::from_secs(read(RETRY_BACKOFF_BASE_SECS, 2, 1, 60)?),
            retry_backoff_cap: Duration::from_secs(read(RETRY_BACKOFF_CAP_SECS, 30, 1, 3600)?),
            shutdown_grace: Duration::from_secs(read(SHUTDOWN_GRACE_SECS, 20, 1, 300)?),
            queue_ttl: Duration::from_secs(read(QUEUE_TTL_SECS, 600, 60, 86_400)?),
        };
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;

    fn from_map(pairs: &[(&str, &str)]) -> Result<JudgeConfig, ConfigError> {
        let map: HashMap<String, String> = pairs
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        JudgeConfig::from_lookup(|name| map.get(name).cloned())
    }

    #[test]
    fn defaults_match_the_design_when_nothing_is_set() {
        let config = from_map(&[]).unwrap();
        assert_eq!(config.concurrency, 2);
        assert_eq!(config.lease, Duration::from_secs(30));
        assert_eq!(config.renew, Duration::from_secs(10));
        assert_eq!(config.max_claim_age, Duration::from_secs(120));
        assert_eq!(config.reaper_interval, Duration::from_secs(5));
        assert_eq!(config.claim_poll, Duration::from_millis(1000));
        assert_eq!(config.retry_backoff_base, Duration::from_secs(2));
        assert_eq!(config.retry_backoff_cap, Duration::from_secs(30));
        assert_eq!(config.shutdown_grace, Duration::from_secs(20));
        assert_eq!(config.queue_ttl, Duration::from_secs(600));
    }

    #[test]
    fn values_set_in_the_environment_replace_the_defaults() {
        let config = from_map(&[
            ("JUDGE_CONCURRENCY", " 4 "),
            ("LEASE_SECS", "20"),
            ("RENEW_SECS", "5"),
        ])
        .unwrap();
        assert_eq!(config.concurrency, 4);
        assert_eq!(config.lease, Duration::from_secs(20));
        assert_eq!(config.renew, Duration::from_secs(5));
    }

    #[test]
    fn a_value_that_is_not_a_whole_number_names_the_variable() {
        let error = from_map(&[("JUDGE_CONCURRENCY", "many")]).unwrap_err();
        assert_eq!(
            error,
            ConfigError::NotANumber {
                name: "JUDGE_CONCURRENCY"
            }
        );
    }

    #[test]
    fn zero_concurrency_is_out_of_range() {
        let error = from_map(&[("JUDGE_CONCURRENCY", "0")]).unwrap_err();
        assert_eq!(
            error,
            ConfigError::OutOfRange {
                name: "JUDGE_CONCURRENCY",
                min: 1,
                max: 16
            }
        );
    }

    #[test]
    fn renewal_must_be_shorter_than_the_lease() {
        let error = from_map(&[("LEASE_SECS", "10"), ("RENEW_SECS", "10")]).unwrap_err();
        assert_eq!(error, ConfigError::RenewNotShorterThanLease);
    }
}
