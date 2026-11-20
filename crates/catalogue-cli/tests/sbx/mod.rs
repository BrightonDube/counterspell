// SPDX-License-Identifier: MIT
//! Shared harness for the real-daemon pack-runner tests.
//!
//! Every helper runs against the rootless sandbox daemon that
//! `SANDBOX_DOCKER_SOCKET` names — after `daemon::verify` has proven the
//! identity checks, never against the main Docker socket. These tests are
//! `#[ignore]`d by default because they need that daemon.
//!
//! Tests that touch the real starter packs also need
//! `COUNTERSPELL_PACKS_PRIVATE_ROOT`: a checkout of `counterspell-packs-private`
//! holding each slug's `reference/` and `incorrect/` files.

#![cfg(feature = "sandbox")]
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::time::Duration;

use bollard::Docker;
use counterspell_judge::languages::LanguageCatalog;
use counterspell_judge::sandbox::backend::{Backend, ManagedContainer};
use counterspell_judge::sandbox::bollard_backend::BollardBackend;
use counterspell_judge::sandbox::config::{OciRuntime, SandboxConfig};
use counterspell_judge::sandbox::daemon::{connect, verify};

/// The longest a single daemon call may take in scenario tests.
pub const API_TIMEOUT: Duration = Duration::from_secs(30);
/// How long a scenario's runs may take before the harness itself gives up.
pub const RUN_CAP: Duration = Duration::from_secs(20);

/// The repository root: `crates/catalogue-cli/` is two levels down.
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The private pack-tree checkout (`COUNTERSPELL_PACKS_PRIVATE_ROOT`). `None`
/// when unset, so pack tests can skip.
pub fn private_packs_root() -> Option<PathBuf> {
    std::env::var("COUNTERSPELL_PACKS_PRIVATE_ROOT")
        .ok()
        .map(PathBuf::from)
        .or_else(|| {
            let sibling = repo_root().join("../counterspell-packs-private");
            sibling.is_dir().then_some(sibling)
        })
}

/// One pack's private tree inside the checkout (`packs/<slug>/`).
pub fn private_pack_dir(slug: &str) -> Option<PathBuf> {
    private_packs_root().map(|root| root.join("packs").join(slug))
}

/// A scenario config: like production's, but with the socket from the environment and
/// modest caps so tests stay quick.
pub fn test_config() -> SandboxConfig {
    let socket = std::env::var("SANDBOX_DOCKER_SOCKET")
        .expect("SANDBOX_DOCKER_SOCKET must name the rootless sandbox daemon socket");
    let languages_file = std::env::var("LANGUAGES_FILE").map_or_else(
        |_| repo_root().join("languages/languages.toml"),
        PathBuf::from,
    );
    SandboxConfig {
        docker_socket: socket.into(),
        runtime: OciRuntime::Runc,
        max_containers: 4,
        reaper_interval: Duration::from_secs(10),
        orphan_age: Duration::from_secs(40),
        kill_slack: Duration::from_millis(500),
        run_wall_cap: RUN_CAP,
        max_consecutive_abnormal: 3,
        cleanup_budget: Duration::from_secs(5),
        api_timeout: API_TIMEOUT,
        languages_file,
    }
}

/// Connects to the sandbox daemon and proves its identity. Panics when the daemon fails
/// the rootless checks — a scenario test must never run against the main daemon.
pub async fn sandbox() -> (Docker, SandboxConfig) {
    let cfg = test_config();
    let docker = connect(&cfg.docker_socket, cfg.api_timeout).expect("the socket path is accepted");
    verify(&docker, cfg.runtime, cfg.api_timeout)
        .await
        .expect("the daemon passes the rootless identity check");
    (docker, cfg)
}

/// The shipped catalogue, loading the real driver files from `languages/`.
pub fn catalogue(cfg: &SandboxConfig) -> LanguageCatalog {
    LanguageCatalog::load(&cfg.languages_file).expect("the shipped catalogue loads")
}

/// The managed containers the daemon reports, through the backend's label filter.
async fn managed_containers(backend: &BollardBackend) -> Vec<ManagedContainer> {
    backend
        .list_managed()
        .await
        .expect("the daemon lists containers")
}

/// Asserts no managed container is still *running* within `within`, then reaps
/// every leftover and returns how many there were. Abnormal-end containers are
/// killed and removed on the call path, but a removal can lag the verdict.
pub async fn drain_dead(docker: &Docker, cfg: &SandboxConfig, within: Duration) -> usize {
    let backend = BollardBackend::new(docker.clone(), cfg.api_timeout);
    let deadline = tokio::time::Instant::now() + within;
    let mut leftovers = managed_containers(&backend).await;
    loop {
        let mut survivors = Vec::new();
        for container in &leftovers {
            // A container can vanish between listing and inspection when the
            // daemon finishes a removal — that is the clean outcome.
            let Ok(inspected) = docker.inspect_container(&container.id, None).await else {
                continue;
            };
            if inspected
                .state
                .and_then(|state| state.running)
                .unwrap_or(false)
            {
                survivors.push(container.name.clone());
            }
        }
        if survivors.is_empty() || tokio::time::Instant::now() >= deadline {
            assert!(
                survivors.is_empty(),
                "containers still running — the program was not contained: {survivors:?}"
            );
            break;
        }
        tokio::time::sleep(Duration::from_millis(400)).await;
        leftovers = managed_containers(&backend).await;
    }
    let mut reaped = 0;
    for container in &leftovers {
        if backend.remove_managed(&container.id).await.is_ok() {
            reaped += 1;
        }
    }
    reaped
}

/// One daemon-backed test at a time within a binary: `drain_dead` counts every
/// managed container, so two overlapping tests would flag each other's live
/// containers as leftovers.
pub static ONE_RUN_AT_A_TIME: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());
