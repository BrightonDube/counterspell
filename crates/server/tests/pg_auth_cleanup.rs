//! The auth cleanup removes rows that stopped working more than a day ago (auth T33, requirement
//! R3.9 and the session limits in R10.2).
//!
//! Email tokens are retired, but stale rows can still exist from before the retirement, so the
//! cleanup still sweeps the table. The test seeds them with SQL, since nothing issues them now.
//!
//! Run with `DATABASE_URL` set: `cargo test -p counterspell-server pg_auth_cleanup -- --include-ignored`.

use std::sync::Arc;

use counterspell_common::ids::UserId;
use counterspell_server::auth::{
    cleanup::purge,
    clock::{Clock, ManualClock},
    oauth::{flow::FlowStore, ProviderId},
    session::{AuthMethod, SessionService},
    tokens::TokenHash,
};
use sqlx::{types::Uuid, PgPool};
use time::{Duration, OffsetDateTime};

fn start() -> OffsetDateTime {
    OffsetDateTime::from_unix_timestamp(1_800_000_000).expect("valid timestamp")
}

async fn count(pool: &PgPool, table: &str) -> i64 {
    let sql = format!("SELECT count(*) FROM {table}");
    let (count,): (i64,) = sqlx::query_as(&sql).fetch_one(pool).await.expect("count");
    count
}

async fn user(pool: &PgPool, handle: &str) -> UserId {
    let (id,): (Uuid,) =
        sqlx::query_as("INSERT INTO users (handle, display_name) VALUES ($1, $1) RETURNING id")
            .bind(handle)
            .fetch_one(pool)
            .await
            .expect("insert user");
    UserId::from_uuid(id)
}

/// Writes a stale `email_tokens` row the way the retired token store used to: a 15-minute window
/// starting at `issued`.
// test-change: ACC-1
async fn old_email_token(pool: &PgPool, email: &str, issued: OffsetDateTime) {
    sqlx::query(
        "INSERT INTO email_tokens (token_hash, purpose, email, created_at, expires_at)
         VALUES ($1, 'magic_link', $2, $3, $3 + interval '15 minutes')",
    )
    .bind(TokenHash::of(email).as_bytes().as_slice())
    .bind(email)
    .bind(issued)
    .execute(pool)
    .await
    .expect("an old token");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
// test-change: ACC-1
async fn pg_auth_cleanup_removes_only_rows_that_stopped_working_a_day_ago(pool: PgPool) {
    let clock = Arc::new(ManualClock::new(start()));
    let old_user = user(&pool, "old").await;
    let new_user = user(&pool, "new").await;
    let sessions = SessionService::new(pool.clone(), clock.clone());
    let flows = FlowStore::new(pool.clone(), clock.clone());

    // Everything below is created at the start time, so it is long dead when the clock moves on.
    sessions
        .create(old_user, AuthMethod::MagicLink)
        .await
        .expect("an old session");
    old_email_token(&pool, "old@example.co", start()).await;
    flows
        .create_login(ProviderId::GitHub, "/")
        .await
        .expect("an old flow");

    clock.set(start() + Duration::days(32));
    sessions
        .create(new_user, AuthMethod::MagicLink)
        .await
        .expect("a live session");
    old_email_token(&pool, "new@example.co", clock.now()).await;
    flows
        .create_login(ProviderId::Google, "/")
        .await
        .expect("a live flow");

    let removed = purge(&pool, clock.now()).await.expect("cleanup runs");

    assert_eq!(removed, 3, "the old session, token and flow are removed");
    assert_eq!(count(&pool, "sessions").await, 1);
    assert_eq!(count(&pool, "email_tokens").await, 1);
    assert_eq!(count(&pool, "oauth_flows").await, 1);
}
