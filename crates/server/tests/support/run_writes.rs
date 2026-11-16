//! Writes that stand in for the judge when a test needs a run in a known state: numbered events, and the
//! terminal result. These write rows directly, so a test can control timing. The judge's own writes are
//! covered by the judge crate's tests.

use counterspell_common::ids::RunId;
use sqlx::PgPool;

/// Adds non-terminal `test` events numbered `from` to `from + count - 1`.
pub async fn write_events_from(pool: &PgPool, run: RunId, from: i32, count: i32) {
    sqlx::query(
        "INSERT INTO run_events (submission_id, seq, attempt, type, payload) \
         SELECT $1, g, 1, 'test', '{}'::jsonb FROM generate_series($2, $2 + $3 - 1) AS g",
    )
    .bind(run.as_uuid())
    .bind(from)
    .bind(count)
    .execute(pool)
    .await
    .expect("insert events");
}

/// Adds `count` non-terminal events numbered from 1.
pub async fn write_events(pool: &PgPool, run: RunId, count: i32) {
    write_events_from(pool, run, 1, count).await;
}

/// Writes the terminal `done` event at `last_seq` and moves the run to `completed`, as the judge does.
pub async fn finish_run(pool: &PgPool, run: RunId, last_seq: i32) {
    let id = run.as_uuid();
    sqlx::query(
        "INSERT INTO run_events (submission_id, seq, attempt, type, payload) \
         VALUES ($1, $2, 1, 'done', '{\"passed\":1,\"total\":2}'::jsonb)",
    )
    .bind(id)
    .bind(last_seq)
    .execute(pool)
    .await
    .expect("done event");
    for status in ["claimed", "compiling", "running"] {
        sqlx::query("UPDATE submissions SET status = $2 WHERE id = $1")
            .bind(id)
            .bind(status)
            .execute(pool)
            .await
            .expect("move status");
    }
    sqlx::query(
        "UPDATE submissions SET status = 'completed', verdict = 'wrong_answer', passed = 1, total = 2, \
         score = 50, runtime_ms = 41, memory_kb = 9210, started_at = now(), finished_at = now(), \
         last_event_seq = $2 WHERE id = $1",
    )
    .bind(id)
    .bind(last_seq)
    .execute(pool)
    .await
    .expect("complete run");
}
