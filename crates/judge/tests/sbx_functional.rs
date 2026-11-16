// SPDX-License-Identifier: MIT
//! Functional sandbox scenarios S01–S04, S27 and S28 (judge-sandbox T26, R8.4, R13.7,
//! R13.8, R14.7, R15.1, R16.3).
//!
//! Ignored by default: they run real code on the rootless sandbox daemon named by
//! `SANDBOX_DOCKER_SOCKET`, never on the host.
//!
//! `cargo test -p counterspell-judge --features sandbox --test sbx_functional -- --ignored`
#![cfg(feature = "sandbox")]

mod sbx;

use counterspell_common::events::PublicTest;
use counterspell_common::status::{FailureReason, RunMode, Verdict};
use counterspell_judge::executor::RunOutcome;

/// The verdict of a completed run, or a panic describing what happened instead.
fn verdict_of(observation: &sbx::RunObservation) -> Verdict {
    let outcome = observation
        .outcome
        .as_ref()
        .expect("the run produces an outcome");
    let RunOutcome::Completed(result) = outcome else {
        panic!("expected a completed run: {outcome:?}");
    };
    result.verdict
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s01_a_correct_solution_is_accepted_with_compiling_first() {
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let observation = sbx::run_source(
        &docker,
        &cfg,
        &catalog,
        "python",
        "add",
        &sbx::solution("add_correct.py"),
        RunMode::Submit,
    )
    .await;

    assert_eq!(observation.event_names().first(), Some(&"compiling"));
    assert_eq!(verdict_of(&observation), Verdict::Accepted);
    assert_eq!(
        observation.event_names(),
        [
            "compiling",
            "test",
            "test",
            "test",
            "test",
            "test",
            "familiar"
        ]
    );
    assert!(
        sbx::cleared_within(&docker, &cfg, std::time::Duration::from_secs(15))
            .await
            .is_empty(),
        "no container survives the run"
    );
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s02_a_wrong_solution_fails_with_sample_detail_and_redacted_hidden_tests() {
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let observation = sbx::run_source(
        &docker,
        &cfg,
        &catalog,
        "python",
        "add",
        &sbx::solution("add_wrong.py"),
        RunMode::Submit,
    )
    .await;

    assert_eq!(verdict_of(&observation), Verdict::WrongAnswer);
    let mut samples = 0;
    let mut hidden = 0;
    for event in &observation.events {
        let counterspell_common::events::PublicEvent::Test(test) = event else {
            continue;
        };
        match test {
            PublicTest::Sample { .. } => samples += 1,
            PublicTest::Hidden { .. } => hidden += 1,
        }
    }
    assert_eq!(samples, 2, "both sample tests keep their public detail");
    assert_eq!(
        hidden, 3,
        "every hidden test record is verdict-and-time only"
    );
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s03a_a_syntax_error_is_a_compile_error_not_a_crash() {
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let observation = sbx::run_source(
        &docker,
        &cfg,
        &catalog,
        "python",
        "add",
        "def add(a, b)\n    return a + b\n", // no colon: does not compile
        RunMode::Submit,
    )
    .await;

    assert_eq!(observation.event_names().first(), Some(&"compiling"));
    assert_eq!(verdict_of(&observation), Verdict::CompileError);
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s03b_a_disabled_language_fails_the_run() {
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let observation = sbx::run_source(
        &docker,
        &cfg,
        &catalog,
        "java", // present in the catalogue but disabled
        "add",
        "class Add { int add(int a, int b) { return a + b; } }",
        RunMode::Submit,
    )
    .await;

    let outcome = observation.outcome.expect("the run produces an outcome");
    assert!(
        matches!(
            outcome,
            RunOutcome::Failed(FailureReason::LanguageUnavailable)
        ),
        "a disabled language fails the run: {outcome:?}"
    );
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s04_an_exception_is_a_runtime_error() {
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let observation = sbx::run_source(
        &docker,
        &cfg,
        &catalog,
        "python",
        "add",
        "def add(a, b):\n    raise ValueError('boom')\n",
        RunMode::Submit,
    )
    .await;

    let verdict = verdict_of(&observation);
    assert_eq!(
        verdict,
        Verdict::RuntimeError,
        "an exception ends the tests, not the run"
    );
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s27_imports_alone_are_not_hostile() {
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let observation = sbx::run_source(
        &docker,
        &cfg,
        &catalog,
        "python",
        "add",
        &sbx::hostile("import_all.py"),
        RunMode::Submit,
    )
    .await;

    assert_eq!(
        verdict_of(&observation),
        Verdict::Accepted,
        "importing os/subprocess/socket/ctypes and answering correctly must pass"
    );
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s28_remember_and_replay_never_leaks_hidden_input() {
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let observation = sbx::run_source(
        &docker,
        &cfg,
        &catalog,
        "python",
        "add",
        &sbx::hostile("remember_replay.py"),
        RunMode::Submit,
    )
    .await;

    // Samples run first and keep their detail; every record after the first hidden test
    // is verdict-and-time only, whatever the program printed.
    let names = observation.event_names();
    assert_eq!(names.first(), Some(&"compiling"));
    let tests: Vec<&PublicTest> = observation
        .events
        .iter()
        .filter_map(|event| match event {
            counterspell_common::events::PublicEvent::Test(test) => Some(test),
            _ => None,
        })
        .collect();
    assert!(
        tests.len() >= 3,
        "the run reached the hidden phase: {names:?}"
    );
    assert!(
        matches!(tests[0], PublicTest::Sample { ordinal: 1, .. })
            && matches!(tests[1], PublicTest::Sample { ordinal: 2, .. }),
        "sample tests run first with their detail: {tests:?}"
    );
    for test in &tests[2..] {
        assert!(
            matches!(test, PublicTest::Hidden { .. }),
            "after the first hidden test no record may carry output: {test:?}"
        );
    }
    let verdict = verdict_of(&observation);
    assert_ne!(
        verdict,
        Verdict::Accepted,
        "replaying hidden input is extra output, not a verdict"
    );
}
