//! Immutability tests for catalogue versions and child rows (problem-catalogue C02).
//!
//! Run with `cargo test -p counterspell-judge --test pg_catalogue_immutability -- --ignored`.

mod catalogue_support;

use catalogue_support as s;
use sqlx::PgPool;

/// Builds track, problem and version `1` of `two-sum`.
async fn base(pool: &PgPool) {
    s::track(pool, "arrays").await.expect("track");
    s::problem(pool, "two-sum", "arrays")
        .await
        .expect("problem");
    s::version(pool, "two-sum", 1).await.expect("version");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_immutability_refuses_title_update(pool: PgPool) {
    base(&pool).await;
    let error =
        sqlx::query("UPDATE problem_versions SET title = 'Renamed' WHERE problem_slug = 'two-sum'")
            .execute(&pool)
            .await
            .expect_err("a version title must not change after insert");
    assert!(
        s::message(&error).contains("immutable"),
        "got: {}",
        s::message(&error)
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_immutability_refuses_version_delete(pool: PgPool) {
    base(&pool).await;
    let error = sqlx::query("DELETE FROM problem_versions WHERE problem_slug = 'two-sum'")
        .execute(&pool)
        .await
        .expect_err("a version must not be deleted");
    assert!(
        s::message(&error).contains("immutable"),
        "got: {}",
        s::message(&error)
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_immutability_sandbox_fill_without_hidden_tests_refused(pool: PgPool) {
    base(&pool).await;
    let error = s::sandbox_fill(&pool, "two-sum", 1)
        .await
        .expect_err("the sandbox fill needs at least one hidden test");
    assert!(
        s::message(&error).contains("hidden tests"),
        "got: {}",
        s::message(&error)
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_immutability_sandbox_fill_with_hidden_tests_passes(pool: PgPool) {
    base(&pool).await;
    s::hidden_test(&pool, "two-sum", 1, 1)
        .await
        .expect("hidden test");
    s::sandbox_fill(&pool, "two-sum", 1)
        .await
        .expect("the one-time fill is allowed once hidden tests exist");
    let stamped: bool = sqlx::query_scalar(
        "SELECT sandbox_validated_at IS NOT NULL FROM problem_versions WHERE problem_slug = 'two-sum' AND version = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("version row");
    assert!(stamped, "the fill must set sandbox_validated_at");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_immutability_refuses_child_insert_after_fill(pool: PgPool) {
    base(&pool).await;
    s::hidden_test(&pool, "two-sum", 1, 1)
        .await
        .expect("hidden test");
    s::sandbox_fill(&pool, "two-sum", 1).await.expect("fill");
    let error = sqlx::query(
        "INSERT INTO problem_examples (problem_slug, version, ordinal, input, expected) \
         VALUES ('two-sum', 1, 3, '[3]', '3')",
    )
    .execute(&pool)
    .await
    .expect_err("a sandbox-validated version must not gain examples");
    assert!(
        s::message(&error).contains("frozen"),
        "got: {}",
        s::message(&error)
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_immutability_refuses_child_update_and_delete(pool: PgPool) {
    base(&pool).await;
    s::examples(&pool, "two-sum", 1, 2).await.expect("examples");
    s::hints(&pool, "two-sum", 1, 4).await.expect("hints");

    let update = sqlx::query("UPDATE problem_examples SET expected = '99' WHERE ordinal = 1")
        .execute(&pool)
        .await
        .expect_err("an example must not change");
    assert!(
        s::message(&update).contains("append-only"),
        "got: {}",
        s::message(&update)
    );

    let delete = sqlx::query("DELETE FROM problem_hints WHERE rung = 1")
        .execute(&pool)
        .await
        .expect_err("a hint must not be deleted");
    assert!(
        s::message(&delete).contains("append-only"),
        "got: {}",
        s::message(&delete)
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_immutability_refuses_event_update_and_delete(pool: PgPool) {
    base(&pool).await;
    sqlx::query(
        "INSERT INTO catalogue_events (problem_slug, action, version, operator) VALUES ('two-sum', 'import', 1, 'operator')",
    )
    .execute(&pool)
    .await
    .expect("an event row must insert");

    let update = sqlx::query("UPDATE catalogue_events SET action = 'publish'")
        .execute(&pool)
        .await
        .expect_err("events must not change");
    assert!(
        s::message(&update).contains("append-only"),
        "got: {}",
        s::message(&update)
    );

    let delete = sqlx::query("DELETE FROM catalogue_events")
        .execute(&pool)
        .await
        .expect_err("events must not be deleted");
    assert!(
        s::message(&delete).contains("append-only"),
        "got: {}",
        s::message(&delete)
    );
}
