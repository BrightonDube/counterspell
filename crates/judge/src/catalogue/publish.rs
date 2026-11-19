//! Publish and unpublish transitions (problem-catalogue C21, CAT-P1..CAT-P5).
//!
//! Publication is a `problems` row update guarded by the `catalogue_publish_gate`
//! trigger: the pinned version must exist, be sandbox-validated, carry 2..=5
//! examples, exactly four hints and at least one concept tag. When the trigger
//! refuses, its check text is reported verbatim — the database is the authority
//! on the gate, this module only decides *which* version to ask for.
//!
//! `publish` resolves a missing `--version` to the latest sandbox-validated
//! version, so the default is a visible decision at the call site. `unpublish`
//! keeps `published_version` (history is append-only) and is idempotent: a
//! second unpublish changes no row and writes no event.

use sqlx::PgPool;

/// The result of a `publish` call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PublishOutcome {
    /// The problem now points at `version`.
    Published {
        /// The version that was published.
        version: i32,
    },
    /// The update could not run: the slug does not exist or no stored version
    /// carries a sandbox stamp.
    Refused {
        /// The refusal code, safe for the operator log.
        code: &'static str,
    },
    /// The database publish gate refused; `check` is the trigger's own text.
    GateRefused {
        /// The failed check text raised by the trigger.
        check: String,
    },
}

/// The result of an `unpublish` call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UnpublishOutcome {
    /// The problem moved back to `unpublished`; `published_version` is kept.
    Unpublished,
    /// Nothing changed: the slug was already unpublished (no event written) or
    /// does not exist.
    Refused {
        /// The refusal code, safe for the operator log.
        code: &'static str,
    },
}

/// Publishes `slug`, pinning `version` or the latest sandbox-validated version.
///
/// The UPDATE, the gate trigger and the `catalogue_events` row commit in one
/// transaction, so a refused publish leaves no partial state.
///
/// # Errors
///
/// Returns the database error only for infrastructure failures; a publish-gate
/// refusal is reported as [`PublishOutcome::GateRefused`] with the trigger text.
pub async fn publish(
    pool: &PgPool,
    slug: &str,
    version: Option<i32>,
    operator: &str,
) -> Result<PublishOutcome, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let version = match version {
        Some(pinned) => pinned,
        None => match latest_validated(&mut tx, slug).await? {
            Some(resolved) => resolved,
            None => {
                return Ok(PublishOutcome::Refused {
                    code: "no_validated_version",
                });
            }
        },
    };
    let update = sqlx::query(
        "UPDATE problems SET publication_state = 'published', published_version = $2, \
         published_at = now(), unpublished_at = NULL WHERE slug = $1",
    )
    .bind(slug)
    .bind(version)
    .execute(&mut *tx)
    .await;
    let updated = match update {
        Ok(done) => done,
        Err(sqlx::Error::Database(error)) => {
            return Ok(PublishOutcome::GateRefused {
                check: error.message().to_string(),
            });
        }
        Err(error) => return Err(error),
    };
    if updated.rows_affected() == 0 {
        return Ok(PublishOutcome::Refused {
            code: "not_in_catalogue",
        });
    }
    sqlx::query(
        "INSERT INTO catalogue_events (problem_slug, action, version, operator, detail) \
         VALUES ($1, 'publish', $2, $3, $4)",
    )
    .bind(slug)
    .bind(version)
    .bind(operator)
    .bind(serde_json::json!({}))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(PublishOutcome::Published { version })
}

/// Moves `slug` back to `unpublished`, keeping `published_version`.
///
/// Idempotent: when the row is already unpublished nothing is written — no
/// `unpublished_at` bump and no second event (CAT-P5).
///
/// # Errors
///
/// Returns the database error from the UPDATE, the lookup or the event insert.
pub async fn unpublish(
    pool: &PgPool,
    slug: &str,
    operator: &str,
) -> Result<UnpublishOutcome, sqlx::Error> {
    let mut tx = pool.begin().await?;
    let updated = sqlx::query(
        "UPDATE problems SET publication_state = 'unpublished', unpublished_at = now() \
         WHERE slug = $1 AND publication_state = 'published'",
    )
    .bind(slug)
    .execute(&mut *tx)
    .await?;
    if updated.rows_affected() == 0 {
        let code = if slug_exists(&mut tx, slug).await? {
            "already_unpublished"
        } else {
            "not_in_catalogue"
        };
        return Ok(UnpublishOutcome::Refused { code });
    }
    sqlx::query(
        "INSERT INTO catalogue_events (problem_slug, action, version, operator, detail) \
         VALUES ($1, 'unpublish', $2, $3, $4)",
    )
    .bind(slug)
    .bind(Option::<i32>::None)
    .bind(operator)
    .bind(serde_json::json!({}))
    .execute(&mut *tx)
    .await?;
    tx.commit().await?;
    Ok(UnpublishOutcome::Unpublished)
}

/// The highest stored version carrying a sandbox stamp, if any.
async fn latest_validated(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    slug: &str,
) -> Result<Option<i32>, sqlx::Error> {
    sqlx::query_scalar(
        "SELECT max(version) FROM problem_versions \
         WHERE problem_slug = $1 AND sandbox_validated_at IS NOT NULL",
    )
    .bind(slug)
    .fetch_one(&mut **tx)
    .await
}

/// Whether `slug` names a catalogue problem.
async fn slug_exists(
    tx: &mut sqlx::Transaction<'_, sqlx::Postgres>,
    slug: &str,
) -> Result<bool, sqlx::Error> {
    sqlx::query_scalar("SELECT EXISTS (SELECT 1 FROM problems WHERE slug = $1)")
        .bind(slug)
        .fetch_one(&mut **tx)
        .await
}
