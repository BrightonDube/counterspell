//! Real-daemon tests for the bollard session (judge-sandbox T21, R5.2, R7.1, R7.2, R9.3,
//! R9.4, R10.1).
//!
//! Ignored by default. They need `SANDBOX_DOCKER_SOCKET` pointing at the rootless sandbox
//! daemon, and the pinned Python image present there (the judge never pulls). Every command
//! here is trusted and fixed: no hostile program runs in these tests.
//!
//! Run on the sandbox daemon only:
//! `cargo test -p counterspell-judge --features sandbox --test sbx_bollard_session -- --ignored`
#![cfg(feature = "sandbox")]

use std::path::Path;
use std::time::Duration;

use counterspell_judge::languages::FileName;
use counterspell_judge::sandbox::backend::{Backend, BackendSession};
use counterspell_judge::sandbox::bollard_backend::BollardBackend;
use counterspell_judge::sandbox::config::OciRuntime;
use counterspell_judge::sandbox::daemon::{connect, verify};
use counterspell_judge::sandbox::spec::{
    build_tar, driver_file_name, ExecSpec, SandboxFiles, SessionSpec,
};
use uuid::Uuid;

mod support;
use support::shipped_catalogue;

const API_TIMEOUT: Duration = Duration::from_secs(30);

/// Connects to the sandbox daemon named by the environment and runs the identity check.
async fn sandbox_backend() -> (BollardBackend, bollard::Docker) {
    let socket = std::env::var("SANDBOX_DOCKER_SOCKET")
        .expect("SANDBOX_DOCKER_SOCKET must name the rootless sandbox daemon socket");
    let docker = connect(Path::new(&socket), API_TIMEOUT).expect("the socket path is accepted");
    verify(&docker, OciRuntime::Runc, API_TIMEOUT)
        .await
        .expect("the daemon passes the rootless identity check");
    (BollardBackend::new(docker.clone(), API_TIMEOUT), docker)
}

/// A Python session spec from the fixture catalogue (same pinned image, stand-in driver).
fn python_session_spec() -> SessionSpec {
    let (_dir, catalog) = shipped_catalogue(false);
    let python = catalog.get("python").expect("python is in the fixture");
    SessionSpec::new(
        python,
        Duration::from_secs(20),
        OciRuntime::Runc,
        Uuid::new_v4(),
        "sbx-test",
        1,
        i64::MAX / 2,
    )
    .expect("the worker id is a safe label")
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_exec_roundtrip() {
    let (backend, _docker) = sandbox_backend().await;
    let mut session = backend
        .create(&python_session_spec())
        .await
        .expect("create");
    session.start().await.expect("start");

    let spec = ExecSpec {
        argv: vec![
            "python3".to_owned(),
            "-c".to_owned(),
            "import sys; sys.stdout.write(sys.stdin.read().upper())".to_owned(),
        ],
        ..ExecSpec::extract()
    };
    let output = session.exec(&spec, b"abc").await.expect("exec");
    assert_eq!(output.exit_code, 0);
    assert_eq!(output.stdout, b"ABC");
    assert!(!output.overflowed);

    session.kill().await.expect("kill");
    session.remove().await.expect("remove");
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_tar_delivery() {
    let (backend, _docker) = sandbox_backend().await;
    let (_dir, catalog) = shipped_catalogue(false);
    let python = catalog.get("python").expect("python is in the fixture");
    let mut session = backend
        .create(&python_session_spec())
        .await
        .expect("create");
    session.start().await.expect("start");

    let files = SandboxFiles::new(
        FileName::parse("solution.py").expect("valid name"),
        driver_file_name(python).expect("driver name is valid"),
        b"print('hi')\n".to_vec(),
        b"print('driver')\n".to_vec(),
        b"{}".to_vec(),
    )
    .expect("three distinct names");
    let archive = build_tar(&files).expect("archive builds");

    let extract = session
        .exec(&ExecSpec::extract(), &archive)
        .await
        .expect("tar extract");
    assert_eq!(extract.exit_code, 0, "tar -x accepts the archive");

    let read = ExecSpec {
        argv: vec!["cat".to_owned(), "/box/solution.py".to_owned()],
        ..ExecSpec::extract()
    };
    let output = session.exec(&read, b"").await.expect("cat");
    assert_eq!(output.stdout, b"print('hi')\n");

    session.kill().await.expect("kill");
    session.remove().await.expect("remove");
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_kill_remove_confirms_absence() {
    let (backend, docker) = sandbox_backend().await;
    let mut session = backend
        .create(&python_session_spec())
        .await
        .expect("create");
    let name = session.name().to_owned();
    session.start().await.expect("start");

    session.kill().await.expect("kill");
    session.remove().await.expect("remove confirms absence");

    let gone = docker
        .inspect_container(
            &name,
            None::<bollard::query_parameters::InspectContainerOptions>,
        )
        .await;
    assert!(gone.is_err(), "the daemon no longer knows the container");
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_oom_counter_visible() {
    let (backend, _docker) = sandbox_backend().await;
    let mut session = backend
        .create(&python_session_spec())
        .await
        .expect("create");
    session.start().await.expect("start");

    let read = ExecSpec {
        argv: vec!["cat".to_owned(), "/sys/fs/cgroup/memory.events".to_owned()],
        ..ExecSpec::extract()
    };
    let output = session.exec(&read, b"").await.expect("read memory.events");
    assert_eq!(output.exit_code, 0);
    let text = String::from_utf8_lossy(&output.stdout);
    assert!(
        text.lines().any(|line| line.starts_with("oom_kill ")),
        "the counter is visible: {text}"
    );

    session.kill().await.expect("kill");
    session.remove().await.expect("remove");
}
