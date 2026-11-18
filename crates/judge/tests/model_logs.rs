// SPDX-License-Identifier: MIT
//! Model test for log hygiene (judge-sandbox T35, R13.5): a program's stdout,
//! stderr and exception text all carry a canary through the sandbox boundary;
//! none of it may appear in `tracing` output. `sbx_logs` repeats this on the
//! real daemon.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use counterspell_common::ids::RunId;
use counterspell_common::status::RunMode;
use counterspell_engine::Comparator;
use counterspell_judge::executor::{RunExecutor, RunInput};
use counterspell_judge::sandbox::config::{OciRuntime, SandboxConfig};
use counterspell_judge::sandbox::preflight::{PreflightCheck, PreflightReport, PreflightState};
use counterspell_judge::sandbox::problem::{ProblemSpec, TestCase};
use counterspell_judge::sandbox::sandbox_executor::SandboxExecutor;
use counterspell_judge::sandbox::testkit::{Behaviour, FakeBackend, StaticProblemTests};
use counterspell_judge::testkit::RecordingContext;
use serde_json::json;

mod support;
use support::{shipped_catalogue, LogCapture};

const CANARY: &str = "CANARY-MODEL-7f3a";

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

/// Three tests so one run consumes three scripted calls.
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
        tests: (1..=3u32)
            .map(|ordinal| TestCase {
                ordinal,
                is_sample: ordinal == 1,
                args: vec![json!([2, 7]), json!(9)],
                expected: json!([0, 1]),
            })
            .collect(),
        reference_python: None,
        bench: None,
        generator_schema: None,
        reference_complexity: None,
    }
}

#[tokio::test]
async fn the_canary_never_reaches_the_logs() {
    let capture = LogCapture::new();
    capture.install_global();

    let (dir, catalog) = shipped_catalogue(false);
    let backend = FakeBackend::new();
    backend.script_program([
        // Call 1: a parseable answer whose payload is the canary — it travels the
        // whole answer-parse-and-compare path before being marked wrong.
        Behaviour::Answer(format!("{{\"t\":1,\"out\":\"{CANARY}\"}}\n").into_bytes()),
        // Call 2: a raw line the parser rejects outright (protocol violation).
        Behaviour::Answer(format!("{CANARY}\n").into_bytes()),
        // Call 3: the canary on stderr and in the exception the driver reports.
        Behaviour::Stderr(b"Traceback (most recent call last):\n".to_vec()),
        Behaviour::Stderr(format!("ValueError: {CANARY}\n").into_bytes()),
        Behaviour::Exit {
            code: 1,
            oom_kill: 0,
        },
    ]);
    let executor = SandboxExecutor::new(
        catalog,
        StaticProblemTests::new([problem()]),
        backend.clone(),
        config(dir.path().join("languages.toml")),
        vec![passing()],
        "worker-1",
    );
    let (ctx, _log, _stop) = RecordingContext::with_stop(1);
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

    let outcome = executor.execute(input, ctx).await;
    assert!(outcome.is_ok(), "the run reports a verdict: {outcome:?}");

    // Control: the capture works, so absence below is not vacuous.
    tracing::info!("log-capture sentinel");

    let logs = capture.text();
    assert!(logs.contains("log-capture sentinel"), "the capture records");
    assert!(
        !logs.contains(CANARY),
        "user bytes must never reach the logs: {logs}"
    );
    assert_eq!(
        backend.ledger().created(),
        1,
        "one container ran the program"
    );
}
