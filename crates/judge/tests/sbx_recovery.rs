// SPDX-License-Identifier: MIT
//! Cleanup and crash scenarios S16, S17, S24 (judge-sandbox T31, R10, R5.9).
//!
//! A worker that dies mid-run must not leak its container: PID 1 self-expiry
//! plus the sweeper remove it within sixty seconds of creation, and the
//! sweeper alone is shown to be enough. A finished or stopped run leaves the
//! daemon listing no container for it. The daemon's own container list is the
//! out-of-band proof — never what the run reported. Ignored by default.
//!
//! `cargo test -p counterspell-judge --features sandbox --test sbx_recovery -- --ignored`
#![cfg(feature = "sandbox")]

mod sbx;

use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use bollard::Docker;
use counterspell_common::status::{RunMode, Verdict};
use counterspell_judge::executor::{ExecError, RunOutcome};
use counterspell_judge::sandbox::backend::ManagedContainer;
use counterspell_judge::sandbox::bollard_backend::BollardBackend;
use counterspell_judge::sandbox::config::SandboxConfig;
use counterspell_judge::sandbox::reaper::run_sweeper;
use tokio::sync::watch;

/// Seconds after the container's creation by which nothing labelled may remain.
const GONE_BOUND: i64 = 60;
/// PID-1 sleep slack the policy adds on top of the run cap.
const IDLE_EXTRA: i64 = 5;

/// The run's verdict, or a panic describing the failure instead.
fn verdict(observation: &sbx::RunObservation) -> Verdict {
    let outcome = observation
        .outcome
        .as_ref()
        .expect("the run produces an outcome");
    let RunOutcome::Completed(result) = outcome else {
        panic!("a cleaned-up run still gets a completed outcome: {outcome:?}");
    };
    result.verdict
}

fn now_unix() -> i64 {
    i64::try_from(
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("the clock is past the epoch")
            .as_secs(),
    )
    .unwrap_or(i64::MAX)
}

/// The managed containers belonging to `run_id` (`cs-sbx-<run>-<ordinal>`).
/// Preflight probes carry a different run id, so they never match.
async fn run_containers(backend: &BollardBackend, run_id: &str) -> Vec<ManagedContainer> {
    sbx::managed_containers(backend)
        .await
        .into_iter()
        .filter(|container| container.name.contains(run_id))
        .collect()
}

/// Polls until no container of `run_id` remains or `created_at + GONE_BOUND`
/// passes, then returns the last list (empty on success).
async fn gone_by(backend: &BollardBackend, run_id: &str, created_at: i64) -> Vec<ManagedContainer> {
    let deadline = Instant::now()
        + Duration::from_secs(u64::try_from(created_at + GONE_BOUND - now_unix()).unwrap_or(0));
    let mut left = run_containers(backend, run_id).await;
    while !left.is_empty() && Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(500)).await;
        left = run_containers(backend, run_id).await;
    }
    left
}

/// Builds the `sbx_victim` example and returns its binary path. Incremental, so
/// repeated calls within a suite cost a no-op rebuild check.
#[allow(clippy::disallowed_methods)] // S16's trusted helper is itself spawned as a process.
async fn victim_binary() -> PathBuf {
    let status = tokio::task::spawn_blocking(|| {
        Command::new("cargo")
            .args([
                "build",
                "-q",
                "-p",
                "counterspell-judge",
                "--features",
                "sandbox,testkit",
                "--example",
                "sbx_victim",
            ])
            .current_dir(sbx::repo_root())
            .status()
    })
    .await
    .expect("the build task runs")
    .expect("cargo builds the victim example");
    assert!(status.success(), "cargo build sbx_victim: {status}");
    let target = std::env::var("CARGO_TARGET_DIR")
        .map_or_else(|_| sbx::repo_root().join("target"), PathBuf::from);
    let binary = target.join("debug/examples/sbx_victim");
    assert!(
        binary.is_file(),
        "the victim binary exists: {}",
        binary.display()
    );
    binary
}

/// Spawns the victim example with `run_cap_secs` as its run cap, waits until
/// its run's container appears on the daemon, and returns the child plus the
/// container. The victim prints `VICTIM-RUN:<uuid>` once it submits the run.
#[allow(clippy::disallowed_methods)] // S16 kills the helper with `Child::kill`, as the spec asks.
async fn spawn_victim(
    docker: &Docker,
    cfg: &SandboxConfig,
    run_cap_secs: u64,
) -> (Child, String, ManagedContainer) {
    let binary = victim_binary().await;
    let child = Command::new(&binary)
        .env("SANDBOX_DOCKER_SOCKET", &cfg.docker_socket)
        .env("SBX_VICTIM_RUN_CAP_SECS", run_cap_secs.to_string())
        .stdout(Stdio::piped())
        .spawn()
        .expect("the victim process spawns");
    let run_id = tokio::task::spawn_blocking(move || {
        let mut child = child;
        let stdout = child.stdout.take().expect("victim stdout");
        for line in BufReader::new(stdout).lines() {
            let line = line.expect("victim stdout is readable");
            if let Some(id) = line.strip_prefix("VICTIM-RUN:") {
                return (id.to_owned(), child);
            }
        }
        panic!("the victim exited without submitting a run");
    })
    .await
    .expect("the victim stdout task runs");
    let (run_id, child) = run_id;
    let backend = BollardBackend::new(docker.clone(), cfg.api_timeout);
    let deadline = Instant::now() + Duration::from_secs(60);
    let container = loop {
        if let Some(container) = run_containers(&backend, &run_id).await.into_iter().next() {
            break container;
        }
        assert!(Instant::now() < deadline, "the run's container appears");
        tokio::time::sleep(Duration::from_millis(300)).await;
    };
    (child, run_id, container)
}

