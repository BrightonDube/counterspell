// SPDX-License-Identifier: MIT
//! Output scenarios S09–S11 (judge-sandbox T28, R6, R6.3, R6.5).
//!
//! stdout is protocol: it is capped, and only the bytes cap ends a flood that
//! never forms a line. stderr is diagnostics: a flood there answers nothing, so
//! the call ends at the deadline — never at the output cap, and never accepted.
//! Ignored by default.
//!
//! `cargo test -p counterspell-judge --features sandbox --test sbx_output -- --ignored`
#![cfg(feature = "sandbox")]

mod sbx;

use std::time::Duration;

use counterspell_common::events::{PublicEvent, PublicTest};
use counterspell_common::status::{RunMode, TestVerdict, Verdict};
use counterspell_judge::executor::RunOutcome;

/// Runs `source` on `add` and asserts no managed container is still running
/// once the run ends (corpses are reaped like the sweeper would).
async fn run_source(name_or_source: Hostile<'_>) -> sbx::RunObservation {
    let (docker, mut cfg) = sbx::sandbox().await;
    // A dev-daemon margin over the production 500 ms slack (the spec bound,
    // R5.5 = 1 s, is re-verified on the VPS in T37).
    cfg.kill_slack = Duration::from_secs(2);
    let catalog = sbx::catalogue(&cfg);
    let source = match &name_or_source {
        Hostile::Corpus(name) => sbx::hostile(name),
        Hostile::Inline(text) => (*text).to_owned(),
    };
    let observation = sbx::run_source(
        &docker,
        &cfg,
        &catalog,
        "python",
        "add",
        &source,
        RunMode::Submit,
    )
    .await;
    sbx::drain_dead(&docker, &cfg, Duration::from_secs(20)).await;
    observation
}

/// Either a hostile corpus file or an inline source string.
enum Hostile<'a> {
    /// A file under `tests/hostile/`.
    Corpus(&'a str),
    /// Source text written directly into the run.
    Inline(&'a str),
}

/// The run's verdict, or a panic describing the failure instead.
fn verdict(observation: &sbx::RunObservation) -> Verdict {
    let outcome = observation
        .outcome
        .as_ref()
        .expect("the run produces an outcome");
    let RunOutcome::Completed(result) = outcome else {
        panic!("a flood still gets a completed run: {outcome:?}");
    };
    result.verdict
}

/// Every per-test verdict in emission order.
fn test_verdicts(observation: &sbx::RunObservation) -> Vec<TestVerdict> {
    observation
        .events
        .iter()
        .filter_map(|event| match event {
            PublicEvent::Test(
                PublicTest::Hidden { verdict, .. } | PublicTest::Sample { verdict, .. },
            ) => Some(*verdict),
            _ => None,
        })
        .collect()
}

/// The test worker's resident memory, from procfs. The worker buffers stderr
/// and stdout caps in-process; a flood must not grow it.
fn resident_bytes() -> u64 {
    let status = std::fs::read_to_string("/proc/self/status").expect("procfs is readable");
    let kb: u64 = status
        .lines()
        .find_map(|line| {
            line.strip_prefix("VmRSS:")
                .and_then(|rest| rest.trim().strip_suffix(" kB"))
                .and_then(|num| num.trim().parse().ok())
        })
        .expect("VmRSS is present");
    kb * 1024
}

/// S09 — a text flood on the real stdout stream is cut at the byte cap.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s09a_a_stdout_flood_is_cut_at_the_budget() {
    let observation = run_source(Hostile::Corpus("stdout_flood.py")).await;
    assert_eq!(verdict(&observation), Verdict::OutputLimit);
    assert!(
        test_verdicts(&observation).contains(&TestVerdict::OutputLimit),
        "the cap, not a parse failure, ends the call: {:?}",
        test_verdicts(&observation)
    );
}

/// S09 — the same flood through raw `os.write` is cut identically.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s09b_a_raw_stdout_flood_is_cut_at_the_budget() {
    let observation = run_source(Hostile::Corpus("stdout_flood_raw.py")).await;
    assert_eq!(verdict(&observation), Verdict::OutputLimit);
}

/// S10 — an endless stderr flood answers nothing, so the missed deadline ends
/// the call as `time_limit`. stderr alone never trips the output cap.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s10_a_stderr_flood_is_a_time_limit_not_output_limit() {
    let before = resident_bytes();
    let observation = run_source(Hostile::Corpus("stderr_flood.py")).await;
    assert_eq!(verdict(&observation), Verdict::TimeLimit);
    let verdicts = test_verdicts(&observation);
    assert!(
        !verdicts.contains(&TestVerdict::OutputLimit),
        "stderr is never protocol output: {verdicts:?}"
    );
    let after = resident_bytes();
    assert!(
        after.saturating_sub(before) < 256 * 1024 * 1024,
        "the worker's RSS stays bounded during the flood: {before} -> {after}"
    );
}

/// S10 — when the flood ends in an exception, the stored stderr tail still
/// gives the driver its exception channel: the run is a runtime error.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s10b_a_flood_then_exception_is_a_runtime_error() {
    let source = "import os\n\
def add(a, b):\n    \
    for _ in range(400):\n        \
        os.write(2, b'flood\\n' * 700)\n    \
    raise RuntimeError('boom')\n";
    let observation = run_source(Hostile::Inline(source)).await;
    assert_eq!(verdict(&observation), Verdict::RuntimeError);
}

/// S11 — one line far past the budget is cut the same way as a flood.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s11_one_giant_line_is_cut_at_the_budget() {
    let observation = run_source(Hostile::Corpus("long_line.py")).await;
    assert_eq!(verdict(&observation), Verdict::OutputLimit);
}
