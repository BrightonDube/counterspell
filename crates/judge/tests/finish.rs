//! Database tests for the terminal write (run-lifecycle T14, design Path 5).
//!
//! A run ends in exactly one transaction: the fence, the result columns, the test
//! rows, one `done` (or `error`) event and the job state. Three barriers protect
//! it: the fence, the frozen-row trigger and the unique terminal event index.
//!
//! Run with `cargo test -p counterspell-judge --test finish -- --ignored pg_`.

use counterspell_common::events::{PublicEvent, PublicTest, SampleDetail};
use counterspell_common::status::{FailureReason, TestVerdict, Verdict};
use counterspell_judge::queue::emit::emit;
use counterspell_judge::queue::finish::{finish, finish_failed, FinalResult, TestRecord};
use counterspell_judge::queue::QueueError;
use counterspell_judge::testkit::{assert_invariants, force_claim, seed_run, token_with};
use sqlx::PgPool;
use uuid::Uuid;

const UNIQUE_VIOLATION: &str = "23505";

/// A result with two hidden tests and one sample, all passing.
fn passing_result() -> FinalResult {
    FinalResult {
        verdict: Verdict::Accepted,
        passed: 3,
        total: 3,
        score: 100,
        runtime_ms: 12,
        compile_output: None,
        tests: vec![
            TestRecord::Sample {
                ordinal: 1,
                verdict: TestVerdict::Pass,
                time_ms: 3,
                detail: SampleDetail::new("[2,7]", "[0,1]", "[0,1]"),
            },
            TestRecord::Hidden {
                ordinal: 2,
                verdict: TestVerdict::Pass,
                time_ms: 4,
            },
            TestRecord::Hidden {
                ordinal: 3,
                verdict: TestVerdict::Pass,
                time_ms: 5,
            },
        ],
    }
}

/// Moves a claimed run to `running` so that a finish is allowed.
async fn start_running(
    pool: &PgPool,
    run: counterspell_common::ids::RunId,
) -> counterspell_judge::queue::ClaimToken {
    let token = force_claim(pool, run, "worker-a").await.unwrap();
    emit(pool, &token, &PublicEvent::Compiling).await.unwrap();
    emit(
        pool,
        &token,
        &PublicEvent::Test(PublicTest::Hidden {
            ordinal: 1,
            verdict: TestVerdict::Pass,
            time_ms: 1,
        }),
    )
    .await
    .unwrap();
    token
}

