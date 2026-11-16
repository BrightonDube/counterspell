//! Publish-gate tests (problem-catalogue C02, CAT-S4 and P7).
//!
//! Run with `cargo test -p counterspell-judge --test pg_catalogue_publish_gate -- --ignored`.

mod catalogue_support;

use catalogue_support as s;
use sqlx::PgPool;

/// Builds a version with the given example and hint counts, one concept tag, and optionally a sandbox fill.
async fn build(pool: &PgPool, examples: i32, hints: i32, sandboxed: bool) {
    s::track(pool, "arrays").await.expect("track");
    s::problem(pool, "two-sum", "arrays")
        .await
        .expect("problem");
    s::version(pool, "two-sum", 1).await.expect("version");
    s::tag_concept(pool, "two-sum", "hash-maps")
        .await
        .expect("concept");
    s::examples(pool, "two-sum", 1, examples)
        .await
        .expect("examples");
    s::hints(pool, "two-sum", 1, hints).await.expect("hints");
    if sandboxed {
        s::hidden_test(pool, "two-sum", 1, 1)
            .await
            .expect("hidden test");
        s::sandbox_fill(pool, "two-sum", 1).await.expect("fill");
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_publish_gate_valid_version_publishes(pool: PgPool) {
    build(&pool, 2, 4, true).await;
    s::publish(&pool, "two-sum", 1)
        .await
        .expect("a sandbox-validated version with 2 examples and 4 hints must publish");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_publish_gate_unvalidated_version_refused(pool: PgPool) {
    build(&pool, 2, 4, false).await;
    let error = s::publish(&pool, "two-sum", 1)
        .await
        .expect_err("an unvalidated version must not publish");
    assert!(
        s::message(&error).contains("sandbox-validated"),
        "got: {}",
        s::message(&error)
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_publish_gate_three_hints_refused(pool: PgPool) {
    build(&pool, 2, 3, true).await;
    let error = s::publish(&pool, "two-sum", 1)
        .await
        .expect_err("a version with three hints must not publish");
    assert!(
        s::message(&error).contains("four hints"),
        "got: {}",
        s::message(&error)
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_publish_gate_six_examples_refused(pool: PgPool) {
    build(&pool, 6, 4, true).await;
    let error = s::publish(&pool, "two-sum", 1)
        .await
        .expect_err("a version with six examples must not publish");
    assert!(
        s::message(&error).contains("2 to 5 examples"),
        "got: {}",
        s::message(&error)
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_publish_gate_missing_taxonomy_refused(pool: PgPool) {
    build(&pool, 2, 4, true).await;
    // A problem with no concept tag: create a second problem without tags.
    s::problem(&pool, "no-tags", "arrays")
        .await
        .expect("second problem");
    s::version(&pool, "no-tags", 1)
        .await
        .expect("second version");
    s::examples(&pool, "no-tags", 1, 2).await.expect("examples");
    s::hints(&pool, "no-tags", 1, 4).await.expect("hints");
    s::hidden_test(&pool, "no-tags", 1, 1)
        .await
        .expect("hidden");
    s::sandbox_fill(&pool, "no-tags", 1).await.expect("fill");
    let error = s::publish(&pool, "no-tags", 1)
        .await
        .expect_err("a problem with no concept tag must not publish");
    assert!(
        s::message(&error).contains("concept"),
        "got: {}",
        s::message(&error)
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_publish_gate_unknown_track_refused_by_key(pool: PgPool) {
    // A track that does not exist cannot be used at all. The foreign key stops the problem row,
    // so no version can reach the publish gate with an unknown track.
    let error = s::problem(&pool, "two-sum", "no-such-track")
        .await
        .expect_err("an unknown track must be refused");
    assert_eq!(s::code(&error), "23503", "got: {}", s::message(&error));
}
