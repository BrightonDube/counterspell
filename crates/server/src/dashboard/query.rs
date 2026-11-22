//! The dashboard read model: one PostgreSQL statement per request (dashboard-fsrs T03).
//!
//! The statement takes the owner's id as its only user selector. Every user-scoped step starts from
//! the owner's completed runs, so no aggregate is built over other learners' rows. The statement
//! returns one row: a JSON object with the solved counts, weak concepts, recent runs and candidate
//! problems, and the UTC dates of the owner's completed runs for the streak.

use counterspell_common::ids::UserId;
use serde::Deserialize;
use serde_json::Value;
use sqlx::PgPool;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

use crate::reviews::{
    types::{DueCard, DueReview, ReviewChallenge, ReviewChallengeKind},
    CardState,
};

use super::recommendations::{rank_suggestions, Candidate, WeakSignal};
use super::streak::streak_from_dates;
use super::types::{
    ConceptRef, DashboardResponse, Difficulty, DifficultyCounts, ProblemSummary, RecentSubmission,
    ReviewSummary, SolvedSummary, StreakSummary, WeakConcept,
};

/// The most suggested problems the dashboard shows.
const SUGGESTION_LIMIT: usize = 3;

/// The most weak concepts the response lists. The suggestions still rank against every weak concept.
const WEAK_CONCEPT_LIMIT: usize = 10;

