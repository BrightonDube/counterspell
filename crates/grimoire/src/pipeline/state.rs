//! The stage and status of one pack, and the moves allowed between statuses.
//!
//! The state file holds only a slug, a stage and a status. Unknown keys are
//! refused, so a hidden value cannot be stored in it by mistake (PIP-32, PIP-45).

use std::fmt;

use serde::{Deserialize, Serialize};

/// A pipeline stage, S0 (topic) to S13 (summary and pull request).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Stage {
    S0,
    S1,
    S2,
    S3,
    S4,
    S5,
    S6,
    S7,
    S8,
    S9,
    S10,
    S11,
    S12,
    S13,
}

/// The status of a pack in the review loop (design §7).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PackStatus {
    InProgress,
    StaticComplete,
    SandboxChecked,
    InReview,
    Rework,
    Stale,
    Passed,
    Escalated,
    AwaitingApproval,
    Approved,
}

impl PackStatus {
    /// Returns `true` when design §7 allows a pack to move from `self` to `next`.
    #[must_use]
    pub fn can_move_to(self, next: PackStatus) -> bool {
        use PackStatus::{
            Approved, AwaitingApproval, Escalated, InProgress, InReview, Passed, Rework,
            SandboxChecked, Stale, StaticComplete,
        };
        match self {
            InProgress => next == StaticComplete,
            StaticComplete => next == SandboxChecked,
            SandboxChecked | Rework | Stale => next == InReview,
            InReview => matches!(next, Rework | Escalated | Stale | Passed),
            Passed => next == AwaitingApproval,
            AwaitingApproval => matches!(next, Approved | Rework),
            // Escalated and Approved are terminal for the automated pipeline.
            Escalated | Approved => false,
        }
    }
}

/// A refused move between two statuses. Carries both ends so the caller can report them.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TransitionError {
    pub from: PackStatus,
    pub to: PackStatus,
}

impl fmt::Display for TransitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "a pack in {:?} cannot move to {:?}",
            self.from, self.to
        )
    }
}

impl std::error::Error for TransitionError {}

/// The state file for one pack, `.kiro/pipeline/state/<slug>.toml`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PackState {
    pub slug: String,
    pub stage: Stage,
    pub status: PackStatus,
}

impl PackState {
    /// Parses a state file from its text. Unknown keys and unknown statuses are errors.
    ///
    /// # Errors
    ///
    /// Returns the TOML error when the text is not a valid state file.
    pub fn from_toml(text: &str) -> Result<Self, toml::de::Error> {
        toml::from_str(text)
    }

    /// Writes the state as TOML text.
    ///
    /// # Errors
    ///
    /// Returns the TOML error when the state cannot be serialised.
    pub fn to_toml(&self) -> Result<String, toml::ser::Error> {
        toml::to_string(self)
    }

    /// Moves the pack to `next` when design §7 allows it. A refused move leaves the status as it was.
    ///
    /// # Errors
    ///
    /// Returns [`TransitionError`] when the move is not allowed.
    pub fn move_to(&mut self, next: PackStatus) -> Result<(), TransitionError> {
        if !self.status.can_move_to(next) {
            return Err(TransitionError {
                from: self.status,
                to: next,
            });
        }
        self.status = next;
        Ok(())
    }
}
