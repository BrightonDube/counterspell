//! The run state machine: which status may follow which, and how statuses
//! map to job states. This is the only Rust copy of the table. The database
//! function `cs_status_transition_allowed` mirrors it and a test compares them.

use crate::status::{JobState, RunStatus};

/// Returns true when a run may move from `from` to `to`.
///
/// Keeping the same status is not a transition, so it is always allowed.
/// Terminal statuses allow nothing else.
#[must_use]
pub fn can_transition(from: RunStatus, to: RunStatus) -> bool {
    use RunStatus::{Claimed, Compiling, Completed, Expired, Failed, GhostHunting, InternalError};
    use RunStatus::{Queued, Running};
    match from {
        Queued => matches!(to, Queued | Claimed | Expired | Failed),
        Claimed => matches!(to, Claimed | Compiling | Queued | Failed | InternalError),
        Compiling => matches!(
            to,
            Compiling | Running | Completed | Queued | Failed | InternalError
        ),
        Running => matches!(
            to,
            Running | GhostHunting | Completed | Queued | Failed | InternalError
        ),
        GhostHunting => matches!(
            to,
            GhostHunting | Completed | Queued | Failed | InternalError
        ),
        Completed | Failed | Expired | InternalError => to == from,
    }
}

/// The job state that matches a run status. The mapping is total and fixed.
#[must_use]
pub const fn job_state_for(status: RunStatus) -> JobState {
    match status {
        RunStatus::Queued => JobState::Queued,
        RunStatus::Claimed
        | RunStatus::Compiling
        | RunStatus::Running
        | RunStatus::GhostHunting => JobState::Claimed,
        RunStatus::Completed => JobState::Succeeded,
        RunStatus::Failed | RunStatus::Expired => JobState::Failed,
        RunStatus::InternalError => JobState::Dead,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn queued_run_can_be_claimed_expired_or_failed() {
        assert!(can_transition(RunStatus::Queued, RunStatus::Claimed));
        assert!(can_transition(RunStatus::Queued, RunStatus::Expired));
        assert!(can_transition(RunStatus::Queued, RunStatus::Failed));
        assert!(!can_transition(RunStatus::Queued, RunStatus::Completed));
    }

    #[test]
    fn a_lost_lease_sends_an_active_run_back_to_the_queue() {
        assert!(can_transition(RunStatus::Running, RunStatus::Queued));
        assert!(can_transition(RunStatus::GhostHunting, RunStatus::Queued));
    }

    #[test]
    fn terminal_runs_never_change() {
        for from in [
            RunStatus::Completed,
            RunStatus::Failed,
            RunStatus::Expired,
            RunStatus::InternalError,
        ] {
            for to in RunStatus::ALL {
                if *to != from {
                    assert!(
                        !can_transition(from, *to),
                        "{from:?} must not move to {to:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn job_state_follows_the_mapping_table() {
        assert_eq!(job_state_for(RunStatus::Queued), JobState::Queued);
        assert_eq!(job_state_for(RunStatus::Running), JobState::Claimed);
        assert_eq!(job_state_for(RunStatus::Completed), JobState::Succeeded);
        assert_eq!(job_state_for(RunStatus::Failed), JobState::Failed);
        assert_eq!(job_state_for(RunStatus::Expired), JobState::Failed);
        assert_eq!(job_state_for(RunStatus::InternalError), JobState::Dead);
    }

    #[test]
    fn active_means_the_job_is_claimed() {
        for status in RunStatus::ALL {
            assert_eq!(
                status.is_active(),
                job_state_for(*status) == JobState::Claimed,
                "mismatch for {status:?}"
            );
        }
    }
}
