//! The fence: proves, inside a transaction, that the caller still holds the claim.
//!
//! Every judge write starts with this check. It locks the job row, so a second
//! writer waits for the first and then sees the new owner. An old token matches
//! nothing and the caller gets `LeaseLost`.

use super::{ClaimToken, QueueError};
use sqlx::{Postgres, Transaction};

/// Locks the job row if `token` still matches it (claimed, same owner, same attempt).
///
/// The lock is held until the transaction ends. Lock order is always the job row
/// first, then the submission row.
///
/// # Errors
///
/// Returns `LeaseLost` when the claim no longer matches, or a database error.
pub(crate) async fn lock_claim(
    tx: &mut Transaction<'_, Postgres>,
    token: &ClaimToken,
) -> Result<(), QueueError> {
    let held: Option<i32> = sqlx::query_scalar(
        "SELECT 1 FROM jobs \
         WHERE id = $1 AND state = 'claimed' AND lease_owner = $2 AND attempts = $3 \
         FOR UPDATE",
    )
    .bind(token.job_id())
    .bind(token.lease_owner())
    .bind(token.attempts())
    .fetch_optional(&mut **tx)
    .await?;
    held.map(|_| ()).ok_or(QueueError::LeaseLost)
}
