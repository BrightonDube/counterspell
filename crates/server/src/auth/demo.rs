//! The judge demo account: seeding it at start-up and resetting it every day.
//!
//! The seed runs through the database function `cs_seed_demo_user`, and the reset runs through
//! `cs_reset_demo_account`. Those functions are the only code that may change the demo account.
//! This module never writes to `users` or to the demo's rows directly.

use std::{sync::Arc, time::Duration as StdDuration};

use sqlx::PgPool;
use time::{Duration, OffsetDateTime, Time};
use tokio::task::JoinHandle;

use super::{
    clock::Clock,
    config::DemoConfig,
    errors::{database, AuthError},
    password::{HashGate, Verdict},
};

/// How often the background task checks whether a reset is due.
const CHECK_EVERY: StdDuration = StdDuration::from_secs(5 * 60);

/// Creates or updates the single demo account from the configured address and password.
///
/// The password is hashed only when the stored hash does not already accept it, so a restart with
/// the same password does not change the stored value.
///
/// # Errors
///
/// Returns [`AuthError::Internal`] when hashing or the database fails, and [`AuthError::Busy`]
/// when the hash gate is full.
pub async fn seed_demo(
    pool: &PgPool,
    demo: &DemoConfig,
    hashes: &HashGate,
) -> Result<(), AuthError> {
    let password = demo.password.expose().clone();
    let existing: Option<(Option<String>,)> =
        sqlx::query_as("SELECT password_hash FROM users WHERE is_demo")
            .fetch_optional(pool)
            .await
            .map_err(database)?;
    let reusable = match existing.and_then(|(stored,)| stored) {
        Some(stored) => {
            let verdict = hashes.verify(password.clone(), stored.clone()).await?;
            (verdict != Verdict::Rejected).then_some(stored)
        }
        None => None,
    };
    let hash = match reusable {
        Some(stored) => stored,
        None => hashes.hash(password).await?,
    };
    sqlx::query("SELECT cs_seed_demo_user($1, $2)")
        .bind(&demo.email)
        .bind(hash)
        .execute(pool)
        .await
        .map_err(database)?;
    Ok(())
}

/// The most recent scheduled reset at or before `now`: `hour`:00 UTC today, or yesterday when
/// today's time has not come yet.
#[must_use]
pub fn latest_scheduled_reset(now: OffsetDateTime, hour: u8) -> OffsetDateTime {
    let today = now.replace_time(Time::MIDNIGHT) + Duration::hours(i64::from(hour));
    if today > now {
        today - Duration::days(1)
    } else {
        today
    }
}

/// Resets the demo account if it has not been reset since the latest scheduled reset.
///
/// The clock supplies both the reset time and the schedule. The database checks the stored reset
/// time under its advisory lock, so a restart or two concurrent checks reset at most once.
///
/// Returns the number of finished runs removed. Zero means no reset ran, or the reset ran and had
/// no finished runs to remove.
///
/// # Errors
///
/// Returns [`AuthError::Internal`] when the database call fails.
pub async fn reset_if_due(pool: &PgPool, clock: &dyn Clock, hour: u8) -> Result<i64, AuthError> {
    let now = clock.now();
    let due = latest_scheduled_reset(now, hour);
    let (removed,): (i64,) = sqlx::query_as("SELECT cs_reset_demo_account($1, $2)")
        .bind(now)
        .bind(due)
        .fetch_one(pool)
        .await
        .map_err(database)?;
    Ok(removed)
}

/// Starts the background task that resets the demo account at each scheduled time.
///
/// The task checks once at start-up and then every five minutes, so a reset missed while the
/// server was down runs on the next start. Abort the returned handle to stop the task.
pub fn spawn_daily_reset(pool: PgPool, clock: Arc<dyn Clock>, hour: u8) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match reset_if_due(&pool, clock.as_ref(), hour).await {
                Ok(removed) if removed > 0 => {
                    tracing::info!(removed, "demo account reset removed finished runs");
                }
                Ok(_) => {}
                Err(error) => {
                    tracing::warn!(code = error.code(), "demo account reset check failed");
                }
            }
            tokio::time::sleep(CHECK_EVERY).await;
        }
    })
}

#[cfg(test)]
mod tests {
    use time::macros::datetime;

    use super::latest_scheduled_reset;

    #[test]
    fn the_latest_reset_is_today_once_the_hour_has_passed() {
        assert_eq!(
            latest_scheduled_reset(datetime!(2026-11-13 12:00 UTC), 12),
            datetime!(2026-11-13 12:00 UTC)
        );
        assert_eq!(
            latest_scheduled_reset(datetime!(2026-11-13 23:59 UTC), 12),
            datetime!(2026-11-13 12:00 UTC)
        );
    }

    #[test]
    fn the_latest_reset_is_yesterday_before_the_hour() {
        assert_eq!(
            latest_scheduled_reset(datetime!(2026-11-13 11:59 UTC), 12),
            datetime!(2026-11-12 12:00 UTC)
        );
        assert_eq!(
            latest_scheduled_reset(datetime!(2026-11-13 00:00 UTC), 0),
            datetime!(2026-11-13 00:00 UTC)
        );
    }
}
