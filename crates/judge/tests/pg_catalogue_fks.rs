//! Foreign-key tests for the catalogue (problem-catalogue C03, CAT-F1 to CAT-F3, CAT-S8, CAT-S9).
//!
//! Run with `cargo test -p counterspell-judge --test pg_catalogue_fks -- --ignored`.

mod catalogue_support;

use catalogue_support as s;
use sqlx::PgPool;
use uuid::Uuid;

const FOREIGN_KEY_VIOLATION: &str = "23503";
const FK_MIGRATION: &str =
    include_str!("../../../migrations/20261116100100_catalogue_foreign_keys.sql");

/// Inserts a `users` row so a submission can reference it.
async fn ensure_user(pool: &PgPool, user: Uuid) {
    sqlx::query(
        "INSERT INTO users (id, handle, display_name) VALUES ($1, $2, 'Test user') ON CONFLICT (id) DO NOTHING",
    )
    .bind(user)
    .bind(user.simple().to_string())
    .execute(pool)
    .await
    .expect("the test user must insert");
}

/// Inserts one queued submission for `slug`, pinned to `version` when given.
async fn submit(pool: &PgPool, slug: &str, version: Option<i32>) -> Result<(), sqlx::Error> {
    let user = Uuid::new_v4();
    ensure_user(pool, user).await;
    sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, problem_version) \
         VALUES ($1, $2, $3, 'python', 'submit', 'print(1)', $4, $5)",
    )
    .bind(Uuid::new_v4())
    .bind(user)
    .bind(slug)
    .bind(vec![0u8; 32])
    .bind(version)
    .execute(pool)
    .await
    .map(|_| ())
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_fks_reject_submission_for_unknown_slug(pool: PgPool) {
    let error = submit(&pool, "no-such-problem", None)
        .await
        .expect_err("a submission for an unknown problem must be refused");
    assert_eq!(
        s::code(&error),
        FOREIGN_KEY_VIOLATION,
        "got: {}",
        s::message(&error)
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_fks_pin_submission_to_existing_version(pool: PgPool) {
    s::track(&pool, "arrays").await.expect("track");
    s::problem(&pool, "two-sum", "arrays")
        .await
        .expect("problem");
    s::version(&pool, "two-sum", 1).await.expect("version 1");

    let error = submit(&pool, "two-sum", Some(2))
        .await
        .expect_err("a submission may only pin a version that exists");
    assert_eq!(
        s::code(&error),
        FOREIGN_KEY_VIOLATION,
        "got: {}",
        s::message(&error)
    );

    submit(&pool, "two-sum", Some(1))
        .await
        .expect("a submission pinned to an existing version must be accepted");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_fks_are_not_valid_and_restrict_on_delete(pool: PgPool) {
    // convalidated = false means the foreign key checks new rows only (NOT VALID).
    // confdeltype = 'r' means ON DELETE RESTRICT.
    let rows: Vec<(String, bool, String)> = sqlx::query_as(
        "SELECT conname, convalidated, confdeltype::text FROM pg_constraint \
         WHERE conname IN ('submissions_problem_fk', 'submissions_problem_version_fk', 'problems_published_version_fk') \
         ORDER BY conname",
    )
    .fetch_all(&pool)
    .await
    .expect("constraints should be queryable");

    assert_eq!(
        rows.len(),
        3,
        "all three catalogue foreign keys must exist: {rows:?}"
    );
    for (name, validated, delete_action) in &rows {
        assert_eq!(delete_action, "r", "{name} must restrict deletes");
        let expect_not_valid = name != "problems_published_version_fk";
        assert_eq!(
            !validated, expect_not_valid,
            "{name}: NOT VALID is expected only on the submission keys"
        );
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_fks_migration_is_idempotent(pool: PgPool) {
    sqlx::raw_sql(FK_MIGRATION)
        .execute(&pool)
        .await
        .expect("running the foreign-key migration a second time must succeed");
    let count: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM pg_constraint WHERE conname IN ('submissions_problem_fk', 'submissions_problem_version_fk', 'problems_published_version_fk')",
    )
    .fetch_one(&pool)
    .await
    .expect("constraints should be queryable");
    assert_eq!(
        count, 3,
        "a second run must not duplicate or drop constraints"
    );
}
