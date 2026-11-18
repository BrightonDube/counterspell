// SPDX-License-Identifier: MIT
//! Real-daemon log hygiene (judge-sandbox T35, R13.5): hostile programs write a
//! canary to the protocol pipe, to stderr and inside an exception message; none
//! of it may appear in `tracing` output captured from the worker.

#![cfg(feature = "sandbox")]

mod sbx;
mod support;

use std::time::Duration;

use counterspell_common::status::{RunMode, Verdict};
use counterspell_judge::executor::RunOutcome;
use support::LogCapture;

/// The verdict of a completed run, or a panic describing the failure instead.
fn verdict(observation: &sbx::RunObservation) -> Verdict {
    let outcome = observation
        .outcome
        .as_ref()
        .unwrap_or_else(|e| panic!("the run must complete, not fail: {e:?}"));
    let RunOutcome::Completed(result) = outcome else {
        panic!("the run must produce a verdict, not a failure: {outcome:?}");
    };
    result.verdict
}

/// Both corpus programs run in one test so a single global capture covers the
/// whole scenario.
#[tokio::test]
#[ignore = "needs the rootless sandbox daemon (SANDBOX_DOCKER_SOCKET)"]
async fn user_output_never_reaches_the_logs() {
    let capture = LogCapture::new();
    capture.install_global();

    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);

    let fd = sbx::run_source(
        &docker,
        &cfg,
        &catalog,
        "python",
        "add",
        &sbx::hostile("log_canary_fd.py"),
        RunMode::Submit,
    )
    .await;
    assert_eq!(
        verdict(&fd),
        Verdict::WrongAnswer,
        "the fd canary is a protocol violation, never a pass"
    );

    let exc = sbx::run_source(
        &docker,
        &cfg,
        &catalog,
        "python",
        "add",
        &sbx::hostile("log_canary_exc.py"),
        RunMode::Submit,
    )
    .await;
    assert_ne!(
        verdict(&exc),
        Verdict::Accepted,
        "a raising program is never accepted"
    );

    sbx::drain_dead(&docker, &cfg, Duration::from_secs(60)).await;

    // Control: the capture works, so absence below is not vacuous.
    tracing::info!("log-capture sentinel");

    let logs = capture.text();
    assert!(logs.contains("log-capture sentinel"), "the capture records");
    for canary in ["CANARY-FD-8c3e", "CANARY-EXC-4d91"] {
        assert!(
            !logs.contains(canary),
            "user bytes must never reach the logs ({canary}): {logs}"
        );
    }
}
