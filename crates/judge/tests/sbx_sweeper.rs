// SPDX-License-Identifier: MIT
//! Real-daemon test for the sweeper (judge-sandbox T23, R10.3 and R10.4).
//!
//! Ignored by default. It needs `SANDBOX_DOCKER_SOCKET` set to the rootless sandbox
//! daemon's socket, and it never runs against the main Docker daemon.
//!
//! Run on the sandbox daemon only:
//! `cargo test -p counterspell-judge --features sandbox --test sbx_sweeper -- --ignored`
#![cfg(feature = "sandbox")]

use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use bollard::query_parameters::InspectContainerOptions;
use counterspell_judge::sandbox::backend::{Backend, BackendSession};
use counterspell_judge::sandbox::bollard_backend::{
    create_started_unchecked, remove_forced, BollardBackend,
};
use counterspell_judge::sandbox::config::OciRuntime;
use counterspell_judge::sandbox::daemon::{connect, verify};
use counterspell_judge::sandbox::policy::ContainerPolicy;
use counterspell_judge::sandbox::reaper::sweep_once;
use counterspell_judge::sandbox::spec::SessionSpec;

mod sbx_support;
use sbx_support::catalogue;
use uuid::Uuid;

const API_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET pointing at the rootless sandbox daemon"]
async fn sbx_sweeper_removes_orphan_and_ignores_unlabelled() {
    let socket = std::env::var("SANDBOX_DOCKER_SOCKET")
        .expect("SANDBOX_DOCKER_SOCKET must name the rootless sandbox daemon socket");
    let socket: PathBuf = socket.into();
    let docker = connect(&socket, API_TIMEOUT).expect("the socket path is accepted");
    verify(&docker, OciRuntime::Runc, API_TIMEOUT)
        .await
        .expect("the daemon passes the rootless identity check");
    let backend = BollardBackend::new(docker.clone(), API_TIMEOUT);

    let (_dir, catalog) = catalogue();
    let python = catalog.get("python").expect("python is in the catalogue");

    // A managed container, created but never started: an orphan by construction.
    let spec = SessionSpec::new(
        python,
        Duration::from_secs(60),
        OciRuntime::Runc,
        Uuid::new_v4(),
        "sbx-sweeper",
        1,
        i64::MAX / 2,
    )
    .expect("the session spec builds");
    let session = backend.create(&spec).await.expect("the orphan is created");
    let orphan_id = session.name().to_owned();

    // A running container without the managed label, as any unrelated container on the
    // host. The sweeper must not touch it.
    let mut stray_policy =
        ContainerPolicy::for_language(python, Duration::from_secs(60), OciRuntime::Runc);
    stray_policy.labels.remove("counterspell.managed");
    let stray_name = format!("counterspell-sbx-stray-{}", Uuid::new_v4());
    let stray_id = create_started_unchecked(&docker, &stray_policy, &stray_name, API_TIMEOUT)
        .await
        .expect("the stray container creates");

    // Make the orphan strictly older than a zero orphan age.
    tokio::time::sleep(Duration::from_millis(1_100)).await;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("the clock works")
        .as_secs();
    let now = i64::try_from(now).unwrap_or(i64::MAX);
    let outcome = sweep_once(&backend, Duration::ZERO, now)
        .await
        .expect("sweep runs");
    assert!(outcome.removed >= 1, "the orphan was swept: {outcome:?}");

    // The managed orphan is gone; the unlabelled stray survives.
    let orphan_inspect = docker
        .inspect_container(&orphan_id, None::<InspectContainerOptions>)
        .await;
    assert!(
        matches!(
            orphan_inspect,
            Err(bollard::errors::Error::DockerResponseServerError {
                status_code: 404,
                ..
            })
        ),
        "the orphan must be gone: {orphan_inspect:?}"
    );
    docker
        .inspect_container(&stray_id, None::<InspectContainerOptions>)
        .await
        .expect("the unlabelled container is still there");
    remove_forced(&docker, &stray_id, API_TIMEOUT)
        .await
        .expect("the stray is removed by the test");
}
