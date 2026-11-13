//! Database tests for the judge demo daily reset (auth T28, requirements R16.7 and R16.8).
//!
//! The reset runs at most once per scheduled window. The schedule is computed from the injected
//! clock, and the database checks `demo_reset_at` under its advisory lock, so a restart or two
//! concurrent checks never reset twice.
//!
//! Run with `DATABASE_URL=<throwaway postgres 17> cargo test -p counterspell-server --test pg_auth_demo_reset -- --ignored pg_`.

use counterspell_server::auth::{clock::ManualClock, demo::reset_if_due};
use sqlx::PgPool;
use time::macros::datetime;
use uuid::Uuid;

async fn seed(pool: &PgPool) -> sqlx::Result<Uuid> {
    let (id,): (Uuid,) = sqlx::query_as("SELECT cs_seed_demo_user($1, '$argon2id$v=19$seed')")
        .bind("judge@example.co")
        .fetch_one(pool)
        .await?;
    Ok(id)
}

async fn verified_user(pool: &PgPool, handle: &str) -> sqlx::Result<Uuid> {
    let (id,): (Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, email_verified_at, handle, display_name)
         VALUES ($1, now(), $2, $2) RETURNING id",
    )
    .bind(format!("{handle}@example.co"))
    .bind(handle)
    .fetch_one(pool)
    .await?;
    Ok(id)
}

/// Adds one submission for `user`. It is finished when `finished` is true, otherwise it is still running.
async fn add_submission(pool: &PgPool, user: Uuid, finished: bool) -> sqlx::Result<()> {
    let (status, verdict) = if finished {
        ("completed", Some("accepted"))
    } else {
        ("running", None)
    };
    let finished_at = if finished { "now()" } else { "NULL" };
    sqlx::query(&format!(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256,
                                 status, verdict, finished_at)
         VALUES ($1, $2, 'two-sum', 'python', 'run', 'print(1)', decode(repeat('00', 32), 'hex'),
                 $3, $4, {finished_at})"
    ))
    .bind(Uuid::new_v4())
    .bind(user)
    .bind(status)
    .bind(verdict)
    .execute(pool)
    .await?;
    Ok(())
}

async fn count_submissions(pool: &PgPool, user: Uuid) -> sqlx::Result<i64> {
    let (count,): (i64,) = sqlx::query_as("SELECT count(*) FROM submissions WHERE user_id = $1")
        .bind(user)
        .fetch_one(pool)
        .await?;
    Ok(count)
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_reset_waits_for_the_next_scheduled_hour(pool: PgPool) {
    let demo = seed(&pool).await.expect("seed");
    let clock = ManualClock::new(datetime!(2026-11-13 12:30 UTC));

    add_submission(&pool, demo, true).await.expect("first run");
    let removed = reset_if_due(&pool, &clock, 12).await.expect("first reset");
    assert_eq!(removed, 1, "the first check after the hour resets the demo");

    add_submission(&pool, demo, true).await.expect("second run");
    clock.set(datetime!(2026-11-13 23:59 UTC));
    let removed = reset_if_due(&pool, &clock, 12).await.expect("same day");
    assert_eq!(removed, 0, "no second reset before the next 12:00 UTC");
    assert_eq!(count_submissions(&pool, demo).await.expect("count"), 1);

    clock.set(datetime!(2026-11-14 11:59 UTC));
    let removed = reset_if_due(&pool, &clock, 12)
        .await
        .expect("before next hour");
    assert_eq!(removed, 0, "11:59 UTC is still before the scheduled reset");

    clock.set(datetime!(2026-11-14 12:00 UTC));
    let removed = reset_if_due(&pool, &clock, 12).await.expect("next hour");
    assert_eq!(removed, 1, "the reset runs once the scheduled hour arrives");
    assert_eq!(count_submissions(&pool, demo).await.expect("count"), 0);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_a_restart_after_a_missed_reset_runs_it_once(pool: PgPool) {
    let demo = seed(&pool).await.expect("seed");
    let clock = ManualClock::new(datetime!(2026-11-10 12:30 UTC));
    reset_if_due(&pool, &clock, 12)
        .await
        .expect("reset on 10 Nov");

    add_submission(&pool, demo, true).await.expect("run");
    clock.set(datetime!(2026-11-15 09:00 UTC));
    let first_start = reset_if_due(&pool, &clock, 12)
        .await
        .expect("start-up check");
    let second_start = reset_if_due(&pool, &clock, 12)
        .await
        .expect("restart check");

    assert_eq!(first_start, 1, "the missed 14 Nov reset runs at start-up");
    assert_eq!(second_start, 0, "a restart does not repeat the reset");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_concurrent_checks_do_not_remove_rows_twice(pool: PgPool) {
    let demo = seed(&pool).await.expect("seed");
    let clock = ManualClock::new(datetime!(2026-11-13 13:00 UTC));
    add_submission(&pool, demo, true).await.expect("run");

    let (first, second) = tokio::join!(
        reset_if_due(&pool, &clock, 12),
        reset_if_due(&pool, &clock, 12)
    );
    let total = first.expect("first") + second.expect("second");
    assert_eq!(total, 1, "the finished run is removed exactly once");
    assert_eq!(count_submissions(&pool, demo).await.expect("count"), 0);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_reset_keeps_running_work_other_users_and_demo_sessions(pool: PgPool) {
    let demo = seed(&pool).await.expect("seed");
    let other = verified_user(&pool, "lerato").await.expect("other user");
    add_submission(&pool, demo, true)
        .await
        .expect("finished demo run");
    add_submission(&pool, demo, false)
        .await
        .expect("running demo run");
    add_submission(&pool, other, true)
        .await
        .expect("finished other run");
    add_submission(&pool, other, false)
        .await
        .expect("running other run");

    sqlx::query(
        "INSERT INTO sessions (id, user_id, auth_method, created_at, last_renewed_at,
                               expires_at, absolute_expires_at)
         VALUES (repeat('a', 64), $1, 'password', now(), now(), now() + interval '30 days',
                 now() + interval '60 days')",
    )
    .bind(demo)
    .execute(&pool)
    .await
    .expect("demo session");

    let other_before: Vec<(String, bool)> = sqlx::query_as(
        "SELECT status, finished_at IS NOT NULL FROM submissions WHERE user_id = $1
         ORDER BY status, id",
    )
    .bind(other)
    .fetch_all(&pool)
    .await
    .expect("other before");

    let clock = ManualClock::new(datetime!(2026-11-13 13:00 UTC));
    let removed = reset_if_due(&pool, &clock, 12).await.expect("reset");
    assert_eq!(removed, 1, "only the demo's finished run is removed");
    assert_eq!(count_submissions(&pool, demo).await.expect("demo rows"), 1);

    let other_after: Vec<(String, bool)> = sqlx::query_as(
        "SELECT status, finished_at IS NOT NULL FROM submissions WHERE user_id = $1
         ORDER BY status, id",
    )
    .bind(other)
    .fetch_all(&pool)
    .await
    .expect("other after");
    assert_eq!(
        other_before, other_after,
        "another user's rows are unchanged"
    );

    let (sessions,): (i64,) = sqlx::query_as("SELECT count(*) FROM sessions WHERE user_id = $1")
        .bind(demo)
        .fetch_one(&pool)
        .await
        .expect("sessions");
    assert_eq!(sessions, 1, "the demo's sessions survive a reset");
}
