// SPDX-License-Identifier: MIT
//! Concurrency scenarios S18, S19 (judge-sandbox T32, R11).
//!
//! Runs that share one executor share one container semaphore: a fork bomb in
//! one slot must not starve or taint a correct solution in the next, and the
//! daemon's own container list — sampled out-of-band — may never hold more
//! than `SANDBOX_MAX_CONTAINERS`. Ignored by default.
//!
//! `cargo test -p counterspell-judge --features sandbox --test sbx_concurrency -- --ignored`
#![cfg(feature = "sandbox")]

mod sbx;

use std::sync::Arc;
use std::time::{Duration, Instant};

use counterspell_common::status::{RunMode, Verdict};
use counterspell_judge::executor::RunOutcome;
use tokio::sync::watch;

/// The run's verdict, or a panic describing the failure instead.
fn verdict(observation: &sbx::RunObservation) -> Verdict {
    let outcome = observation
        .outcome
        .as_ref()
        .expect("the run produces an outcome");
    let RunOutcome::Completed(result) = outcome else {
        panic!("a concurrent run still gets a completed outcome: {outcome:?}");
    };
    result.verdict
}

/// S18 — a fork bomb and a correct solution run concurrently through one
/// executor (the `JUDGE_CONCURRENCY=2` case: two in-flight runs). The good run
/// is accepted inside its own limits; the bomb is contained, never accepted.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s18_a_bomb_in_one_slot_cannot_hurt_the_other() {
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let executor = sbx::shared_executor(&docker, &cfg, &catalog).await;

    let (stop_sample, signal) = watch::channel(false);
    let sampler = tokio::spawn(sbx::sample_managed_peak(
        docker.clone(),
        cfg.clone(),
        Duration::from_millis(200),
        signal,
    ));

    let bomb_executor = executor.clone();
    let bomb = tokio::spawn(async move {
        sbx::execute_on(
            &bomb_executor,
            "python",
            "add",
            &sbx::hostile("fork_bomb.py"),
            RunMode::Submit,
        )
        .await
    });
    let good_executor = executor.clone();
    let good = tokio::spawn(async move {
        sbx::execute_on(
            &good_executor,
            "python",
            "add",
            &sbx::solution("add_correct.py"),
            RunMode::Submit,
        )
        .await
    });

    let bomb_obs = bomb.await.expect("the bomb run completes");
    let good_obs = good.await.expect("the good run completes");
    let _ = stop_sample.send(true);
    let peak = sampler.await.expect("the sampler finishes");
    assert!(
        peak.running <= cfg.max_containers,
        "the semaphore bound held while both runs were live: peak running {} (listed {})",
        peak.running,
        peak.listed
    );

    assert_eq!(
        verdict(&good_obs),
        Verdict::Accepted,
        "the good run is accepted while a fork bomb runs beside it"
    );
    assert!(
        good_obs.run_wall <= sbx::RUN_CAP + Duration::from_secs(10),
        "the good run stayed inside its limits: {:?}",
        good_obs.run_wall
    );
    assert_ne!(
        bomb_obs.outcome.as_ref().ok().map(|outcome| match outcome {
            RunOutcome::Completed(result) => result.verdict,
            RunOutcome::Failed(_) => Verdict::RuntimeError,
        }),
        Some(Verdict::Accepted),
        "the fork bomb is never accepted"
    );
    println!(
        "S18 timing: good={:?} bomb={:?} peak_running={} peak_listed={}",
        good_obs.run_wall, bomb_obs.run_wall, peak.running, peak.listed
    );
}

/// S19 — more runs than slots: six runs through one executor with
/// `SANDBOX_MAX_CONTAINERS=4`, sampling the daemon's container list the whole
/// time. The observed peak may never exceed the limit.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s19_never_more_containers_than_the_limit() {
    let (docker, cfg) = sbx::sandbox().await;
    // Earlier scenarios' kills can still be tearing down — a running leftover
    // would count against the sampled peak without being this executor's
    // container. Start from a clean daemon so the peak is only this batch's.
    sbx::drain_dead(&docker, &cfg, Duration::from_secs(90)).await;
    let catalog = sbx::catalogue(&cfg);
    let executor = Arc::clone(&sbx::shared_executor(&docker, &cfg, &catalog).await);

    let (stop_sample, signal) = watch::channel(false);
    let sampler = tokio::spawn(sbx::sample_managed_peak(
        docker.clone(),
        cfg.clone(),
        Duration::from_millis(200),
        signal,
    ));

    let started = Instant::now();
    let mut handles = Vec::new();
    // Six runs, four slots: two waves. Four slow runs keep slots occupied long
    // enough for the sampler to catch any overshoot; two quick ones queue.
    let sources = [
        sbx::hostile("sleep_forever.py"),
        sbx::hostile("infinite_loop.py"),
        sbx::hostile("sleep_forever.py"),
        sbx::hostile("infinite_loop.py"),
        sbx::solution("add_correct.py"),
        sbx::solution("add_wrong.py"),
    ];
    for source in sources {
        let executor = executor.clone();
        handles.push(tokio::spawn(async move {
            sbx::execute_on(&executor, "python", "add", &source, RunMode::Submit).await
        }));
    }
    let mut walls = Vec::new();
    for handle in handles {
        let observation = handle.await.expect("every queued run completes");
        walls.push(format!("{:?}", observation.run_wall));
    }
    let _ = stop_sample.send(true);
    let peak = sampler.await.expect("the sampler finishes");
    println!(
        "S19 timing: batch={:?} per-run={walls:?} peak_running={} peak_listed={} limit={}",
        started.elapsed(),
        peak.running,
        peak.listed,
        cfg.max_containers
    );
    // The semaphore bounds live sessions; a kill's corpse can still be listed
    // while the next run's container already runs. `running` is the bound the
    // requirement means — the listed peak is recorded for the report.
    assert!(
        peak.running <= cfg.max_containers,
        "the daemon never held more running containers than the limit: {} > {}",
        peak.running,
        cfg.max_containers
    );
    assert!(
        peak.running >= 2,
        "runs actually overlapped — a peak of {} would mean serial execution",
        peak.running
    );
}
