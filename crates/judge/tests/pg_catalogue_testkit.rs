//! Tests for the catalogue test kit (problem-catalogue C05, CAT-X2).
//!
//! The kit inserts rows through the same triggers as production. These tests check two things:
//! the fixtures pass every trigger, and the canary strings in hidden data are visible only to the
//! judge role.
//!
//! Run with `cargo test -p counterspell-judge --test pg_catalogue_testkit -- --ignored`.

use counterspell_judge::catalogue::testkit::{
    insert_published_fixture, insert_static_fixture, CANARY_HIDDEN,
};
use sqlx::PgPool;

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_testkit_published_fixture_passes_triggers(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("the published fixture must pass every trigger");
    let state: String =
        sqlx::query_scalar("SELECT publication_state FROM problems WHERE slug = 'two-sum'")
            .fetch_one(&pool)
            .await
            .expect("the problem row");
    assert_eq!(state, "published");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_testkit_static_fixture_has_no_hidden_rows(pool: PgPool) {
    insert_static_fixture(&pool, "two-sum", 1)
        .await
        .expect("the static fixture must insert");
    let hidden: i64 = sqlx::query_scalar("SELECT count(*) FROM hidden_tests")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(hidden, 0, "a static fixture carries no hidden tests");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_testkit_canary_is_visible_only_to_judge(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("the published fixture must insert");
    let pattern = format!("%{CANARY_HIDDEN}%");

    // The judge role reads the canary from hidden_tests.
    let mut judge = pool.begin().await.expect("transaction");
    sqlx::query("SET LOCAL ROLE counterspell_judge")
        .execute(&mut *judge)
        .await
        .expect("the test may assume the judge role");
    let seen_by_judge: i64 =
        sqlx::query_scalar("SELECT count(*) FROM hidden_tests WHERE input::text LIKE $1")
            .bind(&pattern)
            .fetch_one(&mut *judge)
            .await
            .expect("the judge may read hidden tests");
    judge.rollback().await.expect("rollback");
    assert!(
        seen_by_judge > 0,
        "the judge must see the canary in hidden tests"
    );

    // The server role cannot read hidden tests at all.
    let mut server = pool.begin().await.expect("transaction");
    sqlx::query("SET LOCAL ROLE counterspell_server")
        .execute(&mut *server)
        .await
        .expect("the test may assume the server role");
    let hidden_read = sqlx::query_scalar::<_, i64>("SELECT count(*) FROM hidden_tests")
        .fetch_one(&mut *server)
        .await;
    assert!(
        hidden_read.is_err(),
        "the server role must not read hidden tests"
    );
    server.rollback().await.expect("rollback");

    // Public tables never contain the canary, even though the fixture put it in hidden data.
    let mut public = pool.begin().await.expect("transaction");
    sqlx::query("SET LOCAL ROLE counterspell_server")
        .execute(&mut *public)
        .await
        .expect("server role");
    let leaked: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM problem_versions WHERE to_jsonb(problem_versions)::text LIKE $1",
    )
    .bind(&pattern)
    .fetch_one(&mut *public)
    .await
    .expect("the server may read problem versions");
    public.rollback().await.expect("rollback");
    assert_eq!(
        leaked, 0,
        "the canary must not appear in public version rows"
    );
}
