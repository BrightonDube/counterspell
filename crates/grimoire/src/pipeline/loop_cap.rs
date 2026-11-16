//! The review round counter and escalation (PIP-25, PIP-26, design §7).
//!
//! A new content hash starts a new round. The same hash does not, so re-running a
//! review never uses up a round. The cap of three is checked in one place.

use std::fmt;

use super::state::PackStatus;

/// The most review rounds one pack may have (PIP-25).
pub const MAX_ROUNDS: u8 = 3;

/// Why a round cannot start or a result cannot be given.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopError {
    /// A fourth round was asked for. The pack is escalated instead.
    RoundLimitReached,
    /// The result was asked for before any round started.
    NoRoundStarted,
}

impl fmt::Display for LoopError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LoopError::RoundLimitReached => {
                write!(formatter, "a pack gets at most {MAX_ROUNDS} review rounds")
            }
            LoopError::NoRoundStarted => write!(formatter, "no review round has started"),
        }
    }
}

impl std::error::Error for LoopError {}

/// The round counter for one pack.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct ReviewLoop {
    round: u8,
    last_hash: Option<String>,
}

impl ReviewLoop {
    /// A loop with no rounds yet.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// The number of the current round, or 0 before the first one.
    #[must_use]
    pub fn round(&self) -> u8 {
        self.round
    }

    /// Starts a round on `content_sha256`.
    ///
    /// A hash equal to the last round's hash returns the current round number and changes nothing.
    /// A new hash starts the next round.
    ///
    /// # Errors
    ///
    /// Returns [`LoopError::RoundLimitReached`] when a new hash would be round four.
    pub fn start_round(&mut self, content_sha256: &str) -> Result<u8, LoopError> {
        if self.last_hash.as_deref() == Some(content_sha256) {
            return Ok(self.round);
        }
        if self.round >= MAX_ROUNDS {
            return Err(LoopError::RoundLimitReached);
        }
        self.round += 1;
        self.last_hash = Some(content_sha256.to_owned());
        Ok(self.round)
    }

    /// The status a pack moves to after the current round: `Passed`, `Rework`, or `Escalated` on round three.
    ///
    /// # Errors
    ///
    /// Returns [`LoopError::NoRoundStarted`] when no round has started.
    pub fn finish_round(&self, passed: bool) -> Result<PackStatus, LoopError> {
        if self.round == 0 {
            return Err(LoopError::NoRoundStarted);
        }
        Ok(if passed {
            PackStatus::Passed
        } else if self.round >= MAX_ROUNDS {
            PackStatus::Escalated
        } else {
            PackStatus::Rework
        })
    }

    /// Returns `true` when `content_sha256` differs from the hash of the current round (PIP-26).
    #[must_use]
    pub fn is_stale(&self, content_sha256: &str) -> bool {
        self.last_hash
            .as_deref()
            .is_some_and(|last| last != content_sha256)
    }
}
