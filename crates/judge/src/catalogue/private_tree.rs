//! Private-tree preflight and join over listings (problem-catalogue C08a, CAT-PT1 to CAT-PT3, CAT-X4).
//!
//! No disk access happens here. The caller checks the root, resolves symlinks and lists the
//! directories, then passes the results in. That keeps the rules testable without a filesystem.

use std::fmt;

/// What the caller found at the private root.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RootState {
    /// The path does not exist.
    Missing,
    /// The path exists but cannot be read.
    Unreadable,
    /// The path can be read. `inside_public` is true when its resolved path lies under the public tree.
    Readable {
        /// True when the resolved root is inside the public tree.
        inside_public: bool,
    },
}

/// Why the private root cannot be used.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PreflightError {
    /// The root does not exist.
    Missing,
    /// The root cannot be read.
    Unreadable,
    /// The private root sits inside the public tree.
    InsidePublic,
}

impl PreflightError {
    /// Returns the stable code used in reports.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Missing => "private_tree_missing",
            Self::Unreadable => "private_tree_unreadable",
            Self::InsidePublic => "private_root_inside_public",
        }
    }

    /// Returns the process exit status for this refusal. Every preflight refusal exits with 2.
    #[must_use]
    pub const fn exit_status(self) -> u8 {
        2
    }
}

impl fmt::Display for PreflightError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.code())
    }
}

impl std::error::Error for PreflightError {}

/// Checks the private root before any database connection is opened.
///
/// # Errors
///
/// Returns the matching [`PreflightError`] for a missing, unreadable or public-nested root.
pub fn preflight(root: RootState) -> Result<(), PreflightError> {
    match root {
        RootState::Missing => Err(PreflightError::Missing),
        RootState::Unreadable => Err(PreflightError::Unreadable),
        RootState::Readable {
            inside_public: true,
        } => Err(PreflightError::InsidePublic),
        RootState::Readable {
            inside_public: false,
        } => Ok(()),
    }
}

/// The result of pairing one slug across the public and private trees.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum JoinOutcome {
    /// Both parts exist.
    Paired(String),
    /// The public pack exists, but its private part does not.
    PrivatePartMissing(String),
    /// A private directory exists with no public pack.
    OrphanPrivatePack(String),
}

impl JoinOutcome {
    /// Returns the stable code for a problem outcome. A paired slug has no problem code.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::Paired(_) => "paired",
            Self::PrivatePartMissing(_) => "private_part_missing",
            Self::OrphanPrivatePack(_) => "orphan_private_pack",
        }
    }
}

/// Pairs the public slugs with the private slugs.
///
/// Public slugs come first, in their order. Private slugs with no public pack come after.
#[must_use]
pub fn join(public_slugs: &[&str], private_slugs: &[&str]) -> Vec<JoinOutcome> {
    let mut outcomes: Vec<JoinOutcome> = public_slugs
        .iter()
        .map(|slug| {
            if private_slugs.contains(slug) {
                JoinOutcome::Paired((*slug).to_string())
            } else {
                JoinOutcome::PrivatePartMissing((*slug).to_string())
            }
        })
        .collect();
    outcomes.extend(
        private_slugs
            .iter()
            .filter(|slug| !public_slugs.contains(slug))
            .map(|slug| JoinOutcome::OrphanPrivatePack((*slug).to_string())),
    );
    outcomes
}
