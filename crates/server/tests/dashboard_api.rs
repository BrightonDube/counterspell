//! HTTP tests for `GET /api/dashboard` (dashboard-fsrs T05, DASH-1, DASH-7 and DASH-8).
//!
//! These tests need no database. A signed-out caller is refused before any query runs. The
//! signed-in path with real data is in `pg_dashboard_api.rs`.

mod support;

use std::{sync::Arc, time::Duration};

use counterspell_server::{
    ratelimit::{Limiter, MemoryLimiter},
    testkit::BearerTestResolver,
    AppState,
};
use sqlx::postgres::PgPoolOptions;
use support::{config_for, get, lazy_pool, serve, Fixture};

#[tokio::test]
async fn signed_out_request_to_the_dashboard_gets_401_with_the_dashboard_code() {
    let fixture = Fixture::new();
    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    let state = AppState::new(lazy_pool(), config_for(&fixture.dir), limiter);
    let address = serve(counterspell_server::build_router(state)).await;

    let reply = get(address, "/api/dashboard", &[]).await;

    assert_eq!(reply.status, 401);
    let json: serde_json::Value = serde_json::from_str(&reply.body).expect("json body");
    assert_eq!(json["code"], "unauthenticated");
}

#[tokio::test]
async fn a_failed_database_read_returns_503_without_sql_or_connection_detail() {
    let fixture = Fixture::new();
    // Port 1 refuses the connection at once. A short acquire timeout keeps the test fast.
    let pool = PgPoolOptions::new()
        .acquire_timeout(Duration::from_millis(500))
        .connect_lazy("postgres://app@127.0.0.1:1/counterspell")
        .expect("lazy pool");
    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    let state = AppState::new(pool, config_for(&fixture.dir), limiter)
        .with_user_resolver(BearerTestResolver);
    let address = serve(counterspell_server::build_router(state)).await;

    let reply = get(
        address,
        "/api/dashboard",
        &[("authorization", "Bearer test-user-402")],
    )
    .await;

    assert_eq!(reply.status, 503);
    let json: serde_json::Value = serde_json::from_str(&reply.body).expect("json body");
    assert_eq!(json["code"], "dashboard_unavailable");
    for leak in ["127.0.0.1", "postgres", "connect", "sqlx", "SELECT"] {
        assert!(
            !reply.body.contains(leak),
            "503 body leaked {leak}: {}",
            reply.body
        );
    }
}
