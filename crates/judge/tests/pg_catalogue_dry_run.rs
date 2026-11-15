//! Dry-run tests (problem-catalogue C15, CAT-I13, CAT-I19, CAT-P5).
//!
//! A dry run reads the stored version and prints the plan. It must write nothing: row counts and
//! the event log stay the same.
//!
//! Run with `cargo test -p counterspell-judge --test pg_catalogue_dry_run -- --ignored`.

use counterspell_judge::catalogue::plan::{Plan, Refusal};
use counterspell_judge::catalogue::testkit::static_import as import;
use counterspell_judge::catalogue::write::{dry_run_static, write_static};
use sqlx::PgPool;

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

/// Counts of the problem, version and event tables. The dry run must not change any of them.
async fn counts(pool: &PgPool) -> (i64, i64, i64) {
    sqlx::query_as(
        "SELECT (SELECT count(*) FROM problems), (SELECT count(*) FROM problem_versions), \
         (SELECT count(*) FROM catalogue_events)",
    )
    .fetch_one(pool)
    .await
    .expect("counts")
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_dry_run_no_writes(pool: PgPool) {
    seed_taxonomy(&pool).await;

    let empty = dry_run_static(&pool, &import(1, 0xaa, "New."))
        .await
        .expect("dry run on an empty catalogue");
    assert_eq!(empty.plan, Plan::Create);
    assert_eq!(
        counts(&pool).await,
        (0, 0, 0),
        "a dry run on an empty catalogue writes nothing"
    );

    write_static(&pool, &import(1, 0xaa, "Stored."), "operator")
        .await
        .expect("version 1");
    let stored = counts(&pool).await;

    let same = dry_run_static(&pool, &import(1, 0xaa, "Stored."))
        .await
        .expect("dry run on the same pack");
    assert_eq!(same.plan, Plan::Unchanged);
    assert!(
        same.diff.added.is_empty() && same.diff.removed.is_empty() && same.diff.changed.is_empty()
    );

    let bumped = dry_run_static(&pool, &import(2, 0xbb, "Bumped."))
        .await
        .expect("dry run on a bumped pack");
    assert_eq!(bumped.plan, Plan::Create);
    assert_eq!(bumped.diff.changed, vec!["public/problem.md".to_string()]);

    let same_number = dry_run_static(&pool, &import(1, 0xcc, "Changed."))
        .await
        .expect("dry run on a changed pack with the same number");
    assert_eq!(
        same_number.plan,
        Plan::Refuse(Refusal::VersionNotBumped {
            changed: vec!["public/problem.md".to_string()],
        })
    );

    assert_eq!(
        counts(&pool).await,
        stored,
        "dry runs leave every row and event as they were"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_dry_run_reports_generator_change(pool: PgPool) {
    seed_taxonomy(&pool).await;
    write_static(&pool, &import(1, 0xaa, "Stored."), "operator")
        .await
        .expect("version 1");

    let mut newer_generator = import(1, 0xaa, "Stored.");
    newer_generator.generator_version = "gen-2".to_string();
    let report = dry_run_static(&pool, &newer_generator)
        .await
        .expect("dry run with a new generator version");
    assert_eq!(
        report.generator_change,
        Some(("gen-1".to_string(), "gen-2".to_string())),
        "the stored and current generator versions are both shown (CAT-I19)"
    );
    assert_eq!(
        report.plan,
        Plan::Unchanged,
        "a generator change never regenerates data (D12)"
    );
}
