//! Plain-SQL fixtures for the catalogue database tests.
//!
//! The helpers insert rows with ordinary statements, so the database triggers under test are
//! the only rules that apply. Hidden rows carry canary strings, never real data.

#![allow(dead_code)]

use serde_json::json;
use sqlx::postgres::PgQueryResult;
use sqlx::PgPool;

/// The result of one fixture statement.
pub type Step = Result<PgQueryResult, sqlx::Error>;

/// Returns the database message of a failed statement, or the plain error text.
pub fn message(error: &sqlx::Error) -> String {
    error
        .as_database_error()
        .map_or_else(|| error.to_string(), |db| db.message().to_string())
}

/// Returns the SQLSTATE of a failed statement.
pub fn code(error: &sqlx::Error) -> String {
    error
        .as_database_error()
        .and_then(|db| db.code().map(std::borrow::Cow::into_owned))
        .unwrap_or_default()
}

pub async fn track(pool: &PgPool, slug: &str) -> Step {
    sqlx::query("INSERT INTO tracks (slug, name) VALUES ($1, 'Arrays') ON CONFLICT DO NOTHING")
        .bind(slug)
        .execute(pool)
        .await
}

pub async fn problem(pool: &PgPool, slug: &str, track_slug: &str) -> Step {
    sqlx::query(
        "INSERT INTO problems (slug, title, track_slug, difficulty) VALUES ($1, 'Two Sum', $2, 'easy')",
    )
    .bind(slug)
    .bind(track_slug)
    .execute(pool)
    .await
}

/// Inserts version `version` of `slug` with valid static fields and no sandbox stamp.
pub async fn version(pool: &PgPool, slug: &str, version: i32) -> Step {
    sqlx::query(
        "INSERT INTO problem_versions (problem_slug, version, content_sha256, manifest, title, track_slug, \
         difficulty, statement_markdown, constraints, signature, input_schema, comparator, limits, \
         engine_version, generator_version, hidden_count) \
         VALUES ($1, $2, $3, $4, 'Two Sum', 'arrays', 'easy', 'Statement', $5, $6, $7, 'exact', $8, \
         'engine-1', 'gen-1', 5)",
    )
    .bind(slug)
    .bind(version)
    .bind(vec![u8::try_from(version).unwrap_or(7); 32])
    .bind(json!({}))
    .bind(json!([]))
    .bind(json!({"name": "two_sum"}))
    .bind(json!({"kind": "int"}))
    .bind(json!({"time_ms": 2000, "mem_mb": 256}))
    .execute(pool)
    .await
}

/// Inserts `count` examples with ordinals 1..=count.
pub async fn examples(
    pool: &PgPool,
    slug: &str,
    version: i32,
    count: i32,
) -> Result<(), sqlx::Error> {
    for ordinal in 1..=count {
        sqlx::query(
            "INSERT INTO problem_examples (problem_slug, version, ordinal, input, expected) \
             VALUES ($1, $2, $3, $4, $5)",
        )
        .bind(slug)
        .bind(version)
        .bind(ordinal)
        .bind(json!([ordinal]))
        .bind(json!(ordinal))
        .execute(pool)
        .await?;
    }
    Ok(())
}

/// Inserts hint rungs 1..=count.
pub async fn hints(pool: &PgPool, slug: &str, version: i32, count: i32) -> Result<(), sqlx::Error> {
    for rung in 1..=count {
        sqlx::query(
            "INSERT INTO problem_hints (problem_slug, version, rung, body) VALUES ($1, $2, $3, 'A nudge.')",
        )
        .bind(slug)
        .bind(version)
        .bind(rung)
        .execute(pool)
        .await?;
    }
    Ok(())
}

/// Adds a concept and tags the problem with it.
pub async fn tag_concept(pool: &PgPool, slug: &str, concept: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO concepts (id, name) VALUES ($1, 'Hash maps') ON CONFLICT DO NOTHING")
        .bind(concept)
        .execute(pool)
        .await?;
    sqlx::query("INSERT INTO problem_concepts (problem_slug, concept_id) VALUES ($1, $2)")
        .bind(slug)
        .bind(concept)
        .execute(pool)
        .await?;
    Ok(())
}

/// Inserts one starter row per given language id.
pub async fn starters(
    pool: &PgPool,
    slug: &str,
    version: i32,
    langs: &[&str],
) -> Result<(), sqlx::Error> {
    for lang in langs {
        sqlx::query(
            "INSERT INTO problem_starters (problem_slug, version, lang, code) \
             VALUES ($1, $2, $3, 'def solve(): pass')",
        )
        .bind(slug)
        .bind(version)
        .bind(lang)
        .execute(pool)
        .await?;
    }
    Ok(())
}

/// Inserts one hidden test. The input carries a canary string, never real data.
pub async fn hidden_test(pool: &PgPool, slug: &str, version: i32, ordinal: i32) -> Step {
    sqlx::query(
        "INSERT INTO hidden_tests (problem_slug, version, ordinal, source, input, expected) \
         VALUES ($1, $2, $3, 'curated', $4, $5)",
    )
    .bind(slug)
    .bind(version)
    .bind(ordinal)
    .bind(json!(["CANARY-HIDDEN-7f3a"]))
    .bind(json!("CANARY-EXPECTED-9b1c"))
    .execute(pool)
    .await
}

/// Stamps the one-time sandbox fill on a version.
pub async fn sandbox_fill(pool: &PgPool, slug: &str, version: i32) -> Step {
    sqlx::query(
        "UPDATE problem_versions SET sandbox_validated_at = now(), sandbox_image_digest = 'sha256:test' \
         WHERE problem_slug = $1 AND version = $2",
    )
    .bind(slug)
    .bind(version)
    .execute(pool)
    .await
}

/// Publishes `version` of `slug`.
pub async fn publish(pool: &PgPool, slug: &str, version: i32) -> Step {
    sqlx::query(
        "UPDATE problems SET publication_state = 'published', published_version = $2, published_at = now() \
         WHERE slug = $1",
    )
    .bind(slug)
    .bind(version)
    .execute(pool)
    .await
}
