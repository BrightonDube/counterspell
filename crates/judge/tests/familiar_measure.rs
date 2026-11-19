//! Tests for `measure_run` (efficiency-familiar T20): the failure matrix, event order and
//! error propagation. They use the recording run context and a scripted bench collector, so
//! no sandbox, database or host load is involved.

use counterspell_common::events::PublicEvent;
use counterspell_common::familiar::payload::MeasuredColumns;
use counterspell_common::familiar::ClassId;
use counterspell_common::status::{RunMode, Verdict};
use counterspell_judge::executor::ExecError;
use counterspell_judge::familiar::assess::BenchPoint;
use counterspell_judge::familiar::gate::BenchGate;
use counterspell_judge::familiar::load::LoadSource;
use counterspell_judge::familiar::{measure_run, BenchOutcome, BenchPlan, MeasureInput};
use counterspell_judge::testkit::{EventLog, RecordingContext, ScriptedCollector};
use std::io;

/// A load source that always reads the same value.
struct FixedLoad(f64);

impl LoadSource for FixedLoad {
    fn one_minute(&self) -> io::Result<f64> {
        Ok(self.0)
    }
}

/// A plan for the Submit mode, sized for the tests.
fn plan() -> BenchPlan {
    BenchPlan {
        mode: RunMode::Submit,
        sizes: vec![256, 512, 1024, 2048],
        size_param: 0,
        problem_slug: "two-sum".to_owned(),
        problem_version: 1,
        budget: std::time::Duration::from_secs(4),
    }
}

/// Four points that grow linearly in `n`, with a known spread.
fn linear_points() -> Vec<BenchPoint> {
    [256_u64, 512, 1024, 2048]
        .into_iter()
        .map(|n| BenchPoint {
            n,
            median_ns: n * 100,
            spread: Some(0.02),
        })
        .collect()
}

/// The `payload` of the `familiar` event in `events`, which must be present.
fn familiar_payload(events: &[PublicEvent]) -> serde_json::Value {
    events
        .iter()
        .find(|event| event.event_type() == "familiar")
        .map(PublicEvent::payload)
        .expect("a familiar event")
}

/// Runs `measure_run` for the given verdict, collector answer and load reading.
async fn measure(
    verdict: Option<Verdict>,
    answer: Result<BenchOutcome, ExecError>,
    load: f64,
    plan: Option<&BenchPlan>,
) -> (Result<MeasuredColumns, ExecError>, EventLog, usize) {
    let (ctx, log) = RecordingContext::without_stop(1);
    let gate = BenchGate::new(true);
    let mut collector = ScriptedCollector::new(answer);
    let input = MeasureInput {
        verdict,
        mode: RunMode::Submit,
        reference: ClassId::Linear,
        baseline: None,
        plan,
        max_load: 3.0,
    };
    let outcome = measure_run(&ctx, &gate, &FixedLoad(load), Some(&mut collector), &input).await;
    (outcome, log, collector.calls())
}

#[tokio::test]
async fn a_wrong_answer_never_runs_the_bench_and_gets_the_crack_mood() {
    let (outcome, log, calls) = measure(
        Some(Verdict::WrongAnswer),
        Ok(BenchOutcome::Complete(linear_points())),
        0.5,
        Some(&plan()),
    )
    .await;

    assert!(outcome.is_ok());
    assert_eq!(calls, 0, "a failed run must not be timed");
    let events = log.events();
    assert_eq!(events.len(), 1);
    let payload = familiar_payload(&events);
    assert_eq!(payload["mood"], 1);
}

#[tokio::test]
async fn a_problem_without_a_bench_is_pleased_with_no_bench_spec() {
    let (outcome, log, calls) =
        measure(Some(Verdict::Accepted), Ok(BenchOutcome::Failed), 0.5, None).await;

    assert!(outcome.is_ok());
    assert_eq!(calls, 0);
    let events = log.events();
    assert!(events
        .iter()
        .all(|event| event.event_type() != "complexity"));
    let payload = familiar_payload(&events);
    assert_eq!(payload["mood"], 4);
    assert_eq!(payload["reason"], "no_bench_spec");
}

#[tokio::test]
async fn a_busy_host_discards_the_bench_and_is_pleased_with_host_busy() {
    let (outcome, log, calls) = measure(
        Some(Verdict::Accepted),
        Ok(BenchOutcome::Complete(linear_points())),
        9.0,
        Some(&plan()),
    )
    .await;

    assert!(outcome.is_ok());
    assert_eq!(calls, 1, "the bench runs, then its result is discarded");
    let events = log.events();
    assert!(events
        .iter()
        .all(|event| event.event_type() != "complexity"));
    let payload = familiar_payload(&events);
    assert_eq!(payload["mood"], 4);
    assert_eq!(payload["reason"], "host_busy");
}

