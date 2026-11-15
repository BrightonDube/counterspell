//! Liveness for the judge container (deploy-ops R10.2, R4.4).
//!
//! The worker writes a heartbeat file every few seconds. `counterspell-judge healthcheck` reads
//! that file's modification time and exits 0 when it is recent, or 1 otherwise. The file is a
//! liveness signal only. It carries no run data and no secret.
//!
//! The file is a separate process signal because the claim loop can wait for a free slot while
//! long runs finish. A heartbeat written from the claim loop would then look stale during a busy
//! but healthy period.

use std::{
    io,
    path::{Path, PathBuf},
    time::{Duration, SystemTime},
};

use tokio::sync::watch;

/// Environment variable that overrides the heartbeat file path. Tests use it.
pub const HEARTBEAT_FILE_ENV: &str = "JUDGE_HEARTBEAT_FILE";

/// Default heartbeat path. `/tmp` is a `tmpfs` in the judge container (deploy-ops compose), so
/// writes never touch the disk.
pub const DEFAULT_HEARTBEAT_FILE: &str = "/tmp/counterspell-judge.heartbeat";

/// A heartbeat older than this counts as stale.
pub const MAX_AGE: Duration = Duration::from_secs(60);

/// How often the worker refreshes the heartbeat. Well under [`MAX_AGE`].
pub const BEAT_EVERY: Duration = Duration::from_secs(10);

/// Why the healthcheck failed. Neither variant holds a path or a connection string.
#[derive(Debug, PartialEq, Eq)]
pub enum HealthError {
    /// The heartbeat file does not exist or cannot be read.
    Missing,
    /// The heartbeat file is older than [`MAX_AGE`], or its time is in the future.
    Stale,
}

/// Returns the heartbeat path from the environment, or the default path.
#[must_use]
pub fn heartbeat_path() -> PathBuf {
    std::env::var_os(HEARTBEAT_FILE_ENV)
        .map_or_else(|| PathBuf::from(DEFAULT_HEARTBEAT_FILE), PathBuf::from)
}

/// Says whether a heartbeat written at `modified` is fresh at `now`.
///
/// A time in the future is not fresh. It means the clock went backwards or the file is wrong.
#[must_use]
pub fn is_fresh(modified: SystemTime, now: SystemTime, max_age: Duration) -> bool {
    now.duration_since(modified).is_ok_and(|age| age <= max_age)
}

/// Checks the heartbeat file at `path` against `now`.
///
/// # Errors
///
/// Returns [`HealthError::Missing`] when the file cannot be read, and [`HealthError::Stale`] when
/// it is too old.
pub fn check(path: &Path, now: SystemTime, max_age: Duration) -> Result<Duration, HealthError> {
    let modified = std::fs::metadata(path)
        .and_then(|metadata| metadata.modified())
        .map_err(|_| HealthError::Missing)?;
    if is_fresh(modified, now, max_age) {
        Ok(now.duration_since(modified).unwrap_or_default())
    } else {
        Err(HealthError::Stale)
    }
}

/// Builds the one-line report for the `healthcheck` subcommand. `Ok` means healthy (exit 0);
/// `Err` means unhealthy (exit 1). Neither line includes the file path.
///
/// # Errors
///
/// Returns the unhealthy line when the heartbeat is missing or stale.
pub fn report(path: &Path, now: SystemTime) -> Result<String, String> {
    match check(path, now, MAX_AGE) {
        Ok(age) => Ok(format!("judge healthy: heartbeat {}s old", age.as_secs())),
        Err(HealthError::Missing) => Err("judge unhealthy: heartbeat missing".to_owned()),
        Err(HealthError::Stale) => Err("judge unhealthy: heartbeat stale".to_owned()),
    }
}

/// Writes the heartbeat file, replacing any earlier one.
///
/// # Errors
///
/// Returns the I/O error when the file cannot be written.
pub fn touch(path: &Path) -> io::Result<()> {
    let seconds = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    std::fs::write(path, format!("{seconds}\n"))
}

/// Refreshes the heartbeat every `every` until `shutdown` is set.
///
/// A failed write is logged and tried again on the next tick. The worker keeps running, and the
/// healthcheck reports the stale file.
pub async fn beat_until_shutdown(
    path: PathBuf,
    every: Duration,
    mut shutdown: watch::Receiver<bool>,
) {
    loop {
        if let Err(error) = touch(&path) {
            tracing::warn!(error = %error, "heartbeat write failed");
        }
        tokio::select! {
            () = tokio::time::sleep(every) => {}
            changed = shutdown.changed() => {
                // An error means the sender is gone, which also ends the beat.
                if changed.is_err() || *shutdown.borrow() {
                    break;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn heartbeat_exactly_at_the_limit_is_still_fresh() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000);
        let modified = now - MAX_AGE;

        assert!(is_fresh(modified, now, MAX_AGE));
    }

    #[test]
    fn heartbeat_one_second_past_the_limit_is_stale() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000);
        let modified = now - MAX_AGE - Duration::from_secs(1);

        assert!(!is_fresh(modified, now, MAX_AGE));
    }

    #[test]
    fn heartbeat_in_the_future_is_not_fresh() {
        let now = SystemTime::UNIX_EPOCH + Duration::from_secs(1_000);
        let modified = now + Duration::from_secs(5);

        assert!(!is_fresh(modified, now, MAX_AGE));
    }
}
