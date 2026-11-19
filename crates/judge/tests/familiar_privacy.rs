// SPDX-License-Identifier: MIT
//! Privacy of the measured Familiar on the judge side (efficiency-familiar T43, R7.9, R12).
//!
//! A canary sits in a hidden test's input, in its expected output and in the program's
//! stderr. The run is accepted, so the bench, the fit and the mood all run. The canary
//! must not appear in any `bench_*`, `complexity` or `familiar` event, in the measured
//! columns (including `fit`), or in the logs. The scripted backend stands in for Docker,
//! so no program runs on the host.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use counterspell_common::familiar::ClassId;
use counterspell_common::ids::RunId;
use counterspell_common::status::{RunMode, Verdict};
use counterspell_engine::{Comparator, Schema};
use counterspell_judge::executor::{RunExecutor, RunInput, RunOutcome};
use counterspell_judge::sandbox::config::{OciRuntime, SandboxConfig};
use counterspell_judge::sandbox::preflight::{PreflightCheck, PreflightReport, PreflightState};
use counterspell_judge::sandbox::problem::{BenchDecl, ProblemSpec, TestCase};
use counterspell_judge::sandbox::sandbox_executor::SandboxExecutor;
use counterspell_judge::sandbox::testkit::{Behaviour, FakeBackend, StaticProblemTests};
use counterspell_judge::testkit::RecordingContext;
use serde_json::{json, Value};

mod support;
use support::{shipped_catalogue, LogCapture};

const HIDDEN_INPUT_CANARY: &str = "CANARY-HIDDEN-INPUT-91c2";
const HIDDEN_EXPECTED_CANARY: &str = "CANARY-HIDDEN-EXPECTED-4b7e";
const STDERR_CANARY: &str = "CANARY-STDERR-d05a";

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

/// A benched problem whose hidden test carries canaries in its input and expected output.
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
                args: vec![json!([3, 4]), json!(HIDDEN_INPUT_CANARY)],
                expected: json!(HIDDEN_EXPECTED_CANARY),
            },
        ],
        reference_python: None,
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
        input_schema: None,
        known_ghosts: Vec::new(),
        concept_tags: Vec::new(),
    }
}

fn answer(id: u32, out: &Value) -> Behaviour {
    Behaviour::Answer(format!("{{\"t\":{id},\"out\":{out}}}\n").into_bytes())
}

#[tokio::test]
async fn no_canary_reaches_measured_events_columns_or_logs() {
    let capture = LogCapture::new();
    capture.install_global();

    let (dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    let mut script = vec![
        Behaviour::Stderr(format!("debug: {STDERR_CANARY}\n").into_bytes()),
        answer(1, &json!([0, 1])),
        // The hidden answer equals the hidden expected value, so the run is accepted and
        // the bench, fit and mood all run.
        answer(2, &json!(HIDDEN_EXPECTED_CANARY)),
    ];
    script.extend((0..21u32).map(|id| answer(id, &json!([0, 1]))));
    backend.script_program(script);
    let executor = SandboxExecutor::new(
        catalog,
        StaticProblemTests::new([problem()]),
        backend.clone(),
        config(dir.path().join("languages.toml")),
        vec![passing()],
        "worker-1",
    );
    let (ctx, log, _stop) = RecordingContext::with_stop(1);
    let input = RunInput {
        run_id: RunId::new(),
        problem_slug: "two-sum".to_owned(),
        problem_version: 1,
        lang: "python".to_owned(),
        mode: RunMode::Submit,
        ghost_input: None,
        code: "def two_sum(nums, target):\n    return [0, 1]\n".to_owned(),
        attempt: 1,
    };

    let outcome = executor
        .execute(input, ctx)
        .await
        .expect("the run completes");

    let RunOutcome::Completed(result) = outcome else {
        panic!("an accepted run completes: {outcome:?}");
    };
    assert_eq!(
        result.verdict,
        Verdict::Accepted,
        "the hidden canary was matched"
    );
    let measured = result
        .measured
        .expect("an accepted benched run is measured");
    assert!(
        measured.fit.is_some(),
        "the fit ran, so the check below is not vacuous"
    );

    let measured_events: Vec<String> = log
        .events()
        .iter()
        .filter(|event| {
            matches!(
                event.event_type(),
                "bench_start" | "bench_point" | "complexity" | "familiar"
            )
        })
        .map(|event| event.payload().to_string())
        .collect();
    assert!(
        measured_events
            .iter()
            .any(|e| e.contains("\"familiar\"") || e.contains("mood")),
        "the familiar event was emitted: {measured_events:?}"
    );
    let columns = format!("{measured:?}");
    tracing::info!("log-capture sentinel");
    let logs = capture.text();
    assert!(logs.contains("log-capture sentinel"), "the capture records");

    for canary in [HIDDEN_INPUT_CANARY, HIDDEN_EXPECTED_CANARY, STDERR_CANARY] {
        for event in &measured_events {
            assert!(!event.contains(canary), "{canary} leaked into {event}");
        }
        assert!(!columns.contains(canary), "{canary} leaked into {columns}");
        assert!(!logs.contains(canary), "{canary} leaked into the logs");
    }
    assert_eq!(backend.ledger().live(), 0, "the container is removed");
}
