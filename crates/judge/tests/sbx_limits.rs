// SPDX-License-Identifier: MIT
//! Time, memory and process scenarios S05–S08 (judge-sandbox T27, R5, R9.3, R3.9,
//! R3.10).
//!
//! A hostile program that stalls, stops itself or explodes in memory or process
//! count is contained by the cgroup policy and the worker's deadlines — never by
//! anything the program does to itself. Ignored by default.
//!
//! `cargo test -p counterspell-judge --features sandbox --test sbx_limits -- --ignored`
#![cfg(feature = "sandbox")]

mod sbx;

use std::time::Duration;

use counterspell_common::events::PublicEvent;
use counterspell_common::status::{RunMode, TestVerdict, Verdict};
use counterspell_judge::executor::{ExecError, RunOutcome};
use counterspell_judge::sandbox::backend::{Backend, BackendSession};
use counterspell_judge::sandbox::bollard_backend::{parse_oom_kill, BollardBackend};
use counterspell_judge::sandbox::preflight;

/// `add.json`'s per-test `time_limit_ms`.
const TEST_LIMIT_MS: u32 = 2_000;
/// Per-call wall bound: the limit, the configured kill slack (0.5 s) and daemon
/// margin. Requirement R5.5 wants kill + verdict inside `KILL_SLACK` (1 s).
const CALL_BOUND_MS: u32 = TEST_LIMIT_MS + 2_000;
/// Run verdict bound: the harness run cap plus slack for the last cleanup.
const RUN_BOUND: Duration = Duration::from_secs(23);

/// Runs a hostile source on `add` and returns the observation, asserting no
/// managed container is still running once the run ends. `settle` is how long a
/// kill may take to show on the daemon — process bombs take longer because the
/// kernel reaps tens of tasks before `running` flips to false.
async fn run_hostile(name: &str, settle: Duration) -> sbx::RunObservation {
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
    // Containment: no container still running inside the bound; corpses are
    // reaped. Docker's removal of a killed container is asynchronous and lags
    // under load, so a dead label-holder is tolerated, never a running one.
    sbx::drain_dead(&docker, &cfg, settle).await;
    observation
}

/// The verdict of a completed run, or a panic describing the failure instead.
fn verdict(observation: &sbx::RunObservation) -> Verdict {
    let outcome = observation
        .outcome
        .as_ref()
        .expect("the run produces an outcome");
    let RunOutcome::Completed(result) = outcome else {
        panic!("a contained program still gets a completed run: {outcome:?}");
    };
    result.verdict
}

/// Asserts the run finished inside the run-cap bound and no single test record
/// ran longer than the call bound.
fn assert_within_bounds(observation: &sbx::RunObservation) {
    assert!(
        observation.run_wall <= RUN_BOUND,
        "the run was decided within the cap plus slack: {:?}",
        observation.run_wall
    );
    for event in &observation.events {
        let PublicEvent::Test(test) = event else {
            continue;
        };
        let time_ms = match test {
            counterspell_common::events::PublicTest::Hidden { time_ms, .. }
            | counterspell_common::events::PublicTest::Sample { time_ms, .. } => *time_ms,
        };
        assert!(
            time_ms <= CALL_BOUND_MS,
            "a call finished within limit + kill slack: {time_ms} ms"
        );
    }
}

/// The per-test verdicts as plain `TestVerdict`s, in order.
fn test_verdicts(observation: &sbx::RunObservation) -> Vec<TestVerdict> {
    observation
        .events
        .iter()
        .filter_map(|event| match event {
            PublicEvent::Test(
                counterspell_common::events::PublicTest::Hidden { verdict, .. }
                | counterspell_common::events::PublicTest::Sample { verdict, .. },
            ) => Some(*verdict),
            _ => None,
        })
        .collect()
}

/// S05 — an infinite loop never answers; every call ends at the deadline.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s05_an_infinite_loop_is_a_time_limit_inside_the_bounds() {
    let observation = run_hostile("infinite_loop.py", Duration::from_secs(20)).await;
    assert_eq!(verdict(&observation), Verdict::TimeLimit);
    assert!(
        test_verdicts(&observation)
            .iter()
            .all(|verdict| *verdict == TestVerdict::TimeLimit),
        "every ended call is a time_limit: {:?}",
        test_verdicts(&observation)
    );
    assert_within_bounds(&observation);
}

/// S06 — a loop at import time also dies at the deadline: the process stays
/// alive, so the call times out rather than ending as a driver exit.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s06a_an_import_time_loop_is_a_time_limit() {
    let observation = run_hostile("import_loop.py", Duration::from_secs(20)).await;
    assert_eq!(verdict(&observation), Verdict::TimeLimit);
    assert_within_bounds(&observation);
}

