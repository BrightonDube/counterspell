//! Tests for the dashboard's authenticated-user extractor (dashboard-fsrs T01, DASH-1.2 and DASH-1.3).
//!
//! The owner must come from the session resolver. A signed-out request gets `401 unauthenticated`.

mod support;

use std::sync::Arc;

use axum::{routing::get, Router};
use counterspell_server::{
    dashboard::DashboardUser,
    ratelimit::{Limiter, MemoryLimiter},
    testkit::BearerTestResolver,
    AppState,
};
use support::{config_for, get as http_get, lazy_pool, serve, Fixture};

async fn whoami(DashboardUser(user): DashboardUser) -> String {
    user.as_uuid().to_string()
}

fn app(resolver_is_bearer: bool, static_dir: &std::path::Path) -> AppState {
    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    let state = AppState::new(lazy_pool(), config_for(static_dir), limiter);
    if resolver_is_bearer {
        state.with_user_resolver(BearerTestResolver)
    } else {
        state
    }
}

#[tokio::test]
async fn signed_out_request_gets_401_with_the_dashboard_code() {
    let fixture = Fixture::new();
    let router = Router::new()
        .route("/probe", get(whoami))
        .with_state(app(false, &fixture.dir));
    let address = serve(router).await;

    let reply = http_get(address, "/probe", &[]).await;

    assert_eq!(reply.status, 401);
    let json: serde_json::Value = serde_json::from_str(&reply.body).expect("json body");
    assert_eq!(json["code"], "unauthenticated");
}

#[tokio::test]
async fn signed_in_request_reaches_the_handler_with_the_session_owner() {
    let fixture = Fixture::new();
    let router = Router::new()
        .route("/probe", get(whoami))
        .with_state(app(true, &fixture.dir));
    let address = serve(router).await;

    let reply = http_get(
        address,
        "/probe",
        &[("authorization", "Bearer test-user-7")],
    )
    .await;

    assert_eq!(reply.status, 200);
    assert_eq!(reply.body, "00000000-0000-0000-0000-000000000007");
}
