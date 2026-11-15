//! Read queries for the public catalogue list (problem-catalogue C30, CAT-A1 to CAT-A3).
//!
//! Only published problems appear. The caller's status is computed in SQL from that caller's own
//! submissions only, so one learner's runs can never change another learner's status.

use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use super::dto::{ConceptRef, ProblemSummary, SolveStatus, TrackRef};
use crate::error::ApiError;

/// The most problems one list response can hold (CAT-A3, design decision D15).
pub const LIST_CAP: i64 = 200;

/// The filters a list request may carry. Each one is checked before any query runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProblemFilter {
    /// A track slug, if the request named one.
    pub track: Option<String>,
    /// A concept slug, if the request named one.
    pub concept: Option<String>,
    /// `easy`, `medium` or `hard`, if the request named one.
    pub difficulty: Option<String>,
}

impl ProblemFilter {
    /// Checks the raw query values. A malformed slug or an unknown difficulty gives `400
    /// invalid_filter`. A well-formed slug that matches nothing simply gives an empty list.
    ///
    /// # Errors
    ///
    /// Returns the `invalid_filter` error for the first value that fails its check.
    pub fn parse(
        track: Option<&str>,
        concept: Option<&str>,
        difficulty: Option<&str>,
    ) -> Result<Self, ApiError> {
        if let Some(value) = track {
            check_slug(value, "track")?;
        }
        if let Some(value) = concept {
            check_slug(value, "concept")?;
        }
        if let Some(value) = difficulty {
            if !matches!(value, "easy" | "medium" | "hard") {
                return Err(invalid("difficulty must be easy, medium or hard."));
            }
        }
        Ok(Self {
            track: track.map(str::to_owned),
            concept: concept.map(str::to_owned),
            difficulty: difficulty.map(str::to_owned),
        })
    }
}

fn check_slug(value: &str, name: &str) -> Result<(), ApiError> {
    let valid = !value.is_empty()
        && value.len() <= 40
        && value.starts_with(|c: char| c.is_ascii_lowercase())
        && value
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-');
    if valid {
        Ok(())
    } else {
        Err(invalid(&format!(
            "{name} must be a lower-case slug of up to 40 characters."
        )))
    }
}

fn invalid(message: &str) -> ApiError {
    ApiError::bad_request("invalid_filter", message.to_owned())
}

type ListRow = (
    String,
    String,
    String,
    String,
    String,
    Value,
    Option<String>,
);

/// Lists published problems that match the filters, for the caller `viewer` (`None` when anonymous).
///
/// # Errors
///
/// Returns the database error when the query fails, or a decode error when a stored concept list
/// is not in the expected shape.
pub async fn list_problems(
    pool: &PgPool,
    viewer: Option<Uuid>,
    filter: &ProblemFilter,
) -> Result<Vec<ProblemSummary>, sqlx::Error> {
    let rows: Vec<ListRow> = sqlx::query_as(
        "SELECT p.slug, p.title, p.difficulty, t.slug, t.name, \
                COALESCE(c.concepts, '[]'::jsonb), \
                CASE WHEN $1::uuid IS NULL THEN NULL \
                     WHEN bool_or(s.status = 'completed' AND s.verdict = 'accepted') THEN 'solved' \
                     WHEN bool_or(s.id IS NOT NULL) THEN 'attempted' \
                     ELSE 'unattempted' END \
         FROM problems p \
         JOIN tracks t ON t.slug = p.track_slug \
         LEFT JOIN LATERAL ( \
             SELECT jsonb_agg(jsonb_build_object('slug', k.id, 'name', k.name) ORDER BY k.id) AS concepts \
             FROM problem_concepts pc JOIN concepts k ON k.id = pc.concept_id \
             WHERE pc.problem_slug = p.slug \
         ) c ON true \
         LEFT JOIN submissions s ON s.problem_slug = p.slug AND s.user_id = $1 \
         WHERE p.publication_state = 'published' \
           AND ($2::text IS NULL OR t.slug = $2) \
           AND ($3::text IS NULL OR p.difficulty = $3) \
           AND ($4::text IS NULL OR EXISTS ( \
               SELECT 1 FROM problem_concepts f WHERE f.problem_slug = p.slug AND f.concept_id = $4)) \
         GROUP BY p.slug, p.title, p.difficulty, t.slug, t.name, t.sort_order, c.concepts \
         ORDER BY t.sort_order, p.slug \
         LIMIT $5",
    )
    .bind(viewer)
    .bind(filter.track.as_deref())
    .bind(filter.difficulty.as_deref())
    .bind(filter.concept.as_deref())
    .bind(LIST_CAP)
    .fetch_all(pool)
    .await?;

    rows.into_iter()
        .map(
            |(slug, title, difficulty, track_slug, track_name, concepts, status)| {
                let concepts: Vec<ConceptRef> = serde_json::from_value(concepts)
                    .map_err(|error| sqlx::Error::Decode(Box::new(error)))?;
                Ok(ProblemSummary {
                    slug,
                    title,
                    track: TrackRef {
                        slug: track_slug,
                        name: track_name,
                    },
                    difficulty,
                    concepts,
                    status: status.as_deref().and_then(SolveStatus::from_sql),
                })
            },
        )
        .collect()
}
