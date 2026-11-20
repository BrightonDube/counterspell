// SPDX-License-Identifier: MIT
//! Model tests for `SandboxBenchCollector` (language-drivers T08): the scripted
//! fake backend stands in for the daemon, so each test sees exactly which call
//! lines the bench wrote, how many, and what the collector returned — without
//! Docker.

use std::time::Duration;

use counterspell_common::status::RunMode;
use counterspell_engine::Schema;
use counterspell_judge::executor::ExecError;
use counterspell_judge::familiar::assess::BenchPoint;
use counterspell_judge::familiar::{BenchCollector, BenchOutcome, BenchPlan};
use counterspell_judge::languages::LanguageCatalog;
use counterspell_judge::sandbox::backend::{BackendSession, InfraError};
use counterspell_judge::sandbox::bench::{
    DiscardBench, SandboxBenchCollector, TIMED_REPS, WARM_UPS,
};
use counterspell_judge::sandbox::config::OciRuntime;
use counterspell_judge::sandbox::session::{open_session, Opened};
use counterspell_judge::sandbox::spec::SessionSpec;
use counterspell_judge::sandbox::testkit::{Behaviour, FakeBackend};
use counterspell_judge::testkit::RecordingContext;
use tokio::sync::watch;
use uuid::Uuid;

mod support;
use support::shipped_catalogue;

const RUN: &str = "018b6b24-8a2c-7b3d-9c1e-7b2c5d4e6f70";
const ARCHIVE: &[u8] = b"not really a tar, the fake never reads it";

fn session_spec(catalog: &LanguageCatalog, ordinal: u32) -> SessionSpec {
    SessionSpec::new(
        catalog.get("python").unwrap(),
        Duration::from_secs(20),
        OciRuntime::Runc,
        Uuid::parse_str(RUN).unwrap(),
        "worker-1",
        ordinal,
        1_800_000_000,
    )
    .unwrap()
}

/// A two-parameter signature: the list parameter (index 0) is the size parameter.
fn schema() -> Schema {
    Schema::Tuple {
        items: vec![
            Schema::IntArray {
                min_len: 0,
                max_len: 4,
                min: 0,
                max: 100,
                sorted: false,
                unique: false,
            },
            Schema::Int { min: 0, max: 10 },
        ],
    }
}

fn plan(sizes: &[u64], budget: Duration) -> BenchPlan {
    BenchPlan {
        mode: RunMode::Submit,
        sizes: sizes.to_vec(),
        size_param: 0,
        problem_slug: "add".to_owned(),
        problem_version: 1,
        budget,
    }
}

fn answer(id: u64, out: i64) -> Behaviour {
    Behaviour::Answer(format!("{{\"t\":{id},\"out\":{out}}}\n").into_bytes())
}

async fn opened(backend: &FakeBackend, catalog: &LanguageCatalog) -> Opened<impl BackendSession> {
    open_session(
        backend,
        &session_spec(catalog, 1),
        catalog.get("python").unwrap(),
        ARCHIVE,
    )
    .await
    .expect("the session opens")
}

/// The calls the fake recorded, as raw lines.
fn call_lines(backend: &FakeBackend) -> Vec<Vec<u8>> {
    backend.ledger().inputs()
}

