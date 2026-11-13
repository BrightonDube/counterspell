//! Privilege tests for the server and judge database roles (R19). Each role may do
//! exactly what the privilege matrix in `design.md` allows. Every denied statement
//! runs inside its own savepoint, so one denial does not abort the test transaction.
//!
//! Run with `cargo test -p counterspell-judge --test roles -- --ignored pg_`.

use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

const INSUFFICIENT_PRIVILEGE: &str = "42501";

/// Inserts one queued run and its job as the database owner. Returns the run id.
async fn seed_run(pool: &PgPool) -> Uuid {
    let run = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256) \
         VALUES ($1, $2, 'two-sum', 'python', 'submit', 'print(1)', decode('00', 'hex'))",
    )
    .bind(run)
    .bind(Uuid::new_v4())
    .execute(pool)
    .await
    .unwrap();
    sqlx::query("INSERT INTO jobs (submission_id) VALUES ($1)")
        .bind(run)
        .execute(pool)
        .await
        .unwrap();
    run
}

/// Runs `sql` (with the run id as `$1`) and requires success.
async fn allowed(tx: &mut Transaction<'_, Postgres>, sql: &str, run: Uuid) {
    sqlx::query(sql)
        .bind(run)
        .execute(&mut **tx)
        .await
        .unwrap_or_else(|error| panic!("expected to be allowed: {sql}: {error}"));
}

/// Runs `sql` inside a savepoint and requires the database to refuse it for lack of privilege.
async fn denied(tx: &mut Transaction<'_, Postgres>, sql: &str, run: Uuid) {
    sqlx::query("SAVEPOINT probe")
        .execute(&mut **tx)
        .await
        .unwrap();
    let result = sqlx::query(sql).bind(run).execute(&mut **tx).await;
    sqlx::query("ROLLBACK TO SAVEPOINT probe")
        .execute(&mut **tx)
        .await
        .unwrap();
    let code = match result {
        Ok(_) => panic!("expected to be denied, but it succeeded: {sql}"),
        Err(error) => error
            .as_database_error()
            .and_then(|db| db.code().map(std::borrow::Cow::into_owned))
            .unwrap_or_else(|| panic!("not a database error for {sql}: {error}")),
    };
    assert_eq!(code, INSUFFICIENT_PRIVILEGE, "{sql}");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_role_privileges_server(pool: PgPool) {
    let run = seed_run(&pool).await;
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL ROLE counterspell_server")
        .execute(&mut *tx)
        .await
        .unwrap();

    // The server may create runs and their jobs, read everything, and ask for cancellation.
    allowed(
        &mut tx,
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256) \
         VALUES ($1, gen_random_uuid(), 'two-sum', 'python', 'run', 'x', decode('00', 'hex'))",
        Uuid::new_v4(),
    )
    .await;
    allowed(
        &mut tx,
        "SELECT count(*) FROM submissions WHERE id = $1",
        run,
    )
    .await;
    allowed(
        &mut tx,
        "SELECT count(*) FROM run_events WHERE submission_id = $1",
        run,
    )
    .await;
    allowed(
        &mut tx,
        "SELECT count(*) FROM test_results WHERE submission_id = $1",
        run,
    )
    .await;
    allowed(
        &mut tx,
        "UPDATE jobs SET cancel_requested_at = now() WHERE submission_id = $1",
        run,
    )
    .await;

    // The server cannot forge events, results, verdicts or job state.
    denied(
        &mut tx,
        "INSERT INTO run_events (submission_id, seq, attempt, type, payload) \
         VALUES ($1, 1, 1, 'done', '{}')",
        run,
    )
    .await;
    denied(
        &mut tx,
        "INSERT INTO test_results (submission_id, ordinal, is_sample, verdict, time_ms) \
         VALUES ($1, 1, false, 'pass', 1)",
        run,
    )
    .await;
    denied(
        &mut tx,
        "UPDATE submissions SET status = 'completed', verdict = 'accepted', finished_at = now() \
         WHERE id = $1",
        run,
    )
    .await;
    denied(
        &mut tx,
        "UPDATE jobs SET state = 'succeeded' WHERE submission_id = $1",
        run,
    )
    .await;
    denied(
        &mut tx,
        "UPDATE jobs SET lease_owner = 'me' WHERE submission_id = $1",
        run,
    )
    .await;
    tx.rollback().await.unwrap();
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_role_privileges_judge(pool: PgPool) {
    let run = seed_run(&pool).await;
    let mut tx = pool.begin().await.unwrap();
    sqlx::query("SET LOCAL ROLE counterspell_judge")
        .execute(&mut *tx)
        .await
        .unwrap();

    // The judge may claim a job, move the run along, and write events and results.
    allowed(
        &mut tx,
        "UPDATE jobs SET state = 'claimed', lease_owner = 'worker-a', \
             lease_expires_at = now() + interval '30 seconds', attempts = 0 \
         WHERE submission_id = $1",
        run,
    )
    .await;
    allowed(
        &mut tx,
        "UPDATE submissions SET status = 'claimed', started_at = now() WHERE id = $1",
        run,
    )
    .await;
    allowed(
        &mut tx,
        "INSERT INTO run_events (submission_id, seq, attempt, type, payload) \
         VALUES ($1, 1, 1, 'compiling', '{}')",
        run,
    )
    .await;
    allowed(
        &mut tx,
        "INSERT INTO test_results (submission_id, ordinal, is_sample, verdict, time_ms) \
         VALUES ($1, 1, false, 'pass', 1)",
        run,
    )
    .await;

    // The judge cannot delete runs, change who owns them, or change their code.
    denied(&mut tx, "DELETE FROM submissions WHERE id = $1", run).await;
    denied(
        &mut tx,
        "UPDATE submissions SET user_id = gen_random_uuid() WHERE id = $1",
        run,
    )
    .await;
    denied(
        &mut tx,
        "UPDATE submissions SET code = 'print(2)' WHERE id = $1",
        run,
    )
    .await;
    denied(
        &mut tx,
        "UPDATE submissions SET code_sha256 = decode('01', 'hex') WHERE id = $1",
        run,
    )
    .await;
    denied(
        &mut tx,
        "UPDATE jobs SET submission_id = gen_random_uuid() WHERE submission_id = $1",
        run,
    )
    .await;
    denied(&mut tx, "INSERT INTO jobs (submission_id) VALUES ($1)", run).await;
    tx.rollback().await.unwrap();
}
