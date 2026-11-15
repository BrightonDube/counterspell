//! Taxonomy file parser for tracks and concepts (problem-catalogue C07, CAT-T1, CAT-T2).
//!
//! The file is `packs/catalogue.toml`. Unknown keys are errors, so a typo in a reviewed file
//! fails at parse time instead of being ignored.

use serde::Deserialize;

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
