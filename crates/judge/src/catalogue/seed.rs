//! Operator listing and the seed-demo publish step (problem-catalogue C22,
//! CAT-D1..CAT-D3).
//!
//! `list` is a plain read of the `problems` table for the operator. The seed
//! commands live in the CLI (they walk the pack trees); this module holds only
//! the database-shaped pieces so the two seed paths cannot drift apart: after
//! a `seed-demo` import succeeds, [`publish_validated`] pins the latest
//! sandbox-validated version — the same default `publish` uses for a bare
//! `publish <slug>`.

use sqlx::PgPool;

use super::publish::{self, PublishOutcome};

/// One `problems` row as the `list` command prints it.
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
pub struct CatalogueEntry {
    /// The problem slug.
    pub slug: String,
    /// `unpublished` or `published`.
    pub state: String,
    /// The pinned published version, when published.
    pub version: Option<i32>,
}

/// Every catalogue problem, ordered by slug.
///
/// # Errors
///
/// Returns the database error from the SELECT.
pub async fn list(pool: &PgPool) -> Result<Vec<CatalogueEntry>, sqlx::Error> {
    sqlx::query_as(
        "SELECT slug, publication_state AS state, published_version AS version \
         FROM problems ORDER BY slug",
    )
    .fetch_all(pool)
    .await
}

/// Publishes the latest sandbox-validated version of `slug` — the publish step
/// of `seed-demo`, identical to a bare `publish <slug>` invocation.
///
/// # Errors
///
/// Returns the database error from the publish transaction.
pub async fn publish_validated(
    pool: &PgPool,
    slug: &str,
    operator: &str,
) -> Result<PublishOutcome, sqlx::Error> {
    publish::publish(pool, slug, None, operator).await
}
