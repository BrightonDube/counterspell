//! HTTP tests for `GET /api/dashboard` (dashboard-fsrs T05, DASH-1, DASH-7 and DASH-8).
//!
//! These tests need no database. A signed-out caller is refused before any query runs. The
//! signed-in path with real data is in `pg_dashboard_api.rs`.

mod support;

use std::sync::Arc;

use counterspell_server::{
    ratelimit::{Limiter, MemoryLimiter},
    AppState,
};
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
