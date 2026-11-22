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
use uuid::Uuid;

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

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_active_run_has_no_verdict_and_the_response_never_carries_code(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("published problem");
    sqlx::query(
        "INSERT INTO users (id, handle, display_name) VALUES ($1, 'learner402', 'Learner') ON CONFLICT DO NOTHING",
    )
    .bind(Uuid::from_u128(402))
    .execute(&pool)
    .await
    .expect("user row");
    sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, status) \
         VALUES ($1, $2, 'two-sum', 'python', 'submit', 'print(\"CODE-CANARY-402\")', $3, 'queued')",
    )
    .bind(Uuid::new_v4())
    .bind(Uuid::from_u128(402))
    .bind(vec![7u8; 32])
    .execute(&pool)
    .await
    .expect("queued submission row");
    let fixture = Fixture::new();
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

    assert_eq!(reply.status, 200, "body: {}", reply.body);
    assert!(
        !reply.body.contains("CODE-CANARY-402"),
        "submitted code leaked: {}",
        reply.body
    );
    let json: Value = serde_json::from_str(&reply.body).expect("json body");
    let mut keys: Vec<&str> = json
        .as_object()
        .expect("object")
        .keys()
        .map(String::as_str)
        .collect();
    keys.sort_unstable();
    // test-change: CP-4 (the public dashboard contract includes bounded activity days)
    assert_eq!(
        keys,
        [
            "activity_days",
            "recent_submissions",
            "review_summary",
            "solved",
            "streak",
            "suggested_challenges",
            "weak_concepts"
        ]
    );
    let run = &json["recent_submissions"][0];
    assert_eq!(run["status"], "queued");
    assert!(
        run["verdict"].is_null(),
        "an active run has no verdict: {run}"
    );
    assert!(run["finished_at"].is_null());
    assert!(
        run.get("code").is_none(),
        "a run must not carry its code: {run}"
    );
}
