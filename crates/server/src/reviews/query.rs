//! Due-review reads (dashboard-fsrs T15).
//!
//! The query starts from the authenticated owner's cards before joining catalogue rows. Sibling
//! problems must share the due concept, be published, and be unsolved by that same owner. If none
//! exists, the first normal published unsolved problem is returned and labelled `normal`.

use counterspell_common::ids::UserId;
use serde_json::json;
use sqlx::PgPool;
use time::OffsetDateTime;

use crate::dashboard::types::{ConceptRef, Difficulty};

use super::{
    types::{DueCard, DueReview, ReviewChallenge, ReviewChallengeKind},
    CardState,
};

const DUE_REVIEW_LIMIT: i64 = 20;

const DUE_REVIEWS_SQL: &str = r#"
WITH due_cards AS (
    SELECT cc.concept_id, c.name AS concept_name, cc.state, cc.due, cc.version
    FROM concept_cards cc
    JOIN concepts c ON c.id = cc.concept_id
    WHERE cc.user_id = $1 AND cc.due <= $2
    ORDER BY cc.due ASC, cc.concept_id ASC
    LIMIT $3
),
solved AS (
    SELECT DISTINCT s.problem_slug
    FROM submissions s
    JOIN problems p ON p.slug = s.problem_slug AND p.publication_state = 'published'
    WHERE s.user_id = $1
      AND s.status = 'completed'
      AND s.verdict = 'accepted'
      AND s.mode = 'submit'
      AND s.ghost_input IS NULL
),
siblings AS (
    SELECT dc.concept_id, p.slug, p.title, p.difficulty,
           ROW_NUMBER() OVER (PARTITION BY dc.concept_id ORDER BY p.slug ASC) AS choice
    FROM due_cards dc
    JOIN problem_concepts pc ON pc.concept_id = dc.concept_id
    JOIN problems p ON p.slug = pc.problem_slug AND p.publication_state = 'published'
    WHERE NOT EXISTS (SELECT 1 FROM solved s WHERE s.problem_slug = p.slug)
),
fallbacks AS (
    SELECT p.slug, p.title, p.difficulty,
           ROW_NUMBER() OVER (ORDER BY p.slug ASC) AS choice
    FROM problems p
    WHERE p.publication_state = 'published'
      AND NOT EXISTS (SELECT 1 FROM solved s WHERE s.problem_slug = p.slug)
)
SELECT dc.concept_id, dc.concept_name, dc.state, dc.version,
       to_char(dc.due AT TIME ZONE 'UTC', 'YYYY-MM-DD"T"HH24:MI:SS"Z"') AS due,
       sb.slug AS sibling_slug, sb.title AS sibling_title, sb.difficulty AS sibling_difficulty,
       fb.slug AS fallback_slug, fb.title AS fallback_title, fb.difficulty AS fallback_difficulty
FROM due_cards dc
LEFT JOIN siblings sb ON sb.concept_id = dc.concept_id AND sb.choice = 1
LEFT JOIN fallbacks fb ON fb.choice = 1
ORDER BY dc.due ASC, dc.concept_id ASC
"#;

/// Reads the caller's due cards at `now`.
///
/// # Errors
///
/// Returns the database error, or a decode error when stored card state or difficulty text does not
/// match the public contract.
pub async fn read_due_reviews(
    pool: &PgPool,
    owner: UserId,
    now: OffsetDateTime,
) -> Result<Vec<DueReview>, sqlx::Error> {
    let rows = sqlx::query_as::<_, RawDueReview>(DUE_REVIEWS_SQL)
        .bind(owner.as_uuid())
        .bind(now)
        .bind(DUE_REVIEW_LIMIT)
        .fetch_all(pool)
        .await?;
    rows.into_iter().map(map_due_review).collect()
}

#[derive(sqlx::FromRow)]
struct RawDueReview {
    concept_id: String,
    concept_name: String,
    state: String,
    version: i64,
    due: String,
    sibling_slug: Option<String>,
    sibling_title: Option<String>,
    sibling_difficulty: Option<String>,
    fallback_slug: Option<String>,
    fallback_title: Option<String>,
    fallback_difficulty: Option<String>,
}

fn map_due_review(raw: RawDueReview) -> Result<DueReview, sqlx::Error> {
    let state =
        CardState::parse(&raw.state).map_err(|error| sqlx::Error::Decode(Box::new(error)))?;
    let challenge = match (raw.sibling_slug, raw.sibling_title, raw.sibling_difficulty) {
        (Some(slug), Some(title), Some(difficulty)) => Some(ReviewChallenge {
            kind: ReviewChallengeKind::Sibling,
            slug,
            title,
            difficulty: decode_difficulty(&difficulty)?,
        }),
        _ => match (
            raw.fallback_slug,
            raw.fallback_title,
            raw.fallback_difficulty,
        ) {
            (Some(slug), Some(title), Some(difficulty)) => Some(ReviewChallenge {
                kind: ReviewChallengeKind::Normal,
                slug,
                title,
                difficulty: decode_difficulty(&difficulty)?,
            }),
            _ => None,
        },
    };

    Ok(DueReview {
        concept: ConceptRef {
            slug: raw.concept_id,
            name: raw.concept_name,
        },
        card: DueCard {
            state,
            due: raw.due,
            version: raw.version,
        },
        challenge,
    })
}

fn decode_difficulty(value: &str) -> Result<Difficulty, sqlx::Error> {
    serde_json::from_value(json!(value)).map_err(|error| sqlx::Error::Decode(Box::new(error)))
}
