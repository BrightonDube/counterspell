//! Judge runtime reads (problem-catalogue C24, CAT-L4, CAT-R4, CAT-X1, P9).
//!
//! The tests run as the judge role (`SET ROLE counterspell_judge`), so they prove the grants
//! that the judge really has, not the owner's wider access.
//!
//! Run with `cargo test -p counterspell-judge --test pg_catalogue_judge_reads -- --ignored`.

use counterspell_judge::catalogue::testkit::{insert_published_fixture, CANARY_HIDDEN};
use counterspell_judge::problem_store::{
    concept_tags_for, known_ghosts_for, load, LoadMode, StoreError,
};
use sqlx::{PgConnection, PgPool};

const JUDGE_ROLE: &str = "counterspell_judge";

/// Opens a connection that runs as the judge role.
async fn judge_connection(pool: &PgPool) -> PgConnection {
    let mut conn = pool.acquire().await.expect("connection").detach();
    sqlx::query(&format!("SET ROLE {JUDGE_ROLE}"))
        .execute(&mut conn)
        .await
        .expect("set judge role");
    conn
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_judge_reads_run_has_no_hidden_rows(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("fixture");
    let mut conn = judge_connection(&pool).await;

    let run = load(&mut conn, "two-sum", 1, LoadMode::Run)
        .await
        .expect("run load")
        .expect("pinned version exists");
    assert_eq!(run.examples.len(), 2, "Run loads the two public examples");
    assert!(run.hidden.is_empty(), "Run never loads hidden tests");

    let submit = load(&mut conn, "two-sum", 1, LoadMode::Submit)
        .await
        .expect("submit load")
        .expect("pinned version exists");
    assert_eq!(submit.hidden.len(), 5, "Submit loads every hidden test");
    assert!(
        submit
            .hidden
            .iter()
            .any(|case| case.input.to_string().contains(CANARY_HIDDEN)),
        "Submit sees the hidden canary values"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_judge_reads_unpublished_keeps_pinned_version(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("fixture");
    sqlx::query(
        "UPDATE problems SET publication_state = 'unpublished', unpublished_at = now() WHERE slug = 'two-sum'",
    )
    .execute(&pool)
    .await
    .expect("unpublish");
    let mut conn = judge_connection(&pool).await;

    let pinned = load(&mut conn, "two-sum", 1, LoadMode::Submit)
        .await
        .expect("load after unpublish");
    assert!(
        pinned.is_some(),
        "a queued run still finds its pinned version (P9)"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_judge_reads_missing_version_is_none(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("fixture");
    let mut conn = judge_connection(&pool).await;

    let missing = load(&mut conn, "two-sum", 9, LoadMode::Run)
        .await
        .expect("missing version is not an infrastructure error");
    assert!(missing.is_none());
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_judge_reads_judge_role_cannot_write(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("fixture");
    let mut conn = judge_connection(&pool).await;

    let write =
        sqlx::query("UPDATE problem_versions SET title = 'Changed' WHERE problem_slug = 'two-sum'")
            .execute(&mut conn)
            .await;
    assert!(write.is_err(), "the judge role has no write grant");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_judge_reads_submit_hunt_data_under_the_judge_role(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("fixture");
    let mut conn = judge_connection(&pool).await;

    // The submit hunt reads authored ghosts and the problem's concept tags with
    // the judge role's grants — `problem_concepts` is public-level data the
    // judge needs for the autopsy (ide-workspace T25).
    let ghosts = known_ghosts_for(&mut conn, "two-sum", 1)
        .await
        .expect("known ghosts load under the judge role");
    assert_eq!(ghosts.len(), 1);
    assert_eq!(ghosts[0].ghost_id, "ghost-1");
    assert_eq!(ghosts[0].kind, "wrong");
    let tags = concept_tags_for(&mut conn, "two-sum")
        .await
        .expect("concept tags load under the judge role");
    assert_eq!(tags, ["hash-maps"]);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_judge_reads_store_error_is_infrastructure(pool: PgPool) {
    // A connection without the judge grants cannot read hidden data. The error is not `None`.
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("fixture");
    let mut conn = pool.acquire().await.expect("connection").detach();
    sqlx::query("SET ROLE counterspell_server")
        .execute(&mut conn)
        .await
        .expect("set server role");
    let result = load(&mut conn, "two-sum", 1, LoadMode::Submit).await;
    assert!(
        matches!(result, Err(StoreError::Database(_))),
        "a permission failure is an infrastructure error, not a missing problem"
    );
}
