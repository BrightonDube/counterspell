//! HTTP contract tests for the review endpoints (dashboard-fsrs T15, FSRS-3.5).

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
async fn signed_out_request_to_due_reviews_gets_401() {
    let fixture = Fixture::new();
    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    let state = AppState::new(lazy_pool(), config_for(&fixture.dir), limiter);
    let address = serve(counterspell_server::build_router(state)).await;

    let reply = get(address, "/api/reviews/due", &[]).await;

    assert_eq!(reply.status, 401);
    let json: serde_json::Value = serde_json::from_str(&reply.body).expect("json body");
    assert_eq!(json["code"], "unauthenticated");
}

#[tokio::test]
async fn a_failed_due_review_read_returns_503_without_database_detail() {
    let fixture = Fixture::new();
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
        "/api/reviews/due",
        &[("authorization", "Bearer test-user-701")],
    )
    .await;

    assert_eq!(reply.status, 503);
    let json: serde_json::Value = serde_json::from_str(&reply.body).expect("json body");
    assert_eq!(json["code"], "reviews_unavailable");
    for leak in ["127.0.0.1", "postgres", "connect", "sqlx", "SELECT"] {
        assert!(
            !reply.body.contains(leak),
            "503 body leaked {leak}: {}",
            reply.body
        );
    }
}
