//! Database isolation and aggregate tests for the dashboard (dashboard-fsrs T06, DASH-2 to DASH-8).
//!
//! Two learners each get their own canary concept and their own runs. A learner's dashboard must not
//! contain the other learner's canary, run ids or solved problems. The published fixtures carry
//! hidden-test canaries from the judge's test kit, and none of them may appear in a response.
//! Duplicate accepted runs counting once is covered by `pg_dashboard_query`.
//!
//! Run with `DATABASE_URL=<throwaway postgres 17> cargo test -p counterspell-server --test pg_dashboard_privacy -- --ignored pg_`.

use counterspell_common::ids::UserId;
use counterspell_judge::catalogue::testkit::{
    insert_published_fixture, insert_static_fixture, CANARY_EXPECTED, CANARY_GHOST, CANARY_HIDDEN,
    CANARY_REFERENCE,
};
use counterspell_server::dashboard::{
    query::read_dashboard,
    types::{DashboardResponse, SuggestionReason},
};
use sqlx::PgPool;
use time::{Date, Month, OffsetDateTime};
use uuid::Uuid;

/// The UTC instant the fixtures are measured against.
fn now() -> OffsetDateTime {
    Date::from_calendar_date(2026, Month::November, 15)
        .expect("valid date")
        .with_hms(12, 0, 0)
        .expect("valid time")
        .assume_utc()
}

/// A published problem that no fixture solves, used by the unpublished-canary test.
const UNPUBLISHED_CANARY: &str = "aa-unpublished-canary";

async fn make_user(pool: &PgPool, n: u128) -> UserId {
    let id = Uuid::from_u128(n);
    sqlx::query(
        "INSERT INTO users (id, handle, display_name) VALUES ($1, $2, 'Learner') ON CONFLICT DO NOTHING",
    )
    .bind(id)
    .bind(format!("learner{n}"))
    .execute(pool)
    .await
    .expect("user row");
    UserId::from_uuid(id)
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

/// Saves one run for `user` and returns its id. A finished status gets `at` as its finish time.
/// The status and failure reason follow the run-lifecycle CHECK constraints.
async fn insert_run(
    pool: &PgPool,
    user: UserId,
    problem_slug: &str,
    status: &str,
    verdict: Option<&str>,
    failure_reason: Option<&str>,
    at: &str,
) -> Uuid {
    let id = Uuid::new_v4();
    let finished = matches!(
        status,
        "completed" | "failed" | "expired" | "internal_error"
    )
    .then_some(at);
    sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, status, \
         verdict, failure_reason, passed, total, score, created_at, finished_at) \
         VALUES ($1, $2, $3, 'python', 'submit', 'pass', $4, $5, $6, $7, 1, 1, 1, $8::timestamptz, $9::timestamptz)",
    )
    .bind(id)
    .bind(user.as_uuid())
    .bind(problem_slug)
    .bind(vec![9u8; 32])
    .bind(status)
    .bind(verdict)
    .bind(failure_reason)
    .bind(at)
    .bind(finished)
    .execute(pool)
    .await
    .expect("submission row");
    id
}

fn response_text(response: &DashboardResponse) -> String {
    serde_json::to_string(response).expect("response serialises")
}

