// SPDX-License-Identifier: MIT
//! The sweeper: removes managed containers that outlived their run (judge-sandbox T23,
//! design section 12, R10.3 and R10.4).
//!
//! [`sweep_once`] lists containers with the `counterspell.managed=true` label through
//! the backend and force-removes the ones older than the orphan age. [`run`] repeats the
//! sweep at start-up and every interval until the stop signal fires. The sweeper never
//! touches a container without the label — the backend's list only returns managed ones.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use tokio::sync::watch;

use super::backend::{Backend, InfraError};

/// What one sweep observed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SweepOutcome {
    /// Managed containers the backend listed.
    pub listed: usize,
    /// Containers removed because they were older than the orphan age.
    pub removed: usize,
    /// Containers whose removal failed. The errors are logged, not returned — one stuck
    /// container does not save the others.
    pub failed: usize,
}

/// The current unix time in seconds.
fn now_unix() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| {
            i64::try_from(since.as_secs()).unwrap_or(i64::MAX)
        })
}

/// Lists managed containers and force-removes every one strictly older than
/// `orphan_age`. A removal error is logged and counted; the sweep continues. A list
/// error is returned.
///
/// # Errors
///
/// Returns [`InfraError`] when the container list itself fails.
pub async fn sweep_once<B: Backend>(
    backend: &B,
    orphan_age: Duration,
    now: i64,
) -> Result<SweepOutcome, InfraError> {
    let containers = backend.list_managed().await?;
    let mut outcome = SweepOutcome {
        listed: containers.len(),
        removed: 0,
        failed: 0,
    };
    let too_old = i64::try_from(orphan_age.as_secs()).unwrap_or(i64::MAX);
    for container in containers {
        if now.saturating_sub(container.created_at) <= too_old {
            continue;
        }
        match backend.remove_managed(&container.id).await {
            Ok(()) => outcome.removed += 1,
            Err(error) => {
                outcome.failed += 1;
                tracing::warn!(container = %container.id, %error, "sweeper could not remove an orphan");
            }
        }
    }
    Ok(outcome)
}

/// Sweeps at start-up and every `interval` until `stop` fires. A failed sweep is logged
/// and the loop continues — the next interval tries again. Runs until stopped; spawn it.
pub async fn run_sweeper<B: Backend>(
    backend: B,
    interval: Duration,
    orphan_age: Duration,
    mut stop: watch::Receiver<bool>,
) {
    let mut ticks = tokio::time::interval(interval);
    loop {
        tokio::select! {
            _ = ticks.tick() => {
                match sweep_once(&backend, orphan_age, now_unix()).await {
                    Ok(outcome) if outcome.removed > 0 || outcome.failed > 0 => {
                        tracing::info!(?outcome, "sandbox sweep removed orphans");
                    }
                    Ok(_) => {}
                    Err(error) => {
                        tracing::warn!(%error, "sandbox sweep could not list containers");
                    }
                }
            }
            _ = stop.changed() => break,
        }
    }
}
