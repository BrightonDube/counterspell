//! Reference and known-ghost loaders for the judge (problem-catalogue C25, CAT-L4, CAT-X1).
//!
//! Run with `cargo test -p counterspell-judge --test pg_catalogue_judge_ghosts -- --ignored`.

use counterspell_judge::catalogue::testkit::{
    insert_published_fixture, CANARY_GHOST, CANARY_REFERENCE,
};
use counterspell_judge::problem_store::{known_ghosts_for, reference_for};
use sqlx::{PgConnection, PgPool};

/// Opens a connection that runs as `role`, so the test uses that role's real grants.
async fn as_role(pool: &PgPool, role: &str) -> PgConnection {
    let mut conn = pool.acquire().await.expect("connection").detach();
    sqlx::query(&format!("SET ROLE {role}"))
        .execute(&mut conn)
        .await
        .expect("set role");
    conn
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_judge_ghosts_load_pinned_version_only(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("version 1 fixture");
    insert_published_fixture(&pool, "two-sum", 2)
        .await
        .expect("version 2 fixture");
    let mut judge = as_role(&pool, "counterspell_judge").await;

    let reference = reference_for(&mut judge, "two-sum", 1)
        .await
        .expect("reference read")
        .expect("version 1 has a reference");
    assert!(reference.contains(CANARY_REFERENCE));

    let ghosts = known_ghosts_for(&mut judge, "two-sum", 1)
        .await
        .expect("ghost read");
    assert_eq!(ghosts.len(), 1, "exactly the version 1 ghost");
    assert_eq!(ghosts[0].ghost_id, "ghost-1");
    assert_eq!(ghosts[0].kind, "wrong");
    assert!(ghosts[0].input.to_string().contains(CANARY_GHOST));

    let missing = known_ghosts_for(&mut judge, "two-sum", 9)
        .await
        .expect("missing version read");
    assert!(missing.is_empty(), "no ghost rows for an unknown version");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_judge_ghosts_denied_to_server_role(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("fixture");
    let mut server = as_role(&pool, "counterspell_server").await;

    let ghosts = known_ghosts_for(&mut server, "two-sum", 1).await;
    assert!(ghosts.is_err(), "the server role cannot read known ghosts");
    let reference = reference_for(&mut server, "two-sum", 1).await;
    assert!(reference.is_err(), "the server role cannot read references");
}
