//! Tests for `GET` and `PATCH /api/me/familiar` (efficiency-familiar T32, requirement R10).
//!
//! The first group needs no database: authentication, content type and body checks all run before any
//! query. The `pg_` group needs a database and is run with
//! `cargo test -p counterspell-server --test familiar_prefs -- --ignored pg_` with `DATABASE_URL` set.

mod support;

use std::sync::Arc;

use counterspell_common::ids::UserId;
use counterspell_server::{
    build_router,
    ratelimit::{Limiter, MemoryLimiter},
    testkit::BearerTestResolver,
    AppState,
};
use serde_json::{json, Value};
use support::{config_for, get, lazy_pool, patch, serve, Fixture, Reply};
use uuid::Uuid;

/// A router with a bearer-token resolver and no database connection.
async fn offline_app() -> std::net::SocketAddr {
    let fixture = Fixture::new();
    let state = AppState::new(
        lazy_pool(),
        config_for(&fixture.dir),
        Limiter::Memory(Arc::new(MemoryLimiter::default())),
    )
    .with_user_resolver(BearerTestResolver);
    serve(build_router(state)).await
}

const JSON_TYPE: (&str, &str) = ("Content-Type", "application/json");
const SIGNED_IN: (&str, &str) = ("Authorization", "Bearer test-user-1");

fn error_code(reply: &Reply) -> String {
    let body: Value = serde_json::from_str(&reply.body).expect("JSON error body");
    body["error"]["code"]
        .as_str()
        .or_else(|| body["code"].as_str())
        .unwrap_or_default()
        .to_owned()
}

#[tokio::test]
async fn a_patch_without_a_session_is_refused_with_401() {
    let address = offline_app().await;
    let reply = patch(
        address,
        "/api/me/familiar",
        &[JSON_TYPE],
        r#"{"muted":true}"#,
    )
    .await;
    assert_eq!(reply.status, 401);
}

#[tokio::test]
async fn a_get_without_a_session_is_refused_with_401() {
    let address = offline_app().await;
    let reply = get(address, "/api/me/familiar", &[]).await;
    assert_eq!(reply.status, 401);
}

#[tokio::test]
async fn a_patch_that_is_not_json_is_refused_with_415() {
    let address = offline_app().await;
    let reply = patch(
        address,
        "/api/me/familiar",
        &[SIGNED_IN, ("Content-Type", "text/plain")],
        "{}",
    )
    .await;
    assert_eq!(reply.status, 415);
}

#[tokio::test]
async fn an_unknown_field_is_refused_with_400_and_names_no_value() {
    let address = offline_app().await;
    let reply = patch(
        address,
        "/api/me/familiar",
        &[SIGNED_IN, JSON_TYPE],
        r#"{"volume":9}"#,
    )
    .await;
    assert_eq!(reply.status, 400);
    assert_eq!(error_code(&reply), "invalid_request");
}

#[tokio::test]
async fn a_wrong_type_is_refused_with_400() {
    let address = offline_app().await;
    let reply = patch(
        address,
        "/api/me/familiar",
        &[SIGNED_IN, JSON_TYPE],
        r#"{"muted":"yes"}"#,
    )
    .await;
    assert_eq!(reply.status, 400);
}

#[tokio::test]
async fn a_value_outside_the_allowed_set_is_refused_with_400() {
    let address = offline_app().await;
    let reply = patch(
        address,
        "/api/me/familiar",
        &[SIGNED_IN, JSON_TYPE],
        r#"{"skin":"ghost"}"#,
    )
    .await;
    assert_eq!(reply.status, 400);
}

#[tokio::test]
async fn a_body_over_one_kibibyte_is_refused_with_400() {
    let address = offline_app().await;
    let padding = "x".repeat(2000);
    let body = json!({ "speech": "off", "pad": padding }).to_string();
    let reply = patch(address, "/api/me/familiar", &[SIGNED_IN, JSON_TYPE], &body).await;
    assert_eq!(reply.status, 400);
}

#[tokio::test]
async fn a_body_that_is_not_valid_json_is_refused_with_400() {
    let address = offline_app().await;
    let reply = patch(
        address,
        "/api/me/familiar",
        &[SIGNED_IN, JSON_TYPE],
        "{not json",
    )
    .await;
    assert_eq!(reply.status, 400);
}