#[tokio::test]
async fn a_complete_bench_times_every_size_and_every_rep() {
    let (_dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    // 3 sizes × (2 warm-ups + 5 timed) = 21 calls, ids 0..=20.
    backend.script_program((0..21).map(|id| answer(id, 5)));
    let mut opened = opened(&backend, &catalog).await;
    let (_tx, mut stop) = watch::channel(false);
    let (ctx, _log) = RecordingContext::without_stop(1);

    let outcome = {
        let mut collector = SandboxBenchCollector::new(
            &mut opened,
            catalog.get("python").unwrap(),
            schema(),
            Duration::from_secs(5),
            &ctx,
            &mut stop,
        );
        collector
            .collect(&plan(&[4, 16, 64], Duration::from_secs(30)))
            .await
            .expect("the bench completes")
    };

    let BenchOutcome::Complete(points) = outcome else {
        panic!("a scripted-complete bench must be Complete: {outcome:?}");
    };
    assert_eq!(
        points.iter().map(|p| p.n).collect::<Vec<_>>(),
        vec![4, 16, 64],
        "one point per size, in order"
    );
    for point in &points {
        let BenchPoint { median_ns, .. } = point;
        assert!(*median_ns > 0, "every timed size has a positive median");
    }

    let lines = call_lines(&backend);
    assert_eq!(lines.len(), 21, "one line per bench call");
    for (index, line) in lines.iter().enumerate() {
        let text = String::from_utf8_lossy(line);
        assert!(
            text.contains(&format!("\"t\":{index}}}")),
            "call ids are sequential: {text}"
        );
    }
    // Every repeat of a size sends the same seeded input (only the call id differs).
    for size in 0..3 {
        let base = size * (WARM_UPS + TIMED_REPS);
        for rep in 1..(WARM_UPS + TIMED_REPS) {
            let before = String::from_utf8_lossy(&lines[base]);
            let after = String::from_utf8_lossy(&lines[base + rep]);
            assert_eq!(
                before.split(",\"t\":").next(),
                after.split(",\"t\":").next(),
                "every repeat of a size uses the same seeded input"
            );
        }
    }
}

#[tokio::test]
async fn a_zero_budget_reports_incomplete_before_any_call() {
    let (_dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    let mut opened = opened(&backend, &catalog).await;
    let (_tx, mut stop) = watch::channel(false);
    let (ctx, _log) = RecordingContext::without_stop(1);

    let outcome = {
        let mut collector = SandboxBenchCollector::new(
            &mut opened,
            catalog.get("python").unwrap(),
            schema(),
            Duration::from_secs(5),
            &ctx,
            &mut stop,
        );
        collector
            .collect(&plan(&[4, 16], Duration::ZERO))
            .await
            .expect("the bench completes")
    };

    assert_eq!(
        outcome,
        BenchOutcome::Incomplete(vec![]),
        "no budget means no timed points"
    );
    assert!(
        call_lines(&backend).is_empty(),
        "nothing was sent when the budget was already spent"
    );
}

#[tokio::test]
async fn a_driver_crash_fails_the_bench() {
    let (_dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    backend.script_program((0..7).map(|id| answer(id, 5)).chain([Behaviour::Exit {
        code: 1,
        oom_kill: 0,
    }]));
    let mut opened = opened(&backend, &catalog).await;
    let (_tx, mut stop) = watch::channel(false);
    let (ctx, _log) = RecordingContext::without_stop(1);

    let outcome = {
        let mut collector = SandboxBenchCollector::new(
            &mut opened,
            catalog.get("python").unwrap(),
            schema(),
            Duration::from_secs(5),
            &ctx,
            &mut stop,
        );
        collector
            .collect(&plan(&[4, 16], Duration::from_secs(30)))
            .await
            .expect("a crashed bench is a bench outcome, not an error")
    };

    assert_eq!(outcome, BenchOutcome::Failed);
}

#[tokio::test]
async fn a_hang_fails_the_bench_within_the_call_deadline() {
    let (_dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    backend.script_program([Behaviour::Hang]);
    let mut opened = opened(&backend, &catalog).await;
    let (_tx, mut stop) = watch::channel(false);
    let (ctx, _log) = RecordingContext::without_stop(1);

    let started = std::time::Instant::now();
    let outcome = {
        let mut collector = SandboxBenchCollector::new(
            &mut opened,
            catalog.get("python").unwrap(),
            schema(),
            Duration::from_millis(50),
            &ctx,
            &mut stop,
        );
        collector
            .collect(&plan(&[4], Duration::from_secs(30)))
            .await
            .expect("a hung bench is a bench outcome, not an error")
    };

    assert_eq!(outcome, BenchOutcome::Failed);
    assert!(
        started.elapsed() < Duration::from_secs(5),
        "the call deadline bounds the hang"
    );
}

#[tokio::test]
async fn the_stop_signal_fails_the_bench_as_stopped() {
    let (_dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    let mut opened = opened(&backend, &catalog).await;
    let (_tx, mut stop) = watch::channel(true);
    let (ctx, _log) = RecordingContext::without_stop(1);

    let result = {
        let mut collector = SandboxBenchCollector::new(
            &mut opened,
            catalog.get("python").unwrap(),
            schema(),
            Duration::from_secs(5),
            &ctx,
            &mut stop,
        );
        collector
            .collect(&plan(&[4], Duration::from_secs(30)))
            .await
    };

    assert!(
        matches!(result, Err(ExecError::Stopped)),
        "a stopped bench reports Stopped: {result:?}"
    );
}

#[tokio::test]
async fn sandbox_failures_are_infrastructure_never_a_verdict() {
    let (_dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    // `open_session` is ops 1-6 (create, read_policy, start, two execs, driver);
    // op 7 is the bench driver's open_driver.
    backend.fail_call(7, InfraError::ApiTimeout);
    let mut opened = opened(&backend, &catalog).await;
    let (_tx, mut stop) = watch::channel(false);
    let (ctx, _log) = RecordingContext::without_stop(1);

    let result = {
        let mut collector = SandboxBenchCollector::new(
            &mut opened,
            catalog.get("python").unwrap(),
            schema(),
            Duration::from_secs(5),
            &ctx,
            &mut stop,
        );
        collector
            .collect(&plan(&[4], Duration::from_secs(30)))
            .await
    };

    assert!(
        matches!(result, Err(ExecError::Infrastructure(_))),
        "a daemon failure is infrastructure, never a bench verdict: {result:?}"
    );
    assert_eq!(
        backend.fault_fired_on(),
        Some("open_driver"),
        "the fault fired on the bench driver open"
    );
}

/// efficiency-familiar T25 (R13.1, R13.2): pack validation times the reference through this
/// same collector. Prepared inputs are sent exactly as given, every size still gets the two
/// warm-ups and five timed calls, and a sink that keeps nothing receives no run event.
#[tokio::test]
async fn a_bench_over_prepared_inputs_sends_those_inputs_and_writes_no_run_record() {
    let (_dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    let calls = 2 * (WARM_UPS + TIMED_REPS);
    backend.script_program((0..calls as u64).map(|id| answer(id, 5)));
    let mut opened = opened(&backend, &catalog).await;
    let (_tx, mut stop) = watch::channel(false);
    let inputs = vec![
        (4_u64, serde_json::json!([[9, 8, 7, 6], 1])),
        (
            16_u64,
            serde_json::json!([[5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5, 5], 2]),
        ),
    ];
    let sink = DiscardBench;

    let outcome = {
        let mut collector = SandboxBenchCollector::for_inputs(
            &mut opened,
            catalog.get("python").unwrap(),
            &inputs,
            Duration::from_secs(5),
            &sink,
            &mut stop,
        );
        collector
            .collect(&plan(&[4, 16], Duration::from_secs(30)))
            .await
            .expect("the bench completes")
    };

    let BenchOutcome::Complete(points) = outcome else {
        panic!("a scripted-complete bench must be Complete: {outcome:?}");
    };
    assert_eq!(points.iter().map(|p| p.n).collect::<Vec<_>>(), vec![4, 16]);
    let lines = call_lines(&backend);
    assert_eq!(lines.len(), calls, "warm-ups and timed calls for each size");
    let first = String::from_utf8_lossy(&lines[0]);
    let last = String::from_utf8_lossy(&lines[calls - 1]);
    assert!(
        first.contains("[[9,8,7,6],1]"),
        "the prepared input: {first}"
    );
    assert!(
        last.contains("[[5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5],2]"),
        "{last}"
    );
}

/// A prepared bench without an input for a planned size fails inside the bench, not the judge.
#[tokio::test]
async fn a_planned_size_without_a_prepared_input_fails_the_bench() {
    let (_dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    let mut opened = opened(&backend, &catalog).await;
    let (_tx, mut stop) = watch::channel(false);
    let inputs = vec![(4_u64, serde_json::json!([[1], 1]))];

    let outcome = {
        let mut collector = SandboxBenchCollector::for_inputs(
            &mut opened,
            catalog.get("python").unwrap(),
            &inputs,
            Duration::from_secs(5),
            &DiscardBench,
            &mut stop,
        );
        collector
            .collect(&plan(&[8], Duration::from_secs(30)))
            .await
            .expect("not a judge error")
    };

    assert_eq!(outcome, BenchOutcome::Failed);
    assert!(
        call_lines(&backend).is_empty(),
        "nothing was sent for the missing size"
    );
}
