//! HTTP tests for `POST /api/hints` (hints spec H13). Each `pg_` test runs against a real database
//! and needs `DATABASE_URL`. Run with `-- --ignored`. No test reaches an AI provider: AI is either
//! off or answered from the cache.

mod support;

use std::sync::Arc;

use counterspell_server::{
    build_router,
    hints::{handlers::HintState, store},
    ratelimit::{Limiter, MemoryLimiter},
    testkit::BearerTestResolver,
    AppState,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use support::{config_for, post, serve, Fixture, Reply};
use uuid::Uuid;

fn app(pool: PgPool, static_dir: &std::path::Path, hints: HintState) -> AppState {
    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    AppState::new(pool, config_for(static_dir), limiter)
        .with_user_resolver(BearerTestResolver)
        .with_hints(hints)
}

/// The bearer header for the test identity `test-user-<number>`. The few strings it makes are
/// leaked on purpose, so header arrays can borrow them for the whole test.
fn auth(number: u128) -> &'static str {
    Box::leak(format!("Bearer test-user-{number}").into_boxed_str())
}

fn learner(number: u128) -> Uuid {
    Uuid::from_u128(number)
}

/// Creates a finished submission for `user` with the given code, on the published `two-sum`.
async fn finished_submission(pool: &PgPool, user: Uuid, code: &str) -> Uuid {
    counterspell_server::testkit::ensure_user(
        pool,
        counterspell_common::ids::UserId::from_uuid(user),
    )
    .await
    .expect("user fixture");
    support::publish_two_sum(pool).await;
    let submission = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, status, verdict, finished_at)
         VALUES ($1, $2, 'two-sum', 'python', 'run', $3, $4, 'completed', 'wrong_answer', now())",
    )
    .bind(submission)
    .bind(user)
    .bind(code)
    .bind(vec![0u8; 32])
    .execute(pool)
    .await
    .expect("a finished submission can be created");
    submission
}

fn body_of(submission: Uuid, rung: u8, ai: bool) -> String {
    json!({ "submission_id": submission, "rung": rung, "ai": ai }).to_string()
}

fn parse(reply: &Reply) -> Value {
    serde_json::from_str(&reply.body).expect("the reply is JSON")
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "needs DATABASE_URL"]
async fn pg_ladder_reveals_in_order_and_locks_the_later_rung(pool: PgPool) {
    let fixture = Fixture::new();
    let owner = learner(4001);
    let submission = finished_submission(&pool, owner, "print(1)").await;
    let address = serve(build_router(app(
        pool.clone(),
        &fixture.dir,
        HintState::disabled(pool.clone()),
    )))
    .await;
    let headers = [
        ("authorization", auth(4001)),
        ("content-type", "application/json"),
    ];

    let locked = post(
        address,
        "/api/hints",
        &headers,
        &body_of(submission, 2, false),
    )
    .await;
    let first = post(
        address,
        "/api/hints",
        &headers,
        &body_of(submission, 1, false),
    )
    .await;
    let second = post(
        address,
        "/api/hints",
        &headers,
        &body_of(submission, 2, false),
    )
    .await;

    assert_eq!(locked.status, 409);
    assert_eq!(parse(&locked)["code"], "previous_rung_required");
    assert_eq!(parse(&locked)["missing"], 1);
    assert_eq!(first.status, 200);
    assert_eq!(parse(&first)["source"], "ladder");
    assert_eq!(second.status, 200);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "needs DATABASE_URL"]
