//! Tests for the executor seam (run-lifecycle T16, design Path 7).
//!
//! These run without a database. A recording context stands in for the queue, so each
//! test can check exactly which public events an executor emitted and what it returned.
//!
//! Run with `cargo test -p counterspell-judge executor`.

use counterspell_common::events::PublicEvent;
use counterspell_common::ids::RunId;
use counterspell_common::status::{FailureReason, RunMode, Verdict};
use counterspell_judge::executor::{ExecError, RunExecutor, RunInput, RunOutcome};
use counterspell_judge::testkit::{RecordingContext, Script, ScriptedExecutor};
use std::time::Duration;

/// The input every test passes to the executor.
fn input() -> RunInput {
    RunInput {
        run_id: RunId::new(),
        problem_slug: "two-sum".to_owned(),
        lang: "python".to_owned(),
        mode: RunMode::Submit,
        code: "print(1)".to_owned(),
        attempt: 1,
    }
}

/// The event names recorded so far, in order.
fn names(events: &[PublicEvent]) -> Vec<&'static str> {
    events.iter().map(PublicEvent::event_type).collect()
}

#[tokio::test]
async fn executor_recording_context_keeps_events_in_order() {
    let (ctx, log) = RecordingContext::without_stop(1);
    let executor = ScriptedExecutor::new(Script::Pass {
        tests: 2,
        delay: Duration::ZERO,
    });

    executor.execute(input(), ctx).await.unwrap();

    assert_eq!(
        names(&log.events()),
        ["compiling", "test", "test"],
        "the executor emits compiling before its first test"
    );
}

#[tokio::test]
async fn executor_pass_returns_an_accepted_completion() {
    let (ctx, _log) = RecordingContext::without_stop(1);
    let executor = ScriptedExecutor::new(Script::Pass {
        tests: 3,
        delay: Duration::ZERO,
    });

    let outcome = executor.execute(input(), ctx).await.unwrap();

    let RunOutcome::Completed(result) = outcome else {
        panic!("a passing script must complete");
    };
    assert_eq!(result.verdict, Verdict::Accepted);
    assert_eq!((result.passed, result.total), (3, 3));
    assert_eq!(result.tests.len(), 3);
}

#[tokio::test]
async fn executor_fail_returns_the_scripted_verdict() {
    let (ctx, _log) = RecordingContext::without_stop(1);
    let executor = ScriptedExecutor::new(Script::Fail {
        verdict: Verdict::WrongAnswer,
    });

    let RunOutcome::Completed(result) = executor.execute(input(), ctx).await.unwrap() else {
        panic!("a wrong answer is still a completed run");
    };
    assert_eq!(result.verdict, Verdict::WrongAnswer);
    assert_eq!(result.passed, 0);
}

#[tokio::test]
async fn executor_hang_stops_only_when_the_stop_signal_fires() {
    let (ctx, _log, stop) = RecordingContext::with_stop(1);
    let executor = ScriptedExecutor::new(Script::Hang);
    let task = tokio::spawn(async move { executor.execute(input(), ctx).await });

    stop.send(true).unwrap();

    let result = task.await.unwrap();
    assert!(matches!(result, Err(ExecError::Stopped)), "{result:?}");
}

#[tokio::test]
async fn executor_infrastructure_failure_is_reported_not_a_verdict() {
    let (ctx, _log) = RecordingContext::without_stop(1);
    let executor = ScriptedExecutor::new(Script::Infra("sandbox image missing".to_owned()));

    let result = executor.execute(input(), ctx).await;

    assert!(
        matches!(result, Err(ExecError::Infrastructure(_))),
        "{result:?}"
    );
}

#[tokio::test]
async fn executor_crash_after_events_stops_without_a_result() {
    let (ctx, log) = RecordingContext::without_stop(1);
    let executor = ScriptedExecutor::new(Script::CrashAfter(2));

    let result = executor.execute(input(), ctx).await;

    assert!(result.is_err(), "a crash never returns a completion");
    assert_eq!(
        log.events().len(),
        2,
        "exactly two events were emitted first"
    );
}

#[tokio::test]
async fn executor_panic_is_visible_to_the_caller_as_a_join_error() {
    let (ctx, _log) = RecordingContext::without_stop(1);
    let executor = ScriptedExecutor::new(Script::Panic);

    let task = tokio::spawn(async move { executor.execute(input(), ctx).await });

    let error = task
        .await
        .expect_err("the panic must surface as a JoinError");
    assert!(error.is_panic());
}

#[tokio::test]
async fn executor_failed_reason_is_passed_through() {
    let (ctx, _log) = RecordingContext::without_stop(1);
    let executor = ScriptedExecutor::new(Script::Unavailable(FailureReason::LanguageUnavailable));

    let RunOutcome::Failed(reason) = executor.execute(input(), ctx).await.unwrap() else {
        panic!("an unavailable language ends as a failure");
    };
    assert_eq!(reason, FailureReason::LanguageUnavailable);
}
