//! Appending run events. Every event is written through a `PublicEvent`, so no
//! caller can store raw JSON in `run_events`.
//!
//! Each append does three things in one transaction: it checks the fence, moves the
//! run's status as the event requires, and takes the next sequence number. The
//! sequence counter lives on the submission row, so `seq` order equals commit order.

use super::fence::lock_claim;
use super::{ClaimToken, QueueError};
use counterspell_common::events::PublicEvent;
use counterspell_common::ids::RunId;
use counterspell_common::run_state::status_after_event;
use counterspell_common::status::RunStatus;
use sqlx::{PgPool, Postgres, Transaction};

/// Stores one public event for the run held by `token` and returns its sequence number.
///
/// Terminal events (`done` and `error`) are refused here. Only the terminal write
/// and the retry path may end a run, because they also set the result columns.
///
/// # Errors
///
/// Returns `LeaseLost` when the claim no longer matches (nothing is written),
/// `EventRejected` when the event is terminal or the run cannot take it now, or a
/// database error.
pub async fn emit(
    pool: &PgPool,
    token: &ClaimToken,
    event: &PublicEvent,
) -> Result<i32, QueueError> {
    if matches!(event, PublicEvent::Done(_) | PublicEvent::Error(_)) {
        return Err(QueueError::EventRejected);
    }
    let mut tx = pool.begin().await?;
    lock_claim(&mut tx, token).await?;
    let seq = append_event_locked(&mut tx, token.run_id(), token.attempts() + 1, event).await?;
    tx.commit().await?;
    Ok(seq)
}

/// Appends an event inside a transaction whose claim is already fenced.
///
/// The caller must lock the claim first. This function locks the submission row,
/// checks the status rule, bumps the counter, stores the event and sends a
/// `NOTIFY` that carries only the run id. An `error` event also sets `finished_at`
/// and `failure_reason` in the same update, which the table's CHECK requires. The
/// notice is sent at commit.
///
/// # Errors
///
/// Returns `EventRejected` when the status rule refuses the event, `NotFound` when
/// the run does not exist, or a database error.
pub(crate) async fn append_event_locked(
    tx: &mut Transaction<'_, Postgres>,
    run: RunId,
    attempt: i16,
    event: &PublicEvent,
) -> Result<i32, QueueError> {
    let current_text: String =
        sqlx::query_scalar("SELECT status FROM submissions WHERE id = $1 FOR UPDATE")
            .bind(run.as_uuid())
            .fetch_optional(&mut **tx)
            .await?
            .ok_or(QueueError::NotFound)?;
    let current = RunStatus::parse(&current_text).ok_or_else(|| {
        QueueError::Database(sqlx::Error::ColumnDecode {
            index: "status".to_owned(),
            source: "unknown run status in the database".into(),
        })
    })?;
    let next = status_after_event(current, event).ok_or(QueueError::EventRejected)?;
    let failure = match event {
        PublicEvent::Error(error) => Some(error.reason().as_str()),
        _ => None,
    };

    let seq: i32 = sqlx::query_scalar(
        "UPDATE submissions \
         SET last_event_seq = last_event_seq + 1, status = $2, \
             finished_at = CASE WHEN $3::text IS NULL THEN finished_at ELSE now() END, \
             failure_reason = COALESCE($3, failure_reason) \
         WHERE id = $1 AND finished_at IS NULL \
         RETURNING last_event_seq",
    )
    .bind(run.as_uuid())
    .bind(next.as_str())
    .bind(failure)
    .fetch_one(&mut **tx)
    .await?;

    sqlx::query(
        "INSERT INTO run_events (submission_id, seq, attempt, type, payload) \
         VALUES ($1, $2, $3, $4, $5)",
    )
    .bind(run.as_uuid())
    .bind(seq)
    .bind(attempt)
    .bind(event.event_type())
    .bind(event.payload())
    .execute(&mut **tx)
    .await?;

    sqlx::query("SELECT pg_notify('run_events', $1)")
        .bind(run.to_string())
        .execute(&mut **tx)
        .await?;
    Ok(seq)
}
