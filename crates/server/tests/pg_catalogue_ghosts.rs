//! Database tests for `GhostLookup` (problem-catalogue C33, CAT-L6).
//!
//! A custom ghost run keeps its input in `submissions.ghost_input`. The lookup must hand that
//! input back only to the submission's owner, and say nothing for anyone else.
//!
//! Run with `DATABASE_URL=<throwaway postgres 17> cargo test -p counterspell-server --test pg_catalogue_ghosts -- --ignored pg_`.

use counterspell_common::ids::{RunId, UserId};
use counterspell_judge::catalogue::testkit::insert_published_fixture;
use counterspell_server::catalogue::ghosts::GhostLookup;
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

async fn make_user(pool: &PgPool, n: u128) -> UserId {
    let id = Uuid::from_u128(n);
    sqlx::query("INSERT INTO users (id, handle, display_name) VALUES ($1, $2, 'Learner') ON CONFLICT DO NOTHING")
        .bind(id)
        .bind(format!("learner{n}"))
        .execute(pool)
        .await
        .expect("user row");
    UserId::from_uuid(id)
}

/// Saves a run owned by `owner`, with or without a custom ghost input.
async fn save_run(pool: &PgPool, owner: UserId, ghost_input: Option<serde_json::Value>) -> RunId {
    let id = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, status, ghost_input) \
         VALUES ($1, $2, 'two-sum', 'python', 'run', 'pass', $3, 'queued', $4)",
    )
    .bind(id)
    .bind(owner.as_uuid())
    .bind(vec![9u8; 32])
    .bind(ghost_input)
    .execute(pool)
    .await
    .expect("submission row");
    RunId::from_uuid(id)
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_the_owner_gets_the_stored_ghost_input(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("published");
    let owner = make_user(&pool, 1).await;
    let run = save_run(&pool, owner, Some(json!({"nums": [0, 0], "target": 0}))).await;

    let found = GhostLookup::new(pool.clone()).stored_for(owner, run).await;

    assert_eq!(
        found.expect("lookup works").expect("a ghost").input,
        json!({"nums": [0, 0], "target": 0})
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_another_users_submission_returns_none(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("published");
    let owner = make_user(&pool, 1).await;
    let stranger = make_user(&pool, 2).await;
    let run = save_run(&pool, owner, Some(json!([1, 2]))).await;

    let found = GhostLookup::new(pool.clone())
        .stored_for(stranger, run)
        .await;

    assert!(
        found.expect("lookup works").is_none(),
        "a foreign run must not leak its ghost"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_a_submission_without_a_ghost_returns_none(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("published");
    let owner = make_user(&pool, 1).await;
    let run = save_run(&pool, owner, None).await;

    let found = GhostLookup::new(pool.clone()).stored_for(owner, run).await;

    assert!(found.expect("lookup works").is_none());
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_an_unknown_submission_returns_none(pool: PgPool) {
    let owner = make_user(&pool, 1).await;

    let found = GhostLookup::new(pool.clone())
        .stored_for(owner, RunId::from_uuid(Uuid::new_v4()))
        .await;

    assert!(found.expect("lookup works").is_none());
}