fn suggested(response: &DashboardResponse) -> Vec<(&str, SuggestionReason)> {
    response
        .suggested_challenges
        .iter()
        .map(|challenge| (challenge.slug.as_str(), challenge.reason))
        .collect()
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_each_learner_sees_only_their_own_progress_and_suggestions(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("two-sum");
    insert_published_fixture(&pool, "lookup-pairs", 1)
        .await
        .expect("lookup-pairs");
    add_concept(&pool, "alice-canary", "Alice canary concept").await;
    add_concept(&pool, "bob-canary", "Bob canary concept").await;
    tag(&pool, "two-sum", "alice-canary").await;
    tag(&pool, "lookup-pairs", "bob-canary").await;
    let alice = make_user(&pool, 601).await;
    let bob = make_user(&pool, 602).await;

    insert_run(
        &pool,
        alice,
        "two-sum",
        "completed",
        Some("wrong_answer"),
        None,
        "2026-11-14T10:00:00Z",
    )
    .await;
    let bob_solved = insert_run(
        &pool,
        bob,
        "lookup-pairs",
        "completed",
        Some("accepted"),
        None,
        "2026-11-15T09:00:00Z",
    )
    .await;
    let bob_failed = insert_run(
        &pool,
        bob,
        "two-sum",
        "completed",
        Some("wrong_answer"),
        None,
        "2026-11-15T08:00:00Z",
    )
    .await;

    // Concept names and problem slugs in suggestions are public catalogue data, so the canary checks
    // below read only the private sections: solved counts, weak concepts and recent runs.
    let alice_view = read_dashboard(&pool, alice, now())
        .await
        .expect("alice reads");
    let alice_recent =
        serde_json::to_string(&alice_view.recent_submissions).expect("recent serialises");

    assert_eq!(
        alice_view.solved.total, 0,
        "bob's accepted run must not count for alice"
    );
    let weak: Vec<(&str, u32)> = alice_view
        .weak_concepts
        .iter()
        .map(|c| (c.slug.as_str(), c.failure_count))
        .collect();
    assert_eq!(
        weak,
        [("alice-canary", 1), ("hash-maps", 1)],
        "each count is alice's own failure; bob's failure on two-sum would make it 2"
    );
    assert!(
        !alice_recent.contains(&bob_solved.to_string()),
        "bob's run id leaked: {alice_recent}"
    );
    assert!(
        !alice_recent.contains(&bob_failed.to_string()),
        "bob's run id leaked: {alice_recent}"
    );
    assert_eq!(
        suggested(&alice_view),
        [
            ("two-sum", SuggestionReason::PracticeWeakConcept),
            ("lookup-pairs", SuggestionReason::PracticeWeakConcept)
        ],
        "alice has not solved either problem; two-sum shares both weak concepts"
    );

    let bob_view = read_dashboard(&pool, bob, now()).await.expect("bob reads");
    let bob_text = response_text(&bob_view);

    assert_eq!(bob_view.solved.total, 1);
    let bob_weak: Vec<(&str, u32)> = bob_view
        .weak_concepts
        .iter()
        .map(|c| (c.slug.as_str(), c.failure_count))
        .collect();
    assert_eq!(
        bob_weak,
        [("alice-canary", 1), ("hash-maps", 1)],
        "each count is bob's own failure on two-sum; alice's failure on it would make it 2"
    );
    let bob_recent: Vec<String> = bob_view
        .recent_submissions
        .iter()
        .map(|run| run.id.to_string())
        .collect();
    assert_eq!(
        bob_recent,
        [bob_solved.to_string(), bob_failed.to_string()],
        "newest first"
    );
    assert!(
        !bob_text.contains(&alice.as_uuid().to_string()),
        "alice's user id leaked: {bob_text}"
    );
    assert_eq!(
        suggested(&bob_view),
        [("two-sum", SuggestionReason::PracticeWeakConcept)]
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_hidden_test_data_never_reaches_any_dashboard(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("two-sum");
    let learner = make_user(&pool, 603).await;
    insert_run(
        &pool,
        learner,
        "two-sum",
        "completed",
        Some("accepted"),
        None,
        "2026-11-15T10:00:00Z",
    )
    .await;

    let response = read_dashboard(&pool, learner, now())
        .await
        .expect("dashboard reads");
    let text = response_text(&response);

    for canary in [
        CANARY_HIDDEN,
        CANARY_EXPECTED,
        CANARY_REFERENCE,
        CANARY_GHOST,
    ] {
        assert!(
            !text.contains(canary),
            "hidden canary {canary} reached the dashboard: {text}"
        );
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_queued_expired_and_internal_error_runs_are_not_solved_or_activity(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("two-sum");
    insert_published_fixture(&pool, "lookup-pairs", 1)
        .await
        .expect("lookup-pairs");
    let learner = make_user(&pool, 604).await;
    insert_run(
        &pool,
        learner,
        "two-sum",
        "queued",
        None,
        None,
        "2026-11-15T10:00:00Z",
    )
    .await;
    insert_run(
        &pool,
        learner,
        "lookup-pairs",
        "expired",
        None,
        Some("expired"),
        "2026-11-15T10:01:00Z",
    )
    .await;
    insert_run(
        &pool,
        learner,
        "two-sum",
        "internal_error",
        None,
        Some("retries_exhausted"),
        "2026-11-15T10:02:00Z",
    )
    .await;

    let response = read_dashboard(&pool, learner, now())
        .await
        .expect("dashboard reads");

    assert_eq!(response.solved.total, 0);
    assert_eq!(
        response.streak.current_days, 0,
        "no completed run means no streak"
    );
    assert_eq!(response.streak.last_activity_date, None);
    assert!(
        response.weak_concepts.is_empty(),
        "an unfinished run is not a failed attempt"
    );
    assert_eq!(
        response.recent_submissions.len(),
        3,
        "the learner still sees their own runs"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_recent_submissions_are_the_ten_newest_runs_in_order(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("two-sum");
    let learner = make_user(&pool, 605).await;
    let mut ids = Vec::new();
    for minute in 0..12 {
        let at = format!("2026-11-15T10:{minute:02}:00Z");
        ids.push(
            insert_run(
                &pool,
                learner,
                "two-sum",
                "completed",
                Some("wrong_answer"),
                None,
                &at,
            )
            .await,
        );
    }

    let response = read_dashboard(&pool, learner, now())
        .await
        .expect("dashboard reads");

    let returned: Vec<String> = response
        .recent_submissions
        .iter()
        .map(|run| run.id.to_string())
        .collect();
    let expected: Vec<String> = ids[2..].iter().rev().map(Uuid::to_string).collect();
    assert_eq!(returned, expected, "ten newest, newest first");
    assert!(
        !response_text(&response).contains(&ids[0].to_string()),
        "the oldest run is past the cap"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_suggestions_skip_solved_and_unpublished_problems_and_stop_at_three(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("two-sum");
    insert_published_fixture(&pool, "lookup-pairs", 1)
        .await
        .expect("lookup-pairs");
    insert_published_fixture(&pool, "pairs-two", 1)
        .await
        .expect("pairs-two");
    insert_published_fixture(&pool, "pairs-three", 1)
        .await
        .expect("pairs-three");
    insert_published_fixture(&pool, "pairs-four", 1)
        .await
        .expect("pairs-four");
    insert_static_fixture(&pool, UNPUBLISHED_CANARY, 1)
        .await
        .expect("unpublished canary");
    let learner = make_user(&pool, 606).await;
    insert_run(
        &pool,
        learner,
        "two-sum",
        "completed",
        Some("accepted"),
        None,
        "2026-11-15T10:00:00Z",
    )
    .await;

    let response = read_dashboard(&pool, learner, now())
        .await
        .expect("dashboard reads");

    let slugs: Vec<&str> = response
        .suggested_challenges
        .iter()
        .map(|c| c.slug.as_str())
        .collect();
    assert_eq!(slugs, ["lookup-pairs", "pairs-four", "pairs-three"]);
    assert!(
        !response_text(&response).contains(UNPUBLISHED_CANARY),
        "an unpublished problem was suggested"
    );
}
