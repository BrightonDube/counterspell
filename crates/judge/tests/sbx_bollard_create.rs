//! Real-daemon test for the bollard create path (judge-sandbox T20, R2.2, R3.15, R1.9).
//!
//! Ignored by default. It needs `SANDBOX_DOCKER_SOCKET` set to the rootless sandbox daemon's
//! socket, and it never runs against the main Docker daemon. The identity check runs before
//! any container is created, so a daemon that is not rootless (or lacks cgroup delegation)
//! stops the test with no container started.
//!
//! Run on the sandbox daemon only:
//! `cargo test -p counterspell-judge --features sandbox --test sbx_bollard_create -- --ignored`
#![cfg(feature = "sandbox")]

use std::path::{Path, PathBuf};
use std::time::Duration;

use counterspell_judge::languages::LanguageCatalog;
use counterspell_judge::sandbox::bollard_backend::{create_checked, remove_forced};
use counterspell_judge::sandbox::config::OciRuntime;
use counterspell_judge::sandbox::daemon::{connect, verify};
use counterspell_judge::sandbox::policy::ContainerPolicy;

const API_TIMEOUT: Duration = Duration::from_secs(30);

/// The shipped language catalogue, so the test uses the same pinned image as production.
fn shipped_catalogue_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../languages/languages.toml")
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET pointing at the rootless sandbox daemon"]
async fn sbx_create_readback_matches() {
    let socket = std::env::var("SANDBOX_DOCKER_SOCKET")
        .expect("SANDBOX_DOCKER_SOCKET must name the rootless sandbox daemon socket");
    let docker = connect(Path::new(&socket), API_TIMEOUT).expect("the socket path is accepted");
    verify(&docker, OciRuntime::Runc, API_TIMEOUT)
        .await
        .expect("the daemon passes the rootless identity check");

    let catalog = LanguageCatalog::load(&shipped_catalogue_path()).expect("languages.toml loads");
    let python = catalog.get("python").expect("python is listed");
    let policy = ContainerPolicy::for_language(python, Duration::from_secs(20), OciRuntime::Runc);

    let name = format!("counterspell-sbx-create-{}", uuid::Uuid::new_v4());
    let id = create_checked(&docker, &policy, &name, API_TIMEOUT)
        .await
        .expect("the pinned image is present and the read-back matches the policy");
    remove_forced(&docker, &id, API_TIMEOUT)
        .await
        .expect("the test container is removed");
}
