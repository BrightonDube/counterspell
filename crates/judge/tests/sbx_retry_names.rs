// SPDX-License-Identifier: MIT
//! Real-daemon test for retried attempts (IDE-3 criterion 7, run-lifecycle retry).
//!
//! A worker killed mid-run leaves its container on the daemon until the sweeper removes it.
//! The retry of the same run must still be able to create its own first container. Before the
//! attempt was part of the name, the daemon answered 409 and every retry ended
//! `sandbox_infrastructure` until the job went dead.
//!
//! Ignored by default. Needs `SANDBOX_DOCKER_SOCKET` set to the rootless sandbox daemon's socket:
//! `cargo test -p counterspell-judge --features sandbox --test sbx_retry_names -- --ignored`
#![cfg(feature = "sandbox")]

use std::path::PathBuf;
use std::time::Duration;

use counterspell_judge::sandbox::backend::{Backend, BackendSession};
use counterspell_judge::sandbox::bollard_backend::{remove_forced, BollardBackend};
use counterspell_judge::sandbox::config::OciRuntime;
use counterspell_judge::sandbox::daemon::{connect, verify};
use counterspell_judge::sandbox::spec::SessionSpec;

mod sbx_support;
use sbx_support::catalogue;
use uuid::Uuid;

const API_TIMEOUT: Duration = Duration::from_secs(30);

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET pointing at the rootless sandbox daemon"]
async fn sbx_retry_creates_its_first_container_beside_the_crashed_attempts() {
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
    let run = Uuid::new_v4();
    let spec = |attempt: u16| {
        SessionSpec::for_attempt(
            python,
            Duration::from_secs(60),
            OciRuntime::Runc,
            run,
            "sbx-retry",
            attempt,
            0,
            i64::MAX / 2,
        )
        .expect("the session spec builds")
    };

    // Attempt 1's first container, left behind as a killed worker would leave it.
    let left_behind = backend
        .create(&spec(1))
        .await
        .expect("attempt 1 creates its container");
    let left_behind = left_behind.name().to_owned();

    // Attempt 2 of the same run opens its first container while that one still exists.
    let retried = backend.create(&spec(2)).await;
    let retried_name = retried.as_ref().map(|s| s.name().to_owned());

    remove_forced(&docker, &left_behind, API_TIMEOUT)
        .await
        .expect("the left-behind container is removed by the test");
    if let Ok(name) = &retried_name {
        remove_forced(&docker, name, API_TIMEOUT)
            .await
            .expect("the retry's container is removed by the test");
    }
    let name = retried_name.expect("the retry must not collide with the crashed attempt");
    assert_ne!(name, left_behind);
}
