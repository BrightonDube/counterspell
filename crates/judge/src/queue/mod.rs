//! Postgres job queue: claiming, leases, fenced writes, run events, terminal
//! writes and reaping. Postgres is the only source of truth for these rows.

pub mod claim;
pub mod emit;
pub(crate) mod fence;
pub mod lease;

use counterspell_common::ids::RunId;
use thiserror::Error;

/// Why a queue operation failed. `LeaseLost` is an expected outcome, not a crash.
#[derive(Debug, Error)]
pub enum QueueError {
    /// This worker no longer holds the job's lease, so it must not write.
    #[error("the lease for this run is no longer held by this worker")]
    LeaseLost,
    /// No row matched the request.
    #[error("the run was not found")]
    NotFound,
    /// The run is in a state that cannot take this event, for example after it finished.
    #[error("the run cannot take this event now")]
    EventRejected,
    /// The database rejected or failed the statement.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),
}

/// Proof that one worker holds one attempt of one job. Every fenced write needs it.
///
/// The fields name the claim exactly: the job, the run, the worker's id and the
/// attempt count at claim time. A later claim of the same job has a different
/// attempt count, so an old token can never match it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClaimToken {
    job_id: i64,
    run_id: RunId,
    lease_owner: String,
    attempts: i16,
}

impl ClaimToken {
    /// Builds a token from the values read in a claim. Only the claim code and the
    /// test kit create tokens, so a worker cannot invent a claim it does not hold.
    pub(crate) const fn new(
        job_id: i64,
        run_id: RunId,
        lease_owner: String,
        attempts: i16,
    ) -> Self {
        Self {
            job_id,
            run_id,
            lease_owner,
            attempts,
        }
    }

    /// The job row this claim belongs to.
    #[must_use]
    pub const fn job_id(&self) -> i64 {
        self.job_id
    }

    /// The run this claim is for.
    #[must_use]
    pub const fn run_id(&self) -> RunId {
        self.run_id
    }

    /// The id of the worker that holds the lease.
    #[must_use]
    pub fn lease_owner(&self) -> &str {
        &self.lease_owner
    }

    /// The attempt count stored on the job when this claim was made.
    #[must_use]
    pub const fn attempts(&self) -> i16 {
        self.attempts
    }
}
