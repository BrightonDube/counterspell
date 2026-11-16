//! Database tests for the problem-catalogue core tables (problem-catalogue C01).
//!
//! These need `DATABASE_URL` and run with
//! `cargo test -p counterspell-judge --test pg_catalogue_schema -- --ignored`.
//! They use the database owner, as the schema tests do.

use serde_json::json;
use sqlx::PgPool;

const CHECK_VIOLATION: &str = "23514";

/// Returns the SQLSTATE of a rejected statement, and fails the test if the statement succeeded.
fn rejected_with(result: Result<sqlx::postgres::PgQueryResult, sqlx::Error>) -> String {
    match result {
        Ok(_) => panic!("the database accepted a statement it must reject"),
        Err(error) => error
            .as_database_error()
            .and_then(|db| db.code().map(std::borrow::Cow::into_owned))
            .unwrap_or_else(|| panic!("expected a database error, got: {error}")),
    }
}

async fn insert_track(
    pool: &PgPool,
    slug: &str,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query("INSERT INTO tracks (slug, name) VALUES ($1, 'Arrays')")
        .bind(slug)
        .execute(pool)
        .await
}

async fn insert_problem(
    pool: &PgPool,
    slug: &str,
    difficulty: &str,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query(
        "INSERT INTO problems (slug, title, track_slug, difficulty) VALUES ($1, 'Two Sum', 'arrays', $2)",
    )
    .bind(slug)
    .bind(difficulty)
    .execute(pool)
    .await
}

/// Inserts one version with a hash of `hash_len` bytes and `version` as the version number.
async fn insert_version(
    pool: &PgPool,
    slug: &str,
    version: i32,
    hash_len: usize,
) -> Result<sqlx::postgres::PgQueryResult, sqlx::Error> {
    sqlx::query(
        "INSERT INTO problem_versions (problem_slug, version, content_sha256, manifest, title, track_slug, \
         difficulty, statement_markdown, constraints, signature, input_schema, comparator, limits, \
         engine_version, generator_version, hidden_count) \
         VALUES ($1, $2, $3, $4, 'Two Sum', 'arrays', 'easy', 'Statement', $5, $6, $7, 'exact', $8, \
         'engine-1', 'gen-1', 5)",
    )
    .bind(slug)
    .bind(version)
    .bind(vec![0u8; hash_len])
    .bind(json!({}))
    .bind(json!([]))
    .bind(json!({"name": "two_sum"}))
    .bind(json!({"kind": "int"}))
    .bind(json!({"time_ms": 2000, "mem_mb": 256}))
    .execute(pool)
    .await
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_schema_creates_all_tables(pool: PgPool) {
    let names = [
        "tracks",
        "concepts",
        "problems",
        "problem_concepts",
        "problem_versions",
        "problem_examples",
        "problem_starters",
        "problem_hints",
        "hidden_tests",
        "hidden_generation",
        "hidden_sources",
        "reference_solutions",
        "known_ghosts",
        "catalogue_events",
    ];
    let found: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM information_schema.tables WHERE table_schema = 'public' AND table_name = ANY($1)",
    )
    .bind(names.iter().map(ToString::to_string).collect::<Vec<_>>())
    .fetch_one(&pool)
    .await
    .expect("the catalogue tables should be queryable");
    assert_eq!(
        found, 14,
        "all 14 catalogue tables must exist after migration"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_schema_rejects_bad_track_slug(pool: PgPool) {
    assert_eq!(
        rejected_with(insert_track(&pool, "Bad_Slug").await),
        CHECK_VIOLATION
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_schema_rejects_difficulty_intro(pool: PgPool) {
    insert_track(&pool, "arrays")
        .await
        .expect("a valid track must insert");
    assert_eq!(
        rejected_with(insert_problem(&pool, "two-sum", "intro").await),
        CHECK_VIOLATION
    );
    insert_problem(&pool, "two-sum", "easy")
        .await
        .expect("difficulty easy must be accepted, so the helper is not the cause");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_schema_checks_content_hash_length(pool: PgPool) {
    insert_track(&pool, "arrays").await.expect("track");
    insert_problem(&pool, "two-sum", "easy")
        .await
        .expect("problem");
    assert_eq!(
        rejected_with(insert_version(&pool, "two-sum", 1, 31).await),
        CHECK_VIOLATION,
        "a 31-byte content hash must be rejected"
    );
    insert_version(&pool, "two-sum", 1, 32)
        .await
        .expect("a 32-byte content hash must be accepted, so the helper is not the cause");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_schema_rejects_version_zero(pool: PgPool) {
    insert_track(&pool, "arrays").await.expect("track");
    insert_problem(&pool, "two-sum", "easy")
        .await
        .expect("problem");
    assert_eq!(
        rejected_with(insert_version(&pool, "two-sum", 0, 32).await),
        CHECK_VIOLATION
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_schema_keeps_generator_seed_out_of_versions(pool: PgPool) {
    let seed_in_versions: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM information_schema.columns \
         WHERE table_schema = 'public' AND table_name = 'problem_versions' AND column_name = 'generator_seed'",
    )
    .fetch_one(&pool)
    .await
    .expect("columns should be queryable");
    assert_eq!(
        seed_in_versions, 0,
        "problem_versions must not hold generator_seed (CAT-PT9)"
    );

    let seed_in_generation: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM information_schema.columns \
         WHERE table_schema = 'public' AND table_name = 'hidden_generation' AND column_name = 'generator_seed'",
    )
    .fetch_one(&pool)
    .await
    .expect("columns should be queryable");
    assert_eq!(seed_in_generation, 1, "the seed lives in hidden_generation");
}
