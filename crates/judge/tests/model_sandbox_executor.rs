// SPDX-License-Identifier: MIT
//! Model tests for `SandboxExecutor` (judge-sandbox T24, R15.1–R15.3, R9.5, R12.3).
//!
//! The scripted fake backend and the recording context stand in for the daemon and the
//! queue, so each test sees exactly which events were emitted and what the executor
//! returned — without Docker or Postgres.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use counterspell_common::events::PublicEvent;
use counterspell_common::familiar::ClassId;
use counterspell_common::ids::RunId;
use counterspell_common::status::{FailureReason, RunMode, Verdict};
use counterspell_engine::{Comparator, Schema};
use counterspell_judge::executor::{ExecError, RunExecutor, RunInput, RunOutcome};
use counterspell_judge::languages::LanguageCatalog;
use counterspell_judge::sandbox::backend::InfraError;
use counterspell_judge::sandbox::config::{OciRuntime, SandboxConfig};
use counterspell_judge::sandbox::preflight::{PreflightCheck, PreflightReport, PreflightState};
use counterspell_judge::sandbox::problem::{
    BenchDecl, EmptyProblemTests, ProblemSpec, ProblemTests, TestCase,
};
use counterspell_judge::sandbox::sandbox_executor::SandboxExecutor;
use counterspell_judge::sandbox::testkit::{
    Behaviour, DirProblemTests, FakeBackend, StaticProblemTests,
};
use counterspell_judge::testkit::RecordingContext;
use serde_json::{json, Value};

mod support;
use support::shipped_catalogue;

fn config(languages_file: PathBuf) -> SandboxConfig {
    SandboxConfig {
        docker_socket: PathBuf::from("/run/user/0/sandbox.sock"),
        runtime: OciRuntime::Runc,
        max_containers: 4,
        reaper_interval: Duration::from_secs(10),
        orphan_age: Duration::from_secs(40),
        kill_slack: Duration::from_millis(500),
        run_wall_cap: Duration::from_secs(20),
        max_consecutive_abnormal: 3,
        cleanup_budget: Duration::from_secs(5),
        api_timeout: Duration::from_secs(30),
        languages_file,
    }
}

/// A preflight state holding one all-ok report.
fn passing() -> Arc<PreflightState> {
    let state = Arc::new(PreflightState::new());
    state.update(PreflightReport {
        checks: vec![PreflightCheck {
            name: "probe",
            expected: "ok".to_owned(),
            observed: "ok".to_owned(),
            ok: true,
        }],
    });
    state
}

/// The problem every test loads: one sample test and one hidden test.
fn problem() -> ProblemSpec {
    ProblemSpec {
        slug: "two-sum".to_owned(),
        version: 1,
        signature: json!({
            "protocol_version": 1,
            "function": "two_sum",
            "parameters": [
                {"name": "nums", "type": {"kind": "list", "items": {"kind": "int"}}},
                {"name": "target", "type": {"kind": "int"}}
            ],
            "returns": {"kind": "list", "items": {"kind": "int"}}
        }),
        comparator: Comparator::Exact,
        time_limit_ms: Some(1_000),
        tests: vec![
            TestCase {
                ordinal: 1,
                is_sample: true,
                args: vec![json!([2, 7]), json!(9)],
                expected: json!([0, 1]),
            },
            TestCase {
                ordinal: 2,
                is_sample: false,
                args: vec![json!([3, 4]), json!(7)],
                expected: json!([0, 1]),
            },
        ],
        reference_python: None,
        bench: None,
        generator_schema: None,
        reference_complexity: None,
    }
}

/// The same problem with a full bench declaration: the `nums` parameter grows.
fn benched_problem() -> ProblemSpec {
    ProblemSpec {
        bench: Some(BenchDecl {
            sizes: vec![16, 64, 256],
            size_param: 0,
        }),
        generator_schema: Some(Schema::Tuple {
            items: vec![
                Schema::IntArray {
                    min_len: 0,
                    max_len: 16,
                    min: 0,
                    max: 10_000,
                    sorted: false,
                    unique: false,
                },
                Schema::Int {
                    min: 0,
                    max: 20_000,
                },
            ],
        }),
        reference_complexity: Some(ClassId::Linear),
        ..problem()
    }
}

/// The answer line a correct program prints for call `ordinal`.
fn answer(ordinal: u32, out: &Value) -> Behaviour {
    Behaviour::Answer(format!("{{\"t\":{ordinal},\"out\":{out}}}\n").into_bytes())
}

