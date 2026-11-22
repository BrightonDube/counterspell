//! Database tests for the dashboard read statement (dashboard-fsrs T03, DASH-2 to DASH-7).
//!
//! The statement must decode against the real catalogue and run tables and return only the owner's
//! data. Problems come from the judge's catalogue test kit, which writes through the production
//! triggers. Timestamps are fixed so the streak and the ordering are deterministic.
//!
//! Run with `DATABASE_URL=<throwaway postgres 17> cargo test -p counterspell-server --test pg_dashboard_query -- --ignored pg_`.

use counterspell_common::ids::UserId;
use counterspell_judge::catalogue::testkit::insert_published_fixture;
use counterspell_server::dashboard::{query::read_dashboard, types::DashboardResponse};
use serde_json::{json, Value};
use sqlx::PgPool;
use time::{Date, Month, OffsetDateTime};
use uuid::Uuid;

/// The UTC instant the fixtures are built around. The streak and the due-review cutoff
/// are measured against it.
fn now() -> OffsetDateTime {
    Date::from_calendar_date(2026, Month::November, 15)
        .expect("valid date")
        .with_hms(12, 0, 0)
        .expect("valid time")
        .assume_utc()
}

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

/// Saves a completed run with a fixed finish and creation time, both as UTC timestamps.
async fn save_completed(pool: &PgPool, user: UserId, problem_slug: &str, verdict: &str, at: &str) {
    sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, status, \
         verdict, passed, total, score, created_at, finished_at) \
         VALUES ($1, $2, $3, 'python', 'submit', 'pass', $4, 'completed', $5, 1, 1, 1, $6::timestamptz, $6::timestamptz)",
    )
    .bind(Uuid::new_v4())
    .bind(user.as_uuid())
    .bind(problem_slug)
    .bind(vec![9u8; 32])
    .bind(verdict)
    .bind(at)
    .execute(pool)
    .await
    .expect("completed submission row");
}

/// Saves a completed sample-only Run (`mode = 'run'`) with the given verdict.
async fn save_completed_run_mode(pool: &PgPool, user: UserId, problem_slug: &str, verdict: &str) {
    sqlx::query(
        "INSERT INTO submissions (id, user_id, problem_slug, lang, mode, code, code_sha256, status, \
         verdict, passed, total, score, created_at, finished_at) \
         VALUES ($1, $2, $3, 'python', 'run', 'pass', $4, 'completed', $5, 1, 1, 1, \
                 '2026-11-14T10:00:00Z'::timestamptz, '2026-11-14T10:00:00Z'::timestamptz)",
    )
    .bind(Uuid::new_v4())
    .bind(user.as_uuid())
    .bind(problem_slug)
    .bind(vec![9u8; 32])
    .bind(verdict)
    .execute(pool)
    .await
    .expect("completed run row");
}

