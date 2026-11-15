//! Database test for `GET /api/dashboard` over the real router (dashboard-fsrs T05, DASH-1).
//!
//! The signed-in caller comes from the bearer test resolver. The request has no user selector, so
//! the response is the caller's own dashboard.
//!
//! Run with `DATABASE_URL=<throwaway postgres 17> cargo test -p counterspell-server --test pg_dashboard_api -- --ignored pg_`.

mod support;

use std::sync::Arc;

use counterspell_judge::catalogue::testkit::insert_published_fixture;
use counterspell_server::{
    ratelimit::{Limiter, MemoryLimiter},
    testkit::BearerTestResolver,
    AppState,
};
use serde_json::Value;
use sqlx::PgPool;
use support::{config_for, get, serve, Fixture};

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_signed_in_learner_gets_every_dashboard_section(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("published problem");
    let fixture = Fixture::new();
    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    let state = AppState::new(pool, config_for(&fixture.dir), limiter)
        .with_user_resolver(BearerTestResolver);
    let address = serve(counterspell_server::build_router(state)).await;

    let reply = get(
        address,
        "/api/dashboard",
        &[("authorization", "Bearer test-user-401")],
    )
    .await;

    assert_eq!(reply.status, 200, "body: {}", reply.body);
    let json: Value = serde_json::from_str(&reply.body).expect("json body");
    for section in [
        "solved",
        "streak",
        "weak_concepts",
        "recent_submissions",
        "suggested_challenges",
    ] {
        assert!(json.get(section).is_some(), "missing {section} in {json}");
    }
    assert_eq!(json["solved"]["total"], 0);
    assert_eq!(json["suggested_challenges"][0]["slug"], "two-sum");
}
