// SPDX-License-Identifier: MIT
//! Filesystem and host-visibility scenarios S13 and S26 (judge-sandbox T30,
//! R3.2, R3.3, R3.5, R3.6, R1.9, R16.4).
//!
//! Verdicts come from `run_source`; side effects come from out-of-band evidence:
//! the daemon's own filesystem-changes report, the container's mount table, and
//! REPORT lines the corpus prints when run as a probe script — never from what
//! the program claims about itself. Ignored by default.
//!
//! `cargo test -p counterspell-judge --features sandbox --test sbx_filesystem -- --ignored`
#![cfg(feature = "sandbox")]

mod sbx;

use std::time::Duration;

use counterspell_common::status::{RunMode, Verdict};
use counterspell_judge::executor::RunOutcome;
use counterspell_judge::sandbox::backend::{Backend, BackendSession};
use counterspell_judge::sandbox::bollard_backend::BollardBackend;
use counterspell_judge::sandbox::preflight;

/// docker-owned layer entries: the three /etc bind mounts it creates and the
/// tmpfs mountpoints themselves (their contents are outside the layer).
const DOCKER_OWNED: &[&str] = &[
    "/etc/hostname",
    "/etc/hosts",
    "/etc/resolv.conf",
    "/box",
    "/tmp",
    "/dev/shm",
];

/// The exact table a policy-conformant container shows: the overlay root, the
/// virtual filesystems, docker's device binds and masked paths, the two tmpfs
/// scratches, and docker's /etc binds. Anything new fails the test.
const ALLOWED_MOUNTS: &[&str] = &[
    "/",
    "/proc",
    "/proc/acpi",
    "/proc/bus",
    "/proc/fs",
    "/proc/interrupts",
    "/proc/irq",
    "/proc/keys",
    "/proc/kcore",
    "/proc/latency_stats",
    "/proc/scsi",
    "/proc/sys",
    "/proc/sysrq-trigger",
    "/proc/timer_list",
    "/sys",
    "/sys/firmware",
    "/sys/fs/cgroup",
    "/dev",
    "/dev/full",
    "/dev/mqueue",
    "/dev/null",
    "/dev/pts",
    "/dev/random",
    "/dev/shm",
    "/dev/tty",
    "/dev/urandom",
    "/dev/zero",
    "/box",
    "/tmp",
    "/etc/hosts",
    "/etc/hostname",
    "/etc/resolv.conf",
];

/// The run's verdict, or a panic describing the failure instead.
fn verdict(observation: &sbx::RunObservation) -> Verdict {
    let outcome = observation
        .outcome
        .as_ref()
        .expect("the run produces an outcome");
    let RunOutcome::Completed(result) = outcome else {
        panic!("a write attempt still gets a completed run: {outcome:?}");
    };
    result.verdict
}

/// Runs a hostile source on `add`, asserting it is never accepted and leaves no
/// container running.
async fn run_hostile(name: &str) {
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
    assert_ne!(
        verdict(&observation),
        Verdict::Accepted,
        "{name} must never be accepted"
    );
    sbx::drain_dead(&docker, &cfg, Duration::from_secs(20)).await;
}

/// S13 — writes outside the scratch tmpfs are refused, fills stop at the tmpfs
/// size, and nothing is ever accepted.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s13_writes_outside_scratch_are_refused() {
    for name in [
        "write_root.py",
        "write_proc.py",
        "write_sys.py",
        "fill_box.py",
        "big_file.py",
    ] {
        run_hostile(name).await;
    }
}