async fn pg_a_stranger_an_unknown_id_and_an_unfinished_run_get_identical_404s(pool: PgPool) {
    let fixture = Fixture::new();
    let owner = learner(4002);
    let finished = finished_submission(&pool, owner, "print(1)").await;
    let unfinished = Uuid::new_v4();
    sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, status)
         VALUES ($1, $2, 'two-sum', 'python', 'run', 'print(2)', $3, 'queued')",
    )
    .bind(unfinished)
    .bind(owner)
    .bind(vec![0u8; 32])
    .execute(&pool)
    .await
    .expect("an unfinished submission can be created");
    let address = serve(build_router(app(
        pool.clone(),
        &fixture.dir,
        HintState::disabled(pool.clone()),
    )))
    .await;
    let stranger = [
        ("authorization", auth(4003)),
        ("content-type", "application/json"),
    ];
    let owner_headers = [
        ("authorization", auth(4002)),
        ("content-type", "application/json"),
    ];
    counterspell_server::testkit::ensure_user(
        &pool,
        counterspell_common::ids::UserId::from_uuid(learner(4003)),
    )
    .await
    .expect("stranger fixture");

    let other = post(
        address,
        "/api/hints",
        &stranger,
        &body_of(finished, 1, false),
    )
    .await;
    let unknown = post(
        address,
        "/api/hints",
        &owner_headers,
        &body_of(Uuid::new_v4(), 1, false),
    )
    .await;
    let not_done = post(
        address,
        "/api/hints",
        &owner_headers,
        &body_of(unfinished, 1, false),
    )
    .await;

    assert_eq!(other.status, 404);
    assert_eq!(unknown.status, 404);
    assert_eq!(not_done.status, 404);
    assert_eq!(
        other.body, unknown.body,
        "a stranger learns nothing about the id"
    );
    assert_eq!(
        unknown.body, not_done.body,
        "an unfinished run looks the same as an unknown id"
    );
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "needs DATABASE_URL"]
async fn pg_a_cached_ai_hint_is_returned_with_no_provider_call(pool: PgPool) {
    let fixture = Fixture::new();
    let owner = learner(4004);
    let submission = finished_submission(&pool, owner, "print(1)").await;
    let address = serve(build_router(app(
        pool.clone(),
        &fixture.dir,
        HintState::disabled(pool.clone()),
    )))
    .await;
    let headers = [
        ("authorization", auth(4004)),
        ("content-type", "application/json"),
    ];
    post(
        address,
        "/api/hints",
        &headers,
        &body_of(submission, 1, false),
    )
    .await;
    store::insert_ai(
        &pool,
        &store::NewAiHint {
            user: owner,
            submission,
            rung: 2,
            provider: "openai",
            model: "gpt-6-luna",
            body: "A stored AI nudge.",
            tokens_in: 100,
            tokens_out: 20,
            cost_micros: 10,
            cost_estimated: false,
            cache_key: [3u8; 32],
        },
    )
    .await
    .expect("the cached hint is stored");

    let reply = post(
        address,
        "/api/hints",
        &headers,
        &body_of(submission, 2, true),
    )
    .await;

    assert_eq!(reply.status, 200);
    let view = parse(&reply);
    assert_eq!(view["source"], "ai");
    assert_eq!(view["label"], "AI hint");
    assert_eq!(view["body"], "A stored AI nudge.");
    assert_eq!(view["sent_to_provider"], true);
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "needs DATABASE_URL"]
async fn pg_an_ai_hint_is_unavailable_while_ai_is_off(pool: PgPool) {
    let fixture = Fixture::new();
    let owner = learner(4005);
    let submission = finished_submission(&pool, owner, "print(1)").await;
    let address = serve(build_router(app(
        pool.clone(),
        &fixture.dir,
        HintState::disabled(pool.clone()),
    )))
    .await;
    let headers = [
        ("authorization", auth(4005)),
        ("content-type", "application/json"),
    ];
    post(
        address,
        "/api/hints",
        &headers,
        &body_of(submission, 1, false),
    )
    .await;

    let reply = post(
        address,
        "/api/hints",
        &headers,
        &body_of(submission, 2, true),
    )
    .await;

    assert_eq!(reply.status, 503);
    assert_eq!(parse(&reply)["code"], "ai_unavailable");
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "needs DATABASE_URL"]
async fn pg_an_ai_hint_for_code_over_8_kb_is_refused_with_422(pool: PgPool) {
    let fixture = Fixture::new();
    let owner = learner(4006);
    let code = "x".repeat(9_000);
    let submission = finished_submission(&pool, owner, &code).await;
    let address = serve(build_router(app(
        pool.clone(),
        &fixture.dir,
        HintState::disabled(pool.clone()),
    )))
    .await;
    let headers = [
        ("authorization", auth(4006)),
        ("content-type", "application/json"),
    ];
    post(
        address,
        "/api/hints",
        &headers,
        &body_of(submission, 1, false),
    )
    .await;

    let reply = post(
        address,
        "/api/hints",
        &headers,
        &body_of(submission, 2, true),
    )
    .await;

    assert_eq!(reply.status, 422);
    assert_eq!(parse(&reply)["code"], "code_too_large_for_hint");
}

#[sqlx::test(migrations = "../../migrations")]
#[ignore = "needs DATABASE_URL"]
async fn pg_a_rung_outside_one_to_four_or_an_unknown_field_is_400(pool: PgPool) {
    let fixture = Fixture::new();
    let owner = learner(4007);
    let submission = finished_submission(&pool, owner, "print(1)").await;
    let address = serve(build_router(app(
        pool.clone(),
        &fixture.dir,
        HintState::disabled(pool.clone()),
    )))
    .await;
    let headers = [
        ("authorization", auth(4007)),
        ("content-type", "application/json"),
    ];

    let rung_five = post(
        address,
        "/api/hints",
        &headers,
        &body_of(submission, 5, false),
    )
    .await;
    let extra_field = post(
        address,
        "/api/hints",
        &headers,
        &json!({ "submission_id": submission, "rung": 1, "model": "x" }).to_string(),
    )
    .await;

    assert_eq!(rung_five.status, 400);
    assert_eq!(parse(&rung_five)["code"], "invalid_request");
    assert_eq!(extra_field.status, 400);
    assert_eq!(parse(&extra_field)["code"], "invalid_request");
}

#[tokio::test]
async fn hints_without_a_session_is_401_and_an_unknown_api_path_is_a_json_404() {
    let fixture = Fixture::new();
    let pool = support::lazy_pool();
    let address = serve(build_router(app(
        pool.clone(),
        &fixture.dir,
        HintState::disabled(pool),
    )))
    .await;
    let body = body_of(Uuid::new_v4(), 1, false);

    let anonymous = post(
        address,
        "/api/hints",
        &[("content-type", "application/json")],
        &body,
    )
    .await;
    let unknown = post(
        address,
        "/api/hints/unknown",
        &[("content-type", "application/json")],
        &body,
    )
    .await;

    assert_eq!(anonymous.status, 401);
    assert_eq!(parse(&anonymous)["code"], "unauthenticated");
    assert_eq!(unknown.status, 404);
    assert_eq!(parse(&unknown)["code"], "not_found");
}
