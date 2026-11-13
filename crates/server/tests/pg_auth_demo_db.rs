//! Database tests for the judge demo account guards and reset (auth T09, requirements R16.6 to R16.8).
//!
//! The demo account can only be created or re-credentialed by the seed helper, can never be
//! deleted or given another identity, and its reset clears only its own finished rows.
//!
//! Run with `DATABASE_URL=<throwaway postgres 17> cargo test -p counterspell-server --test pg_auth_demo_db -- --ignored pg_`.

use sqlx::PgPool;
use uuid::Uuid;

async fn seed(pool: &PgPool, email: &str) -> sqlx::Result<Uuid> {
    let (id,): (Uuid,) = sqlx::query_as("SELECT cs_seed_demo_user($1, '$argon2id$v=19$seed')")
        .bind(email)
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

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_the_demo_account_cannot_be_created_directly(pool: PgPool) {
    let direct = sqlx::query(
        "INSERT INTO users (email, email_verified_at, handle, display_name, is_demo)
         VALUES ('judge@example.co', now(), 'demo', 'Judge', true)",
    )
    .execute(&pool)
    .await;
    assert!(direct.is_err());
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_the_demo_account_cannot_be_deleted_changed_or_given_identities(pool: PgPool) {
    let demo = seed(&pool, "judge@example.co").await.expect("seed");
    let delete = sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(demo)
        .execute(&pool)
        .await;
    assert!(delete.is_err(), "delete");

    let email_change = sqlx::query("UPDATE users SET email = 'other@example.co' WHERE id = $1")
        .bind(demo)
        .execute(&pool)
        .await;
    assert!(email_change.is_err(), "email change");

    let password_change = sqlx::query("UPDATE users SET password_hash = 'x' WHERE id = $1")
        .bind(demo)
        .execute(&pool)
        .await;
    assert!(password_change.is_err(), "password change");

    let identity = sqlx::query(
        "INSERT INTO identities (user_id, provider, provider_user_id) VALUES ($1, 'github', '77')",
    )
    .bind(demo)
    .execute(&pool)
    .await;
    assert!(identity.is_err(), "identity for the demo user");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_seeding_creates_exactly_one_demo_account_and_is_idempotent(pool: PgPool) {
    let first = seed(&pool, "judge@example.co").await.expect("first seed");
    let second = seed(&pool, "judge@example.co").await.expect("second seed");
    assert_eq!(first, second, "the same row is updated");
    let (count, handle): (i64, String) =
        sqlx::query_as("SELECT count(*), max(handle::text) FROM users WHERE is_demo")
            .fetch_one(&pool)
            .await
            .expect("count");
    assert_eq!(count, 1);
    assert_eq!(handle, "demo");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_only_the_server_role_may_run_the_demo_functions(pool: PgPool) {
    for signature in ["cs_reset_demo_account()", "cs_seed_demo_user(text, text)"] {
        let (server,): (bool,) =
            sqlx::query_as("SELECT has_function_privilege('counterspell_server', $1, 'EXECUTE')")
                .bind(signature)
                .fetch_one(&pool)
                .await
                .expect("server");
        let (judge,): (bool,) =
            sqlx::query_as("SELECT has_function_privilege('counterspell_judge', $1, 'EXECUTE')")
                .bind(signature)
                .fetch_one(&pool)
                .await
                .expect("judge");
        assert!(server, "{signature} must be executable by the server role");
        assert!(
            !judge,
            "{signature} must not be executable by the judge role"
        );
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_reset_removes_only_the_demo_finished_rows_and_keeps_other_users(pool: PgPool) {
    let demo = seed(&pool, "judge@example.co").await.expect("seed");
    let other = verified_user(&pool, "lerato").await.expect("other user");

    // A table with a plain foreign key to users, to prove the reset's generic loop.
    sqlx::query(
        "CREATE TABLE demo_scores (
             id bigserial PRIMARY KEY,
             user_id uuid NOT NULL REFERENCES users(id) ON DELETE CASCADE,
             points int NOT NULL)",
    )
    .execute(&pool)
    .await
    .expect("scores table");

    for (user, status, finished) in [
        (demo, "completed", true),
        (demo, "running", false),
        (other, "completed", true),
    ] {
        let verdict = (status == "completed").then_some("accepted");
        let finished_at = finished.then_some("now()");
        sqlx::query(&format!(
            "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256,
                                     status, verdict, finished_at)
             VALUES ($1, $2, 'two-sum', 'python', 'run', 'print(1)', decode(repeat('00', 32), 'hex'),
                     $3, $4, {})",
            finished_at.unwrap_or("NULL")
        ))
        .bind(Uuid::new_v4())
        .bind(user)
        .bind(status)
        .bind(verdict)
        .execute(&pool)
        .await
        .expect("submission");
    }
    sqlx::query("INSERT INTO demo_scores (user_id, points) VALUES ($1, 10), ($2, 20)")
        .bind(demo)
        .bind(other)
        .execute(&pool)
        .await
        .expect("scores");
    sqlx::query("UPDATE users SET xp = 500, streak_days = 4 WHERE id = $1")
        .bind(demo)
        .execute(&pool)
        .await
        .expect("counters");

    let other_before: Vec<(String, i32)> = sqlx::query_as(
        "SELECT status, points FROM submissions s, demo_scores d WHERE s.user_id = $1 AND d.user_id = $1",
    )
    .bind(other)
    .fetch_all(&pool)
    .await
    .expect("before");

    let (removed,): (i64,) = sqlx::query_as("SELECT cs_reset_demo_account()")
        .fetch_one(&pool)
        .await
        .expect("reset");
    assert_eq!(removed, 1, "only the demo's finished submission is removed");

    let (demo_running,): (i64,) = sqlx::query_as(
        "SELECT count(*) FROM submissions WHERE user_id = $1 AND status = 'running'",
    )
    .bind(demo)
    .fetch_one(&pool)
    .await
    .expect("running");
    assert_eq!(demo_running, 1, "a running submission survives the reset");

    let (demo_scores,): (i64,) =
        sqlx::query_as("SELECT count(*) FROM demo_scores WHERE user_id = $1")
            .bind(demo)
            .fetch_one(&pool)
            .await
            .expect("scores");
    assert_eq!(
        demo_scores, 0,
        "the generic loop removes the demo's other rows"
    );

    let (xp, streak, reset_at): (i32, i32, Option<time::OffsetDateTime>) =
        sqlx::query_as("SELECT xp, streak_days, demo_reset_at FROM users WHERE id = $1")
            .bind(demo)
            .fetch_one(&pool)
            .await
            .expect("counters after");
    assert_eq!((xp, streak), (0, 0));
    assert!(reset_at.is_some());

    let other_after: Vec<(String, i32)> = sqlx::query_as(
        "SELECT status, points FROM submissions s, demo_scores d WHERE s.user_id = $1 AND d.user_id = $1",
    )
    .bind(other)
    .fetch_all(&pool)
    .await
    .expect("after");
    assert_eq!(
        other_before, other_after,
        "another user's rows are unchanged"
    );
}