/// S13 — after the same write programs ran inside a production-policy
/// container, the daemon's filesystem-changes report is empty. Writes to the
/// read-only rootfs failed and scratch writes live on tmpfs, outside the layer.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s13_the_daemon_reports_no_filesystem_changes() {
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let python = catalog.get("python").expect("python is in the catalogue");
    let spec = preflight::probe_spec(python, cfg.run_wall_cap, cfg.runtime).expect("probe spec");
    let backend = BollardBackend::new(docker.clone(), cfg.api_timeout);
    let mut session = backend.create(&spec).await.expect("a probe container");
    session.start().await.expect("the container starts");

    for name in [
        "write_root.py",
        "write_proc.py",
        "write_sys.py",
        "fill_box.py",
        "big_file.py",
    ] {
        let script = sbx::hostile(name);
        let output = session
            .exec(&preflight::probe_exec(&["python3", "-"]), script.as_bytes())
            .await
            .unwrap_or_else(|error| panic!("{name} exec ran: {error}"));
        assert_eq!(output.exit_code, 0, "{name} ran to completion");
    }

    let changes = docker
        .container_changes(session.name())
        .await
        .expect("the daemon reports filesystem changes")
        .unwrap_or_default();
    let _ = session.kill().await;
    let _ = session.remove().await;
    let paths: Vec<&str> = changes.iter().map(|change| change.path.as_str()).collect();
    for path in &paths {
        assert!(
            DOCKER_OWNED.contains(path),
            "the daemon reports a filesystem change outside docker's own mounts: {path} (all: {paths:?})"
        );
    }
}

/// S26 — the mount table holds only the mounts the policy creates: an overlay
/// root, the usual virtual filesystems, the two tmpfs scratches, and docker's
/// three /etc bind mounts.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s26_the_mount_table_holds_only_expected_mounts() {
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let python = catalog.get("python").expect("python is in the catalogue");
    let spec = preflight::probe_spec(python, cfg.run_wall_cap, cfg.runtime).expect("probe spec");
    let backend = BollardBackend::new(docker.clone(), cfg.api_timeout);
    let mut session = backend.create(&spec).await.expect("a probe container");
    session.start().await.expect("the container starts");

    let output = session
        .exec(&preflight::probe_exec(&["cat", "/proc/self/mounts"]), &[])
        .await
        .expect("the mounts read runs");
    let _ = session.kill().await;
    let _ = session.remove().await;

    let text = String::from_utf8_lossy(&output.stdout);
    for line in text.lines() {
        let Some(target) = line.split_whitespace().nth(1) else {
            continue;
        };
        assert!(
            ALLOWED_MOUNTS.contains(&target),
            "unexpected mount in the container: {line}"
        );
    }
}

/// S26 — host-visible paths (docker sockets, init's environment, secret files)
/// read as nothing but the container's own mount table.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s26_host_paths_read_nothing_useful() {
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let python = catalog.get("python").expect("python is in the catalogue");
    let spec = preflight::probe_spec(python, cfg.run_wall_cap, cfg.runtime).expect("probe spec");
    let backend = BollardBackend::new(docker.clone(), cfg.api_timeout);
    let mut session = backend.create(&spec).await.expect("a probe container");
    session.start().await.expect("the container starts");

    // read_host may report only the two legitimately visible paths: the mount
    // table itself and PID 1's environment — which holds the image's public
    // build metadata (PATH, HOSTNAME, LANG, the python release's public GPG
    // fingerprint and sha256), never host secrets.
    for (name, allowed) in [
        (
            "read_host.py",
            &["/proc/self/mounts", "/proc/1/environ"][..],
        ),
        ("syscalls_blocked.py", &[][..]),
    ] {
        let script = sbx::hostile(name);
        let output = session
            .exec(&preflight::probe_exec(&["python3", "-"]), script.as_bytes())
            .await
            .unwrap_or_else(|error| panic!("{name} exec ran: {error}"));
        let stdout = String::from_utf8_lossy(&output.stdout);
        let report = stdout
            .lines()
            .find_map(|line| line.strip_prefix("REPORT:"))
            .unwrap_or_else(|| panic!("{name} printed a REPORT: {stdout}"));
        for found in report.split(',').filter(|item| !item.is_empty()) {
            assert!(
                allowed.iter().any(|path| found.starts_with(path)),
                "{name} escaped the sandbox: {found} (report: {report})"
            );
        }
    }
    let _ = session.kill().await;
    let _ = session.remove().await;
}
