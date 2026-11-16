//! The run state machine: which status may follow which, and how statuses
//! map to job states. This is the only Rust copy of the table. The database
//! function `cs_status_transition_allowed` mirrors it and a test compares them.

use crate::events::PublicEvent;
use crate::status::{FailureReason, JobState, RunStatus};

/// The run status that follows an event, or `None` when the event is not allowed now.
///
/// A terminal run accepts no event. Events that do not change the status return it
/// unchanged. Every other answer must be an allowed move in [`can_transition`].
#[must_use]
pub fn status_after_event(current: RunStatus, event: &PublicEvent) -> Option<RunStatus> {
    if current.is_terminal() {
        return None;
    }
    let target = match event {
        PublicEvent::Restarted { .. }
        | PublicEvent::ShrinkStep(_)
        | PublicEvent::Ghost(_)
        | PublicEvent::BenchPoint(_)
        | PublicEvent::Complexity(_)
        | PublicEvent::Familiar(_) => current,
        PublicEvent::Compiling => RunStatus::Compiling,
        PublicEvent::Test(_) => RunStatus::Running,
        PublicEvent::GhostHunting => RunStatus::GhostHunting,
        PublicEvent::Done(_) => RunStatus::Completed,
        PublicEvent::Error(error) => match error.reason() {
            FailureReason::Expired => RunStatus::Expired,
            FailureReason::RetriesExhausted => RunStatus::InternalError,
            FailureReason::Cancelled
            | FailureReason::ProblemUnavailable
            | FailureReason::LanguageUnavailable => RunStatus::Failed,
        },
    };
    can_transition(current, target).then_some(target)
}

/// The event name that ends a run in `status`, or `None` for a run that is not finished.
#[must_use]
pub const fn terminal_event_kind(status: RunStatus) -> Option<&'static str> {
    match status {
        RunStatus::Completed => Some("done"),
        RunStatus::Failed | RunStatus::Expired | RunStatus::InternalError => Some("error"),
        RunStatus::Queued
        | RunStatus::Claimed
        | RunStatus::Compiling
        | RunStatus::Running
        | RunStatus::GhostHunting => None,
    }
}

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
    use crate::events::{BoundedJson, DonePayload, ErrorPayload, PublicTest};
    use crate::status::{TestVerdict, Verdict};
    use proptest::prelude::*;

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
    fn first_test_moves_a_compiling_run_to_running() {
        let test = PublicEvent::Test(PublicTest::Hidden {
            ordinal: 1,
            verdict: TestVerdict::Pass,
            time_ms: 4,
        });
        assert_eq!(
            status_after_event(RunStatus::Compiling, &test),
            Some(RunStatus::Running)
        );
        assert_eq!(
            status_after_event(RunStatus::Running, &test),
            Some(RunStatus::Running)
        );
        // The judge always writes `compiling` before the first test, so a claimed
        // run may not jump straight to a test result.
        assert_eq!(status_after_event(RunStatus::Claimed, &test), None);
    }

    #[test]
    fn error_reason_picks_the_terminal_status() {
        let from_queue = [
            (FailureReason::Expired, RunStatus::Expired),
            (FailureReason::Cancelled, RunStatus::Failed),
        ];
        for (reason, status) in from_queue {
            let event = PublicEvent::Error(ErrorPayload::for_reason(reason));
            assert_eq!(
                status_after_event(RunStatus::Queued, &event),
                Some(status),
                "queued run with {reason:?}"
            );
        }
        let while_running = [
            (FailureReason::RetriesExhausted, RunStatus::InternalError),
            (FailureReason::Cancelled, RunStatus::Failed),
        ];
        for (reason, status) in while_running {
            let event = PublicEvent::Error(ErrorPayload::for_reason(reason));
            assert_eq!(
                status_after_event(RunStatus::Running, &event),
                Some(status),
                "running run with {reason:?}"
            );
        }
    }

    #[test]
    fn expiry_is_refused_once_the_run_is_claimed() {
        let event = PublicEvent::Error(ErrorPayload::for_reason(FailureReason::Expired));
        assert_eq!(status_after_event(RunStatus::Claimed, &event), None);
    }

    #[test]
    fn done_completes_a_running_run_but_not_a_queued_one() {
        let done = PublicEvent::Done(DonePayload::new(Verdict::Accepted, 3, 3, 100, None));
        assert_eq!(
            status_after_event(RunStatus::Running, &done),
            Some(RunStatus::Completed)
        );
        assert_eq!(status_after_event(RunStatus::Queued, &done), None);
    }

    #[test]
    fn a_terminal_run_accepts_no_event() {
        let events = [
            PublicEvent::Compiling,
            PublicEvent::Restarted { attempt: 2 },
            PublicEvent::Error(ErrorPayload::for_reason(FailureReason::Cancelled)),
        ];
        for status in [
            RunStatus::Completed,
            RunStatus::Failed,
            RunStatus::Expired,
            RunStatus::InternalError,
        ] {
            for event in &events {
                assert_eq!(status_after_event(status, event), None, "{status:?}");
            }
        }
    }

    #[test]
    fn terminal_event_kind_names_the_event_that_ends_each_status() {
        assert_eq!(terminal_event_kind(RunStatus::Completed), Some("done"));
        assert_eq!(terminal_event_kind(RunStatus::Failed), Some("error"));
        assert_eq!(terminal_event_kind(RunStatus::Expired), Some("error"));
        assert_eq!(terminal_event_kind(RunStatus::InternalError), Some("error"));
        assert_eq!(terminal_event_kind(RunStatus::Running), None);
        assert_eq!(terminal_event_kind(RunStatus::Queued), None);
    }

    fn any_event() -> impl Strategy<Value = PublicEvent> {
        prop_oneof![
            any::<u16>().prop_map(|attempt| PublicEvent::Restarted { attempt }),
            Just(PublicEvent::Compiling),
            Just(PublicEvent::GhostHunting),
            Just(PublicEvent::Test(PublicTest::Hidden {
                ordinal: 1,
                verdict: TestVerdict::Pass,
                time_ms: 1,
            })),
            Just(PublicEvent::Ghost(
                BoundedJson::from_serialize(&serde_json::json!({})).expect("empty object"),
            )),
            Just(PublicEvent::Done(DonePayload::new(
                Verdict::Accepted,
                1,
                1,
                100,
                None,
            ))),
            prop::sample::select(FailureReason::ALL.to_vec())
                .prop_map(|reason| PublicEvent::Error(ErrorPayload::for_reason(reason))),
        ]
    }

    proptest! {
        #[test]
        fn a_random_walk_of_events_only_makes_table_moves(
            events in prop::collection::vec(any_event(), 0..40)
        ) {
            let mut current = RunStatus::Queued;
            for event in &events {
                if let Some(next) = status_after_event(current, event) {
                    prop_assert!(can_transition(current, next), "{:?} -> {:?}", current, next);
                    current = next;
                }
            }
            prop_assert_eq!(current.is_active(), job_state_for(current) == JobState::Claimed);
        }

        #[test]
        fn once_terminal_no_event_can_move_a_run(
            start in prop::sample::select(RunStatus::ALL.to_vec()),
            events in prop::collection::vec(any_event(), 1..20),
        ) {
            if start.is_terminal() {
                for event in &events {
                    prop_assert_eq!(status_after_event(start, event), None);
                }
            }
        }
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
