//! Background removal of auth rows that can no longer be used.
//!
//! A token, flow or session stays in the database for one day after it stops working. That gives
//! a short window for debugging, and then the row is deleted. The deletes use fixed SQL and never
//! touch a live row.

use std::{sync::Arc, time::Duration as StdDuration};

use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use tokio::task::JoinHandle;

use crate::auth::{
    clock::Clock,
    errors::{database, AuthError},
};

/// How often the background cleanup runs.
const EVERY: StdDuration = StdDuration::from_secs(60 * 60);

/// Deletes every email token, OAuth flow and session that stopped working more than a day before
/// `now`. A spent token counts from the moment it was used. Returns how many rows were removed.
///
/// # Errors
///
/// Returns [`AuthError::Internal`] when a delete fails.
pub async fn purge(pool: &PgPool, now: OffsetDateTime) -> Result<u64, AuthError> {
    let cutoff = now - Duration::days(1);
    let tokens = sqlx::query("DELETE FROM email_tokens WHERE expires_at < $1 OR used_at < $1")
        .bind(cutoff)
        .execute(pool)
        .await
        .map_err(database)?;
    let flows = sqlx::query("DELETE FROM oauth_flows WHERE expires_at < $1")
        .bind(cutoff)
        .execute(pool)
        .await
        .map_err(database)?;
    let sessions =
        sqlx::query("DELETE FROM sessions WHERE expires_at < $1 OR absolute_expires_at < $1")
            .bind(cutoff)
            .execute(pool)
            .await
            .map_err(database)?;
    Ok(tokens.rows_affected() + flows.rows_affected() + sessions.rows_affected())
}

/// Runs [`purge`] once an hour for the life of the process. A failed run is logged and retried on
/// the next hour, and the error text never includes row contents.
pub fn spawn_cleanup(pool: PgPool, clock: Arc<dyn Clock>) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            match purge(&pool, clock.now()).await {
                Ok(removed) if removed > 0 => {
                    tracing::info!(removed, "auth cleanup removed rows that stopped working");
                }
                Ok(_) => {}
                Err(_) => tracing::warn!("auth cleanup failed and will retry"),
            }
            tokio::time::sleep(EVERY).await;
        }
    })
}
