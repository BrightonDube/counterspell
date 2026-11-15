//! The auth cleanup removes rows that stopped working more than a day ago (auth T33, requirement
//! R3.9 and the session limits in R10.2).
//!
//! Run with `DATABASE_URL` set: `cargo test -p counterspell-server pg_auth_cleanup -- --include-ignored`.

use std::sync::Arc;

use counterspell_common::ids::UserId;
use counterspell_server::auth::{
    cleanup::purge,
    clock::{Clock, ManualClock},
    email::EmailAddress,
    oauth::{flow::FlowStore, ProviderId},
    session::{AuthMethod, SessionService},
    tokens::{Purpose, TokenStore},
};
use serde_json::json;
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

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_auth_cleanup_removes_only_rows_that_stopped_working_a_day_ago(pool: PgPool) {
    let clock = Arc::new(ManualClock::new(start()));
    let old_user = user(&pool, "old").await;
    let new_user = user(&pool, "new").await;
    let sessions = SessionService::new(pool.clone(), clock.clone());
    let tokens = TokenStore::new(pool.clone(), clock.clone());
    let flows = FlowStore::new(pool.clone(), clock.clone());
    let old_address = EmailAddress::parse("old@example.co").expect("valid address");
    let new_address = EmailAddress::parse("new@example.co").expect("valid address");

    // Everything below is created at the start time, so it is long dead when the clock moves on.
    sessions
        .create(old_user, AuthMethod::MagicLink)
        .await
        .expect("an old session");
    tokens
        .issue(Purpose::MagicLink, &old_address, None, json!({}))
        .await
        .expect("an old token");
    flows
        .create_login(ProviderId::GitHub, "/")
        .await
        .expect("an old flow");

    clock.set(start() + Duration::days(32));
    sessions
        .create(new_user, AuthMethod::MagicLink)
        .await
        .expect("a live session");
    tokens
        .issue(Purpose::MagicLink, &new_address, None, json!({}))
        .await
        .expect("a live token");
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
