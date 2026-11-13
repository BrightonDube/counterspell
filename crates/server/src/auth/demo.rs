//! The judge demo account: seeding it at start-up. The daily reset is a separate step (T28).
//!
//! The seed runs through the database function `cs_seed_demo_user`, which is the only code that
//! may create or change the demo account. This module never writes to `users` directly.

use sqlx::PgPool;

use super::{
    config::DemoConfig,
    errors::{database, AuthError},
    password::{HashGate, Verdict},
};

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
