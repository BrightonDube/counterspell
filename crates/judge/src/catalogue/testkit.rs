//! Test kit: inserts catalogue rows through the same triggers as production (problem-catalogue C05).
//!
//! Only tests use this module (feature `testkit`). Hidden rows carry canary strings, never real
//! hidden data, so a test can check that a value reached the judge role and did not leak.

use crate::catalogue::write::{KnownGhost, StaticImport};
use serde_json::json;
use sqlx::PgPool;
use std::collections::BTreeMap;

/// Canary written into hidden test inputs. Tests look for it only in the judge's view.
pub const CANARY_HIDDEN: &str = "CANARY-HIDDEN-7f3a";
/// Canary written into hidden expected outputs.
pub const CANARY_EXPECTED: &str = "CANARY-EXPECTED-9b1c";
/// Canary written into the reference solution.
pub const CANARY_REFERENCE: &str = "CANARY-REFERENCE-4d2e";
/// Canary written into a known ghost.
pub const CANARY_GHOST: &str = "CANARY-GHOST-1a8f";

const TRACK: &str = "arrays";
const CONCEPT: &str = "hash-maps";
const HIDDEN_COUNT: i32 = 5;

/// Inserts a track, a concept, a problem and one version with examples, hints, starters and a concept tag.
/// The version has no hidden data and no sandbox stamp, so it cannot be published yet.
///
/// # Errors
///
/// Returns the first database error. A trigger that refuses a row stops the fixture here.
pub async fn insert_static_fixture(
    pool: &PgPool,
    slug: &str,
    version: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO tracks (slug, name) VALUES ($1, 'Arrays') ON CONFLICT DO NOTHING")
        .bind(TRACK)
        .execute(pool)
        .await?;
    sqlx::query("INSERT INTO concepts (id, name) VALUES ($1, 'Hash maps') ON CONFLICT DO NOTHING")
        .bind(CONCEPT)
        .execute(pool)
        .await?;
    sqlx::query(
        "INSERT INTO problems (slug, title, track_slug, difficulty) VALUES ($1, 'Two Sum', $2, 'easy') \
         ON CONFLICT DO NOTHING",
    )
    .bind(slug)
    .bind(TRACK)
    .execute(pool)
    .await?;
    insert_version_row(pool, slug, version).await?;
    for ordinal in 1..=2 {
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
    for rung in 1..=4 {
        sqlx::query(
            "INSERT INTO problem_hints (problem_slug, version, rung, body) VALUES ($1, $2, $3, 'A nudge.')",
        )
        .bind(slug)
        .bind(version)
        .bind(rung)
        .execute(pool)
        .await?;
    }
    for lang in ["python", "javascript"] {
        sqlx::query(
            "INSERT INTO problem_starters (problem_slug, version, lang, code) VALUES ($1, $2, $3, 'def solve(): pass')",
        )
        .bind(slug)
        .bind(version)
        .bind(lang)
        .execute(pool)
        .await?;
    }
    sqlx::query("INSERT INTO problem_concepts (problem_slug, concept_id) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(slug)
        .bind(CONCEPT)
        .execute(pool)
        .await?;
    Ok(())
}

/// Builds the static fixture, adds canary hidden data, stamps the sandbox and publishes the version.
/// Every insert goes through the same triggers as production, so a rule failure surfaces here.
///
/// # Errors
///
/// Returns the first database error from any step.
pub async fn insert_published_fixture(
    pool: &PgPool,
    slug: &str,
    version: i32,
) -> Result<(), sqlx::Error> {
    insert_static_fixture(pool, slug, version).await?;
    for ordinal in 1..=HIDDEN_COUNT {
        sqlx::query(
            "INSERT INTO hidden_tests (problem_slug, version, ordinal, source, input, expected) \
             VALUES ($1, $2, $3, 'curated', $4, $5)",
        )
        .bind(slug)
        .bind(version)
        .bind(ordinal)
        .bind(json!([CANARY_HIDDEN, ordinal]))
        .bind(json!(CANARY_EXPECTED))
        .execute(pool)
        .await?;
    }
    sqlx::query(
        "INSERT INTO hidden_generation (problem_slug, version, generator_seed, generator_version) \
         VALUES ($1, $2, 42, 'gen-1')",
    )
    .bind(slug)
    .bind(version)
    .execute(pool)
    .await?;
    sqlx::query(
        "INSERT INTO hidden_sources (problem_slug, version, path, sha256) VALUES ($1, $2, 'private/hidden/cases.toml', $3)",
    )
    .bind(slug)
    .bind(version)
    .bind(vec![3u8; 32])
    .execute(pool)
    .await?;
    sqlx::query(
        "INSERT INTO reference_solutions (problem_slug, version, lang, code, sha256) VALUES ($1, $2, 'python', $3, $4)",
    )
    .bind(slug)
    .bind(version)
    .bind(format!("def solve(values):\n    return None  # {CANARY_REFERENCE}\n"))
    .bind(vec![4u8; 32])
    .execute(pool)
    .await?;
    sqlx::query(
        "INSERT INTO known_ghosts (problem_slug, version, ghost_id, kind, input, got, exception_class, explanation) \
         VALUES ($1, $2, 'ghost-1', 'wrong', $3, $4, NULL, 'Returns the wrong count.')",
    )
    .bind(slug)
    .bind(version)
    .bind(json!([CANARY_GHOST]))
    .bind(json!(CANARY_GHOST))
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE problem_versions SET sandbox_validated_at = now(), sandbox_image_digest = 'sha256:testkit' \
         WHERE problem_slug = $1 AND version = $2",
    )
    .bind(slug)
    .bind(version)
    .execute(pool)
    .await?;
    sqlx::query(
        "UPDATE problems SET publication_state = 'published', published_version = $2, published_at = now() \
         WHERE slug = $1",
    )
    .bind(slug)
    .bind(version)
    .execute(pool)
    .await?;
    Ok(())
}

async fn insert_version_row(pool: &PgPool, slug: &str, version: i32) -> Result<(), sqlx::Error> {
    sqlx::query(
        "INSERT INTO problem_versions (problem_slug, version, content_sha256, manifest, title, track_slug, \
         difficulty, statement_markdown, constraints, signature, input_schema, comparator, limits, \
         engine_version, generator_version, hidden_count) \
         VALUES ($1, $2, $3, $4, 'Two Sum', $5, 'easy', 'Statement', $6, $7, $8, 'exact', $9, \
         'engine-1', 'gen-1', $10)",
    )
    .bind(slug)
    .bind(version)
    .bind(version_hash(version).to_vec())
    .bind(json!({}))
    .bind(TRACK)
    .bind(json!([]))
    .bind(json!({
        "protocol_version": 1,
        "function": "two_sum",
        "parameters": [
            {"name": "values", "type": {"kind": "list", "items": {"kind": "int"}}},
            {"name": "target", "type": {"kind": "int"}}
        ],
        "returns": {"kind": "list", "items": {"kind": "int"}}
    }))
    .bind(json!({"type": "int", "min": 0, "max": 100}))
    .bind(json!({"time_ms": 2000, "mem_mb": 256}))
    .bind(HIDDEN_COUNT)
    .execute(pool)
    .await
    .map(|_| ())
}

/// Builds a static import for `two-sum` with one public file.
///
/// `body_hash` sets every hash byte, so two calls with different bytes describe two different
/// contents. The pack carries no hidden data, so it is a static import only.
#[must_use]
pub fn static_import(version: i32, body_hash: u8, statement: &str) -> StaticImport {
    let mut manifest_public = BTreeMap::new();
    manifest_public.insert(
        "public/problem.md".to_string(),
        format!("{body_hash:02x}").repeat(32),
    );
    StaticImport {
        slug: "two-sum".to_string(),
        version,
        content_sha256: [body_hash; 32],
        manifest_public,
        title: "Two Sum".to_string(),
        track: "arrays".to_string(),
        difficulty: "easy".to_string(),
        statement_markdown: statement.to_string(),
        constraints: vec!["The list has at most 10 items.".to_string()],
        signature: json!({
            "protocol_version": 1,
            "function": "two_sum",
            "parameters": [
                {"name": "values", "type": {"kind": "list", "items": {"kind": "int"}}},
                {"name": "target", "type": {"kind": "int"}}
            ],
            "returns": {"kind": "list", "items": {"kind": "int"}}
        }),
        input_schema: json!({"kind": "int"}),
        comparator: "exact".to_string(),
        limits: json!({"time_ms": 2000, "mem_mb": 256}),
        reference_complexity: None,
        bench: None,
        complexity_note: None,
        engine_version: "engine-1".to_string(),
        generator_version: "gen-1".to_string(),
        hidden_count: 5,
        concepts: vec!["hash-maps".to_string()],
        examples: vec![(json!([1]), json!(1)), (json!([2]), json!(2))],
        starters: vec![
            (
                "python".to_string(),
                "def two_sum(values, target):\n    raise NotImplementedError\n".to_string(),
            ),
            (
                "javascript".to_string(),
                "function twoSum(values, target) {\n  throw new Error('not implemented');\n}\n"
                    .to_string(),
            ),
        ],
        hints: (1..=4)
            .map(|rung| (rung, format!("Hint {rung}.")))
            .collect(),
        reference_code: "def two_sum(values, target):\n    return 0\n".to_string(),
        known_ghosts: vec![KnownGhost {
            ghost_id: "always-zero".to_string(),
            kind: "wrong".to_string(),
            input: json!([1]),
            got: Some(json!(0)),
            exception_class: None,
            explanation: "Returned 0 for a list with one item.".to_string(),
            concepts: vec!["hash-maps".to_string()],
        }],
    }
}

/// A 32-byte content hash that differs between versions, so one problem can hold several
/// fixture versions. The unique key on `(problem_slug, content_sha256)` needs distinct values.
fn version_hash(version: i32) -> [u8; 32] {
    let mut hash = [5u8; 32];
    hash[31] = u8::try_from(version).unwrap_or(u8::MAX);
    hash
}
