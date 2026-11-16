//! Canary checks for the public catalogue API (problem-catalogue C31, CAT-A4 to CAT-A6, CAT-X1 to CAT-X3).
//!
//! The judge's test kit plants fixed canary strings in hidden tests, reference code and known ghosts.
//! No response body may contain one, whichever route answers and whatever the caller asks for.
//!
//! Run with `DATABASE_URL=<throwaway postgres 17> cargo test -p counterspell-server --test catalogue_no_leak -- --ignored pg_`.

mod support;

use std::sync::Arc;

use counterspell_judge::catalogue::testkit::{
    insert_published_fixture, insert_static_fixture, CANARY_EXPECTED, CANARY_GHOST, CANARY_HIDDEN,
    CANARY_REFERENCE,
};
use counterspell_server::{
    ratelimit::{Limiter, MemoryLimiter},
    testkit::BearerTestResolver,
    AppState,
};
use sqlx::PgPool;
use support::{config_for, get, serve, Fixture};

const CANARIES: [&str; 4] = [
    CANARY_HIDDEN,
    CANARY_EXPECTED,
    CANARY_REFERENCE,
    CANARY_GHOST,
];

async fn app(pool: &PgPool, dir: &std::path::Path) -> std::net::SocketAddr {
    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    let state = AppState::new(pool.clone(), config_for(dir), limiter)
        .with_user_resolver(BearerTestResolver);
    serve(counterspell_server::build_router(state)).await
}

fn assert_no_canary(route: &str, body: &str) {
    for canary in CANARIES {
        assert!(!body.contains(canary), "{route} leaked {canary}");
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_no_canary_appears_in_any_catalogue_response(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("published");
    insert_static_fixture(&pool, "count-pairs", 1)
        .await
        .expect("static only");
    let fixture = Fixture::new();
    let address = app(&pool, &fixture.dir).await;

    let routes = [
        "/api/problems",
        "/api/problems?difficulty=easy",
        "/api/problems/two-sum",
        "/api/problems/count-pairs",
        "/api/problems/no-such-problem",
        "/api/problems?track=CANARY-HIDDEN-7f3a",
        "/api/problems?difficulty=CANARY-EXPECTED-9b1c",
    ];
    for route in routes {
        let reply = get(address, route, &[]).await;
        assert_no_canary(route, &reply.body);
    }
    let detail = get(address, "/api/problems/two-sum", &[]).await;
    assert_eq!(detail.status, 200, "the published detail must answer");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_no_canary_appears_for_a_signed_in_caller(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("published");
    let fixture = Fixture::new();
    let address = app(&pool, &fixture.dir).await;

    let headers = [("Authorization", "Bearer test-user-1")];
    for route in ["/api/problems", "/api/problems/two-sum"] {
        let reply = get(address, route, &headers).await;
        assert_eq!(reply.status, 200, "{route}");
        assert_no_canary(route, &reply.body);
    }
}