/// S06 — an uninterruptible sleep is ended by the deadline.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s06b_sleeping_forever_is_a_time_limit() {
    let observation = run_hostile("sleep_forever.py", Duration::from_secs(20)).await;
    assert_eq!(verdict(&observation), Verdict::TimeLimit);
    assert_within_bounds(&observation);
}

/// S06 — SIGSTOP makes the process unanswerable but the deadline still fires;
/// SIGKILL works on a stopped process.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s06c_stopping_itself_is_a_time_limit() {
    let observation = run_hostile("sigstop_self.py", Duration::from_secs(20)).await;
    assert_eq!(verdict(&observation), Verdict::TimeLimit);
    assert_within_bounds(&observation);
}

/// S07 — steady allocation growth ends at the cgroup memory limit.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s07a_a_memory_bomb_is_contained() {
    let observation = run_hostile("memory_list.py", Duration::from_secs(20)).await;
    assert_ne!(
        verdict(&observation),
        Verdict::Accepted,
        "a memory bomb is never accepted"
    );
    assert!(
        matches!(
            verdict(&observation),
            Verdict::MemoryLimit | Verdict::RuntimeError
        ),
        "memory_limit on oom_kill > 0 or runtime_error on MemoryError: {:?}",
        verdict(&observation)
    );
    if verdict(&observation) == Verdict::MemoryLimit {
        assert!(
            test_verdicts(&observation).contains(&TestVerdict::MemoryLimit),
            "a MemoryLimit run carries the oom evidence in its records"
        );
    }
    assert_within_bounds(&observation);
}

/// S07 — one huge allocation ends the same way.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s07b_a_giant_allocation_is_contained() {
    let observation = run_hostile("memory_alloc.py", Duration::from_secs(20)).await;
    assert_ne!(
        verdict(&observation),
        Verdict::Accepted,
        "a memory bomb is never accepted"
    );
    assert!(
        matches!(
            verdict(&observation),
            Verdict::MemoryLimit | Verdict::RuntimeError
        ),
        "memory_limit on oom_kill > 0 or runtime_error on MemoryError: {:?}",
        verdict(&observation)
    );
    assert_within_bounds(&observation);
}

/// S07 — the `oom_kill` evidence channel is real on this daemon: a probe
/// container's `memory.events` parses, so a `MemoryLimit` verdict above could
/// only have come from the kernel's counter.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s07c_the_oom_kill_counter_is_readable_on_the_daemon() {
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let language = catalog.get("python").expect("python is in the catalogue");
    let spec = preflight::probe_spec(language, cfg.run_wall_cap, cfg.runtime)
        .expect("the probe spec builds");
    let backend = BollardBackend::new(docker.clone(), cfg.api_timeout);
    let mut session = backend.create(&spec).await.expect("a probe container");
    session.start().await.expect("the probe container starts");
    let output = session
        .exec(
            &preflight::probe_exec(&["cat", "/sys/fs/cgroup/memory.events"]),
            &[],
        )
        .await
        .expect("the memory events read runs");
    let count = parse_oom_kill(&String::from_utf8_lossy(&output.stdout))
        .expect("the daemon exposes a parseable oom_kill count");
    let _ = session.kill().await;
    let _ = session.remove().await;
    assert_eq!(count, 0, "a fresh container has no OOM kills");
}

/// S08 — a fork bomb exhausts the container's own PID limit (64), nothing more.
/// Two honest endings: a completed run with a non-accepted verdict, or an
/// infrastructure error when a daemon call on the bombed container times out
/// (the worker reports it instead of guessing). Never `accepted` either way.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s08a_a_fork_bomb_is_contained_by_the_pid_limit() {
    let observation = run_hostile("fork_bomb.py", Duration::from_secs(90)).await;
    match &observation.outcome {
        Ok(RunOutcome::Completed(result)) => assert!(
            matches!(
                result.verdict,
                Verdict::RuntimeError | Verdict::TimeLimit | Verdict::MemoryLimit
            ),
            "the PID limit contains the bomb — never accepted: {:?}",
            result.verdict
        ),
        Err(ExecError::Infrastructure(reason)) => {
            assert!(!reason.is_empty(), "an infra failure still names its cause");
        }
        other => panic!("a contained run completes or reports infra: {other:?}"),
    }
    assert_within_bounds(&observation);
}

/// S08 — a thread bomb is contained the same way (PIDs count tasks).
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s08b_a_thread_bomb_is_contained_by_the_pid_limit() {
    let observation = run_hostile("thread_bomb.py", Duration::from_secs(90)).await;
    assert!(
        matches!(
            verdict(&observation),
            Verdict::RuntimeError | Verdict::TimeLimit | Verdict::MemoryLimit
        ),
        "the PID limit contains the bomb — never accepted: {:?}",
        verdict(&observation)
    );
    assert_within_bounds(&observation);
}
