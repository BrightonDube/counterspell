//! Database tests for fenced event writes (run-lifecycle T12, design Path 4).
//!
//! An event is written only by the worker that holds the claim, and it changes the
//! run status in the same transaction. Sequence numbers must be gap-free.
//!
//! Run with `cargo test -p counterspell-judge --test emit -- --ignored pg_`.

use counterspell_common::events::{ErrorPayload, PublicEvent, PublicTest};
use counterspell_common::status::{FailureReason, TestVerdict};
use counterspell_judge::queue::emit::emit;
use counterspell_judge::queue::QueueError;
use counterspell_judge::testkit::{assert_invariants, force_claim, seed_run, token_with};
use sqlx::PgPool;
use tokio::task::JoinSet;
use uuid::Uuid;

/// A hidden-test result with a fixed ordinal, so tests stay readable.
fn hidden_test(ordinal: u32) -> PublicEvent {
    PublicEvent::Test(PublicTest::Hidden {
        ordinal,
        verdict: TestVerdict::Pass,
        time_ms: 1,
    })
}

/// The run's status as text, read straight from the table.
async fn status_of(pool: &PgPool, run: Uuid) -> String {
    sqlx::query_scalar("SELECT status FROM submissions WHERE id = $1")
        .bind(run)
        .fetch_one(pool)
        .await
        .unwrap()
}

/// How many events the run has stored.
async fn event_count(pool: &PgPool, run: Uuid) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM run_events WHERE submission_id = $1")
        .bind(run)
        .fetch_one(pool)
        .await
        .unwrap()
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_emit_assigns_gapless_seq(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let token = force_claim(&pool, run, "worker-a").await.unwrap();

    let first = emit(&pool, &token, &PublicEvent::Compiling).await.unwrap();
    let second = emit(&pool, &token, &hidden_test(1)).await.unwrap();
    let third = emit(&pool, &token, &hidden_test(2)).await.unwrap();

    assert_eq!((first, second, third), (1, 2, 3));
    assert_invariants(&pool).await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_emit_changes_status_with_event(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let token = force_claim(&pool, run, "worker-a").await.unwrap();

    emit(&pool, &token, &PublicEvent::Compiling).await.unwrap();
    assert_eq!(status_of(&pool, run.as_uuid()).await, "compiling");

    emit(&pool, &token, &hidden_test(1)).await.unwrap();
    assert_eq!(status_of(&pool, run.as_uuid()).await, "running");

    emit(&pool, &token, &PublicEvent::GhostHunting)
        .await
        .unwrap();
    assert_eq!(status_of(&pool, run.as_uuid()).await, "ghost_hunting");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_emit_with_wrong_owner_or_stale_attempt_writes_nothing(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let token = force_claim(&pool, run, "worker-a").await.unwrap();

    let other_owner = token_with(&token, "worker-b", token.attempts());
    let result = emit(&pool, &other_owner, &PublicEvent::Compiling).await;
    assert!(matches!(result, Err(QueueError::LeaseLost)), "{result:?}");

    let stale_attempt = token_with(&token, "worker-a", token.attempts() + 1);
    let result = emit(&pool, &stale_attempt, &PublicEvent::Compiling).await;
    assert!(matches!(result, Err(QueueError::LeaseLost)), "{result:?}");

    assert_eq!(event_count(&pool, run.as_uuid()).await, 0);
    assert_eq!(status_of(&pool, run.as_uuid()).await, "claimed");
    assert_invariants(&pool).await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_event_seq_gapless_under_concurrency(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let token = force_claim(&pool, run, "worker-a").await.unwrap();
    emit(&pool, &token, &PublicEvent::Compiling).await.unwrap();

    // Eight tasks each append ten test results with the same valid fence.
    let mut writers = JoinSet::new();
    for writer in 0..8u32 {
        let pool = pool.clone();
        let token = token.clone();
        writers.spawn(async move {
            for step in 0..10u32 {
                emit(&pool, &token, &hidden_test(writer * 10 + step + 1))
                    .await
                    .unwrap();
            }
        });
    }
    while let Some(result) = writers.join_next().await {
        result.unwrap();
    }

    let seqs: Vec<i32> =
        sqlx::query_scalar("SELECT seq FROM run_events WHERE submission_id = $1 ORDER BY seq")
            .bind(run.as_uuid())
            .fetch_all(&pool)
            .await
            .unwrap();
    let expected: Vec<i32> = (1..=81).collect();
    assert_eq!(
        seqs, expected,
        "sequence must be exactly 1..=81 with no gaps"
    );
    assert_invariants(&pool).await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_emit_refuses_terminal_events(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let token = force_claim(&pool, run, "worker-a").await.unwrap();
    emit(&pool, &token, &PublicEvent::Compiling).await.unwrap();

    // Only the terminal write and the retry path may end a run, so a plain emit must refuse.
    let ended = PublicEvent::Error(ErrorPayload::for_reason(FailureReason::Cancelled));
    let result = emit(&pool, &token, &ended).await;
    assert!(
        matches!(result, Err(QueueError::EventRejected)),
        "{result:?}"
    );
    assert_eq!(status_of(&pool, run.as_uuid()).await, "compiling");
    assert_invariants(&pool).await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_no_append_after_terminal(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let token = force_claim(&pool, run, "worker-a").await.unwrap();
    emit(&pool, &token, &PublicEvent::Compiling).await.unwrap();

    // Finish the run directly as the owner would, leaving the job claimed. This
    // isolates the status rule: a finished run must refuse further events.
    sqlx::query(
        "UPDATE submissions SET status = 'failed', failure_reason = 'cancelled', \
         last_event_seq = last_event_seq + 1, finished_at = now() WHERE id = $1",
    )
    .bind(run.as_uuid())
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query(
        "INSERT INTO run_events (submission_id, seq, attempt, type, payload) \
         VALUES ($1, 2, 1, 'error', '{\"code\":\"cancelled\",\"message\":\"x\"}')",
    )
    .bind(run.as_uuid())
    .execute(&pool)
    .await
    .unwrap();

    let before = event_count(&pool, run.as_uuid()).await;
    let result = emit(&pool, &token, &hidden_test(1)).await;
    assert!(
        matches!(result, Err(QueueError::EventRejected)),
        "{result:?}"
    );
    assert_eq!(event_count(&pool, run.as_uuid()).await, before);
}
