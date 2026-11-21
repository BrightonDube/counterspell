//! Due-review reads (dashboard-fsrs T15).
//!
//! The query starts from the authenticated owner's cards before joining catalogue rows. Sibling
//! problems must share the due concept, be published, and be unsolved by that same owner. If none
//! exists, the first normal published unsolved problem is returned and labelled `normal`.

use counterspell_common::ids::UserId;
use serde_json::{json, Value};
use sqlx::{PgPool, Postgres, Transaction};
use time::OffsetDateTime;

use crate::dashboard::types::{ConceptRef, Difficulty};

use super::{
    types::{DueCard, DueReview, ReviewChallenge, ReviewChallengeKind},
    CardState, FsrsScheduler, PersistedCard, ReviewError, ReviewGrade,
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

/// A review write that cannot be completed safely.
#[derive(Debug)]
pub enum ReviewWriteError {
    /// The card changed since the client's version was read.
    Conflict,
    /// The concept in the URL is not catalogue data.
    UnknownConcept,
    /// Persisted data or scheduler output violated the adapter invariants.
    Scheduler(ReviewError),
    /// PostgreSQL failed.
    Database(sqlx::Error),
}

/// Atomically grades the caller's card for `concept`.
///
/// The card row is locked, the optional version is checked, the card and history are written
/// together, and a challenge is selected before commit. Any failure rolls back the transaction.
///
/// # Errors
///
/// Returns [`ReviewWriteError`] for a stale version, unknown concept, invalid scheduler result, or
/// database failure.
pub async fn review_concept(
    pool: &PgPool,
    owner: UserId,
    concept: &str,
    grade: ReviewGrade,
    expected_version: Option<i64>,
    reviewed_at: OffsetDateTime,
) -> Result<DueReview, ReviewWriteError> {
    let mut tx = pool.begin().await.map_err(ReviewWriteError::Database)?;
    let concept_ref = concept_for_update(&mut tx, concept).await?;
    let previous = card_for_update(&mut tx, owner, concept).await?;
    if expected_version.is_some_and(|version| version != previous.version) {
        return Err(ReviewWriteError::Conflict);
    }

    let calculation = FsrsScheduler::default()
        .review(&previous, grade, reviewed_at)
        .map_err(ReviewWriteError::Scheduler)?;
    update_card(&mut tx, owner, concept, &calculation.next_card).await?;
    insert_history(&mut tx, owner, concept, &calculation).await?;
    let challenge = select_challenge(&mut tx, owner, concept).await?;
    let due = format_timestamp(calculation.next_card.due).map_err(ReviewWriteError::Scheduler)?;
    tx.commit().await.map_err(ReviewWriteError::Database)?;

    Ok(DueReview {
        concept: concept_ref,
        card: DueCard {
            state: calculation.next_card.state,
            due,
            version: calculation.next_card.version,
        },
        challenge,
    })
}

async fn concept_for_update(
    tx: &mut Transaction<'_, Postgres>,
    concept: &str,
) -> Result<ConceptRef, ReviewWriteError> {
    sqlx::query_as::<_, (String, String)>("SELECT id, name FROM concepts WHERE id = $1")
        .bind(concept)
        .fetch_optional(&mut **tx)
        .await
        .map_err(ReviewWriteError::Database)?
        .map(|(slug, name)| ConceptRef { slug, name })
        .ok_or(ReviewWriteError::UnknownConcept)
}

async fn card_for_update(
    tx: &mut Transaction<'_, Postgres>,
    owner: UserId,
    concept: &str,
) -> Result<PersistedCard, ReviewWriteError> {
    if let Some(card) = select_card_for_update(tx, owner, concept).await? {
        return Ok(card);
    }

    sqlx::query(
        "INSERT INTO concept_cards
         (user_id, concept_id, stability, stability_fast, difficulty, reps, lapses, state, due, last_review, version)
         VALUES ($1, $2, 0, 0, 0, 0, 0, 'new', now(), NULL, 0)
         ON CONFLICT (user_id, concept_id) DO NOTHING",
    )
    .bind(owner.as_uuid())
    .bind(concept)
    .execute(&mut **tx)
    .await
    .map_err(ReviewWriteError::Database)?;
    select_card_for_update(tx, owner, concept)
        .await?
        .ok_or_else(|| ReviewWriteError::Scheduler(ReviewError::InvalidCard))
}

async fn select_card_for_update(
    tx: &mut Transaction<'_, Postgres>,
    owner: UserId,
    concept: &str,
) -> Result<Option<PersistedCard>, ReviewWriteError> {
    sqlx::query_as::<_, RawCard>(
        "SELECT stability, stability_fast, difficulty, reps, lapses, state, due, last_review, version
         FROM concept_cards WHERE user_id = $1 AND concept_id = $2 FOR UPDATE",
    )
    .bind(owner.as_uuid())
    .bind(concept)
    .fetch_optional(&mut **tx)
    .await
    .map_err(ReviewWriteError::Database)?
    .map(|raw| raw.into_card().map_err(ReviewWriteError::Scheduler))
    .transpose()
}

async fn update_card(
    tx: &mut Transaction<'_, Postgres>,
    owner: UserId,
    concept: &str,
    card: &PersistedCard,
) -> Result<(), ReviewWriteError> {
    let changed = sqlx::query(
        "UPDATE concept_cards SET
         stability = $3, stability_fast = $4, difficulty = $5, reps = $6, lapses = $7,
         state = $8, due = $9, last_review = $10, version = $11, updated_at = now()
         WHERE user_id = $1 AND concept_id = $2",
    )
    .bind(owner.as_uuid())
    .bind(concept)
    .bind(card.stability)
    .bind(card.stability_fast)
    .bind(card.difficulty)
    .bind(
        i32::try_from(card.reps)
            .map_err(|_| ReviewWriteError::Scheduler(ReviewError::InvalidCard))?,
    )
    .bind(
        i32::try_from(card.lapses)
            .map_err(|_| ReviewWriteError::Scheduler(ReviewError::InvalidCard))?,
    )
    .bind(card.state.as_str())
    .bind(card.due)
    .bind(card.last_review)
    .bind(card.version)
    .execute(&mut **tx)
    .await
    .map_err(ReviewWriteError::Database)?
    .rows_affected();
    if changed == 1 {
        Ok(())
    } else {
        Err(ReviewWriteError::Conflict)
    }
}

async fn insert_history(
    tx: &mut Transaction<'_, Postgres>,
    owner: UserId,
    concept: &str,
    calculation: &super::ReviewCalculation,
) -> Result<(), ReviewWriteError> {
    let previous_snapshot =
        card_snapshot(&calculation.previous_card).map_err(ReviewWriteError::Scheduler)?;
    let next_snapshot =
        card_snapshot(&calculation.next_card).map_err(ReviewWriteError::Scheduler)?;
    sqlx::query(
        "INSERT INTO concept_review_history
         (user_id, concept_id, grade, reviewed_at, previous_card_state, next_card_state, next_interval)
         VALUES ($1, $2, $3, $4, $5, $6, $7::interval)",
    )
    .bind(owner.as_uuid())
    .bind(concept)
    .bind(calculation.grade.as_str())
    .bind(calculation.reviewed_at)
    .bind(previous_snapshot)
    .bind(next_snapshot)
    .bind(format!(
        "{} seconds",
        calculation.next_interval.whole_seconds()
    ))
    .execute(&mut **tx)
    .await
    .map_err(ReviewWriteError::Database)?;
    Ok(())
}

async fn select_challenge(
    tx: &mut Transaction<'_, Postgres>,
    owner: UserId,
    concept: &str,
) -> Result<Option<ReviewChallenge>, ReviewWriteError> {
    if let Some(raw) = sqlx::query_as::<_, RawChallenge>(
        "SELECT p.slug, p.title, p.difficulty
         FROM problem_concepts pc
         JOIN problems p ON p.slug = pc.problem_slug AND p.publication_state = 'published'
         WHERE pc.concept_id = $2
           AND NOT EXISTS (
               SELECT 1 FROM submissions s
               WHERE s.user_id = $1 AND s.problem_slug = p.slug
                 AND s.status = 'completed' AND s.verdict = 'accepted'
                 AND s.mode = 'submit' AND s.ghost_input IS NULL
           )
         ORDER BY p.slug ASC LIMIT 1",
    )
    .bind(owner.as_uuid())
    .bind(concept)
    .fetch_optional(&mut **tx)
    .await
    .map_err(ReviewWriteError::Database)?
    {
        return raw
            .into_challenge(ReviewChallengeKind::Sibling)
            .map(Some)
            .map_err(ReviewWriteError::Database);
    }

    sqlx::query_as::<_, RawChallenge>(
        "SELECT p.slug, p.title, p.difficulty
         FROM problems p
         WHERE p.publication_state = 'published'
           AND NOT EXISTS (
               SELECT 1 FROM submissions s
               WHERE s.user_id = $1 AND s.problem_slug = p.slug
                 AND s.status = 'completed' AND s.verdict = 'accepted'
                 AND s.mode = 'submit' AND s.ghost_input IS NULL
           )
         ORDER BY p.slug ASC LIMIT 1",
    )
    .bind(owner.as_uuid())
    .fetch_optional(&mut **tx)
    .await
    .map_err(ReviewWriteError::Database)?
    .map(|raw| raw.into_challenge(ReviewChallengeKind::Normal))
    .transpose()
    .map_err(ReviewWriteError::Database)
}

fn card_snapshot(card: &PersistedCard) -> Result<Value, ReviewError> {
    Ok(json!({
        "stability": card.stability,
        "stability_fast": card.stability_fast,
        "difficulty": card.difficulty,
        "reps": card.reps,
        "lapses": card.lapses,
        "state": card.state.as_str(),
        "due": format_timestamp(card.due)?,
        "last_review": card.last_review.map(format_timestamp).transpose()?,
        "version": card.version,
    }))
}

fn format_timestamp(value: OffsetDateTime) -> Result<String, ReviewError> {
    value
        .format(&time::format_description::well_known::Rfc3339)
        .map_err(|_| ReviewError::InvalidOutput)
}

#[derive(sqlx::FromRow)]
struct RawCard {
    stability: f64,
    stability_fast: f64,
    difficulty: f64,
    reps: i32,
    lapses: i32,
    state: String,
    due: OffsetDateTime,
    last_review: Option<OffsetDateTime>,
    version: i64,
}

impl RawCard {
    fn into_card(self) -> Result<PersistedCard, ReviewError> {
        Ok(PersistedCard {
            stability: self.stability,
            stability_fast: self.stability_fast,
            difficulty: self.difficulty,
            reps: u32::try_from(self.reps).map_err(|_| ReviewError::InvalidCard)?,
            lapses: u32::try_from(self.lapses).map_err(|_| ReviewError::InvalidCard)?,
            state: CardState::parse(&self.state)?,
            due: self.due,
            last_review: self.last_review,
            version: self.version,
        })
    }
}

#[derive(sqlx::FromRow)]
struct RawChallenge {
    slug: String,
    title: String,
    difficulty: String,
}

impl RawChallenge {
    fn into_challenge(self, kind: ReviewChallengeKind) -> Result<ReviewChallenge, sqlx::Error> {
        Ok(ReviewChallenge {
            kind,
            slug: self.slug,
            title: self.title,
            difficulty: decode_difficulty(&self.difficulty)?,
        })
    }
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
