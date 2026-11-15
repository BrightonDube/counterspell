//! Static import writes (problem-catalogue C14, CAT-I10, CAT-I11, CAT-I12, CAT-P5).
//!
//! Each test writes one pack through `write_static` and checks the database state. The rows are
//! ordinary catalogue rows, so the triggers from C02 apply.
//!
//! Run with `cargo test -p counterspell-judge --test pg_catalogue_import -- --ignored`.

use counterspell_judge::catalogue::plan::Refusal;
use counterspell_judge::catalogue::write::{write_static, KnownGhost, StaticImport, WriteOutcome};
use serde_json::json;
use sqlx::PgPool;
use std::collections::BTreeMap;

/// Builds a static import for `two-sum`. `body_hash` sets every hash byte, so two calls with
/// different bytes describe two different contents.
fn import(version: i32, body_hash: u8, statement: &str) -> StaticImport {
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
        signature: json!({"function": "two_sum"}),
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

/// Inserts the track and concept that the test packs refer to.
async fn seed_taxonomy(pool: &PgPool) {
    sqlx::query("INSERT INTO tracks (slug, name) VALUES ('arrays', 'Arrays')")
        .execute(pool)
        .await
        .expect("track");
    sqlx::query("INSERT INTO concepts (id, name) VALUES ('hash-maps', 'Hash maps')")
        .execute(pool)
        .await
        .expect("concept");
}

/// Row counts for every table that a static import writes. The tuple order is fixed by the query.
async fn counts(pool: &PgPool) -> (i64, i64, i64, i64, i64, i64, i64, i64) {
    sqlx::query_as(
        "SELECT (SELECT count(*) FROM problems), (SELECT count(*) FROM problem_versions), \
         (SELECT count(*) FROM problem_examples), (SELECT count(*) FROM problem_starters), \
         (SELECT count(*) FROM problem_hints), (SELECT count(*) FROM reference_solutions), \
         (SELECT count(*) FROM known_ghosts), (SELECT count(*) FROM catalogue_events)",
    )
    .fetch_one(pool)
    .await
    .expect("counts")
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_import_idempotent(pool: PgPool) {
    seed_taxonomy(&pool).await;
    let pack = import(1, 0xaa, "Find two numbers.");

    let first = write_static(&pool, &pack, "operator")
        .await
        .expect("first import");
    assert_eq!(first, WriteOutcome::Created { version: 1 });
    let after_first = counts(&pool).await;
    assert_eq!(
        after_first,
        (1, 1, 2, 2, 4, 1, 1, 1),
        "one row per child table, one event"
    );

    let second = write_static(&pool, &pack, "operator")
        .await
        .expect("second import");
    assert_eq!(second, WriteOutcome::Unchanged { version: 1 });
    assert_eq!(
        counts(&pool).await,
        after_first,
        "a repeat import writes nothing"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_versions_history(pool: PgPool) {
    seed_taxonomy(&pool).await;
    write_static(&pool, &import(1, 0xaa, "Old statement."), "operator")
        .await
        .expect("version 1");
    let old_row: (String, Vec<u8>) = sqlx::query_as(
        "SELECT statement_markdown, content_sha256 FROM problem_versions WHERE version = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("old row");

    let outcome = write_static(&pool, &import(2, 0xbb, "New statement."), "operator")
        .await
        .expect("version 2");
    assert_eq!(outcome, WriteOutcome::Created { version: 2 });

    let versions: i64 = sqlx::query_scalar("SELECT count(*) FROM problem_versions")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(versions, 2, "the bump adds one version and keeps the first");
    let kept: (String, Vec<u8>) = sqlx::query_as(
        "SELECT statement_markdown, content_sha256 FROM problem_versions WHERE version = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("kept row");
    assert_eq!(kept, old_row, "version 1 keeps its bytes");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_refusals_write_nothing(pool: PgPool) {
    seed_taxonomy(&pool).await;
    write_static(&pool, &import(1, 0xaa, "Stored."), "operator")
        .await
        .expect("version 1");
    let before = counts(&pool).await;

    let same_number = write_static(&pool, &import(1, 0xbb, "Changed, same number."), "operator")
        .await
        .expect("refusal is a value, not an error");
    assert_eq!(
        same_number,
        WriteOutcome::Refused(Refusal::VersionNotBumped {
            changed: vec!["public/problem.md".to_string()],
        })
    );

    write_static(&pool, &import(2, 0xcc, "Stored again."), "operator")
        .await
        .expect("version 2");
    let before_regression = counts(&pool).await;
    let lower = write_static(&pool, &import(1, 0xdd, "Lower number."), "operator")
        .await
        .expect("refusal is a value, not an error");
    assert_eq!(lower, WriteOutcome::Refused(Refusal::VersionRegressed));
    assert_eq!(
        counts(&pool).await,
        before_regression,
        "a regression writes nothing"
    );
    assert_ne!(before, before_regression, "the version 2 import did write");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_rollback_on_failure(pool: PgPool) {
    seed_taxonomy(&pool).await;
    let mut pack = import(1, 0xaa, "Will fail.");
    // The starter limit is 8 KB (CHECK on problem_starters). Inserted after the version row,
    // so a rollback must remove the version too.
    pack.starters[0].1 = "x".repeat(9000);

    let result = write_static(&pool, &pack, "operator").await;
    assert!(result.is_err(), "an oversized starter must fail the import");
    assert_eq!(
        counts(&pool).await,
        (0, 0, 0, 0, 0, 0, 0, 0),
        "a failed import leaves no problem, version or event behind"
    );
}
