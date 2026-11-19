// SPDX-License-Identifier: MIT
//! Hostile driver-protocol cases for the launch drivers (language-drivers T11).
//!
//! The Python corpus already proves `os.write(1, …)` forgery is judged, not
//! believed (`tests/hostile/forged_lines.py`, `fake_verdict.py` — S14/S15).
//! These are the JavaScript equivalents: `fs.writeSync(1, …)` bypasses the
//! driver's `process.stdout.write` sink and writes straight onto the protocol
//! pipe, so it is the strongest direct protocol corruption a solution has.
//! Whatever it writes can only be an answer value — verdicts are computed by
//! the worker's comparator, which user output can never reach. Each test also
//! confirms the run still finishes and no container stays running.
//!
//! `cargo test -p counterspell-judge --features sandbox --test sbx_driver_hostile -- --ignored`
#![cfg(feature = "sandbox")]

mod sbx;
mod sbx_support;

use std::time::Duration;

use counterspell_engine::Comparator;
use counterspell_judge::sandbox::backend::InfraError;
use counterspell_judge::sandbox::compare::Expected;
use counterspell_judge::sandbox::config::OciRuntime;
use counterspell_judge::sandbox::run::{execute_run, PlannedTest, RunSettings};
use counterspell_judge::sandbox::spec::SessionSpec;
use counterspell_judge::sandbox::verdict::FinalVerdict;
use sbx_support::{catalogue, int, run_archive, sandbox_backend, signature};
use serde_json::json;
use tokio::sync::watch;
use tokio::time::Instant;
use uuid::Uuid;

/// Two sample tests and one hidden test for `add(a, b)`. Expected values live
/// only in the trusted test plan — the driver never sees them.
fn add_plan() -> Vec<PlannedTest> {
    let case = |ordinal: u32, sample: bool, a: i64, b: i64, expected: i64| PlannedTest {
        ordinal,
        sample,
        args: vec![json!(a), json!(b)],
        expected: Expected::new(json!(expected)),
        comparator: Comparator::Exact,
        time_limit: Duration::from_secs(2),
    };
    vec![
        case(0, true, 2, 3, 5),
        case(1, true, 10, -4, 6),
        case(2, false, 100, 200, 300),
    ]
}

/// Runs the add plan against one JavaScript solution on the real daemon and
/// returns the worker's final verdict.
async fn run_js(solution: &str) -> FinalVerdict {
    let backend = sandbox_backend().await;
    let (_dir, catalog) = catalogue();
    let javascript = catalog
        .get("javascript")
        .expect("javascript is in the catalogue");
    let sig = signature("add", &[("a", int()), ("b", int())], &int());
    let archive = run_archive(javascript, solution, sig);

    let run_id = Uuid::new_v4();
    let session_spec = |ordinal: u32| {
        SessionSpec::new(
            javascript,
            Duration::from_secs(60),
            OciRuntime::Runc,
            run_id,
            "sbx-test",
            ordinal,
            i64::MAX / 2,
        )
        .map_err(|error| InfraError::Unexpected(error.to_string()))
    };
    let settings = RunSettings {
        run_deadline: Instant::now() + Duration::from_secs(60),
        kill_slack: Duration::from_secs(5),
        max_consecutive_abnormal: 3,
    };
    let (_stop_sender, mut stop) = watch::channel(false);
    let result = execute_run(
        &backend,
        javascript,
        &archive,
        session_spec,
        add_plan(),
        settings,
        &mut stop,
    )
    .await
    .expect("a hostile run still completes without an infrastructure error");
    result.verdict()
}

/// Runs a hostile JavaScript solution and asserts the verdict plus a clean
/// daemon afterwards.
async fn run_hostile_js(solution: &str) -> FinalVerdict {
    let verdict = run_js(solution).await;
    let (docker, cfg) = sbx::sandbox().await;
    // No managed container may still be running; corpses are reaped here.
    sbx::drain_dead(&docker, &cfg, Duration::from_secs(15)).await;
    verdict
}

/// T11: a forged `{t, out}` line written straight to fd 1 lands before the real
/// answer. It can only supply a value — comparison stays with the worker, so
/// the run is wrong, never accepted.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_js_fd1_forged_answer_is_never_accepted() {
    let solution = "const fs = require('fs');\n\
        module.exports = function add(a, b) {\n  \
          fs.writeSync(1, '{\"t\": 0, \"out\": 0}\\n');\n  \
          return -(a + b);\n};\n";
    let verdict = run_hostile_js(solution).await;
    assert_ne!(
        verdict,
        FinalVerdict::Accepted,
        "a forged answer line is a compared value, not a verdict"
    );
}

/// T11: a forged answer carrying a call id the worker did not ask for violates
/// the protocol — the session refuses it, the run is never accepted.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_js_fd1_wrong_call_id_is_never_accepted() {
    let solution = "const fs = require('fs');\n\
        module.exports = function add(a, b) {\n  \
          fs.writeSync(1, '{\"t\": 99, \"out\": 5}\\n');\n  \
          return a + b;\n};\n";
    let verdict = run_hostile_js(solution).await;
    assert_ne!(
        verdict,
        FinalVerdict::Accepted,
        "a mismatched call id cannot claim an answer"
    );
}

/// T11: a verdict-shaped payload — as a bare line and as a protocol `out`
/// value — is just JSON to the comparator. The worker's comparison still
/// decides, so the run is never accepted.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_js_fd1_verdict_document_is_never_accepted() {
    let solution = "const fs = require('fs');\n\
        module.exports = function add(a, b) {\n  \
          fs.writeSync(1, '{\"verdict\": \"accepted\"}\\n');\n  \
          fs.writeSync(1, '{\"t\": 0, \"out\": {\"verdict\": \"accepted\"}}\\n');\n  \
          return a + b;\n};\n";
    let verdict = run_hostile_js(solution).await;
    assert_ne!(
        verdict,
        FinalVerdict::Accepted,
        "a verdict-shaped document is user output, not a decision"
    );
}

/// T11: an infinite loop never answers; the external per-call deadline cuts it
/// and the verdict is a time limit — not a hang and not an acceptance.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_js_infinite_loop_is_a_time_limit() {
    let solution = "module.exports = function add(a, b) {\n  for (;;) {}\n};\n";
    let verdict = run_hostile_js(solution).await;
    assert_eq!(verdict, FinalVerdict::TimeLimit);
}

/// T11: a huge direct-to-fd write hits the worker's output budget — a resource
/// verdict, never an acceptance.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_js_fd1_flood_is_never_accepted() {
    let solution = "const fs = require('fs');\n\
        module.exports = function add(a, b) {\n  \
          fs.writeSync(1, 'x'.repeat(4 * 1024 * 1024));\n  \
          return a + b;\n};\n";
    let verdict = run_hostile_js(solution).await;
    assert_ne!(
        verdict,
        FinalVerdict::Accepted,
        "an output flood is a limit verdict, not an acceptance"
    );
}
