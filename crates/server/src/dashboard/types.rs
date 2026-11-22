//! Response types for `GET /api/dashboard`.
//!
//! The field names are the public JSON contract. Renaming a field is a breaking change for the web app.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::reviews::types::DueReview;

/// The whole dashboard. `Default` is the stable empty shape for a learner with no activity.
#[derive(Debug, Clone, Default, PartialEq, Serialize)]
pub struct DashboardResponse {
    /// Solved counts for the learner.
    pub solved: SolvedSummary,
    /// The practice streak in UTC days.
    pub streak: StreakSummary,
    /// Concepts with the learner's failed attempts, most failures first.
    pub weak_concepts: Vec<WeakConcept>,
    /// The learner's most recent runs, newest first. At most ten.
    pub recent_submissions: Vec<RecentSubmission>,
    /// Unsolved published problems to try next. At most three.
    pub suggested_challenges: Vec<SuggestedChallenge>,
    /// The [stretch] due-review summary (dashboard-fsrs T17). `null` when no card is due.
    /// Additive: older clients can ignore it.
    pub review_summary: Option<ReviewSummary>,
}

/// The learner's spaced-repetition summary. Never fabricated: it exists only when at least one
/// card is due, and it never adds a row to the core fields.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReviewSummary {
    /// How many of the learner's cards are due now.
    pub due_count: u32,
    /// The earliest due review: the concept, the card guard fields and the offered challenge.
    pub next_review: DueReview,
}

/// Solved problem counts. A problem counts once, however many accepted runs it has.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct SolvedSummary {
    /// Every solved problem, across all difficulties.
    pub total: u32,
    /// Solved problems per difficulty. Each level is present, even when its count is zero.
    pub by_difficulty: DifficultyCounts,
}

/// The number of solved problems at each difficulty level.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct DifficultyCounts {
    /// Solved easy problems.
    pub easy: u32,
    /// Solved medium problems.
    pub medium: u32,
    /// Solved hard problems.
    pub hard: u32,
}

/// A catalogue difficulty level. The JSON value is the lowercase name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Difficulty {
    /// The easiest level.
    Easy,
    /// The middle level.
    Medium,
    /// The hardest level.
    Hard,
}

/// The practice streak. `last_activity_date` is a UTC date such as `2026-11-12`, or `null`.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct StreakSummary {
    /// Consecutive UTC days with a completed run, ending today or yesterday.
    pub current_days: u32,
    /// Whether the learner has a completed run today (UTC).
    pub active_today: bool,
    /// The most recent UTC activity date, or `null` when there is none.
    pub last_activity_date: Option<String>,
}

/// A concept the learner has failed on. Timestamps are RFC 3339 in UTC.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct WeakConcept {
    /// The concept's stable slug.
    pub slug: String,
    /// The concept's display name.
    pub name: String,
    /// How many of the learner's failed runs carry this concept.
    pub failure_count: u32,
    /// When the learner last failed a problem with this concept.
    pub last_failed_at: String,
}

/// The public problem fields that dashboard rows show.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProblemSummary {
    /// The problem's URL-safe slug.
    pub slug: String,
    /// The problem's title.
    pub title: String,
    /// The problem's difficulty.
    pub difficulty: Difficulty,
}

/// One of the learner's recent runs. It holds no code, hidden test data or judge diagnostics.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct RecentSubmission {
    /// The run id.
    pub id: Uuid,
    /// The problem the run was for.
    pub problem: ProblemSummary,
    /// The language name, such as `python`.
    pub lang: String,
    /// `run` for sample tests or `submit` for hidden tests.
    pub mode: String,
    /// The lifecycle status, such as `queued` or `completed`.
    pub status: String,
    /// The user-code verdict. Present only once the run has completed.
    pub verdict: Option<String>,
    /// Tests passed, when known.
    pub passed: Option<i32>,
    /// Tests in the run, when known.
    pub total: Option<i32>,
    /// The score, when known.
    pub score: Option<i32>,
    /// When the run was saved, in RFC 3339 UTC.
    pub created_at: String,
    /// When the run finished, in RFC 3339 UTC, or `null` while it is still active.
    pub finished_at: Option<String>,
}

/// A concept shown next to a problem.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ConceptRef {
    /// The concept's stable slug.
    pub slug: String,
    /// The concept's display name.
    pub name: String,
}

/// Why a problem was suggested. The JSON value is a stable `snake_case` code.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionReason {
    /// The problem shares a concept with one of the learner's weak concepts.
    PracticeWeakConcept,
    /// The problem continues the learner's track, used when no weak concept matches.
    ContinueTrack,
}

/// A published problem the learner has not solved yet.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct SuggestedChallenge {
    /// The problem's URL-safe slug.
    pub slug: String,
    /// The problem's title.
    pub title: String,
    /// The problem's difficulty.
    pub difficulty: Difficulty,
    /// The problem's concepts.
    pub concepts: Vec<ConceptRef>,
    /// Why this problem was suggested.
    pub reason: SuggestionReason,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn default_dashboard_is_the_empty_shape_with_zero_counts() {
        let value = serde_json::to_value(DashboardResponse::default()).expect("serialise");
        assert_eq!(
            value,
            json!({
                "solved": {
                    "total": 0,
                    "by_difficulty": {"easy": 0, "medium": 0, "hard": 0}
                },
                "streak": {
                    "current_days": 0,
                    "active_today": false,
                    "last_activity_date": null
                },
                "weak_concepts": [],
                "recent_submissions": [],
                "suggested_challenges": [],
                "review_summary": null
            })
        );
    }

    #[test]
    fn difficulty_and_reason_use_lowercase_and_snake_case_names() {
        assert_eq!(
            serde_json::to_value(Difficulty::Medium).expect("serialise"),
            json!("medium")
        );
        assert_eq!(
            serde_json::to_value(SuggestionReason::PracticeWeakConcept).expect("serialise"),
            json!("practice_weak_concept")
        );
    }
}
