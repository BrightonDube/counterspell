//! Sandbox settings, read from environment variables with the defaults in
//! `design.md` section 6. Values come only from the process environment. This
//! module never reads a `.env` file.

use std::path::PathBuf;
use std::time::Duration;

use crate::config::{whole_number, ConfigError};

const DOCKER_SOCKET: &str = "SANDBOX_DOCKER_SOCKET";
const RUNTIME: &str = "SANDBOX_RUNTIME";
const JUDGE_CONCURRENCY: &str = "JUDGE_CONCURRENCY";
const MAX_CONTAINERS: &str = "SANDBOX_MAX_CONTAINERS";
const RUN_WALL_CAP_SECS: &str = "RUN_WALL_CAP_SECS";
const KILL_SLACK_MS: &str = "SANDBOX_KILL_SLACK_MS";
const MAX_CONSECUTIVE_ABNORMAL: &str = "SANDBOX_MAX_CONSECUTIVE_ABNORMAL";
const CLEANUP_BUDGET_SECS: &str = "SANDBOX_CLEANUP_BUDGET_SECS";
const REAPER_INTERVAL_SECS: &str = "SANDBOX_REAPER_INTERVAL_SECS";
const ORPHAN_AGE_SECS: &str = "SANDBOX_ORPHAN_AGE_SECS";
const API_TIMEOUT_SECS: &str = "SANDBOX_API_TIMEOUT_SECS";
const LANGUAGES_FILE: &str = "LANGUAGES_FILE";

/// Extra seconds an orphan must age beyond the run cap, so the sweeper never
/// removes a container that a run is still using.
const ORPHAN_SLACK_SECS: u64 = 15;

/// The OCI runtime that runs sandbox containers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OciRuntime {
    /// The standard runtime. This is the default.
    Runc,
    /// gVisor. Selectable by configuration but not claimed until it passes the probe on the VPS.
    Runsc,
}

/// Every sandbox setting. Build it once at start-up and pass it down.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxConfig {
    /// The sandbox daemon's Unix socket. There is no default.
    pub docker_socket: PathBuf,
    /// The OCI runtime for sandbox containers.
    pub runtime: OciRuntime,
    /// The most containers that may exist at once.
    pub max_containers: usize,
    /// How often the sweeper looks for orphaned containers.
    pub reaper_interval: Duration,
    /// Age after which a labelled container is treated as an orphan.
    pub orphan_age: Duration,
    /// How long a missed deadline may take to become a recorded verdict.
    pub kill_slack: Duration,
    /// The longest a whole run may last.
    pub run_wall_cap: Duration,
    /// How many abnormal test ends in a row stop the run early.
    pub max_consecutive_abnormal: u32,
    /// How long cleanup may wait for a container to be removed.
    pub cleanup_budget: Duration,
    /// The longest a single daemon call may take.
    pub api_timeout: Duration,
    /// The language catalogue file.
    pub languages_file: PathBuf,
}

impl SandboxConfig {
    /// Reads the settings from the process environment.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] when a value is missing, unusable or out of range.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    /// Reads the settings through `lookup`, so tests never touch the real environment.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] when the socket is missing, the runtime is unknown,
    /// a number is out of range, or the orphan age is not longer than the run cap
    /// plus 15 seconds.
    pub fn from_lookup<F>(lookup: F) -> Result<Self, ConfigError>
    where
        F: Fn(&str) -> Option<String>,
    {
        let docker_socket = match lookup(DOCKER_SOCKET) {
            Some(text) if !text.trim().is_empty() => PathBuf::from(text.trim()),
            _ => {
                return Err(ConfigError::MissingRequired {
                    name: DOCKER_SOCKET,
                })
            }
        };

        let runtime = match lookup(RUNTIME).as_deref().map(str::trim) {
            None | Some("runc") => OciRuntime::Runc,
            Some("runsc") => OciRuntime::Runsc,
            Some(_) => return Err(ConfigError::UnknownRuntime),
        };

        let concurrency = whole_number(&lookup, JUDGE_CONCURRENCY, 2, 1, 16)?;
        let max_containers = whole_number(&lookup, MAX_CONTAINERS, concurrency * 2, 1, 32)?;
        let run_wall_cap_secs = whole_number(&lookup, RUN_WALL_CAP_SECS, 20, 5, 60)?;
        let orphan_age_secs = whole_number(&lookup, ORPHAN_AGE_SECS, 40, 1, 3600)?;

        if orphan_age_secs <= run_wall_cap_secs + ORPHAN_SLACK_SECS {
            return Err(ConfigError::OrphanAgeTooShort);
        }

        let languages_file = lookup(LANGUAGES_FILE)
            .map_or_else(|| PathBuf::from("languages/languages.toml"), PathBuf::from);

        Ok(Self {
            docker_socket,
            runtime,
            max_containers: usize::try_from(max_containers).unwrap_or(32),
            reaper_interval: Duration::from_secs(whole_number(
                &lookup,
                REAPER_INTERVAL_SECS,
                10,
                1,
                300,
            )?),
            orphan_age: Duration::from_secs(orphan_age_secs),
            kill_slack: Duration::from_millis(whole_number(
                &lookup,
                KILL_SLACK_MS,
                1000,
                100,
                10_000,
            )?),
            run_wall_cap: Duration::from_secs(run_wall_cap_secs),
            max_consecutive_abnormal: u32::try_from(whole_number(
                &lookup,
                MAX_CONSECUTIVE_ABNORMAL,
                3,
                1,
                10,
            )?)
            .unwrap_or(10),
            cleanup_budget: Duration::from_secs(whole_number(
                &lookup,
                CLEANUP_BUDGET_SECS,
                5,
                1,
                60,
            )?),
            api_timeout: Duration::from_secs(whole_number(&lookup, API_TIMEOUT_SECS, 10, 1, 120)?),
            languages_file,
        })
    }
}
