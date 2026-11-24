//! Database tests for `GET /api/problems` (problem-catalogue C30, CAT-A1 to CAT-A3, CAT-P4).
//!
//! The list shows only published problems, filters by track, difficulty and concept, and sets each
//! status from the caller's own submissions only. Fixtures come from the judge's catalogue test kit,
//! which writes through the same triggers as production.
//!
//! Run with `DATABASE_URL=<throwaway postgres 17> cargo test -p counterspell-server --test pg_catalogue_api -- --ignored pg_`.

mod support;

use std::sync::Arc;

use counterspell_judge::catalogue::testkit::{insert_published_fixture, insert_static_fixture};
use counterspell_server::{
    ratelimit::{Limiter, MemoryLimiter},
    testkit::BearerTestResolver,
    AppState,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use support::{config_for, get, serve, Fixture};
use uuid::Uuid;

/// Builds the real router over the test pool, with the bearer test resolver.
async fn app(pool: &PgPool, dir: &std::path::Path) -> std::net::SocketAddr {
    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    let state = AppState::new(pool.clone(), config_for(dir), limiter)
        .with_user_resolver(BearerTestResolver);
    serve(counterspell_server::build_router(state)).await
}

/// Inserts a learner whose id is `Uuid::from_u128(n)`, the id the bearer test resolver uses.
async fn make_user(pool: &PgPool, n: u128) {
    let id = Uuid::from_u128(n);
    sqlx::query("INSERT INTO users (id, handle, display_name) VALUES ($1, $2, 'Learner') ON CONFLICT DO NOTHING")
        .bind(id)
        .bind(format!("learner{n}"))
        .execute(pool)
        .await
        .expect("user row");
}

/// Saves a finished or unfinished submission for a learner.
async fn save_run(pool: &PgPool, n: u128, slug: &str, status: &str, verdict: Option<&str>) {
    sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, status, verdict, finished_at) \
         VALUES ($1, $2, $3, 'python', 'submit', 'pass', $4, $5, $6, CASE WHEN $5 = 'completed' THEN now() END)",
    )
    .bind(Uuid::new_v4())
    .bind(Uuid::from_u128(n))
    .bind(slug)
    .bind(vec![9u8; 32])
    .bind(status)
    .bind(verdict)
    .execute(pool)
    .await
    .expect("submission row");
}

fn slugs(body: &Value) -> Vec<String> {
    body["problems"]
        .as_array()
        .expect("problems array")
        .iter()
        .map(|item| item["slug"].as_str().expect("slug").to_owned())
        .collect()
}

