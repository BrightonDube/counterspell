//! `GET /api/runs/{id}/events`: the live event stream for one run (design Path 9, requirements R12 and R13).
//!
//! The stream never trusts a notification. It checks ownership first, then replays every event after the
//! client's cursor from Postgres, and only then waits for a wake-up hint or the poll timer. Every event
//! query contains the owner predicate, so no code path can return another user's events.

use std::{collections::VecDeque, convert::Infallible, str::FromStr, time::Duration};

use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{
        sse::{Event, Sse},
        IntoResponse, Response,
    },
};
use counterspell_common::ids::{RunId, UserId};
use futures_util::stream;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;
use tokio::sync::broadcast::{self, error::RecvError};

use crate::{error::ApiError, runs::wake::Wake, AppState};

/// How many events one replay query reads. Must match the `LIMIT` in [`REPLAY_SQL`].
pub const REPLAY_BATCH: usize = 200;

/// Reads the events of one run after a cursor. The owner join is part of the query, so a run that belongs
/// to someone else returns no rows.
pub const REPLAY_SQL: &str = r"
    SELECT e.seq, e.attempt, e.type AS kind, e.payload
      FROM run_events e
      JOIN submissions s ON s.id = e.submission_id
     WHERE e.submission_id = $1 AND s.user_id = $2 AND e.seq > $3
     ORDER BY e.seq
     LIMIT 200
";

/// Reads the last sequence number and whether the run has finished, for the caller's own run only.
const POSITION_SQL: &str = r"
    SELECT last_event_seq, finished_at IS NOT NULL AS finished
      FROM submissions
     WHERE id = $1 AND user_id = $2
";

/// One stored event, as read by [`replay_events_from`].
#[derive(Debug, sqlx::FromRow)]
pub struct EventRow {
    /// The event's position in its run, starting at 1.
    pub seq: i32,
    /// The attempt that wrote the event.
    pub attempt: i16,
    /// The event type, such as `test` or `done`. Also the SSE `event:` name.
    pub kind: String,
    /// The public payload, already checked by the database to be a JSON object.
    pub payload: serde_json::Value,
}

/// Where a run stands, read before any event is sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, sqlx::FromRow)]
pub struct RunPosition {
    /// The highest event sequence number stored for the run.
    pub last_seq: i32,
    /// True once the run has a final result.
    pub finished: bool,
}

/// The query-string form of the cursor, for clients that cannot send the `Last-Event-ID` header.
#[derive(Debug, Deserialize)]
pub struct StreamQuery {
    /// Cursor: the last sequence number the client has seen.
    last_event_id: Option<String>,
}

/// The SSE envelope every event's `data` field carries.
#[derive(Serialize)]
struct Envelope<'a> {
    seq: i32,
    attempt: i16,
    payload: &'a serde_json::Value,
}

/// The `400` returned for a cursor that is not a non-negative whole number.
fn invalid_cursor() -> ApiError {
    ApiError::bad_request(
        "invalid_last_event_id",
        "Last-Event-ID must be a non-negative whole number.",
    )
}

/// Reads the resume point. The `Last-Event-ID` header wins over the query string, and no cursor means 0.
///
/// # Errors
///
/// `400 invalid_last_event_id` when the text is not a non-negative whole number that fits an `int`.
pub fn parse_cursor(header: Option<&str>, query: Option<&str>) -> Result<i32, ApiError> {
    match header.or(query) {
        None => Ok(0),
        Some(text) => {
            if text.is_empty() || !text.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(invalid_cursor());
            }
            text.parse::<i32>().map_err(|_| invalid_cursor())
        }
    }
}

/// Reads one page of a run's events after `cursor`, for the caller's own run only.
///
/// # Errors
///
/// Returns the database error. An empty result means no new events, or a run the caller does not own.
pub async fn replay_events_from(
    pool: &PgPool,
    owner: UserId,
    run: RunId,
    cursor: i32,
) -> Result<Vec<EventRow>, sqlx::Error> {
    sqlx::query_as(REPLAY_SQL)
        .bind(run.as_uuid())
        .bind(owner.as_uuid())
        .bind(cursor)
        .fetch_all(pool)
        .await
}

/// Reads where a run stands. `None` means the run is missing or belongs to someone else.
///
/// # Errors
///
/// Returns the database error.
pub async fn run_position(
    pool: &PgPool,
    owner: UserId,
    run: RunId,
) -> Result<Option<RunPosition>, sqlx::Error> {
    sqlx::query_as(POSITION_SQL)
        .bind(run.as_uuid())
        .bind(owner.as_uuid())
        .fetch_optional(pool)
        .await
}