/// The run's status and job state, as text.
async fn states(pool: &PgPool, run: Uuid) -> (String, String) {
    sqlx::query_as(
        "SELECT s.status, j.state FROM submissions s JOIN jobs j ON j.submission_id = s.id \
         WHERE s.id = $1",
    )
    .bind(run)
    .fetch_one(pool)
    .await
    .unwrap()
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_finish_writes_done_event_matching_columns(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let token = start_running(&pool, run).await;
    let result = FinalResult {
        verdict: Verdict::WrongAnswer,
        passed: 1,
        total: 3,
        score: 33,
        runtime_ms: 9,
        compile_output: None,
        tests: passing_result().tests,
    };

    finish(&pool, &token, &result).await.unwrap();

    let (status, verdict, passed, total, score): (
        String,
        Option<String>,
        Option<i32>,
        Option<i32>,
        Option<i32>,
    ) = sqlx::query_as(
        "SELECT status, verdict, passed, total, score FROM submissions WHERE id = $1",
    )
    .bind(run.as_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(status, "completed");
    assert_eq!(verdict.as_deref(), Some("wrong_answer"));
    assert_eq!((passed, total, score), (Some(1), Some(3), Some(33)));

    let payload: serde_json::Value = sqlx::query_scalar(
        "SELECT payload FROM run_events WHERE submission_id = $1 AND type = 'done'",
    )
    .bind(run.as_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(payload["verdict"], "wrong_answer");
    assert_eq!(payload["passed"], 1);
    assert_eq!(payload["total"], 3);
    assert_eq!(payload["score"], 33);
    assert_invariants(&pool).await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_finish_hidden_tests_store_no_detail(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let token = start_running(&pool, run).await;

    finish(&pool, &token, &passing_result()).await.unwrap();

    let rows: Vec<(i32, bool, Option<serde_json::Value>)> = sqlx::query_as(
        "SELECT ordinal, is_sample, public_detail FROM test_results \
         WHERE submission_id = $1 ORDER BY ordinal",
    )
    .bind(run.as_uuid())
    .fetch_all(&pool)
    .await
    .unwrap();
    assert_eq!(rows.len(), 3);
    assert!(rows[0].1, "the first test is a sample");
    let detail = rows[0].2.as_ref().expect("samples keep their detail");
    assert_eq!(detail["input"], "[2,7]");
    assert_eq!(detail["expected"], "[0,1]");
    for (_, is_sample, detail) in &rows[1..] {
        assert!(!is_sample);
        assert!(detail.is_none(), "hidden tests keep no input or output");
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_terminal_write_exactly_once(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let token = start_running(&pool, run).await;

    // Two tasks race to finish the same claim. Only one may write the result.
    let first = {
        let (pool, token) = (pool.clone(), token.clone());
        tokio::spawn(async move { finish(&pool, &token, &passing_result()).await })
    };
    let second = {
        let (pool, token) = (pool.clone(), token.clone());
        tokio::spawn(async move { finish(&pool, &token, &passing_result()).await })
    };
    let results = [first.await.unwrap(), second.await.unwrap()];
    let wins = results.iter().filter(|result| result.is_ok()).count();
    let losses = results
        .iter()
        .filter(|result| matches!(result, Err(QueueError::LeaseLost)))
        .count();
    assert_eq!((wins, losses), (1, 1), "{results:?}");

    let done_events: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM run_events WHERE submission_id = $1 AND type = 'done'",
    )
    .bind(run.as_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(done_events, 1);
    assert_invariants(&pool).await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_stale_owner_finish_rejected(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let token = start_running(&pool, run).await;
    let stranger = token_with(&token, "worker-b", token.attempts());

    let result = finish(&pool, &stranger, &passing_result()).await;
    assert!(matches!(result, Err(QueueError::LeaseLost)), "{result:?}");

    assert_eq!(states(&pool, run.as_uuid()).await.0, "running");
    let test_rows: i64 =
        sqlx::query_scalar("SELECT count(*) FROM test_results WHERE submission_id = $1")
            .bind(run.as_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(test_rows, 0);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_finish_failed_cancelled_shape(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let token = force_claim(&pool, run, "worker-a").await.unwrap();

    finish_failed(&pool, &token, FailureReason::Cancelled)
        .await
        .unwrap();

    let (status, reason, job_state): (String, Option<String>, String) = sqlx::query_as(
        "SELECT s.status, s.failure_reason, j.state FROM submissions s \
         JOIN jobs j ON j.submission_id = s.id WHERE s.id = $1",
    )
    .bind(run.as_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(status, "failed");
    assert_eq!(reason.as_deref(), Some("cancelled"));
    assert_eq!(job_state, "failed");

    let payload: serde_json::Value = sqlx::query_scalar(
        "SELECT payload FROM run_events WHERE submission_id = $1 AND type = 'error'",
    )
    .bind(run.as_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(payload["code"], "cancelled");
    assert_invariants(&pool).await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_second_terminal_event_rejected(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let token = start_running(&pool, run).await;
    finish(&pool, &token, &passing_result()).await.unwrap();

    // Even a direct insert, bypassing the Rust code, cannot add a second terminal event.
    let error = sqlx::query(
        "INSERT INTO run_events (submission_id, seq, attempt, type, payload) \
         VALUES ($1, 99, 1, 'error', '{\"code\":\"cancelled\",\"message\":\"x\"}')",
    )
    .bind(run.as_uuid())
    .execute(&pool)
    .await
    .expect_err("a second terminal event must be refused");
    let code = error
        .as_database_error()
        .and_then(|db| db.code().map(std::borrow::Cow::into_owned));
    assert_eq!(code.as_deref(), Some(UNIQUE_VIOLATION));
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_finish_is_atomic(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let token = start_running(&pool, run).await;
    // Two test rows with the same ordinal. The submission update runs first, then the
    // second test insert fails, so the whole transaction must roll back.
    let broken = FinalResult {
        tests: vec![
            TestRecord::Hidden {
                ordinal: 5,
                verdict: TestVerdict::Pass,
                time_ms: 1,
            },
            TestRecord::Hidden {
                ordinal: 5,
                verdict: TestVerdict::Pass,
                time_ms: 1,
            },
        ],
        ..passing_result()
    };

    let error = finish(&pool, &token, &broken)
        .await
        .expect_err("a duplicate ordinal must fail the finish");
    let code = match error {
        QueueError::Database(sqlx_error) => sqlx_error
            .as_database_error()
            .and_then(|db| db.code().map(std::borrow::Cow::into_owned)),
        other => panic!("expected a database error, got {other:?}"),
    };
    assert_eq!(code.as_deref(), Some(UNIQUE_VIOLATION));

    assert_eq!(
        states(&pool, run.as_uuid()).await,
        ("running".into(), "claimed".into())
    );
    let rows: i64 =
        sqlx::query_scalar("SELECT count(*) FROM test_results WHERE submission_id = $1")
            .bind(run.as_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(rows, 0, "no test rows survive a failed finish");
    let done: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM run_events WHERE submission_id = $1 AND type = 'done'",
    )
    .bind(run.as_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!(done, 0);
    assert_invariants(&pool).await.unwrap();
}
