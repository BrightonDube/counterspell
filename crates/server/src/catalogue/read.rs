//! Read queries for the public catalogue list (problem-catalogue C30, CAT-A1 to CAT-A3).
//!
//! Only published problems appear. The caller's status is computed in SQL from that caller's own
//! submissions only, so one learner's runs can never change another learner's status.

use std::collections::BTreeMap;

use serde::Deserialize;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

use super::dto::{
    ConceptRef, Example, LimitsView, ParameterView, ProblemDetail, ProblemSummary, SignatureView,
    SolveStatus, TrackRef,
};
use crate::error::ApiError;
use crate::seams::LanguageCatalog;

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

/// The signature as the importer stores it, with grimoire's field names. Every field is required, so a
/// malformed stored value is an error rather than a silently empty signature.
#[derive(Deserialize)]
struct StoredSignature {
    function: String,
    params: Vec<StoredParam>,
    returns: String,
}

/// One stored parameter.
#[derive(Deserialize)]
struct StoredParam {
    name: String,
    #[serde(rename = "type")]
    kind: String,
}

type DetailRow = (
    i32,
    String,
    String,
    String,
    String,
    String,
    Value,
    String,
    Value,
    Value,
    Value,
    i64,
);

/// Reads the pinned published version of one problem (CAT-L1, CAT-A4). Returns `None` for an unknown
/// slug or an unpublished problem, so both cases look the same to the caller (CAT-A6).
///
/// `languages` decides which starter languages are listed as enabled.
///
/// # Errors
///
/// Returns the database error when a query fails, or a decode error when a stored value is not in the
/// expected shape.
pub async fn published_detail(
    pool: &PgPool,
    slug: &str,
    languages: &LanguageCatalog,
) -> Result<Option<ProblemDetail>, sqlx::Error> {
    let row: Option<DetailRow> = sqlx::query_as(
        "SELECT v.version, p.slug, v.title, t.slug, t.name, v.difficulty, \
                COALESCE(c.concepts, '[]'::jsonb), v.statement_markdown, v.constraints, v.signature, \
                v.limits, \
                (SELECT count(*) FROM problem_hints h WHERE h.problem_slug = p.slug AND h.version = v.version) \
         FROM problems p \
         JOIN problem_versions v ON v.problem_slug = p.slug AND v.version = p.published_version \
         JOIN tracks t ON t.slug = v.track_slug \
         LEFT JOIN LATERAL ( \
             SELECT jsonb_agg(jsonb_build_object('slug', k.id, 'name', k.name) ORDER BY k.id) AS concepts \
             FROM problem_concepts pc JOIN concepts k ON k.id = pc.concept_id \
             WHERE pc.problem_slug = p.slug \
         ) c ON true \
         WHERE p.slug = $1 AND p.publication_state = 'published'",
    )
    .bind(slug)
    .fetch_optional(pool)
    .await?;
    let Some((
        version,
        slug,
        title,
        track_slug,
        track_name,
        difficulty,
        concepts,
        statement,
        constraints,
        signature,
        limits,
        hint_count,
    )) = row
    else {
        return Ok(None);
    };

    let examples: Vec<(Value, Value)> = sqlx::query_as(
        "SELECT input, expected FROM problem_examples \
         WHERE problem_slug = $1 AND version = $2 ORDER BY ordinal",
    )
    .bind(&slug)
    .bind(version)
    .fetch_all(pool)
    .await?;

    let starters: Vec<(String, String)> = sqlx::query_as(
        "SELECT lang, code FROM problem_starters WHERE problem_slug = $1 AND version = $2 ORDER BY lang",
    )
    .bind(&slug)
    .bind(version)
    .fetch_all(pool)
    .await?;

    let concepts: Vec<ConceptRef> = serde_json::from_value(concepts).map_err(decode)?;
    let constraints: Vec<String> = serde_json::from_value(constraints).map_err(decode)?;
    let signature: StoredSignature = serde_json::from_value(signature).map_err(decode)?;
    let limits: LimitsView = serde_json::from_value(limits).map_err(decode)?;
    let starters: BTreeMap<String, String> = starters.into_iter().collect();
    let enabled_languages = starters
        .keys()
        .filter(|lang| languages.find(lang).is_some())
        .cloned()
        .collect();

    Ok(Some(ProblemDetail {
        slug,
        title,
        track: TrackRef {
            slug: track_slug,
            name: track_name,
        },
        difficulty,
        concepts,
        statement_markdown: statement,
        constraints,
        examples: examples
            .into_iter()
            .map(|(input, expected)| Example { input, expected })
            .collect(),
        signature: SignatureView {
            function_name: signature.function,
            parameters: signature
                .params
                .into_iter()
                .map(|param| ParameterView {
                    name: param.name,
                    kind: param.kind,
                })
                .collect(),
            return_type: signature.returns,
        },
        starters,
        enabled_languages,
        limits,
        hint_count,
    }))
}

fn decode(error: serde_json::Error) -> sqlx::Error {
    sqlx::Error::Decode(Box::new(error))
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
