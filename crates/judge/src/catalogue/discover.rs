//! Pack discovery and the draft and underscore filter (problem-catalogue C08, CAT-I2, CAT-I3).
//!
//! The filter is pure: the caller reads the directory and each `meta.toml`, then passes the facts
//! in. Every skipped entry carries its reason, so the report can say why.

use std::fmt;

/// What the caller found for one entry in the packs directory.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DirEntry<'a> {
    /// The entry name, for example `count-pairs` or `README.md`.
    pub name: &'a str,
    /// True for a directory.
    pub is_dir: bool,
    /// True when the directory holds a `meta.toml`.
    pub has_meta: bool,
    /// The `status` value from `meta.toml`, when it was read.
    pub status: Option<&'a str>,
}

/// Why an entry is not imported.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SkipReason {
    /// The name starts with `_`, which marks test and dogfood packs.
    Underscore,
    /// The entry is a file, or a directory without `meta.toml`.
    NotAPack,
    /// The pack's status is not `published`.
    Draft,
}

/// The outcome for one directory entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// The entry is a published pack and will be imported.
    Import,
    /// The entry is skipped, and the reason is recorded.
    Skip(SkipReason),
}

/// Why an explicitly named slug is refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiscoverError {
    slug: String,
}

impl fmt::Display for DiscoverError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "slug \"{}\" starts with _ and is never imported",
            self.slug
        )
    }
}

impl std::error::Error for DiscoverError {}

/// Classifies one directory entry.
#[must_use]
pub fn classify(entry: &DirEntry<'_>) -> Decision {
    if entry.name.starts_with('_') {
        return Decision::Skip(SkipReason::Underscore);
    }
    if !entry.is_dir || !entry.has_meta {
        return Decision::Skip(SkipReason::NotAPack);
    }
    if entry.status != Some("published") {
        return Decision::Skip(SkipReason::Draft);
    }
    Decision::Import
}

/// Refuses an explicit `--slug` that starts with `_`.
///
/// # Errors
///
/// Returns [`DiscoverError`] when the slug starts with `_`.
pub fn explicit_slug(slug: &str) -> Result<(), DiscoverError> {
    if slug.starts_with('_') {
        return Err(DiscoverError {
            slug: slug.to_string(),
        });
    }
    Ok(())
}
