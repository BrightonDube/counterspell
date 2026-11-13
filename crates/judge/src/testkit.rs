//! Test helpers for the judge's database tests. Compiled only with the `testkit`
//! feature, so release builds never contain them.
//!
//! These helpers run as a database superuser. They seed rows and age leases with
//! SQL, so tests never sleep to move time forward.

use crate::queue::ClaimToken;
use counterspell_common::ids::{RunId, UserId};
use sqlx::PgPool;

/// Claims a queued run for `owner` without the claim code, as the database owner.
///
/// Sets the job to `claimed` with a lease an hour ahead, and moves a queued run to
/// `claimed`. The attempt count is left as it is. Use this only in tests that are
/// not testing the claim itself.
///
/// # Errors
///
/// Returns the database error when the update fails or no job exists for `run`.
pub async fn force_claim(
    pool: &PgPool,
    run: RunId,
    owner: &str,
) -> Result<ClaimToken, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let (job_id, attempts): (i64, i16) = sqlx::query_as(
        "UPDATE jobs SET state = 'claimed', lease_owner = $2, \
         lease_expires_at = now() + interval '1 hour', claimed_at = now(), updated_at = now() \
         WHERE submission_id = $1 RETURNING id, attempts",
    )
    .bind(run.as_uuid())
    .bind(owner)
    .fetch_one(&mut *tx)
    .await?;
    sqlx::query(
        "UPDATE submissions SET status = 'claimed', started_at = COALESCE(started_at, now()) \
         WHERE id = $1 AND status = 'queued'",
    )
    .bind(run.as_uuid())
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(ClaimToken::new(job_id, run, owner.to_owned(), attempts))
}

/// Copies a token but changes its owner or attempt count. Used to test that a stale
/// or forged claim is refused.
#[must_use]
pub fn token_with(token: &ClaimToken, owner: &str, attempts: i16) -> ClaimToken {
    ClaimToken::new(token.job_id(), token.run_id(), owner.to_owned(), attempts)
}

/// Inserts one queued run and its job in a single transaction.
///
/// # Errors
///
/// Returns the database error when either insert fails.
pub async fn seed_run(pool: &PgPool) -> Result<RunId, sqlx::Error> {
    let run = RunId::new();
    let user = UserId::new();
    let mut tx = pool.begin().await?;
    sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256) \
         VALUES ($1, $2, 'two-sum', 'python', 'submit', 'print(1)', $3)",
    )
    .bind(run.as_uuid())
    .bind(user.as_uuid())
    .bind(vec![0u8; 32])
    .execute(&mut *tx)
    .await?;
    sqlx::query("INSERT INTO jobs (submission_id) VALUES ($1)")
        .bind(run.as_uuid())
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;
    Ok(run)
}

/// Moves a claimed job's lease into the past, as if the worker had stopped renewing.
///
/// # Errors
///
/// Returns the database error when the update fails.
pub async fn age_lease(pool: &PgPool, run: RunId) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE jobs SET lease_expires_at = now() - interval '1 second' \
         WHERE submission_id = $1",
    )
    .bind(run.as_uuid())
    .execute(pool)
    .await?;
    Ok(())
}

/// Moves the run's creation times back by `secs` seconds.
///
/// # Errors
///
/// Returns the database error when the update fails.
pub async fn age_created(pool: &PgPool, run: RunId, secs: i64) -> Result<(), sqlx::Error> {
    sqlx::query(
        "UPDATE submissions SET created_at = created_at - make_interval(secs => $2::float8) \
         WHERE id = $1",
    )
    .bind(run.as_uuid())
    .bind(secs)
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE jobs SET created_at = created_at - make_interval(secs => $2::float8) \
         WHERE submission_id = $1",
    )
    .bind(run.as_uuid())
    .bind(secs)
    .execute(pool)
    .await?;
    Ok(())
}

/// Checks the queue invariants that every test must keep. Each check counts violations
/// with SQL, so a failure names the rule that broke.
///
/// Covered: every submission has a job (P1), job state matches submission status (P7),
/// a finished run has exactly one terminal event that comes last (P6), and event
/// sequence numbers are exactly `1..n` (P8).
///
/// # Errors
///
/// Returns a message that names each failed check, or a database error as text.
pub async fn assert_invariants(pool: &PgPool) -> Result<(), String> {
    const CHECKS: &[(&str, &str)] = &[
        (
            "P1 every submission has a job",
            "SELECT count(*) FROM submissions s \
             WHERE NOT EXISTS (SELECT 1 FROM jobs j WHERE j.submission_id = s.id)",
        ),
        (
            "P7 job state matches submission status",
            "SELECT count(*) FROM submissions s JOIN jobs j ON j.submission_id = s.id \
             WHERE j.state <> CASE s.status \
               WHEN 'queued' THEN 'queued' \
               WHEN 'claimed' THEN 'claimed' \
               WHEN 'compiling' THEN 'claimed' \
               WHEN 'running' THEN 'claimed' \
               WHEN 'ghost_hunting' THEN 'claimed' \
               WHEN 'completed' THEN 'succeeded' \
               WHEN 'failed' THEN 'failed' \
               WHEN 'expired' THEN 'failed' \
               WHEN 'internal_error' THEN 'dead' \
             END",
        ),
        (
            "P6 one terminal event per finished run",
            "SELECT count(*) FROM submissions s \
             WHERE (SELECT count(*) FROM run_events e \
                    WHERE e.submission_id = s.id AND e.type IN ('done', 'error')) \
                   <> (CASE WHEN s.finished_at IS NULL THEN 0 ELSE 1 END)",
        ),
        (
            "P6 terminal event is the last event",
            "SELECT count(*) FROM submissions s \
             JOIN run_events t ON t.submission_id = s.id AND t.type IN ('done', 'error') \
             WHERE EXISTS (SELECT 1 FROM run_events e \
                           WHERE e.submission_id = s.id AND e.seq > t.seq)",
        ),
        (
            "P8 event sequence is 1..n",
            "SELECT count(*) FROM submissions s \
             WHERE s.last_event_seq <> COALESCE((SELECT count(*) FROM run_events e \
                                                 WHERE e.submission_id = s.id), 0) \
                OR COALESCE((SELECT max(e.seq) FROM run_events e \
                             WHERE e.submission_id = s.id), 0) <> s.last_event_seq",
        ),
    ];

    let mut failures = Vec::new();
    for (name, sql) in CHECKS {
        let violations: i64 = sqlx::query_scalar(sql)
            .fetch_one(pool)
            .await
            .map_err(|error| format!("{name}: query failed: {error}"))?;
        if violations != 0 {
            failures.push(format!("{name}: {violations} violation(s)"));
        }
    }
    if failures.is_empty() {
        Ok(())
    } else {
        Err(failures.join("; "))
    }
}