fn input(mode: RunMode) -> RunInput {
    RunInput {
        run_id: RunId::new(),
        problem_slug: "two-sum".to_owned(),
        problem_version: 1,
        lang: "python".to_owned(),
        mode,
        ghost_input: None,
        code: "def two_sum(nums, target):\n    return [0, 1]\n".to_owned(),
        attempt: 1,
    }
}

fn executor(
    catalog: LanguageCatalog,
    backend: &FakeBackend,
    languages_file: PathBuf,
) -> SandboxExecutor<StaticProblemTests, FakeBackend> {
    executor_with(catalog, backend, languages_file, problem())
}

fn executor_with(
    catalog: LanguageCatalog,
    backend: &FakeBackend,
    languages_file: PathBuf,
    spec: ProblemSpec,
) -> SandboxExecutor<StaticProblemTests, FakeBackend> {
    SandboxExecutor::new(
        catalog,
        StaticProblemTests::new([spec]),
        backend.clone(),
        config(languages_file),
        vec![passing()],
        "worker-1",
    )
}

#[tokio::test]
async fn compiling_is_the_first_event_of_every_completed_run() {
    let (dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    backend.script_program([answer(1, &json!([0, 1])), answer(2, &json!([0, 1]))]);
    let executor = executor(catalog, &backend, dir.path().join("languages.toml"));
    let (ctx, log, _stop) = RecordingContext::with_stop(1);

    let outcome = executor
        .execute(input(RunMode::Submit), ctx)
        .await
        .expect("the run completes");

    let names: Vec<_> = log.events().iter().map(PublicEvent::event_type).collect();
    assert_eq!(
        names,
        ["compiling", "test", "test", "familiar"],
        "compiling is first, then one event per executed test, then the familiar"
    );
    let RunOutcome::Completed(result) = outcome else {
        panic!("a passing run must complete: {outcome:?}");
    };
    assert_eq!(result.verdict, Verdict::Accepted);
    assert_eq!(result.passed, 2);
    assert_eq!(result.total, 2);
    assert_eq!(backend.ledger().live(), 0, "the container is removed");
}

#[tokio::test]
async fn a_sample_test_event_carries_its_public_detail() {
    let (dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    backend.script_program([answer(1, &json!([0, 1])), answer(2, &json!([0, 1]))]);
    let executor = executor(catalog, &backend, dir.path().join("languages.toml"));
    let (ctx, log, _stop) = RecordingContext::with_stop(1);

    executor
        .execute(input(RunMode::Submit), ctx)
        .await
        .expect("the run completes");

    let events = log.events();
    assert!(
        matches!(
            &events[1],
            PublicEvent::Test(counterspell_common::events::PublicTest::Sample { .. })
        ),
        "the sample test is visible: {events:?}"
    );
    assert!(
        matches!(
            &events[2],
            PublicEvent::Test(counterspell_common::events::PublicTest::Hidden { .. })
        ),
        "the hidden test carries no detail: {events:?}"
    );
}

#[tokio::test]
async fn the_stop_signal_ends_the_run_as_stopped_with_cleanup() {
    let (dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    backend.script_program([answer(1, &json!([0, 1])), answer(2, &json!([0, 1]))]);
    let executor = executor(catalog, &backend, dir.path().join("languages.toml"));
    let (ctx, _log, stop) = RecordingContext::with_stop(1);
    stop.send(true).expect("the stop signal sends");

    let result = executor.execute(input(RunMode::Submit), ctx).await;

    assert!(matches!(result, Err(ExecError::Stopped)));
    assert_eq!(backend.ledger().live(), 0, "nothing is left running");
}

#[tokio::test]
async fn a_sandbox_error_is_infrastructure_never_a_verdict() {
    let (dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    backend.fail_call(1, InfraError::DaemonUnreachable("down".to_owned()));
    let executor = executor(catalog, &backend, dir.path().join("languages.toml"));
    let (ctx, _log, _stop) = RecordingContext::with_stop(1);

    let result = executor.execute(input(RunMode::Submit), ctx).await;

    assert!(matches!(result, Err(ExecError::Infrastructure(_))));
    assert_eq!(backend.ledger().live(), 0, "nothing is left running");
}

#[tokio::test]
async fn a_failing_preflight_blocks_the_run_before_any_event() {
    let (dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    let no_report = Arc::new(PreflightState::new());
    let executor = SandboxExecutor::new(
        catalog,
        StaticProblemTests::new([problem()]),
        backend.clone(),
        config(dir.path().join("languages.toml")),
        vec![no_report],
        "worker-1",
    );
    let (ctx, log) = RecordingContext::without_stop(1);

    let result = executor.execute(input(RunMode::Submit), ctx).await;

    assert!(matches!(result, Err(ExecError::Infrastructure(_))));
    assert!(
        log.events().is_empty(),
        "no event is emitted when preflight fails"
    );
    assert_eq!(backend.ledger().created(), 0, "no container was created");
}

#[tokio::test]
async fn an_unavailable_language_fails_without_touching_the_sandbox() {
    let (dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    let executor = executor(catalog, &backend, dir.path().join("languages.toml"));
    let (ctx, _log) = RecordingContext::without_stop(1);
    let mut request = input(RunMode::Submit);
    request.lang = "cobol".to_owned();

    let outcome = executor
        .execute(request, ctx)
        .await
        .expect("a Failure, not an error");

    assert!(matches!(
        outcome,
        RunOutcome::Failed(FailureReason::LanguageUnavailable)
    ));
    assert_eq!(backend.ledger().created(), 0);
}

#[tokio::test]
async fn an_unknown_problem_fails_as_problem_unavailable() {
    let (dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    let executor = executor(catalog, &backend, dir.path().join("languages.toml"));
    let (ctx, _log) = RecordingContext::without_stop(1);
    let mut request = input(RunMode::Submit);
    request.problem_slug = "no-such-problem".to_owned();

    let outcome = executor
        .execute(request, ctx)
        .await
        .expect("a Failure, not an error");

    assert!(matches!(
        outcome,
        RunOutcome::Failed(FailureReason::ProblemUnavailable)
    ));
    assert_eq!(backend.ledger().created(), 0);
}

#[tokio::test]
async fn the_production_loader_makes_every_problem_unavailable() {
    let (dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    let executor = SandboxExecutor::new(
        catalog,
        EmptyProblemTests,
        backend.clone(),
        config(dir.path().join("languages.toml")),
        vec![passing()],
        "worker-1",
    );
    let (ctx, _log) = RecordingContext::without_stop(1);

    let outcome = executor
        .execute(input(RunMode::Submit), ctx)
        .await
        .expect("a Failure, not an error");

    assert!(matches!(
        outcome,
        RunOutcome::Failed(FailureReason::ProblemUnavailable)
    ));
    assert_eq!(backend.ledger().created(), 0);
}

#[tokio::test]
async fn a_source_the_precheck_refuses_is_a_compile_error_without_a_container() {
    let (dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    let executor = executor(catalog, &backend, dir.path().join("languages.toml"));
    let (ctx, _log, _stop) = RecordingContext::with_stop(1);
    let mut request = input(RunMode::Submit);
    request.code = "print('x')\x07".to_owned(); // a control character

    let outcome = executor
        .execute(request, ctx)
        .await
        .expect("a verdict, not an error");

    let RunOutcome::Completed(result) = outcome else {
        panic!("a refused source still completes: {outcome:?}");
    };
    assert_eq!(result.verdict, Verdict::CompileError);
    assert!(result.compile_output.is_some());
    assert_eq!(backend.ledger().created(), 0, "no container was created");
}

#[tokio::test]
async fn dir_problem_tests_loads_a_fixture_and_filters_by_mode() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/problems");
    let problems = DirProblemTests::new(dir);

    let run = problems
        .load("two-sum", 1, RunMode::Run)
        .await
        .expect("the fixture parses")
        .expect("two-sum exists");
    assert!(
        run.tests.iter().all(|test| test.is_sample),
        "mode Run loads only the sample tests"
    );
    let submit = problems
        .load("two-sum", 1, RunMode::Submit)
        .await
        .expect("the fixture parses")
        .expect("two-sum exists");
    assert!(
        submit.tests.iter().any(|test| !test.is_sample),
        "mode Submit loads the hidden tests too"
    );
    assert!(
        problems
            .load("no-such-problem", 1, RunMode::Submit)
            .await
            .expect("a missing file is not an error")
            .is_none(),
        "an absent fixture is problem_unavailable"
    );
    assert!(
        problems
            .load("two-sum", 2, RunMode::Submit)
            .await
            .expect("an unpinned version is not an error")
            .is_none(),
        "a version the store does not hold is problem_unavailable"
    );
}

#[tokio::test]
async fn dir_problem_tests_loads_the_bench_declaration() {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/problems");
    let problems = DirProblemTests::new(dir);

    let spec = problems
        .load("two-sum", 1, RunMode::Submit)
        .await
        .expect("the fixture parses")
        .expect("two-sum exists");
    let bench = spec.bench.expect("two-sum declares a bench");
    assert_eq!(bench.sizes, vec![16, 64, 256, 1024, 2048, 4096]);
    assert_eq!(bench.size_param, 0, "the nums parameter grows");
    assert_eq!(spec.reference_complexity, Some(ClassId::Linear));
    match spec.generator_schema {
        Some(Schema::Tuple { items }) => assert_eq!(items.len(), 2),
        other => panic!("the generator schema must be the parameter tuple: {other:?}"),
    }
}

#[tokio::test]
async fn an_accepted_benched_run_measures_and_records_timings() {
    let (dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    // Two correctness calls (ids 1 and 2), then 3 sizes x 7 bench calls (ids 0..=20).
    let mut script = vec![answer(1, &json!([0, 1])), answer(2, &json!([0, 1]))];
    script.extend((0..21u32).map(|id| answer(id, &json!([0, 1]))));
    backend.script_program(script);
    let executor = executor_with(
        catalog,
        &backend,
        dir.path().join("languages.toml"),
        benched_problem(),
    );
    let (ctx, log, _stop) = RecordingContext::with_stop(1);

    let outcome = executor
        .execute(input(RunMode::Submit), ctx)
        .await
        .expect("the run completes");

    let names: Vec<_> = log.events().iter().map(PublicEvent::event_type).collect();
    assert_eq!(
        names[..7],
        [
            "compiling",
            "test",
            "test",
            "bench_start",
            "bench_point",
            "bench_point",
            "bench_point"
        ],
        "the bench runs after the tests"
    );
    // The fit may or may not settle a class on scripted timings; when it does, the
    // complexity event precedes the mood, which always ends the run.
    let tail = &names[7..];
    assert!(
        tail == ["familiar"] || tail == ["complexity", "familiar"],
        "the mood always ends the run: {tail:?}"
    );
    let RunOutcome::Completed(result) = outcome else {
        panic!("a passing run must complete: {outcome:?}");
    };
    assert_eq!(result.verdict, Verdict::Accepted);
    let measured = result.measured.expect("a measured run stores its columns");
    assert!(measured.fit.is_some(), "a fitted bench stores its fit");
    assert!(measured.familiar_mood.is_some());

    let timings = log.timings();
    assert_eq!(timings.len(), 15, "three sizes, five timed repeats each");
    for size in [16_u64, 64, 256] {
        let rows: Vec<_> = timings.iter().filter(|row| row.n == size).collect();
        assert_eq!(rows.len(), 5, "one row per timed repeat at n={size}");
        assert!(
            rows.iter().map(|row| row.repeat).eq(1..=5),
            "repeats count from 1"
        );
        assert!(rows.iter().all(|row| row.ns > 0));
    }
    assert_eq!(backend.ledger().live(), 0, "the container is removed");
}

#[tokio::test]
async fn a_wrong_answer_on_a_benched_problem_never_runs_the_bench() {
    let (dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    backend.script_program([answer(1, &json!([0, 1])), answer(2, &json!([9, 9]))]);
    let executor = executor_with(
        catalog,
        &backend,
        dir.path().join("languages.toml"),
        benched_problem(),
    );
    let (ctx, log, _stop) = RecordingContext::with_stop(1);

    let outcome = executor
        .execute(input(RunMode::Submit), ctx)
        .await
        .expect("the run completes");

    let names: Vec<_> = log.events().iter().map(PublicEvent::event_type).collect();
    assert_eq!(
        names,
        ["compiling", "test", "test", "familiar"],
        "no bench runs for a failed verdict"
    );
    assert!(log.timings().is_empty(), "no samples are stored");
    let RunOutcome::Completed(result) = outcome else {
        panic!("a completed run: {outcome:?}");
    };
    assert_eq!(result.verdict, Verdict::WrongAnswer);
    let measured = result.measured.expect("the mood is still measured");
    assert!(measured.measured_complexity.is_none());
    assert!(measured.familiar_mood.is_some());
}

#[tokio::test]
async fn a_partial_bench_declaration_is_infrastructure() {
    let (dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    backend.script_program([answer(1, &json!([0, 1])), answer(2, &json!([0, 1]))]);
    // The sizes are declared but the schema and reference are missing: a broken store.
    let mut spec = problem();
    spec.bench = Some(BenchDecl {
        sizes: vec![16, 64],
        size_param: 0,
    });
    let executor = executor_with(catalog, &backend, dir.path().join("languages.toml"), spec);
    let (ctx, _log, _stop) = RecordingContext::with_stop(1);

    let outcome = executor.execute(input(RunMode::Submit), ctx).await;
    assert!(
        matches!(outcome, Err(ExecError::Infrastructure(_))),
        "a half-declared bench is a store fault, not a verdict: {outcome:?}"
    );
}

/// A problem with a trusted reference, for custom ghost runs (ide-workspace T02).
fn problem_with_reference() -> ProblemSpec {
    ProblemSpec {
        reference_python: Some("def two_sum(nums, target):\n    return [0, 1]\n".to_owned()),
        ..problem()
    }
}

/// A run carrying one ghost input instead of the stored suite.
fn ghost_input(input_value: Value) -> RunInput {
    let mut run = input(RunMode::Run);
    run.ghost_input = Some(input_value);
    run
}

#[tokio::test]
async fn a_ghost_run_passes_when_the_answers_match() {
    let (dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    // One call per side: first the user answer, then the reference answer.
    backend.script_program([answer(1, &json!([0, 1])), answer(1, &json!([0, 1]))]);
    let executor = executor_with(
        catalog,
        &backend,
        dir.path().join("languages.toml"),
        problem_with_reference(),
    );
    let (ctx, log, _stop) = RecordingContext::with_stop(1);

    let outcome = executor
        .execute(ghost_input(json!([[2, 7], 9])), ctx)
        .await
        .expect("the ghost run completes");

    let names: Vec<_> = log.events().iter().map(PublicEvent::event_type).collect();
    assert_eq!(names, ["compiling", "test"], "one custom test, no familiar");
    let RunOutcome::Completed(result) = outcome else {
        panic!("a passing ghost run must complete: {outcome:?}");
    };
    assert_eq!(result.verdict, Verdict::Accepted);
    assert_eq!((result.passed, result.total), (1, 1));
    assert!(result.measured.is_none(), "a ghost run is never benched");
}

#[tokio::test]
async fn a_ghost_run_is_a_wrong_answer_when_the_sides_differ() {
    let (dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    backend.script_program([answer(1, &json!([9, 9])), answer(1, &json!([0, 1]))]);
    let executor = executor_with(
        catalog,
        &backend,
        dir.path().join("languages.toml"),
        problem_with_reference(),
    );
    let (ctx, _log, _stop) = RecordingContext::with_stop(1);

    let outcome = executor
        .execute(ghost_input(json!([[2, 7], 9])), ctx)
        .await
        .expect("the ghost run completes");

    let RunOutcome::Completed(result) = outcome else {
        panic!("a wrong ghost run still completes: {outcome:?}");
    };
    assert_eq!(result.verdict, Verdict::WrongAnswer);
    assert_eq!((result.passed, result.total), (0, 1));
}

#[tokio::test]
async fn a_ghost_run_is_a_runtime_error_when_the_program_crashes() {
    let (dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    backend.script_program([
        Behaviour::Stderr(b"ValueError: boom\n".to_vec()),
        Behaviour::Exit {
            code: 1,
            oom_kill: 0,
        },
        answer(1, &json!([0, 1])),
    ]);
    let executor = executor_with(
        catalog,
        &backend,
        dir.path().join("languages.toml"),
        problem_with_reference(),
    );
    let (ctx, _log, _stop) = RecordingContext::with_stop(1);

    let outcome = executor
        .execute(ghost_input(json!([[2, 7], 9])), ctx)
        .await
        .expect("the ghost run completes");

    let RunOutcome::Completed(result) = outcome else {
        panic!("a crashed ghost run still completes: {outcome:?}");
    };
    assert_eq!(result.verdict, Verdict::RuntimeError);
}

#[tokio::test]
async fn a_ghost_run_times_out_when_the_program_hangs() {
    let (dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    backend.script_program([Behaviour::Hang, answer(1, &json!([0, 1]))]);
    let mut spec = problem_with_reference();
    spec.time_limit_ms = Some(50);
    let executor = executor_with(catalog, &backend, dir.path().join("languages.toml"), spec);
    let (ctx, _log, _stop) = RecordingContext::with_stop(1);

    let outcome = executor
        .execute(ghost_input(json!([[2, 7], 9])), ctx)
        .await
        .expect("the ghost run completes");

    let RunOutcome::Completed(result) = outcome else {
        panic!("a timed-out ghost run still completes: {outcome:?}");
    };
    assert_eq!(result.verdict, Verdict::TimeLimit);
}

#[tokio::test]
async fn a_ghost_run_without_a_reference_is_problem_unavailable() {
    let (dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    let executor = executor(catalog, &backend, dir.path().join("languages.toml"));
    let (ctx, _log, _stop) = RecordingContext::with_stop(1);

    let outcome = executor
        .execute(ghost_input(json!([[2, 7], 9])), ctx)
        .await
        .expect("the ghost run resolves");

    assert!(
        matches!(
            outcome,
            RunOutcome::Failed(FailureReason::ProblemUnavailable)
        ),
        "no reference means unjudgeable, never a verdict: {outcome:?}"
    );
}
