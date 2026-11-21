//! Public response types for review endpoints.

use serde::Serialize;

use crate::dashboard::types::{ConceptRef, Difficulty};

use super::CardState;

/// The caller's due review cards.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DueReviewsResponse {
    /// Due cards, earliest due first and capped by the endpoint.
    pub reviews: Vec<DueReview>,
}

/// One due card and the challenge offered for that concept.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DueReview {
    /// The concept being reviewed.
    pub concept: ConceptRef,
    /// The learner's persisted card summary.
    pub card: DueCard,
    /// The selected challenge, or `null` when no published unsolved problem exists.
    pub challenge: Option<ReviewChallenge>,
}

/// The card fields the browser needs to submit a guarded review.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DueCard {
    /// The card lifecycle state.
    pub state: CardState,
    /// When the card became due, formatted as RFC 3339 UTC.
    pub due: String,
    /// The optimistic-concurrency version for the next grade write.
    pub version: i64,
}

/// A published problem offered beside a due review.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct ReviewChallenge {
    /// Whether the problem shares the review concept or is the normal fallback.
    pub kind: ReviewChallengeKind,
    /// The problem's URL-safe slug.
    pub slug: String,
    /// The problem's title.
    pub title: String,
    /// The problem's difficulty.
    pub difficulty: Difficulty,
}

/// How a challenge was selected.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReviewChallengeKind {
    /// Shares the due card's concept and is unsolved by the caller.
    Sibling,
    /// The normal unsolved published fallback when no sibling is available.
    Normal,
}
