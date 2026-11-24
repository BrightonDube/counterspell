//! Looks up published problems in Postgres for the run flow (problem-catalogue CAT-L2).

use async_trait::async_trait;
use sqlx::PgPool;

use crate::{
    error::ApiError,
    seams::{ProblemLookup, ProblemRef},
};

/// Reads the published version of a problem from the `problems` table.
///
/// The publication check is part of the SQL `WHERE` clause, so an unpublished row is never
/// returned. A database error is logged and answered as "not found". That fails closed: the run is
/// refused, not queued against an unknown version.
#[derive(Debug, Clone)]
pub struct PgProblemLookup {
    pool: PgPool,
}

impl PgProblemLookup {
    /// Wraps a pool that connects as the server role.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl ProblemLookup for PgProblemLookup {
    async fn published(&self, slug: &str) -> Option<ProblemRef> {
        let row: Result<Option<(String, i32)>, sqlx::Error> = sqlx::query_as(
            "SELECT slug, published_version FROM problems \
             WHERE slug = $1 AND publication_state = 'published'",
        )
        .bind(slug)
        .fetch_optional(&self.pool)
        .await;
        match row {
            Ok(found) => found.map(|(slug, version)| ProblemRef { slug, version }),
            Err(error) => {
                tracing::error!(error = %error, "catalogue lookup failed; treating the problem as unpublished");
                None
            }
        }
    }

    async fn supports_language(&self, problem: &ProblemRef, lang: &str) -> Result<bool, ApiError> {
        let row: Result<bool, sqlx::Error> = sqlx::query_scalar(
            "SELECT EXISTS(SELECT 1 FROM problem_starters \
             WHERE problem_slug = $1 AND version = $2 AND lang = $3)",
        )
        .bind(&problem.slug)
        .bind(problem.version)
        .bind(lang)
        .fetch_one(&self.pool)
        .await;
        match row {
            Ok(supported) => Ok(supported),
            Err(error) => {
                tracing::error!(error = %error, "starter lookup failed");
                Err(ApiError::internal(error))
            }
        }
    }
}
