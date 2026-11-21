//! PostgreSQL tests for `GET /api/reviews/due` (dashboard-fsrs T15, FSRS-3.1/3.2).
//!
//! Run with `DATABASE_URL=<throwaway postgres 17> cargo test -p counterspell-server --test pg_reviews -- --ignored pg_`.

mod support;

use std::sync::Arc;

use counterspell_judge::catalogue::testkit::{insert_published_fixture, insert_static_fixture};
use counterspell_server::{
    ratelimit::{Limiter, MemoryLimiter},
    testkit::BearerTestResolver,
    AppState,
};
use serde_json::Value;
use sqlx::PgPool;
use support::{config_for, get, post, serve, Fixture};
use uuid::Uuid;

async fn make_user(pool: &PgPool, n: u128) -> Uuid {
    let id = Uuid::from_u128(n);
    sqlx::query(
        "INSERT INTO users (id, handle, display_name) VALUES ($1, $2, 'Learner') ON CONFLICT DO NOTHING",
    )
    .bind(id)
    .bind(format!("learner{n}"))
    .execute(pool)
    .await
    .expect("user row");
    id
}

async fn add_concept(pool: &PgPool, id: &str, name: &str) {
    sqlx::query("INSERT INTO concepts (id, name) VALUES ($1, $2) ON CONFLICT DO NOTHING")
        .bind(id)
        .bind(name)
        .execute(pool)
        .await
        .expect("concept row");
}

async fn tag(pool: &PgPool, problem_slug: &str, concept_id: &str) {
    sqlx::query(
        "INSERT INTO problem_concepts (problem_slug, concept_id) VALUES ($1, $2) ON CONFLICT DO NOTHING",
    )
    .bind(problem_slug)
    .bind(concept_id)
    .execute(pool)
    .await
    .expect("problem concept row");
}

async fn add_card(pool: &PgPool, user: Uuid, concept: &str, due: &str, version: i64) {
    sqlx::query(
        "INSERT INTO concept_cards
         (user_id, concept_id, stability, stability_fast, difficulty, reps, lapses, state, due, last_review, version)
         VALUES ($1, $2, 1, 1, 1, 1, 0, 'review', $3::timestamptz, now() - interval '1 day', $4)",
    )
    .bind(user)
    .bind(concept)
    .bind(due)
    .bind(version)
    .execute(pool)
    .await
    .expect("due card");
}

async fn add_solved(pool: &PgPool, user: Uuid, problem_slug: &str) {
    sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, status, verdict, finished_at)
         VALUES ($1, $2, $3, 'python', 'submit', 'pass', $4, 'completed', 'accepted', now())",
    )
    .bind(Uuid::new_v4())
    .bind(user)
    .bind(problem_slug)
    .bind(vec![9u8; 32])
    .execute(pool)
    .await
    .expect("solved submission");
}

