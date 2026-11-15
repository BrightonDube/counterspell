//! Taxonomy file parser for tracks and concepts (problem-catalogue C07, CAT-T1, CAT-T2).
//!
//! The file is `packs/catalogue.toml`. Unknown keys are errors, so a typo in a reviewed file
//! fails at parse time instead of being ignored.

use serde::Deserialize;
use sqlx::PgPool;

/// One track or concept: a slug and a display name.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Entry {
    /// The kebab-case slug, for example `hash-maps`.
    pub slug: String,
    /// The display name.
    pub name: String,
}

/// The parsed taxonomy file.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Taxonomy {
    /// Tracks, in file order.
    #[serde(default)]
    pub tracks: Vec<Entry>,
    /// Concepts, in file order.
    #[serde(default)]
    pub concepts: Vec<Entry>,
}

/// Why a taxonomy file was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TaxonomyError {
    /// The TOML did not match the expected shape.
    Parse(String),
    /// Two entries of the same kind share a slug.
    DuplicateSlug {
        /// `track` or `concept`.
        kind: &'static str,
        /// The repeated slug.
        slug: String,
    },
    /// A slug breaks the kebab-case rule.
    BadSlug(String),
}

/// Returns true when `slug` is 2 to 40 characters, starts with a lower-case letter, and uses only
/// lower-case letters, digits and hyphens.
#[must_use]
pub fn is_valid_slug(slug: &str) -> bool {
    let length = slug.chars().count();
    let mut chars = slug.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_lowercase())
        && (2..=40).contains(&length)
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
}

/// Parses and checks a taxonomy file.
///
/// # Errors
///
/// Returns [`TaxonomyError::Parse`] for unknown keys or bad shapes, [`TaxonomyError::BadSlug`]
/// for a slug outside the kebab rule, and [`TaxonomyError::DuplicateSlug`] for a repeated slug.
pub fn parse_taxonomy(text: &str) -> Result<Taxonomy, TaxonomyError> {
    let taxonomy: Taxonomy =
        toml::from_str(text).map_err(|error| TaxonomyError::Parse(error.message().to_string()))?;
    check_entries("track", &taxonomy.tracks)?;
    check_entries("concept", &taxonomy.concepts)?;
    Ok(taxonomy)
}

fn check_entries(kind: &'static str, entries: &[Entry]) -> Result<(), TaxonomyError> {
    let mut seen: Vec<&str> = Vec::with_capacity(entries.len());
    for entry in entries {
        if !is_valid_slug(&entry.slug) {
            return Err(TaxonomyError::BadSlug(entry.slug.clone()));
        }
        if seen.contains(&entry.slug.as_str()) {
            return Err(TaxonomyError::DuplicateSlug {
                kind,
                slug: entry.slug.clone(),
            });
        }
        seen.push(&entry.slug);
    }
    Ok(())
}

/// Upserts every track and concept from `taxonomy` (CAT-T3).
///
/// Each row is one `INSERT … ON CONFLICT DO UPDATE` statement, so two syncs running at the same
/// time cannot both insert the same slug. Rows missing from the file are kept, never deleted.
///
/// # Errors
///
/// Returns the database error when a statement fails. Earlier statements in the same call are
/// not rolled back, so the caller can run the sync again safely.
pub async fn sync_taxonomy(pool: &PgPool, taxonomy: &Taxonomy) -> Result<(), sqlx::Error> {
    for (index, entry) in taxonomy.tracks.iter().enumerate() {
        sqlx::query(
            "INSERT INTO tracks (slug, name, sort_order) VALUES ($1, $2, $3) \
             ON CONFLICT (slug) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order",
        )
        .bind(&entry.slug)
        .bind(&entry.name)
        .bind(sort_order(index))
        .execute(pool)
        .await?;
    }
    for (index, entry) in taxonomy.concepts.iter().enumerate() {
        sqlx::query(
            "INSERT INTO concepts (id, name, sort_order) VALUES ($1, $2, $3) \
             ON CONFLICT (id) DO UPDATE SET name = EXCLUDED.name, sort_order = EXCLUDED.sort_order",
        )
        .bind(&entry.slug)
        .bind(&entry.name)
        .bind(sort_order(index))
        .execute(pool)
        .await?;
    }
    Ok(())
}

/// Converts a zero-based file position into a sort order. Saturates instead of panicking.
fn sort_order(index: usize) -> i32 {
    i32::try_from(index).unwrap_or(i32::MAX)
}