#[tokio::test]
async fn a_failed_bench_is_pleased_with_bench_failed() {
    let (outcome, log, _calls) = measure(
        Some(Verdict::Accepted),
        Ok(BenchOutcome::Failed),
        0.5,
        Some(&plan()),
    )
    .await;

    assert!(outcome.is_ok());
    let payload = familiar_payload(&log.events());
    assert_eq!(payload["mood"], 4);
    assert_eq!(payload["reason"], "bench_failed");
}

#[tokio::test]
async fn an_incomplete_bench_is_pleased_with_bench_incomplete() {
    let (outcome, log, _calls) = measure(
        Some(Verdict::Accepted),
        Ok(BenchOutcome::Incomplete(linear_points()[..2].to_vec())),
        0.5,
        Some(&plan()),
    )
    .await;

    assert!(outcome.is_ok());
    let payload = familiar_payload(&log.events());
    assert_eq!(payload["mood"], 4);
    assert_eq!(payload["reason"], "bench_incomplete");
}

#[tokio::test]
async fn a_fitted_bench_ends_with_the_familiar_event() {
    let (outcome, log, calls) = measure(
        Some(Verdict::Accepted),
        Ok(BenchOutcome::Complete(linear_points())),
        0.5,
        Some(&plan()),
    )
    .await;

    assert!(outcome.is_ok());
    assert_eq!(calls, 1);
    let types: Vec<&str> = log.events().iter().map(PublicEvent::event_type).collect();
    assert_eq!(types.last(), Some(&"familiar"));
    if types.contains(&"complexity") {
        assert_eq!(
            types[types.len() - 2],
            "complexity",
            "complexity comes just before familiar"
        );
    }
}

#[tokio::test]
async fn a_collector_infrastructure_error_is_returned_and_no_mood_is_emitted() {
    let (outcome, log, _calls) = measure(
        Some(Verdict::Accepted),
        Err(ExecError::Infrastructure("sandbox down".to_owned())),
        0.5,
        Some(&plan()),
    )
    .await;

    assert_eq!(
        outcome,
        Err(ExecError::Infrastructure("sandbox down".to_owned()))
    );
    assert!(
        log.events().is_empty(),
        "a retried run must not show a mood"
    );
}

#[tokio::test]
async fn a_restarted_attempt_gets_a_fresh_result_and_the_gate_is_released() {
    let gate = BenchGate::new(true);
    let input = MeasureInput {
        verdict: Some(Verdict::Accepted),
        mode: RunMode::Submit,
        reference: ClassId::Linear,
        baseline: None,
        plan: Some(&plan()),
        max_load: 3.0,
    };

    // Attempt 1 fails inside the sandbox. It must show no mood and must not keep the gate.
    let (first_ctx, first_log) = RecordingContext::without_stop(1);
    let mut failing = ScriptedCollector::new(Err(ExecError::Infrastructure("daemon".to_owned())));
    let first = measure_run(
        &first_ctx,
        &gate,
        &FixedLoad(0.5),
        Some(&mut failing),
        &input,
    )
    .await;
    assert!(first.is_err());
    assert_eq!(first_log.events().len(), 0);

    // Attempt 2 is a new run context. It must emit exactly one fresh familiar event.
    let (second_ctx, second_log) = RecordingContext::without_stop(2);
    let mut working = ScriptedCollector::new(Ok(BenchOutcome::Complete(linear_points())));
    let second = measure_run(
        &second_ctx,
        &gate,
        &FixedLoad(0.5),
        Some(&mut working),
        &input,
    )
    .await;
    assert!(second.is_ok());
    let events = second_log.events();
    let familiar_count = events
        .iter()
        .filter(|event| event.event_type() == "familiar")
        .count();
    assert_eq!(familiar_count, 1);
}

#[tokio::test]
async fn a_fitted_bench_without_a_usable_baseline_is_capped_at_pleased() {
    // The failure matrix: a baseline that is missing or stale caps the mood at 4 (`no_baseline`),
    // even when the measured class matches the reference.
    let (ctx, log) = RecordingContext::without_stop(1);
    let gate = BenchGate::new(true);
    let mut collector = ScriptedCollector::new(Ok(BenchOutcome::Complete(linear_points())));
    let plan = plan();
    let input = MeasureInput {
        verdict: Some(Verdict::Accepted),
        mode: RunMode::Submit,
        reference: ClassId::Linear,
        baseline: None,
        plan: Some(&plan),
        max_load: 3.0,
    };

    let outcome = measure_run(&ctx, &gate, &FixedLoad(0.5), Some(&mut collector), &input).await;

    assert!(outcome.is_ok());
    let payload = familiar_payload(&log.events());
    assert_eq!(payload["mood"], 4);
    assert_eq!(payload["reason"], "no_baseline");
}

#[tokio::test]
async fn a_stop_signal_from_the_collector_is_returned_unchanged() {
    let (outcome, log, _calls) = measure(
        Some(Verdict::Accepted),
        Err(ExecError::Stopped),
        0.5,
        Some(&plan()),
    )
    .await;

    assert_eq!(outcome, Err(ExecError::Stopped));
    assert_eq!(log.events().len(), 0);
}
