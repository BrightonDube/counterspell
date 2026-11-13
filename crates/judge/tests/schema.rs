//! Database tests for the run-lifecycle schema: CHECK constraints, unique indexes,
//! the frozen-row trigger, the transition guard, and agreement with the Rust table.
//!
//! These tests need `DATABASE_URL` and run with `cargo test -p counterspell-judge
//! --test schema -- --ignored pg_`. They use the database owner, not the roles.

use counterspell_common::run_state::can_transition;
use counterspell_common::status::RunStatus;
use sqlx::PgPool;
use uuid::Uuid;

const CHECK_VIOLATION: &str = "23514";
const UNIQUE_VIOLATION: &str = "23505";

/// Returns the SQLSTATE of a rejected statement, and fails the test if the statement succeeded.
fn rejected_with<T>(result: Result<T, sqlx::Error>) -> String {
    match result {
        Ok(_) => panic!("the database accepted a statement it must reject"),
        Err(error) => error
            .as_database_error()
            .and_then(|db| db.code().map(std::borrow::Cow::into_owned))
            .unwrap_or_else(|| panic!("expected a database error, got: {error}")),
    }
}

/// Inserts one queued submission for `user` and returns its id.
async fn insert_submission(
    pool: &PgPool,
    user: Uuid,
    key: Option<&str>,
) -> Result<Uuid, sqlx::Error> {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, idempotency_key) \
         VALUES ($1, $2, 'two-sum', 'python', 'submit', 'print(1)', $3, $4)",
    )
    .bind(id)
    .bind(user)
    .bind(vec![0u8; 32])
    .bind(key)
    .execute(pool)
    .await?;
    Ok(id)
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_check_constraints_reject_bad_rows(pool: PgPool) {
    let user = Uuid::new_v4();
    let run = insert_submission(&pool, user, None).await.unwrap();

    // A status outside the nine states.
    let unknown_status = sqlx::query("UPDATE submissions SET status = 'nope' WHERE id = $1")
        .bind(run)
        .execute(&pool)
        .await;
    assert_eq!(rejected_with(unknown_status), CHECK_VIOLATION);

    // A verdict on a run that is not completed.
    let verdict_without_completion = sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, status, verdict, finished_at) \
         VALUES ($1, $2, 'two-sum', 'python', 'submit', 'x', $3, 'queued', 'accepted', NULL)",
    )
    .bind(Uuid::new_v4())
    .bind(user)
    .bind(vec![0u8; 32])
    .execute(&pool)
    .await;
    assert_eq!(rejected_with(verdict_without_completion), CHECK_VIOLATION);

    // A completed run must have a finish time.
    let completed_without_finish = sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, status, verdict) \
         VALUES ($1, $2, 'two-sum', 'python', 'submit', 'x', $3, 'completed', 'accepted')",
    )
    .bind(Uuid::new_v4())
    .bind(user)
    .bind(vec![0u8; 32])
    .execute(&pool)
    .await;
    assert_eq!(rejected_with(completed_without_finish), CHECK_VIOLATION);

    // A hidden test may not store its input, output or program output.
    let hidden_with_detail = sqlx::query(
        "INSERT INTO test_results (submission_id, ordinal, is_sample, verdict, time_ms, public_detail) \
         VALUES ($1, 1, false, 'wrong_answer', 3, '{\"input\": \"1\"}')",
    )
    .bind(run)
    .execute(&pool)
    .await;
    assert_eq!(rejected_with(hidden_with_detail), CHECK_VIOLATION);

    // An event payload over 16 KiB is refused.
    let big = "a".repeat(17_000);
    let oversized = sqlx::query(
        "INSERT INTO run_events (submission_id, seq, attempt, type, payload) VALUES ($1, 1, 1, 'test', $2)",
    )
    .bind(run)
    .bind(serde_json::json!({ "blob": big }))
    .execute(&pool)
    .await;
    assert_eq!(rejected_with(oversized), CHECK_VIOLATION);

    // An event type outside the eleven public names is refused.
    let unknown_event = sqlx::query(
        "INSERT INTO run_events (submission_id, seq, attempt, type, payload) VALUES ($1, 1, 1, 'secret', '{}')",
    )
    .bind(run)
    .execute(&pool)
    .await;
    assert_eq!(rejected_with(unknown_event), CHECK_VIOLATION);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_unique_job_per_submission(pool: PgPool) {
    let run = insert_submission(&pool, Uuid::new_v4(), None)
        .await
        .unwrap();
    sqlx::query("INSERT INTO jobs (submission_id) VALUES ($1)")
        .bind(run)
        .execute(&pool)
        .await
        .unwrap();
    let second = sqlx::query("INSERT INTO jobs (submission_id) VALUES ($1)")
        .bind(run)
        .execute(&pool)
        .await;
    assert_eq!(rejected_with(second), UNIQUE_VIOLATION);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_one_terminal_event_per_run(pool: PgPool) {
    let run = insert_submission(&pool, Uuid::new_v4(), None)
        .await
        .unwrap();
    sqlx::query(
        "INSERT INTO run_events (submission_id, seq, attempt, type, payload) VALUES ($1, 1, 1, 'done', '{}')",
    )
    .bind(run)
    .execute(&pool)
    .await
    .unwrap();
    let second = sqlx::query(
        "INSERT INTO run_events (submission_id, seq, attempt, type, payload) VALUES ($1, 2, 1, 'error', '{}')",
    )
    .bind(run)
    .execute(&pool)
    .await;
    assert_eq!(rejected_with(second), UNIQUE_VIOLATION);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_terminal_row_is_frozen(pool: PgPool) {
    let user = Uuid::new_v4();
    let run = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, status, verdict, passed, total, last_event_seq, finished_at) \
         VALUES ($1, $2, 'two-sum', 'python', 'submit', 'x', $3, 'completed', 'accepted', 3, 3, 1, now())",
    )
    .bind(run)
    .bind(user)
    .bind(vec![0u8; 32])
    .execute(&pool)
    .await
    .unwrap();

    let attempts = [
        "UPDATE submissions SET status = 'failed', failure_reason = 'cancelled' WHERE id = $1",
        "UPDATE submissions SET verdict = 'wrong_answer' WHERE id = $1",
        "UPDATE submissions SET passed = 0 WHERE id = $1",
        "UPDATE submissions SET finished_at = NULL WHERE id = $1",
        "UPDATE submissions SET user_id = gen_random_uuid() WHERE id = $1",
        "UPDATE submissions SET code_sha256 = '\\x00' WHERE id = $1",
        "UPDATE submissions SET last_event_seq = 2 WHERE id = $1",
    ];
    for sql in attempts {
        let result = sqlx::query(sql).bind(run).execute(&pool).await;
        assert_eq!(rejected_with(result), CHECK_VIOLATION, "{sql}");
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_transition_guard_rejects_illegal_move(pool: PgPool) {
    let run = insert_submission(&pool, Uuid::new_v4(), None)
        .await
        .unwrap();

    // Positive control: the table allows queued to claimed.
    sqlx::query("UPDATE submissions SET status = 'claimed' WHERE id = $1")
        .bind(run)
        .execute(&pool)
        .await
        .unwrap();

    // Claimed cannot jump straight to running; the first test event must come after compiling.
    let skip_compiling = sqlx::query("UPDATE submissions SET status = 'running' WHERE id = $1")
        .bind(run)
        .execute(&pool)
        .await;
    assert_eq!(rejected_with(skip_compiling), CHECK_VIOLATION);

    // The event counter may never go down.
    sqlx::query("UPDATE submissions SET last_event_seq = 2 WHERE id = $1")
        .bind(run)
        .execute(&pool)
        .await
        .unwrap();
    let decrease = sqlx::query("UPDATE submissions SET last_event_seq = 1 WHERE id = $1")
        .bind(run)
        .execute(&pool)
        .await;
    assert_eq!(rejected_with(decrease), CHECK_VIOLATION);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_status_transition_agreement(pool: PgPool) {
    // The SQL table and the Rust table must agree on all nine by nine pairs.
    let mut checked = 0;
    for from in RunStatus::ALL {
        for to in RunStatus::ALL {
            let sql_allows: bool =
                sqlx::query_scalar("SELECT cs_status_transition_allowed($1, $2)")
                    .bind(from.as_str())
                    .bind(to.as_str())
                    .fetch_one(&pool)
                    .await
                    .unwrap();
            assert_eq!(
                sql_allows,
                can_transition(*from, *to),
                "disagreement for {} -> {}",
                from.as_str(),
                to.as_str()
            );
            checked += 1;
        }
    }
    assert_eq!(checked, 81);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_idempotency_index_is_per_user(pool: PgPool) {
    let user = Uuid::new_v4();
    insert_submission(&pool, user, Some("key-1")).await.unwrap();

    // The same user cannot reuse a key.
    let reused = insert_submission(&pool, user, Some("key-1")).await;
    assert_eq!(rejected_with(reused), UNIQUE_VIOLATION);

    // Another user may use the same key.
    insert_submission(&pool, Uuid::new_v4(), Some("key-1"))
        .await
        .unwrap();

    // Runs without a key never collide.
    insert_submission(&pool, user, None).await.unwrap();
    insert_submission(&pool, user, None).await.unwrap();
}