/// One statement for the whole dashboard. It returns one row: the JSON object and the UTC dates of
/// the owner's completed runs. Only completed runs count as activity, and only accepted ones solve a
/// problem. Only published problems can be solved, suggested or listed as candidates.
///
/// Weak concepts and recent runs are the owner's own history, so they also include problems that
/// have since been unpublished. Their titles and concept names are public catalogue data, and no
/// other learner's rows are read.
const DASHBOARD_SQL: &str = r#"
WITH owned_completed AS (
    SELECT s.id, s.problem_slug, s.verdict, s.finished_at, s.mode, s.ghost_input
    FROM submissions s
    WHERE s.user_id = $1 AND s.status = 'completed'
),
solved AS (
    SELECT DISTINCT p.slug, p.difficulty
    FROM owned_completed oc
    JOIN problems p ON p.slug = oc.problem_slug AND p.publication_state = 'published'
    WHERE oc.verdict = 'accepted' AND oc.mode = 'submit' AND oc.ghost_input IS NULL
),
weak AS (
    SELECT c.id AS slug, c.name,
           count(DISTINCT oc.id) AS failure_count,
           max(oc.finished_at) AS last_failed_at
    FROM owned_completed oc
    JOIN problem_concepts pc ON pc.problem_slug = oc.problem_slug
    JOIN concepts c ON c.id = pc.concept_id
    WHERE oc.verdict <> 'accepted'
    GROUP BY c.id, c.name
),
recent AS (
    SELECT s.id, s.created_at, s.finished_at, s.lang, s.mode, s.status, s.verdict,
           s.passed, s.total, s.score, p.slug, p.title, p.difficulty
    FROM submissions s
    JOIN problems p ON p.slug = s.problem_slug
    WHERE s.user_id = $1
    ORDER BY s.created_at DESC, s.id DESC
    LIMIT 10
),
candidates AS (
    SELECT p.slug, p.title, p.difficulty,
           COALESCE(jsonb_agg(jsonb_build_object('slug', c.id, 'name', c.name)
                              ORDER BY c.id) FILTER (WHERE c.id IS NOT NULL), '[]'::jsonb) AS concepts,
           EXISTS (SELECT 1 FROM solved sv WHERE sv.slug = p.slug) AS solved
    FROM problems p
    LEFT JOIN problem_concepts pc ON pc.problem_slug = p.slug
    LEFT JOIN concepts c ON c.id = pc.concept_id
    WHERE p.publication_state = 'published'
    GROUP BY p.slug, p.title, p.difficulty
),
-- [stretch] T17: the owner's due review cards only, earliest first. An empty set makes
-- review_first empty and the review_summary scalar subquery NULL, so nothing is fabricated.
due_reviews AS (
    SELECT cc.concept_id, c.name AS concept_name, cc.state, cc.due, cc.version
    FROM concept_cards cc
    JOIN concepts c ON c.id = cc.concept_id
    WHERE cc.user_id = $1 AND cc.due <= $2
    ORDER BY cc.due ASC, cc.concept_id ASC
),
review_first AS (
    SELECT * FROM due_reviews ORDER BY due ASC, concept_id ASC LIMIT 1
),
review_sibling AS (
    SELECT p.slug, p.title, p.difficulty
    FROM review_first rf
    JOIN problem_concepts pc ON pc.concept_id = rf.concept_id
    JOIN problems p ON p.slug = pc.problem_slug AND p.publication_state = 'published'
    WHERE NOT EXISTS (SELECT 1 FROM solved sv WHERE sv.slug = p.slug)
    ORDER BY p.slug ASC LIMIT 1
),
review_fallback AS (
    SELECT p.slug, p.title, p.difficulty
    FROM problems p
    WHERE p.publication_state = 'published'
      AND NOT EXISTS (SELECT 1 FROM solved sv WHERE sv.slug = p.slug)
    ORDER BY p.slug ASC LIMIT 1
)
SELECT jsonb_build_object(
    'solved', (SELECT jsonb_build_object(
                   'total', count(*),
                   'easy', count(*) FILTER (WHERE difficulty = 'easy'),
                   'medium', count(*) FILTER (WHERE difficulty = 'medium'),
                   'hard', count(*) FILTER (WHERE difficulty = 'hard'))
               FROM solved),
    'weak_concepts', (SELECT COALESCE(jsonb_agg(jsonb_build_object(
                   'slug', slug,
                   'name', name,
                   'failure_count', failure_count,
                   'last_failed_at', to_char(last_failed_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'))
               ORDER BY failure_count DESC, last_failed_at DESC, slug ASC), '[]'::jsonb)
               FROM weak),
    'recent_submissions', (SELECT COALESCE(jsonb_agg(jsonb_build_object(
                   'id', id,
                   'slug', slug,
                   'title', title,
                   'difficulty', difficulty,
                   'lang', lang,
                   'mode', mode,
                   'status', status,
                   'verdict', verdict,
                   'passed', passed,
                   'total', total,
                   'score', score,
                   'created_at', to_char(created_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'),
                   'finished_at', to_char(finished_at AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'))
               ORDER BY created_at DESC, id DESC), '[]'::jsonb)
               FROM recent),
    'candidates', (SELECT COALESCE(jsonb_agg(jsonb_build_object(
                   'slug', slug,
                   'title', title,
                   'difficulty', difficulty,
                   'concepts', concepts,
                   'solved', solved)
               ORDER BY slug), '[]'::jsonb)
               FROM candidates),
    'review_summary', (SELECT jsonb_build_object(
                   'due_count', (SELECT count(*) FROM due_reviews),
                   'next_review', jsonb_build_object(
                       'concept', jsonb_build_object('slug', rf.concept_id, 'name', rf.concept_name),
                       'card', jsonb_build_object(
                           'state', rf.state,
                           'due', to_char(rf.due AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"'),
                           'version', rf.version),
                       'challenge', COALESCE(
                           (SELECT jsonb_build_object('kind', 'sibling', 'slug', slug,
                                                      'title', title, 'difficulty', difficulty)
                            FROM review_sibling),
                           (SELECT jsonb_build_object('kind', 'normal', 'slug', slug,
                                                      'title', title, 'difficulty', difficulty)
                            FROM review_fallback))))
               FROM review_first rf)
) AS dashboard,
ARRAY(SELECT DISTINCT (finished_at AT TIME ZONE 'UTC')::date
      FROM owned_completed
      ORDER BY 1 DESC) AS activity_days
"#;

/// Reads the dashboard for `owner` in one statement. `now` ends the streak at its UTC date and
/// decides which review cards are due.
///
/// # Errors
///
/// Returns the database error when the statement fails. Returns a decode error when the stored
/// values do not have the shape the dashboard expects, such as an unknown difficulty.
pub async fn read_dashboard(
    pool: &PgPool,
    owner: UserId,
    now: OffsetDateTime,
) -> Result<DashboardResponse, sqlx::Error> {
    let (dashboard, activity_days): (Value, Vec<Date>) = sqlx::query_as(DASHBOARD_SQL)
        .bind(owner.as_uuid())
        .bind(now)
        .fetch_one(pool)
        .await?;
    build_response(dashboard, &activity_days, now.date())
}

/// The statement's JSON object, before it is mapped to the public response.
#[derive(Deserialize)]
struct RawDashboard {
    solved: RawSolved,
    weak_concepts: Vec<RawWeakConcept>,
    recent_submissions: Vec<RawRecentRun>,
    candidates: Vec<RawCandidate>,
    review_summary: Option<RawReviewSummary>,
}

#[derive(Deserialize)]
struct RawSolved {
    total: u32,
    easy: u32,
    medium: u32,
    hard: u32,
}

#[derive(Deserialize)]
struct RawWeakConcept {
    slug: String,
    name: String,
    failure_count: u32,
    last_failed_at: String,
}

#[derive(Deserialize)]
struct RawRecentRun {
    id: Uuid,
    slug: String,
    title: String,
    difficulty: Difficulty,
    lang: String,
    mode: String,
    status: String,
    verdict: Option<String>,
    passed: Option<i32>,
    total: Option<i32>,
    score: Option<i32>,
    created_at: String,
    finished_at: Option<String>,
}

#[derive(Deserialize)]
struct RawCandidate {
    slug: String,
    title: String,
    difficulty: Difficulty,
    concepts: Vec<ConceptRef>,
    solved: bool,
}

/// The [stretch] review summary as the statement writes it. `next_review` is always present when
/// the summary exists; a missing one is a decode error, not a guessed review.
#[derive(Deserialize)]
struct RawReviewSummary {
    due_count: u32,
    next_review: RawDueReview,
}

#[derive(Deserialize)]
struct RawDueReview {
    concept: ConceptRef,
    card: RawDueCard,
    challenge: Option<RawReviewChallenge>,
}

#[derive(Deserialize)]
struct RawDueCard {
    state: String,
    due: String,
    version: i64,
}

#[derive(Deserialize)]
struct RawReviewChallenge {
    kind: ReviewChallengeKind,
    slug: String,
    title: String,
    difficulty: Difficulty,
}

/// Maps the statement's JSON and the activity dates to the public response.
///
/// This is a pure function. It takes no database handle, so the tests can use hand-written rows.
fn build_response(
    dashboard: Value,
    activity_days: &[Date],
    today: Date,
) -> Result<DashboardResponse, sqlx::Error> {
    let raw: RawDashboard =
        serde_json::from_value(dashboard).map_err(|error| sqlx::Error::Decode(Box::new(error)))?;

    let weak: Vec<WeakSignal> = raw
        .weak_concepts
        .iter()
        .map(|concept| WeakSignal {
            slug: concept.slug.clone(),
            failure_count: concept.failure_count,
        })
        .collect();
    // The statement keeps only published problems, so every candidate is published.
    let candidates: Vec<Candidate> = raw
        .candidates
        .into_iter()
        .map(|candidate| Candidate {
            slug: candidate.slug,
            title: candidate.title,
            difficulty: candidate.difficulty,
            concepts: candidate.concepts,
            published: true,
            solved: candidate.solved,
        })
        .collect();
    let streak = streak_from_dates(activity_days, today);

    Ok(DashboardResponse {
        solved: SolvedSummary {
            total: raw.solved.total,
            by_difficulty: DifficultyCounts {
                easy: raw.solved.easy,
                medium: raw.solved.medium,
                hard: raw.solved.hard,
            },
        },
        streak: StreakSummary {
            current_days: streak.current_days,
            active_today: streak.active_today,
            last_activity_date: streak.last_activity.map(iso_date),
        },
        weak_concepts: raw
            .weak_concepts
            .into_iter()
            .take(WEAK_CONCEPT_LIMIT)
            .map(|concept| WeakConcept {
                slug: concept.slug,
                name: concept.name,
                failure_count: concept.failure_count,
                last_failed_at: concept.last_failed_at,
            })
            .collect(),
        recent_submissions: raw
            .recent_submissions
            .into_iter()
            .map(recent_submission)
            .collect(),
        suggested_challenges: rank_suggestions(&candidates, &weak, SUGGESTION_LIMIT),
        review_summary: raw.review_summary.map(review_summary).transpose()?,
    })
}

/// Maps the statement's review JSON to the public summary. Unknown card states are decode errors.
fn review_summary(raw: RawReviewSummary) -> Result<ReviewSummary, sqlx::Error> {
    let next = raw.next_review;
    Ok(ReviewSummary {
        due_count: raw.due_count,
        next_review: DueReview {
            concept: next.concept,
            card: DueCard {
                state: CardState::parse(&next.card.state)
                    .map_err(|error| sqlx::Error::Decode(Box::new(error)))?,
                due: next.card.due,
                version: next.card.version,
            },
            challenge: next.challenge.map(|raw| ReviewChallenge {
                kind: raw.kind,
                slug: raw.slug,
                title: raw.title,
                difficulty: raw.difficulty,
            }),
        },
    })
}

fn recent_submission(run: RawRecentRun) -> RecentSubmission {
    RecentSubmission {
        id: run.id,
        problem: ProblemSummary {
            slug: run.slug,
            title: run.title,
            difficulty: run.difficulty,
        },
        lang: run.lang,
        mode: run.mode,
        status: run.status,
        verdict: run.verdict,
        passed: run.passed,
        total: run.total,
        score: run.score,
        created_at: run.created_at,
        finished_at: run.finished_at,
    }
}

/// Formats a UTC date as `YYYY-MM-DD`, the form the response uses for activity dates.
fn iso_date(date: Date) -> String {
    format!(
        "{:04}-{:02}-{:02}",
        date.year(),
        u8::from(date.month()),
        date.day()
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use time::{Date, Month};

    fn date(year: i32, month: Month, day: u8) -> Date {
        Date::from_calendar_date(year, month, day).expect("valid test date")
    }

    fn empty_read() -> Value {
        json!({
            "solved": {"total": 0, "easy": 0, "medium": 0, "hard": 0},
            "weak_concepts": [],
            "recent_submissions": [],
            "candidates": []
        })
    }

    #[test]
    fn an_empty_read_maps_to_the_stable_empty_shape() {
        let response = build_response(empty_read(), &[], date(2026, Month::November, 15))
            .expect("an empty read maps");

        assert_eq!(
            serde_json::to_value(&response).expect("serialise"),
            json!({
                "solved": {"total": 0, "by_difficulty": {"easy": 0, "medium": 0, "hard": 0}},
                "streak": {"current_days": 0, "active_today": false, "last_activity_date": null},
                "weak_concepts": [],
                "recent_submissions": [],
                "suggested_challenges": [],
                "review_summary": null
            })
        );
    }

    #[test]
    fn solved_counts_come_from_the_read_and_the_streak_from_utc_dates() {
        let mut read = empty_read();
        read["solved"] = json!({"total": 3, "easy": 1, "medium": 2, "hard": 0});
        let days = [
            date(2026, Month::November, 13),
            date(2026, Month::November, 14),
            date(2026, Month::November, 15),
        ];

        let response = build_response(read, &days, date(2026, Month::November, 15)).expect("maps");

        assert_eq!(response.solved.total, 3);
        assert_eq!(
            response.solved.by_difficulty,
            DifficultyCounts {
                easy: 1,
                medium: 2,
                hard: 0
            }
        );
        assert_eq!(
            response.streak,
            StreakSummary {
                current_days: 3,
                active_today: true,
                last_activity_date: Some("2026-11-15".to_owned()),
            }
        );
    }

    #[test]
    fn weak_concepts_keep_the_read_order_and_their_failure_counts() {
        let mut read = empty_read();
        read["weak_concepts"] = json!([
            {"slug": "hash-maps", "name": "Hash maps", "failure_count": 3,
             "last_failed_at": "2026-11-14T18:20:00Z"},
            {"slug": "loops", "name": "Loops", "failure_count": 1,
             "last_failed_at": "2026-11-10T09:00:00Z"}
        ]);

        let response = build_response(read, &[], date(2026, Month::November, 15)).expect("maps");

        let slugs: Vec<&str> = response
            .weak_concepts
            .iter()
            .map(|concept| concept.slug.as_str())
            .collect();
        assert_eq!(slugs, ["hash-maps", "loops"]);
        assert_eq!(response.weak_concepts[0].failure_count, 3);
        assert_eq!(
            response.weak_concepts[0].last_failed_at,
            "2026-11-14T18:20:00Z"
        );
    }

    #[test]
    fn suggestions_skip_solved_problems_and_keep_at_most_three() {
        let mut read = empty_read();
        read["candidates"] = json!([
            {"slug": "alpha", "title": "Alpha", "difficulty": "easy", "concepts": [], "solved": true},
            {"slug": "bravo", "title": "Bravo", "difficulty": "easy", "concepts": [], "solved": false},
            {"slug": "charlie", "title": "Charlie", "difficulty": "medium", "concepts": [], "solved": false},
            {"slug": "delta", "title": "Delta", "difficulty": "hard", "concepts": [], "solved": false},
            {"slug": "echo", "title": "Echo", "difficulty": "hard", "concepts": [], "solved": false}
        ]);

        let response = build_response(read, &[], date(2026, Month::November, 15)).expect("maps");

        let slugs: Vec<&str> = response
            .suggested_challenges
            .iter()
            .map(|challenge| challenge.slug.as_str())
            .collect();
        assert_eq!(slugs, ["bravo", "charlie", "delta"]);
    }

    #[test]
    fn an_unfinished_run_maps_with_null_verdict_and_finish_time() {
        let mut read = empty_read();
        read["recent_submissions"] = json!([{
            "id": "00000000-0000-0000-0000-00000000abcd",
            "slug": "two-sum", "title": "Two Sum", "difficulty": "easy",
            "lang": "python", "mode": "submit", "status": "queued", "verdict": null,
            "passed": null, "total": null, "score": null,
            "created_at": "2026-11-15T08:00:00Z", "finished_at": null
        }]);

        let response = build_response(read, &[], date(2026, Month::November, 15)).expect("maps");

        let recent = &response.recent_submissions[0];
        assert_eq!(recent.status, "queued");
        assert_eq!(recent.verdict, None);
        assert_eq!(recent.finished_at, None);
        assert_eq!(recent.problem.slug, "two-sum");
    }

    #[test]
    fn weak_concepts_are_capped_at_the_documented_limit_in_read_order() {
        let mut read = empty_read();
        let concepts: Vec<Value> = (0..11)
            .map(|index| {
                json!({
                    "slug": format!("concept-{index:02}"),
                    "name": format!("Concept {index}"),
                    "failure_count": 11 - index,
                    "last_failed_at": "2026-11-14T18:20:00Z"
                })
            })
            .collect();
        read["weak_concepts"] = Value::Array(concepts);

        let response = build_response(read, &[], date(2026, Month::November, 15)).expect("maps");

        assert_eq!(
            response.weak_concepts.len(),
            10,
            "the response caps the weak list at ten"
        );
        assert_eq!(response.weak_concepts[0].slug, "concept-00");
        assert_eq!(response.weak_concepts[9].slug, "concept-09");
    }

    #[test]
    fn a_due_review_maps_to_the_additive_summary_and_nothing_else() {
        let mut read = empty_read();
        read["review_summary"] = json!({
            "due_count": 3,
            "next_review": {
                "concept": {"slug": "loops", "name": "Loops"},
                "card": {"state": "review", "due": "2026-11-15T09:00:00Z", "version": 4},
                "challenge": {"kind": "sibling", "slug": "loop-gym",
                              "title": "Loop Gym", "difficulty": "medium"}
            }
        });

        let response = build_response(read, &[], date(2026, Month::November, 15)).expect("maps");

        let summary = response.review_summary.expect("summary present");
        assert_eq!(summary.due_count, 3);
        assert_eq!(summary.next_review.concept.slug, "loops");
        assert_eq!(summary.next_review.card.state, CardState::Review);
        assert_eq!(summary.next_review.card.version, 4);
        assert_eq!(
            summary
                .next_review
                .challenge
                .map(|challenge| challenge.kind),
            Some(ReviewChallengeKind::Sibling)
        );
        assert_eq!(response.solved.total, 0, "core fields stay unchanged");
    }

    #[test]
    fn a_null_review_summary_stays_empty_without_fabricating_a_review() {
        let mut read = empty_read();
        read["review_summary"] = Value::Null;

        let response = build_response(read, &[], date(2026, Month::November, 15)).expect("maps");

        assert_eq!(response.review_summary, None);
    }

    #[test]
    fn an_unknown_card_state_is_a_decode_error_not_a_guessed_state() {
        let mut read = empty_read();
        read["review_summary"] = json!({
            "due_count": 1,
            "next_review": {
                "concept": {"slug": "loops", "name": "Loops"},
                "card": {"state": "suspended", "due": "2026-11-15T09:00:00Z", "version": 1},
                "challenge": null
            }
        });

        assert!(build_response(read, &[], date(2026, Month::November, 15)).is_err());
    }

    #[test]
    fn an_unknown_difficulty_is_a_decode_error_not_a_guessed_level() {
        let mut read = empty_read();
        read["candidates"] = json!([
            {"slug": "odd", "title": "Odd", "difficulty": "expert", "concepts": [], "solved": false}
        ]);

        assert!(build_response(read, &[], date(2026, Month::November, 15)).is_err());
    }
}