/// The user id the bearer test resolver gives for `number`.
fn user(number: u128) -> UserId {
    UserId::from_uuid(Uuid::from_u128(number))
}

/// Connects to the database named by `DATABASE_URL`, for the `pg_` tests only.
async fn pool() -> sqlx::PgPool {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL is set for pg_ tests");
    sqlx::PgPool::connect(&url).await.expect("connect")
}

/// A router backed by the real database.
async fn database_app(pool: sqlx::PgPool) -> std::net::SocketAddr {
    let fixture = Fixture::new();
    let state = AppState::new(
        pool,
        config_for(&fixture.dir),
        Limiter::Memory(Arc::new(MemoryLimiter::default())),
    )
    .with_user_resolver(BearerTestResolver);
    serve(build_router(state)).await
}

fn bearer(number: u128) -> String {
    format!("Bearer test-user-{number}")
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn pg_prefs_round_trip_and_fill_defaults() {
    let pool = pool().await;
    let who = user(7_101);
    counterspell_server::testkit::ensure_user(&pool, who)
        .await
        .expect("user");
    let address = database_app(pool.clone()).await;
    let token = bearer(7_101);

    let reply = patch(
        address,
        "/api/me/familiar",
        &[("Authorization", token.as_str()), JSON_TYPE],
        r#"{"muted":true}"#,
    )
    .await;
    assert_eq!(reply.status, 200);
    let saved: Value = serde_json::from_str(&reply.body).expect("JSON");
    assert_eq!(saved["muted"], json!(true));
    assert_eq!(saved["hidden"], json!(false));
    assert_eq!(saved["skin"], json!("jack-o-lantern"));
    assert_eq!(saved["speech"], json!("auto"));

    let read = get(
        address,
        "/api/me/familiar",
        &[("Authorization", token.as_str())],
    )
    .await;
    assert_eq!(read.status, 200);
    let stored: Value = serde_json::from_str(&read.body).expect("JSON");
    assert_eq!(
        stored["muted"],
        json!(true),
        "the change persists between requests"
    );
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn pg_prefs_one_user_cannot_change_another() {
    let pool = pool().await;
    counterspell_server::testkit::ensure_user(&pool, user(7_102))
        .await
        .expect("user a");
    counterspell_server::testkit::ensure_user(&pool, user(7_103))
        .await
        .expect("user b");
    let address = database_app(pool.clone()).await;

    let a = bearer(7_102);
    let reply = patch(
        address,
        "/api/me/familiar",
        &[("Authorization", a.as_str()), JSON_TYPE],
        r#"{"hidden":true}"#,
    )
    .await;
    assert_eq!(reply.status, 200);

    let b = bearer(7_103);
    let other = get(
        address,
        "/api/me/familiar",
        &[("Authorization", b.as_str())],
    )
    .await;
    let other: Value = serde_json::from_str(&other.body).expect("JSON");
    assert_eq!(
        other["hidden"],
        json!(false),
        "user B keeps its own preferences"
    );
}

#[tokio::test]
#[ignore = "needs DATABASE_URL"]
async fn pg_demo_account_may_change_its_preferences() {
    let pool = pool().await;
    let who = user(7_104);
    // Only seeding may create the demo account. The setting is local to this transaction, as in seeding.
    let mut tx = pool.begin().await.expect("transaction");
    sqlx::query("SELECT set_config('counterspell.demo_seed', 'on', true)")
        .execute(&mut *tx)
        .await
        .expect("seed flag");
    sqlx::query(
        "INSERT INTO users (id, handle, display_name, is_demo) VALUES ($1, $2, 'Demo', true)
         ON CONFLICT (id) DO NOTHING",
    )
    .bind(who.as_uuid())
    .bind(who.as_uuid().simple().to_string())
    .execute(&mut *tx)
    .await
    .expect("demo fixture");
    tx.commit().await.expect("commit");
    let address = database_app(pool.clone()).await;
    let token = bearer(7_104);

    let reply = patch(
        address,
        "/api/me/familiar",
        &[("Authorization", token.as_str()), JSON_TYPE],
        r#"{"hidden":true}"#,
    )
    .await;
    assert_eq!(
        reply.status, 200,
        "the demo account is not refused a preference change"
    );
    let saved: Value = serde_json::from_str(&reply.body).expect("JSON");
    assert_eq!(saved["hidden"], json!(true));
}
