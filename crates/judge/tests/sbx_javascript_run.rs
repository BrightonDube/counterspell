//! Real-daemon run tests for the JavaScript driver through the judge's run path
//! (language-drivers T07).
//!
//! Each test runs a small plan through `execute_run` on the rootless sandbox daemon. The plan,
//! the expected values and the comparator stay in the judge; the container only receives the
//! solution, the driver and the public signature. A test checks the final verdict the worker
//! decides, never an answer the program printed.
//!
//! Ignored by default. Needs `SANDBOX_DOCKER_SOCKET` and the pinned `node:22-slim` image on the
//! rootless daemon. Nothing runs on the host.
//!
//! Run on the sandbox daemon only:
//! `cargo test -p counterspell-judge --features sandbox --test sbx_javascript_run -- --ignored`
#![cfg(feature = "sandbox")]

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

/// Two sample tests and one hidden test for `add(a, b)`. The expected values live only here.
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

/// Runs the add plan against one JavaScript solution and returns the worker's final verdict.
async fn run_add(solution: &str) -> FinalVerdict {
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
    // The sender stays alive for the whole run, so the stop signal never fires by accident.
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
    .expect("the run completes without an infrastructure error");
    result.verdict()
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_run_accepts_a_correct_solution() {
    let verdict = run_add("module.exports = function add(a, b) { return a + b; };\n").await;
    assert_eq!(verdict, FinalVerdict::Accepted);
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_run_reports_a_wrong_answer() {
    let verdict = run_add("module.exports = function add(a, b) { return a + b + 1; };\n").await;
    assert_eq!(verdict, FinalVerdict::WrongAnswer);
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_run_exception_is_a_runtime_error_never_accepted() {
    let verdict =
        run_add("module.exports = function add(a, b) { throw new Error('boom'); };\n").await;
    assert_eq!(verdict, FinalVerdict::RuntimeError);
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_run_fake_verdict_print_cannot_make_it_accepted() {
    // The program prints a verdict-shaped line, then returns a wrong answer. Only the worker's
    // comparison decides, so the run is a wrong answer.
    let solution = "module.exports = function add(a, b) {\n  console.log('{\"verdict\": \"accepted\"}');\n  return 0;\n};\n";
    let verdict = run_add(solution).await;
    assert_eq!(verdict, FinalVerdict::WrongAnswer);
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned node image on the rootless daemon"]
async fn sbx_javascript_run_malformed_return_is_never_accepted() {
    let verdict = run_add("module.exports = function add(a, b) { return true; };\n").await;
    assert_ne!(
        verdict,
        FinalVerdict::Accepted,
        "a refused answer is not an accepted run"
    );
}