async fn address_for(pool: PgPool, fixture: &Fixture) -> std::net::SocketAddr {
    let limiter = Limiter::Memory(Arc::new(MemoryLimiter::default()));
    let state = AppState::new(pool, config_for(&fixture.dir), limiter)
        .with_user_resolver(BearerTestResolver);
    serve(counterspell_server::build_router(state)).await
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_due_reviews_are_owned_ordered_bounded_and_sibling_scoped(pool: PgPool) {
    insert_published_fixture(&pool, "sibling-seed", 1)
        .await
        .expect("sibling fixture");
    insert_published_fixture(&pool, "fallback-seed", 1)
        .await
        .expect("fallback fixture");
    let alice = make_user(&pool, 701).await;
    let bob = make_user(&pool, 702).await;

    for index in 0..22 {
        let concept = format!("due-{index:02}");
        add_concept(&pool, &concept, &format!("Due {index}")).await;
        let due = format!("2020-01-01T00:{index:02}:00Z");
        add_card(&pool, alice, &concept, &due, i64::from(index)).await;
    }
    tag(&pool, "sibling-seed", "due-00").await;
    add_card(&pool, bob, "due-00", "2019-12-31T23:59:00Z", 99).await;

    let fixture = Fixture::new();
    let address = address_for(pool, &fixture).await;
    let reply = get(
        address,
        "/api/reviews/due?user_id=702",
        &[("authorization", "Bearer test-user-701")],
    )
    .await;

    assert_eq!(reply.status, 200, "body: {}", reply.body);
    let json: Value = serde_json::from_str(&reply.body).expect("json body");
    let reviews = json["reviews"].as_array().expect("reviews array");
    assert_eq!(reviews.len(), 20, "due page is bounded");
    assert_eq!(reviews[0]["concept"]["slug"], "due-00");
    assert_eq!(reviews[0]["card"]["version"], 0);
    assert_eq!(reviews[0]["challenge"]["kind"], "sibling");
    assert_eq!(reviews[0]["challenge"]["slug"], "sibling-seed");
    assert_eq!(reviews[1]["concept"]["slug"], "due-01");
    assert_eq!(reviews[1]["challenge"]["kind"], "normal");
    assert_eq!(reviews[1]["challenge"]["slug"], "fallback-seed");
    assert!(
        !reply.body.contains("\"version\":99"),
        "bob's card leaked through a browser user selector: {}",
        reply.body
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_due_reviews_skip_solved_and_unpublished_siblings_for_the_caller(pool: PgPool) {
    insert_published_fixture(&pool, "solved-sibling", 1)
        .await
        .expect("solved sibling");
    insert_static_fixture(&pool, "unpublished-sibling", 1)
        .await
        .expect("unpublished sibling");
    insert_published_fixture(&pool, "fallback-normal", 1)
        .await
        .expect("normal fallback");
    add_concept(&pool, "review-concept", "Review concept").await;
    tag(&pool, "solved-sibling", "review-concept").await;
    tag(&pool, "unpublished-sibling", "review-concept").await;
    let learner = make_user(&pool, 703).await;
    add_solved(&pool, learner, "solved-sibling").await;
    add_card(&pool, learner, "review-concept", "2020-01-01T00:00:00Z", 7).await;

    let fixture = Fixture::new();
    let address = address_for(pool, &fixture).await;
    let reply = get(
        address,
        "/api/reviews/due",
        &[("authorization", "Bearer test-user-703")],
    )
    .await;

    assert_eq!(reply.status, 200, "body: {}", reply.body);
    let json: Value = serde_json::from_str(&reply.body).expect("json body");
    let challenge = &json["reviews"][0]["challenge"];
    assert_eq!(challenge["kind"], "normal");
    assert_eq!(challenge["slug"], "fallback-normal");
    assert!(!reply.body.contains("unpublished-sibling"));
    assert!(!reply.body.contains("solved-sibling"));
}

async fn card_state(pool: &PgPool, user: Uuid, concept: &str) -> (String, i64) {
    sqlx::query_as(
        "SELECT state, version FROM concept_cards WHERE user_id = $1 AND concept_id = $2",
    )
    .bind(user)
    .bind(concept)
    .fetch_one(pool)
    .await
    .expect("card row")
}

async fn history_count(pool: &PgPool, user: Uuid, concept: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT count(*) FROM concept_review_history WHERE user_id = $1 AND concept_id = $2",
    )
    .bind(user)
    .bind(concept)
    .fetch_one(pool)
    .await
    .expect("history count")
}

fn review_headers(user: u128) -> [(&'static str, String); 2] {
    [
        ("authorization", format!("Bearer test-user-{user}")),
        ("content-type", "application/json".to_owned()),
    ]
}

fn review_header_refs<'a>(
    headers: &'a [(&'static str, String); 2],
) -> [(&'static str, &'a str); 2] {
    [
        (headers[0].0, headers[0].1.as_str()),
        (headers[1].0, headers[1].1.as_str()),
    ]
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_review_write_creates_card_history_and_sibling_atomically(pool: PgPool) {
    insert_published_fixture(&pool, "write-sibling", 1)
        .await
        .expect("sibling fixture");
    add_concept(&pool, "atomic-review", "Atomic review").await;
    tag(&pool, "write-sibling", "atomic-review").await;
    let learner = make_user(&pool, 704).await;
    let fixture = Fixture::new();
    let address = address_for(pool.clone(), &fixture).await;

    let header_values = review_headers(704);
    let headers = review_header_refs(&header_values);
    let reply = post(
        address,
        "/api/reviews/atomic-review",
        &headers,
        r#"{"grade":"good","version":0}"#,
    )
    .await;

    assert_eq!(reply.status, 200, "body: {}", reply.body);
    let json: Value = serde_json::from_str(&reply.body).expect("json body");
    assert_eq!(json["concept"]["slug"], "atomic-review");
    assert_eq!(json["card"]["version"], 1);
    assert_eq!(json["challenge"]["kind"], "sibling");
    assert_eq!(json["challenge"]["slug"], "write-sibling");
    assert_eq!(
        card_state(&pool, learner, "atomic-review").await,
        ("review".to_owned(), 1)
    );
    assert_eq!(history_count(&pool, learner, "atomic-review").await, 1);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_stale_review_write_conflicts_without_card_or_history_changes(pool: PgPool) {
    insert_published_fixture(&pool, "conflict-sibling", 1)
        .await
        .expect("sibling fixture");
    add_concept(&pool, "conflict-review", "Conflict review").await;
    tag(&pool, "conflict-sibling", "conflict-review").await;
    let learner = make_user(&pool, 705).await;
    add_card(&pool, learner, "conflict-review", "2020-01-01T00:00:00Z", 3).await;
    let fixture = Fixture::new();
    let address = address_for(pool.clone(), &fixture).await;
    let header_values = review_headers(705);
    let headers = review_header_refs(&header_values);

    let stale = post(
        address,
        "/api/reviews/conflict-review",
        &headers,
        r#"{"grade":"easy","version":2}"#,
    )
    .await;
    assert_eq!(stale.status, 409, "body: {}", stale.body);
    assert_eq!(
        card_state(&pool, learner, "conflict-review").await,
        ("review".to_owned(), 3)
    );
    assert_eq!(history_count(&pool, learner, "conflict-review").await, 0);

    let current = post(
        address,
        "/api/reviews/conflict-review",
        &headers,
        r#"{"grade":"easy","version":3}"#,
    )
    .await;
    assert_eq!(current.status, 200, "body: {}", current.body);
    assert_eq!(
        card_state(&pool, learner, "conflict-review").await,
        ("review".to_owned(), 4)
    );
    assert_eq!(history_count(&pool, learner, "conflict-review").await, 1);

    let retry = post(
        address,
        "/api/reviews/conflict-review",
        &headers,
        r#"{"grade":"easy","version":3}"#,
    )
    .await;
    assert_eq!(retry.status, 409, "body: {}", retry.body);
    assert_eq!(history_count(&pool, learner, "conflict-review").await, 1);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_review_write_rejects_user_selector_and_bad_input(pool: PgPool) {
    add_concept(&pool, "guarded-review", "Guarded review").await;
    let _learner = make_user(&pool, 706).await;
    let fixture = Fixture::new();
    let address = address_for(pool, &fixture).await;
    let header_values = review_headers(706);
    let headers = review_header_refs(&header_values);

    let selector = post(
        address,
        "/api/reviews/guarded-review",
        &headers,
        r#"{"grade":"good","user_id":"00000000-0000-0000-0000-000000000001"}"#,
    )
    .await;
    assert_eq!(selector.status, 400);
    assert!(selector.body.contains("unexpected_user_id"));

    let grade = post(
        address,
        "/api/reviews/guarded-review",
        &headers,
        r#"{"grade":"sort-of"}"#,
    )
    .await;
    assert_eq!(grade.status, 400);
    assert!(grade.body.contains("invalid_grade"));

    let unknown = post(
        address,
        "/api/reviews/missing-concept",
        &headers,
        r#"{"grade":"good"}"#,
    )
    .await;
    assert_eq!(unknown.status, 404);
    assert!(unknown.body.contains("unknown_concept"));
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_scheduler_failure_rolls_back_card_and_history(pool: PgPool) {
    add_concept(&pool, "broken-review", "Broken review").await;
    let learner = make_user(&pool, 707).await;
    sqlx::query(
        "INSERT INTO concept_cards
         (user_id, concept_id, stability, stability_fast, difficulty, reps, lapses, state, due, last_review, version)
         VALUES ($1, $2, 1, 1, 1, 0, 0, 'unsupported', now() - interval '1 day', now(), 0)",
    )
    .bind(learner)
    .bind("broken-review")
    .execute(&pool)
    .await
    .expect("unsupported card fixture");
    let fixture = Fixture::new();
    let address = address_for(pool.clone(), &fixture).await;

    let header_values = review_headers(707);
    let headers = review_header_refs(&header_values);
    let reply = post(
        address,
        "/api/reviews/broken-review",
        &headers,
        r#"{"grade":"good","version":0}"#,
    )
    .await;

    assert_eq!(reply.status, 500, "body: {}", reply.body);
    assert!(reply.body.contains("scheduler_invariant_failed"));
    assert_eq!(
        card_state(&pool, learner, "broken-review").await,
        ("unsupported".to_owned(), 0)
    );
    assert_eq!(history_count(&pool, learner, "broken-review").await, 0);
}