/// Runs the sweeper loop until `stop` fires; the scenario stops it once the
/// daemon is clean so the task does not outlive the test.
fn start_sweeper(docker: &Docker, cfg: &SandboxConfig) -> watch::Sender<bool> {
    let backend = BollardBackend::new(docker.clone(), cfg.api_timeout);
    let (stop, signal) = watch::channel(false);
    let interval = cfg.reaper_interval;
    let orphan_age = cfg.orphan_age;
    tokio::spawn(async move {
        run_sweeper(backend, interval, orphan_age, signal).await;
    });
    stop
}

/// S16 — a `SIGKILL`ed worker leaves no labelled container behind, once with
/// self-expiry and the sweeper both acting (normal policy), and once with the
/// sweeper alone (PID 1 outlives the orphan age).
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s16_sigkilled_worker_leaves_no_labelled_container() {
    let (docker, cfg) = sbx::sandbox().await;
    // The normal test cap gives PID 1 a `sleep 25` — self-expiry stops the
    // container five seconds past the cap and the sweeper removes it.
    sigkill_round(&docker, &cfg, sbx::RUN_CAP.as_secs()).await;
    // `sleep 125` cannot expire inside the sixty-second bound, so the removal
    // is the sweeper's work alone.
    sigkill_round(&docker, &cfg, 120).await;
}

async fn sigkill_round(docker: &Docker, cfg: &SandboxConfig, run_cap_secs: u64) {
    let (mut child, run_id, container) = spawn_victim(docker, cfg, run_cap_secs).await;
    let created_at = container.created_at;
    child.kill().expect("SIGKILL reaches the victim");
    assert!(
        child.wait().expect("the victim exits").code().is_none(),
        "the victim died by signal, not by exit code"
    );
    let stop = start_sweeper(docker, cfg);
    let backend = BollardBackend::new(docker.clone(), cfg.api_timeout);
    let left = gone_by(&backend, &run_id, created_at).await;
    let gone_at = now_unix();
    let _ = stop.send(true);
    assert!(
        left.is_empty(),
        "60 s after creation nothing labelled may remain: {left:?}"
    );
    // With a long PID 1 the container could not have expired by itself — the
    // sweeper's removal of a still-running container is the only explanation.
    if i64::try_from(run_cap_secs).unwrap() + IDLE_EXTRA > GONE_BOUND {
        assert!(
            gone_at < created_at + i64::try_from(run_cap_secs).unwrap() + IDLE_EXTRA,
            "self-expiry could not have fired yet: the sweeper alone removed it"
        );
    }
}

/// S17 — after `execute` returns, on timeout and on cancel, the daemon lists no
/// container for the run.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s17_timeout_and_cancel_leave_no_container() {
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let source = sbx::hostile("sleep_forever.py");

    let timed_out = sbx::run_source(
        &docker,
        &cfg,
        &catalog,
        "python",
        "add",
        &source,
        RunMode::Submit,
    )
    .await;
    assert_eq!(verdict(&timed_out), Verdict::TimeLimit);
    let left = sbx::cleared_within(&docker, &cfg, Duration::from_secs(10)).await;
    assert!(
        left.is_empty(),
        "the timed-out run left a container: {left:?}"
    );

    let (handle, stop, run_id) = sbx::spawn_source(
        &docker,
        &cfg,
        &catalog,
        "python",
        "add",
        &source,
        RunMode::Submit,
    );
    wait_for_container(&docker, &cfg, &run_id).await;
    stop.send(true).expect("the stop signal fires");
    let cancelled = handle.await.expect("the run task completes");
    assert!(
        matches!(cancelled.outcome, Err(ExecError::Stopped)),
        "a cancelled run reports Stopped: {:?}",
        cancelled.outcome
    );
    let left = sbx::cleared_within(&docker, &cfg, Duration::from_secs(10)).await;
    assert!(
        left.is_empty(),
        "the cancelled run left a container: {left:?}"
    );
}

/// S24 — the stop signal during a live call kills and removes the container and
/// the run reports `Stopped`.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s24_stop_during_a_call_kills_and_removes() {
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let (handle, stop, run_id) = sbx::spawn_source(
        &docker,
        &cfg,
        &catalog,
        "python",
        "add",
        &sbx::hostile("sleep_forever.py"),
        RunMode::Submit,
    );
    // The daemon's container list is the proof the call was live when the
    // signal landed — not anything the run itself reported.
    wait_for_container(&docker, &cfg, &run_id).await;
    stop.send(true).expect("the stop signal fires");
    let observation = handle.await.expect("the run task completes");
    assert!(
        matches!(observation.outcome, Err(ExecError::Stopped)),
        "a stopped run reports Stopped: {:?}",
        observation.outcome
    );
    let left =
        sbx::cleared_within(&docker, &cfg, cfg.cleanup_budget + Duration::from_secs(5)).await;
    assert!(
        left.is_empty(),
        "the stopped call's container was killed and removed inside the cleanup budget: {left:?}"
    );
}

/// Polls the daemon until a container of `run_id` exists — the signal that a
/// call is in flight — or fails after thirty seconds.
async fn wait_for_container(docker: &Docker, cfg: &SandboxConfig, run_id: &str) {
    let backend = BollardBackend::new(docker.clone(), cfg.api_timeout);
    let deadline = Instant::now() + Duration::from_secs(30);
    while run_containers(&backend, run_id).await.is_empty() {
        assert!(
            Instant::now() < deadline,
            "a container of the run appears while the call runs"
        );
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
}