/// `GET /api/runs/{id}/events`. Replays every event after the cursor, then streams new ones until the
/// run's terminal event.
///
/// # Errors
///
/// `401` with no session, `400` for a bad cursor, `404 not_found` for a missing or foreign run (the same
/// body for both), and `500` for a database failure.
pub async fn stream_run(
    State(state): State<AppState>,
    headers: HeaderMap,
    Path(id): Path<String>,
    Query(query): Query<StreamQuery>,
) -> Result<Response, ApiError> {
    let user = state
        .user_resolver
        .resolve(&headers)
        .await
        .ok_or_else(ApiError::unauthorized)?;
    let header = match headers.get("last-event-id") {
        Some(value) => Some(value.to_str().map_err(|_| invalid_cursor())?),
        None => None,
    };
    let cursor = parse_cursor(header, query.last_event_id.as_deref())?;

    let not_found = || ApiError::not_found("Run not found.");
    let run = RunId::from_str(&id).map_err(|_| not_found())?;
    let position = run_position(&state.pool, user, run)
        .await
        .map_err(ApiError::internal)?
        .ok_or_else(not_found)?;
    if position.finished && cursor >= position.last_seq {
        return Ok(StatusCode::NO_CONTENT.into_response());
    }

    // Subscribe before the first query, so no wake-up sent during the replay is lost.
    let replay = Replay {
        pool: state.pool.clone(),
        owner: user,
        run,
        cursor,
        waker: state.run_waker.subscribe(),
        pending: VecDeque::new(),
        caught_up: false,
        finished: false,
        poll: Duration::from_millis(state.config.sse.poll_ms),
    };
    Ok(Sse::new(stream::unfold(replay, next_event)).into_response())
}

/// The state carried between stream items.
struct Replay {
    pool: PgPool,
    owner: UserId,
    run: RunId,
    /// The sequence number of the last event sent.
    cursor: i32,
    waker: broadcast::Receiver<Wake>,
    /// Events read from Postgres and not yet sent.
    pending: VecDeque<EventRow>,
    /// True when the last query returned less than a full batch, so the next step should wait.
    caught_up: bool,
    /// True once the terminal event has been sent.
    finished: bool,
    poll: Duration,
}

/// Produces the next SSE event, or ends the stream. The stream ends after a terminal event, or when a
/// database read fails. The client then reconnects with its cursor and loses nothing.
async fn next_event(mut replay: Replay) -> Option<(Result<Event, Infallible>, Replay)> {
    loop {
        if let Some(row) = replay.pending.pop_front() {
            replay.cursor = row.seq;
            replay.finished = is_terminal(&row.kind);
            return match to_sse_event(&row) {
                Ok(event) => Some((Ok(event), replay)),
                Err(error) => {
                    tracing::error!(%error, "could not encode a run event; closing stream");
                    None
                }
            };
        }
        if replay.finished {
            return None;
        }
        if replay.caught_up {
            wait_for_change(&mut replay.waker, replay.run, replay.poll).await;
        }
        match replay_events_from(&replay.pool, replay.owner, replay.run, replay.cursor).await {
            Ok(rows) => {
                replay.caught_up = rows.len() < REPLAY_BATCH;
                replay.pending = rows.into();
            }
            Err(error) => {
                tracing::error!(%error, "event replay failed; closing stream");
                return None;
            }
        }
    }
}

/// Waits until a wake-up for this run arrives or the poll timer ends. Wake-ups for other runs are ignored.
async fn wait_for_change(waker: &mut broadcast::Receiver<Wake>, run: RunId, poll: Duration) {
    let timer = tokio::time::sleep(poll);
    tokio::pin!(timer);
    loop {
        tokio::select! {
            () = &mut timer => return,
            received = waker.recv() => match received {
                Ok(wake) if wake.concerns(run) => return,
                // A wake-up for another run: keep waiting.
                Ok(_) => {}
                // Missed some wake-ups: read again now, because the poll would find the events anyway.
                Err(RecvError::Lagged(_)) => return,
                // The waker is gone: fall back to polling alone.
                Err(RecvError::Closed) => {
                    tokio::time::sleep(poll).await;
                    return;
                }
            },
        }
    }
}

/// Builds the SSE frame: `id` is the sequence number, `event` the type, and `data` the JSON envelope.
fn to_sse_event(row: &EventRow) -> Result<Event, serde_json::Error> {
    let data = serde_json::to_string(&Envelope {
        seq: row.seq,
        attempt: row.attempt,
        payload: &row.payload,
    })?;
    Ok(Event::default()
        .id(row.seq.to_string())
        .event(row.kind.as_str())
        .data(data))
}

/// Whether an event type ends a run's stream.
fn is_terminal(kind: &str) -> bool {
    matches!(kind, "done" | "error")
}

#[cfg(test)]
mod tests {
    use super::{parse_cursor, REPLAY_SQL};

    #[test]
    fn cursor_comes_from_the_header_first() {
        assert_eq!(parse_cursor(Some("5"), Some("9")).expect("valid"), 5);
    }

    #[test]
    fn cursor_comes_from_the_query_when_there_is_no_header() {
        assert_eq!(parse_cursor(None, Some("7")).expect("valid"), 7);
    }

    #[test]
    fn cursor_is_zero_when_neither_is_given() {
        assert_eq!(parse_cursor(None, None).expect("valid"), 0);
    }

    #[test]
    fn cursor_rejects_values_that_are_not_non_negative_integers() {
        for bad in ["-1", "abc", "", "1.5", " 3", "+2", "99999999999"] {
            let error = parse_cursor(Some(bad), None).expect_err(bad);
            assert_eq!(error.status(), axum::http::StatusCode::BAD_REQUEST, "{bad}");
        }
    }

    #[test]
    fn replay_query_joins_the_owner_and_reads_after_the_cursor() {
        assert!(REPLAY_SQL.contains("JOIN submissions s ON s.id = e.submission_id"));
        assert!(REPLAY_SQL.contains("s.user_id = $2"));
        assert!(REPLAY_SQL.contains("e.seq > $3"));
        assert!(REPLAY_SQL.contains("ORDER BY e.seq"));
        assert!(REPLAY_SQL.contains("LIMIT 200"));
    }
}
