//! Schema tests for the `hints` table (hints spec H10). Each `pg_` test runs the real migrations on a
//! fresh database. They need `DATABASE_URL` and are run with `--ignored`.

use counterspell_common::ids::UserId;
use counterspell_server::testkit::ensure_user;
use sqlx::PgPool;
use uuid::Uuid;

/// The migration under test, read at build time so the test always checks the file in the repo.
const MIGRATION: &str = include_str!("../../../migrations/20261116120000_hints_core.sql");

/// One hint row to insert. Only the columns the CHECK constraints look at are varied.
struct Row<'a> {
    rung: Option<i16>,
    source: &'a str,
    provider: Option<&'a str>,
    cache_key: Option<Vec<u8>>,
}

/// Creates a learner and one queued submission for them, and returns both ids.
async fn seed(pool: &PgPool) -> (Uuid, Uuid) {
    let user = Uuid::new_v4();
    ensure_user(pool, UserId::from_uuid(user))
        .await
        .expect("a user row can be created");
    let submission = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, status)
         VALUES ($1, $2, 'two-sum', 'python', 'run', 'print(1)', $3, 'queued')",
    )
    .bind(submission)
    .bind(user)
    .bind(vec![0u8; 32])
    .execute(pool)
    .await
    .expect("a submission row can be created");
    (user, submission)
}

async fn insert(pool: &PgPool, owner: (Uuid, Uuid), row: Row<'_>) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO hints (user_id, submission_id, rung, source, provider, body, cache_key)
         VALUES ($1, $2, $3, $4, $5, 'a hint', $6)",
    )
    .bind(owner.0)
    .bind(owner.1)
    .bind(row.rung)
    .bind(row.source)
    .bind(row.provider)
    .bind(row.cache_key)
    .execute(pool)
    .await
    .map(|_| ())
}

fn key_bytes() -> Vec<u8> {
    vec![7u8; 32]
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "needs DATABASE_URL"]
async fn pg_hints_accept_a_valid_ladder_ai_and_familiar_row(pool: PgPool) {
    let owner = seed(&pool).await;

    let ladder = insert(
        &pool,
        owner,
        Row {
            rung: Some(1),
            source: "ladder",
            provider: None,
            cache_key: None,
        },
    )
    .await;
    let ai = insert(
        &pool,
        owner,
        Row {
            rung: Some(2),
            source: "ai",
            provider: Some("openai"),
            cache_key: Some(key_bytes()),
        },
    )
    .await;
    let familiar = insert(
        &pool,
        owner,
        Row {
            rung: None,
            source: "familiar",
            provider: Some("groq"),
            cache_key: Some(key_bytes()),
        },
    )
    .await;

    assert!(ladder.is_ok(), "a ladder row is valid: {ladder:?}");
    assert!(ai.is_ok(), "an AI row is valid: {ai:?}");
    assert!(familiar.is_ok(), "a familiar row is valid: {familiar:?}");
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "needs DATABASE_URL"]
async fn pg_hints_reject_a_ladder_row_with_a_provider(pool: PgPool) {
    let owner = seed(&pool).await;

    let result = insert(
        &pool,
        owner,
        Row {
            rung: Some(1),
            source: "ladder",
            provider: Some("openai"),
            cache_key: None,
        },
    )
    .await;

    assert!(result.is_err(), "a ladder row must not name a provider");
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "needs DATABASE_URL"]
async fn pg_hints_reject_an_ai_row_without_a_cache_key(pool: PgPool) {
    let owner = seed(&pool).await;

    let result = insert(
        &pool,
        owner,
        Row {
            rung: Some(2),
            source: "ai",
            provider: Some("openai"),
            cache_key: None,
        },
    )
    .await;

    assert!(result.is_err(), "an AI row needs its cache key");
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "needs DATABASE_URL"]
async fn pg_hints_reject_a_familiar_row_with_a_rung(pool: PgPool) {
    let owner = seed(&pool).await;

    let result = insert(
        &pool,
        owner,
        Row {
            rung: Some(1),
            source: "familiar",
            provider: Some("openai"),
            cache_key: Some(key_bytes()),
        },
    )
    .await;

    assert!(
        result.is_err(),
        "a familiar bubble must not use the ladder rung column"
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "needs DATABASE_URL"]
async fn pg_hints_server_role_cannot_update_or_delete(pool: PgPool) {
    let owner = seed(&pool).await;
    insert(
        &pool,
        owner,
        Row {
            rung: Some(1),
            source: "ladder",
            provider: None,
            cache_key: None,
        },
    )
    .await
    .expect("the owner can insert a ladder row");
    let mut connection = pool.acquire().await.expect("a connection");

    sqlx::query("SET ROLE counterspell_server")
        .execute(&mut *connection)
        .await
        .expect("the test can switch to the server role");
    let update = sqlx::query("UPDATE hints SET body = 'changed'")
        .execute(&mut *connection)
        .await;
    let delete = sqlx::query("DELETE FROM hints")
        .execute(&mut *connection)
        .await;
    sqlx::query("RESET ROLE")
        .execute(&mut *connection)
        .await
        .expect("the test can switch back");

    assert!(
        permission_denied(update),
        "the server role may not update hints"
    );
    assert!(
        permission_denied(delete),
        "the server role may not delete hints"
    );
}

/// `true` when the query failed with PostgreSQL's `insufficient_privilege` code (42501).
fn permission_denied(result: Result<sqlx::postgres::PgQueryResult, sqlx::Error>) -> bool {
    match result {
        Err(sqlx::Error::Database(error)) => error.code().as_deref() == Some("42501"),
        _ => false,
    }
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "needs DATABASE_URL"]
async fn pg_hints_migration_runs_twice_without_error(pool: PgPool) {
    sqlx::raw_sql(MIGRATION)
        .execute(&pool)
        .await
        .expect("the migration is safe to run a second time");
}
