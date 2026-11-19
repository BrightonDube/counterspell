//! `GhostLookup`: the stored ghost input of a custom ghost run (problem-catalogue C33, CAT-L6).
//!
//! A custom ghost run keeps its single input in `submissions.ghost_input` (ide-workspace T02).
//! The lookup returns it only to the submission's owner.

use counterspell_common::ids::{RunId, UserId};
use serde_json::Value;
use sqlx::PgPool;

/// The ghost input stored with a submission.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredGhost {
    /// The input the ghost run replays the learner's code on.
    pub input: Value,
}

/// Reads stored ghost inputs through the server's own database role.
#[derive(Debug, Clone)]
pub struct GhostLookup {
    pool: PgPool,
}

impl GhostLookup {
    /// Wraps a pool that connects as the server role.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Returns the ghost input of `submission` when `user` owns it and it has one.
    ///
    /// A missing submission, a foreign submission and a submission without a ghost all return
    /// `None`, so the answer never reveals whether another learner's run exists.
    ///
    /// # Errors
    ///
    /// Returns the database error when the query fails.
    pub async fn stored_for(
        &self,
        user: UserId,
        submission: RunId,
    ) -> Result<Option<StoredGhost>, sqlx::Error> {
        // The owner check is part of the WHERE clause, so a forgotten Rust check cannot leak a row.
        let input: Option<Value> = sqlx::query_scalar(
            "SELECT ghost_input FROM submissions \
             WHERE id = $1 AND user_id = $2 AND ghost_input IS NOT NULL",
        )
        .bind(submission.as_uuid())
        .bind(user.as_uuid())
        .fetch_optional(&self.pool)
        .await?;
        Ok(input.map(|input| StoredGhost { input }))
    }
}
