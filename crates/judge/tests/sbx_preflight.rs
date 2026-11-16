// SPDX-License-Identifier: MIT
//! Real-daemon tests for the sandbox preflight probe (judge-sandbox T22, R4.1 to R4.6).
//!
//! Ignored by default. They need `SANDBOX_DOCKER_SOCKET` set to the rootless sandbox
//! daemon's socket, and they never run against the main Docker daemon. The identity
//! check runs before any container is created.
//!
//! Run on the sandbox daemon only:
//! `cargo test -p counterspell-judge --features sandbox --test sbx_preflight -- --ignored`
#![cfg(feature = "sandbox")]

use std::path::PathBuf;
use std::time::Duration;

use counterspell_judge::sandbox::bollard_backend::{
    create_started_unchecked, kill_container, remove_forced, run_exec, BollardBackend,
};
use counterspell_judge::sandbox::config::OciRuntime;
use counterspell_judge::sandbox::daemon::{connect, verify};
use counterspell_judge::sandbox::policy::ContainerPolicy;
use counterspell_judge::sandbox::preflight::{
    evaluate, probe_exec, probe_outputs, probe_spec, run_probe, PROBE_COMMANDS,
};
use counterspell_judge::sandbox::testkit::{weakened_policy, WeakenedControl};

mod sbx_support;
use sbx_support::catalogue;
use uuid::Uuid;

const API_TIMEOUT: Duration = Duration::from_secs(30);
const RUN_CAP: Duration = Duration::from_secs(20);

/// Connects to the sandbox daemon named by the environment and runs the identity check.
/// Returns the raw client, because the weakened-policy test must bypass `create_checked`
/// to build a container the production path would refuse.
async fn sandbox_client() -> (bollard::Docker, BollardBackend) {
    let socket = std::env::var("SANDBOX_DOCKER_SOCKET")
        .expect("SANDBOX_DOCKER_SOCKET must name the rootless sandbox daemon socket");
    let socket: PathBuf = socket.into();
    let docker = connect(&socket, API_TIMEOUT).expect("the socket path is accepted");
    verify(&docker, OciRuntime::Runc, API_TIMEOUT)
        .await
        .expect("the daemon passes the rootless identity check");
    let backend = BollardBackend::new(docker.clone(), API_TIMEOUT);
    (docker, backend)
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET pointing at the rootless sandbox daemon"]
async fn sbx_preflight_passes() {
    let (_docker, backend) = sandbox_client().await;
    let (_dir, catalog) = catalogue();
    let python = catalog.get("python").expect("python is in the catalogue");
    let spec = probe_spec(python, RUN_CAP, OciRuntime::Runc).expect("the probe spec builds");

    let report = run_probe(&backend, &spec)
        .await
        .expect("the probe ran to completion");
    assert!(report.passed(), "failing checks: {:?}", report.failures());
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET pointing at the rootless sandbox daemon"]
async fn sbx_preflight_fails_when_policy_weakened() {
    let (docker, _backend) = sandbox_client().await;
    let (_dir, catalog) = catalogue();
    let python = catalog.get("python").expect("python is in the catalogue");
    let production = ContainerPolicy::for_language(python, RUN_CAP, OciRuntime::Runc);
    // The weakened constructor is testkit-only: a writable root filesystem makes the
    // write-refusal and root-read-only checks fail.
    let weakened = weakened_policy(
        python,
        RUN_CAP,
        OciRuntime::Runc,
        &[WeakenedControl::WritableRoot],
    );

    let name = format!("counterspell-sbx-weak-{}", Uuid::new_v4());
    let id = create_started_unchecked(&docker, &weakened, &name, API_TIMEOUT)
        .await
        .expect("the weakened container creates and starts");

    let mut results = Vec::with_capacity(PROBE_COMMANDS.len());
    for argv in PROBE_COMMANDS {
        results.push(
            run_exec(&docker, &id, &probe_exec(argv), &[], API_TIMEOUT)
                .await
                .expect("a probe command ran"),
        );
    }
    let outputs = probe_outputs(results).expect("every probe command returned");
    let report = evaluate(&outputs, &production);

    kill_container(&docker, &id, API_TIMEOUT)
        .await
        .expect("the weakened container is killed");
    remove_forced(&docker, &id, API_TIMEOUT)
        .await
        .expect("the weakened container is removed");

    assert!(!report.passed(), "the weakened policy must not pass");
    let failed: Vec<&str> = report.failures().iter().map(|check| check.name).collect();
    for expected in ["root_read_only", "read_only_write_refused"] {
        assert!(
            failed.contains(&expected),
            "{expected} must fail; failing: {failed:?}"
        );
    }
}
