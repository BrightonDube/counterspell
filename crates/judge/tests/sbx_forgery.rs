// SPDX-License-Identifier: MIT
//! Forgery and crash scenarios S14 and S15 (judge-sandbox T26, R8.4, R13.7, R16.3).
//!
//! A program may print anything it likes — a fake result document, a forged protocol
//! line, an early answer. The judge counts only the line that answers the real call, and
//! everything else is a wrong answer or a runtime error. Ignored by default.
//!
//! `cargo test -p counterspell-judge --features sandbox --test sbx_forgery -- --ignored`
#![cfg(feature = "sandbox")]

mod sbx;

use counterspell_common::status::{RunMode, Verdict};
use counterspell_judge::executor::RunOutcome;

/// Runs one hostile source against `add` and returns its verdict, asserting the run
/// still completes (a forgery attempt is a verdict, never a panic or an infra error).
async fn run_hostile(name: &str) -> Verdict {
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let observation = sbx::run_source(
        &docker,
        &cfg,
        &catalog,
        "python",
        "add",
        &sbx::hostile(name),
        RunMode::Submit,
    )
    .await;
    let outcome = observation.outcome.expect("the run produces an outcome");
    let RunOutcome::Completed(result) = outcome else {
        panic!("a hostile program still gets a completed run: {outcome:?}");
    };
    assert!(
        sbx::cleared_within(&docker, &cfg, std::time::Duration::from_secs(15))
            .await
            .is_empty(),
        "no container survives {name}"
    );
    result.verdict
}

/// S14 — the driver sinks `sys.stdout` while user code runs, so `print` lands
/// nowhere; `os.write(1, …)` hits the protocol pipe and violates the one-answer
/// contract. Either way only `verdict::decide` may produce `Pass`.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s14_a_printed_result_document_is_just_output() {
    let verdict = run_hostile("fake_verdict.py").await;
    assert_eq!(
        verdict,
        Verdict::WrongAnswer,
        "a fake result document on stdout is extra output, not a verdict"
    );
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s15a_forged_and_early_answer_lines_are_wrong() {
    let verdict = run_hostile("forged_lines.py").await;
    assert_eq!(
        verdict,
        Verdict::WrongAnswer,
        "forged ids and early answers are protocol violations or wrong answers"
    );
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s15b_an_early_exit_is_a_runtime_error() {
    let verdict = run_hostile("early_exit.py").await;
    assert_eq!(verdict, Verdict::RuntimeError);
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s15c_killing_itself_is_a_runtime_error() {
    let verdict = run_hostile("self_kill.py").await;
    assert_eq!(verdict, Verdict::RuntimeError);
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s15d_closing_stdout_is_a_runtime_error_not_a_hang() {
    let verdict = run_hostile("stdout_close.py").await;
    assert_eq!(
        verdict,
        Verdict::RuntimeError,
        "a closed protocol stream ends the call as a driver exit"
    );
}