fn status_of(body: &Value, slug: &str) -> Value {
    body["problems"]
        .as_array()
        .expect("problems array")
        .iter()
        .find(|item| item["slug"] == slug)
        .map(|item| item["status"].clone())
        .expect("problem present in list")
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_list_shows_published_problems_and_hides_unpublished(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("published");
    insert_static_fixture(&pool, "count-pairs", 1)
        .await
        .expect("static only");
    let fixture = Fixture::new();
    let address = app(&pool, &fixture.dir).await;

    let reply = get(address, "/api/problems", &[]).await;

    assert_eq!(reply.status, 200);
    let body: Value = serde_json::from_str(&reply.body).expect("JSON");
    assert_eq!(slugs(&body), vec!["two-sum".to_owned()]);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_list_filters_by_track_difficulty_and_concept(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("published");
    let fixture = Fixture::new();
    let address = app(&pool, &fixture.dir).await;

    // The fixture uses track `arrays`, difficulty `easy` and concept `hash-maps`.
    for (path, expected) in [
        ("/api/problems?track=arrays", 1),
        ("/api/problems?track=graphs", 0),
        ("/api/problems?difficulty=easy", 1),
        ("/api/problems?difficulty=hard", 0),
        ("/api/problems?concept=hash-maps", 1),
        ("/api/problems?concept=trees", 0),
    ] {
        let reply = get(address, path, &[]).await;
        assert_eq!(reply.status, 200, "{path}");
        let body: Value = serde_json::from_str(&reply.body).expect("JSON");
        assert_eq!(slugs(&body).len(), expected, "{path}");
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_anonymous_callers_see_a_null_status(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("published");
    let fixture = Fixture::new();
    let address = app(&pool, &fixture.dir).await;

    let reply = get(address, "/api/problems", &[]).await;

    let body: Value = serde_json::from_str(&reply.body).expect("JSON");
    assert_eq!(status_of(&body, "two-sum"), Value::Null);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_detail_returns_the_pinned_public_fields(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("published");
    let fixture = Fixture::new();
    let address = app(&pool, &fixture.dir).await;

    let reply = get(address, "/api/problems/two-sum", &[]).await;

    assert_eq!(reply.status, 200);
    let body: Value = serde_json::from_str(&reply.body).expect("JSON");
    assert_eq!(body["slug"], "two-sum");
    assert_eq!(body["title"], "Two Sum");
    assert_eq!(body["track"]["slug"], "arrays");
    assert_eq!(body["difficulty"], "easy");
    assert_eq!(body["constraints"], json!([]));
    assert_eq!(
        body["examples"],
        json!([
            {"input": [1], "expected": 1},
            {"input": [2], "expected": 2}
        ])
    );
    assert_eq!(body["signature"]["function_name"], "two_sum");
    assert_eq!(
        body["signature"]["parameters"],
        json!([
            {"name": "values", "type": "list[int]"},
            {"name": "target", "type": "int"}
        ])
    );
    assert_eq!(body["signature"]["return_type"], "list[int]");
    assert_eq!(body["limits"], json!({"time_ms": 2000, "mem_mb": 256}));
    // Every starter row for the pinned version is enabled (LC03: starter ∩ driver).
    assert_eq!(
        body["enabled_languages"],
        json!(["cpp", "java", "javascript", "python"])
    );
    assert_eq!(body["hint_count"], 4);
    for lang in ["python", "javascript", "java", "cpp"] {
        assert!(body["starters"][lang].is_string(), "{lang} starter");
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_detail_unknown_and_unpublished_slugs_get_the_same_404(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("published");
    insert_static_fixture(&pool, "count-pairs", 1)
        .await
        .expect("static only");
    let fixture = Fixture::new();
    let address = app(&pool, &fixture.dir).await;

    let unknown = get(address, "/api/problems/no-such-problem", &[]).await;
    let unpublished = get(address, "/api/problems/count-pairs", &[]).await;

    assert_eq!(unknown.status, 404);
    assert_eq!(unpublished.status, 404);
    assert_eq!(unknown.body, unpublished.body);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_status_comes_only_from_the_callers_own_submissions(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("published");
    insert_published_fixture(&pool, "count-pairs", 1)
        .await
        .expect("published");
    insert_published_fixture(&pool, "longest-run", 1)
        .await
        .expect("published");
    make_user(&pool, 1).await;
    make_user(&pool, 2).await;
    // Learner 1 solved two-sum and only attempted count-pairs. Learner 2 solved longest-run.
    save_run(&pool, 1, "two-sum", "completed", Some("accepted")).await;
    save_run(&pool, 1, "count-pairs", "completed", Some("wrong_answer")).await;
    save_run(&pool, 2, "longest-run", "completed", Some("accepted")).await;
    let fixture = Fixture::new();
    let address = app(&pool, &fixture.dir).await;

    let reply = get(
        address,
        "/api/problems",
        &[("Authorization", "Bearer test-user-1")],
    )
    .await;

    let body: Value = serde_json::from_str(&reply.body).expect("JSON");
    assert_eq!(status_of(&body, "two-sum"), "solved");
    assert_eq!(status_of(&body, "count-pairs"), "attempted");
    assert_eq!(status_of(&body, "longest-run"), "unattempted");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_unfinished_run_does_not_count_as_solved(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("published");
    make_user(&pool, 1).await;
    save_run(&pool, 1, "two-sum", "queued", None).await;
    let fixture = Fixture::new();
    let address = app(&pool, &fixture.dir).await;

    let reply = get(
        address,
        "/api/problems",
        &[("Authorization", "Bearer test-user-1")],
    )
    .await;

    let body: Value = serde_json::from_str(&reply.body).expect("JSON");
    assert_eq!(status_of(&body, "two-sum"), "attempted");
}

/// T39 finding 1: an accepted sample-only Run must show as attempted, never solved.
#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_accepted_sample_run_is_attempted_not_solved(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("published");
    make_user(&pool, 1).await;
    sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, status, verdict, finished_at) \
         VALUES ($1, $2, 'two-sum', 'python', 'run', 'pass', $3, 'completed', 'accepted', now())",
    )
    .bind(Uuid::new_v4())
    .bind(Uuid::from_u128(1))
    .bind(vec![9u8; 32])
    .execute(&pool)
    .await
    .expect("run row");
    let fixture = Fixture::new();
    let address = app(&pool, &fixture.dir).await;

    let reply = get(
        address,
        "/api/problems",
        &[("Authorization", "Bearer test-user-1")],
    )
    .await;

    let body: Value = serde_json::from_str(&reply.body).expect("JSON");
    assert_eq!(status_of(&body, "two-sum"), "attempted");
}
