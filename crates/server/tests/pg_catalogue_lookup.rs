//! Database tests for the server's catalogue lookup (problem-catalogue C28, requirements CAT-L2 and
//! CAT-A9).
//!
//! The lookup returns only published problems and the version that is published now. The server
//! role cannot read any hidden table, so the test asserts that with `has_table_privilege`.
//!
//! Run with `DATABASE_URL=<throwaway postgres 17> cargo test -p counterspell-server --test pg_catalogue_lookup -- --ignored pg_`.

use counterspell_judge::catalogue::testkit::{insert_published_fixture, insert_static_fixture};
use counterspell_server::{catalogue::lookup::PgProblemLookup, seams::ProblemLookup};
use sqlx::PgPool;

async fn role_can(pool: &PgPool, role: &str, table: &str, privilege: &str) -> bool {
    let (granted,): (bool,) = sqlx::query_as("SELECT has_table_privilege($1, $2, $3)")
        .bind(role)
        .bind(table)
        .bind(privilege)
        .fetch_one(pool)
        .await
        .expect("privilege query");
    granted
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_lookup_returns_the_published_version(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 2)
        .await
        .expect("published fixture");
    let lookup = PgProblemLookup::new(pool.clone());

    let found = lookup.published("two-sum").await;

    assert_eq!(
        found,
        Some(counterspell_server::seams::ProblemRef {
            slug: "two-sum".to_owned(),
            version: 2,
        })
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_lookup_hides_an_unpublished_problem(pool: PgPool) {
    insert_static_fixture(&pool, "two-sum", 1)
        .await
        .expect("static fixture");
    let lookup = PgProblemLookup::new(pool.clone());

    assert_eq!(lookup.published("two-sum").await, None);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_lookup_hides_a_problem_after_unpublish(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("published fixture");
    sqlx::query(
        "UPDATE problems SET publication_state = 'unpublished', unpublished_at = now() WHERE slug = $1",
    )
    .bind("two-sum")
    .execute(&pool)
    .await
    .expect("unpublish");
    let lookup = PgProblemLookup::new(pool.clone());

    assert_eq!(lookup.published("two-sum").await, None);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_lookup_returns_none_for_an_unknown_slug(pool: PgPool) {
    let lookup = PgProblemLookup::new(pool.clone());

    assert_eq!(lookup.published("no-such-problem").await, None);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_server_role_can_read_problems_but_not_hidden_tables(pool: PgPool) {
    assert!(role_can(&pool, "counterspell_server", "problems", "SELECT").await);
    for table in ["hidden_tests", "reference_solutions", "known_ghosts"] {
        assert!(
            !role_can(&pool, "counterspell_server", table, "SELECT").await,
            "the server role must not read {table} (CAT-A9)"
        );
    }
}