fn to_json(response: &DashboardResponse) -> Value {
    serde_json::to_value(response).expect("response serialises")
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_new_learner_gets_the_stable_empty_dashboard(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("published problem");
    let learner = make_user(&pool, 101).await;

    let response = read_dashboard(&pool, learner, now())
        .await
        .expect("dashboard reads for a learner with no runs");

    assert_eq!(
        to_json(&response),
        json!({
            "solved": {"total": 0, "by_difficulty": {"easy": 0, "medium": 0, "hard": 0}},
            "streak": {"current_days": 0, "active_today": false, "last_activity_date": null},
            "weak_concepts": [],
            "recent_submissions": [],
            "suggested_challenges": [{
                "slug": "two-sum",
                "title": "Two Sum",
                "difficulty": "easy",
                "concepts": [{"slug": "hash-maps", "name": "Hash maps"}],
                "reason": "continue_track"
            }],
            "review_summary": null,
            "activity_days": []
        })
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_read_maps_one_learners_runs_and_leaves_the_other_learner_untouched(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("two-sum");
    insert_published_fixture(&pool, "lookup-pairs", 1)
        .await
        .expect("lookup-pairs");
    let alice = make_user(&pool, 201).await;
    let bob = make_user(&pool, 202).await;
    save_completed(&pool, alice, "two-sum", "accepted", "2026-11-15T10:00:00Z").await;
    save_completed(
        &pool,
        alice,
        "lookup-pairs",
        "wrong_answer",
        "2026-11-14T09:00:00Z",
    )
    .await;

    let response = read_dashboard(&pool, alice, now())
        .await
        .expect("alice's dashboard reads");

    assert_eq!(response.solved.total, 1);
    assert_eq!(response.solved.by_difficulty.easy, 1);
    assert_eq!(response.streak.current_days, 2);
    assert!(response.streak.active_today);
    // test-change: CP-4 (dashboard returns real UTC activity days for the signed-in learner)
    assert_eq!(response.activity_days, ["2026-11-15", "2026-11-14"]);
    assert_eq!(
        response.streak.last_activity_date.as_deref(),
        Some("2026-11-15")
    );
    assert_eq!(response.weak_concepts.len(), 1);
    assert_eq!(response.weak_concepts[0].slug, "hash-maps");
    assert_eq!(response.weak_concepts[0].failure_count, 1);
    let recent: Vec<&str> = response
        .recent_submissions
        .iter()
        .map(|run| run.problem.slug.as_str())
        .collect();
    assert_eq!(recent, ["two-sum", "lookup-pairs"]);
    let suggested: Vec<&str> = response
        .suggested_challenges
        .iter()
        .map(|challenge| challenge.slug.as_str())
        .collect();
    assert_eq!(suggested, ["lookup-pairs"], "alice has solved two-sum");

    let other = read_dashboard(&pool, bob, now())
        .await
        .expect("bob's dashboard reads");
    assert_eq!(
        other.solved.total, 0,
        "alice's accepted run must not count for bob"
    );
    assert_eq!(other.recent_submissions.len(), 0);
    assert_eq!(other.weak_concepts.len(), 0);
    // test-change: CP-4 (activity days are owner-scoped)
    assert_eq!(other.activity_days, Vec::<String>::new());
    let other_suggested: Vec<&str> = other
        .suggested_challenges
        .iter()
        .map(|challenge| challenge.slug.as_str())
        .collect();
    assert!(
        other_suggested.contains(&"two-sum"),
        "two-sum is unsolved for bob, so it must still be suggested"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_repeated_accepted_runs_count_one_solved_problem(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("two-sum");
    let learner = make_user(&pool, 301).await;
    save_completed(
        &pool,
        learner,
        "two-sum",
        "accepted",
        "2026-11-14T10:00:00Z",
    )
    .await;
    save_completed(
        &pool,
        learner,
        "two-sum",
        "accepted",
        "2026-11-15T10:00:00Z",
    )
    .await;

    let response = read_dashboard(&pool, learner, now())
        .await
        .expect("dashboard reads");

    assert_eq!(
        response.solved.total, 1,
        "one problem, however many accepted runs"
    );
    assert_eq!(response.solved.by_difficulty.easy, 1);
}

/// T39 finding 1: a sample-only Run that passes proves nothing about the hidden tests, so it must
/// not count as solved. Only an accepted Submit does (DASH-4, judge-sandbox T39).
#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_an_accepted_sample_run_is_not_a_solved_problem(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("two-sum");
    let learner = make_user(&pool, 302).await;
    save_completed_run_mode(&pool, learner, "two-sum", "accepted").await;

    let response = read_dashboard(&pool, learner, now())
        .await
        .expect("dashboard reads");

    assert_eq!(response.solved.total, 0, "a sample Run is not a solve");
    assert_eq!(response.solved.by_difficulty.easy, 0);
}

/// T17: a due review card owned by the learner produces the additive `review_summary`; another
/// learner's card and a sibling already solved by the owner are excluded.
#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_a_due_card_produces_the_review_summary_without_changing_core_fields(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("sibling problem");
    insert_published_fixture(&pool, "three-sum", 1)
        .await
        .expect("fallback problem");
    let learner = make_user(&pool, 401).await;
    let other = make_user(&pool, 402).await;

    sqlx::query("INSERT INTO concepts (id, name) VALUES ('dp', 'Dynamic programming') ON CONFLICT DO NOTHING")
        .execute(&pool)
        .await
        .expect("concept");
    sqlx::query("INSERT INTO problem_concepts (problem_slug, concept_id) VALUES ('two-sum', 'dp') ON CONFLICT DO NOTHING")
        .execute(&pool)
        .await
        .expect("tag");
    sqlx::query(
        "INSERT INTO concept_cards
         (user_id, concept_id, stability, stability_fast, difficulty, reps, lapses, state, due, last_review, version)
         VALUES ($1, 'dp', 1, 1, 1, 1, 0, 'review', '2026-11-15T08:00:00Z'::timestamptz,
                 '2026-11-14T08:00:00Z'::timestamptz, 7)",
    )
    .bind(learner.as_uuid())
    .execute(&pool)
    .await
    .expect("due card");
    // Another learner's due card must not leak into this dashboard.
    sqlx::query(
        "INSERT INTO concept_cards
         (user_id, concept_id, stability, stability_fast, difficulty, reps, lapses, state, due, last_review, version)
         VALUES ($1, 'dp', 1, 1, 1, 1, 0, 'review', '2020-01-01T00:00:00Z'::timestamptz,
                 '2019-12-31T00:00:00Z'::timestamptz, 99)",
    )
    .bind(other.as_uuid())
    .execute(&pool)
    .await
    .expect("other user's card");

    let response = read_dashboard(&pool, learner, now())
        .await
        .expect("dashboard reads");

    let summary = response
        .review_summary
        .as_ref()
        .expect("a due card produces a summary");
    assert_eq!(summary.due_count, 1);
    assert_eq!(summary.next_review.concept.slug, "dp");
    assert_eq!(summary.next_review.card.version, 7);
    assert_eq!(
        summary
            .next_review
            .challenge
            .as_ref()
            .map(|challenge| (challenge.slug.as_str(), challenge.kind)),
        Some((
            "two-sum",
            counterspell_server::reviews::types::ReviewChallengeKind::Sibling
        ))
    );
    assert_eq!(
        response.recent_submissions.len(),
        0,
        "core fields are untouched by the additive field"
    );
}

/// T17: when the only sibling is already solved, the summary offers the normal fallback and
/// labels it `normal`; with no due cards at all the summary is `null`.
#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_a_solved_sibling_falls_back_to_a_normal_challenge(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("sibling problem");
    insert_published_fixture(&pool, "three-sum", 1)
        .await
        .expect("fallback problem");
    let learner = make_user(&pool, 501).await;

    sqlx::query("INSERT INTO concepts (id, name) VALUES ('dp', 'Dynamic programming') ON CONFLICT DO NOTHING")
        .execute(&pool)
        .await
        .expect("concept");
    sqlx::query("INSERT INTO problem_concepts (problem_slug, concept_id) VALUES ('two-sum', 'dp') ON CONFLICT DO NOTHING")
        .execute(&pool)
        .await
        .expect("tag");
    sqlx::query(
        "INSERT INTO concept_cards
         (user_id, concept_id, stability, stability_fast, difficulty, reps, lapses, state, due, last_review, version)
         VALUES ($1, 'dp', 1, 1, 1, 1, 0, 'review', '2026-11-15T08:00:00Z'::timestamptz,
                 '2026-11-14T08:00:00Z'::timestamptz, 3)",
    )
    .bind(learner.as_uuid())
    .execute(&pool)
    .await
    .expect("due card");
    save_completed(
        &pool,
        learner,
        "two-sum",
        "accepted",
        "2026-11-14T10:00:00Z",
    )
    .await;

    let response = read_dashboard(&pool, learner, now())
        .await
        .expect("dashboard reads");

    let summary = response
        .review_summary
        .expect("a due card produces a summary");
    let challenge = summary.next_review.challenge.expect("fallback exists");
    assert_eq!(
        (challenge.slug.as_str(), challenge.kind),
        (
            "three-sum",
            counterspell_server::reviews::types::ReviewChallengeKind::Normal
        )
    );
}
