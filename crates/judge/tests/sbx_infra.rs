// SPDX-License-Identifier: MIT
//! Infrastructure and runtime scenarios S21, S22, S23, S25 (judge-sandbox T33,
//! R4, R9.4, R12.4, R12.5). S20 itself is proven by `sbx_preflight.rs`: the
//! probe passes against the production policy and fails against a deliberately
//! weakened one — this file covers what happens when the *environment* is
//! broken, not the container.
//!
//! Ignored by default:
//! `cargo test -p counterspell-judge --features sandbox --test sbx_infra -- --ignored`
#![cfg(feature = "sandbox")]

mod sbx;

use std::sync::Arc;
use std::time::{Duration, Instant};

use counterspell_common::status::RunMode;
use counterspell_judge::executor::{ExecError, RunExecutor};
use counterspell_judge::languages::PinnedImage;
use counterspell_judge::sandbox::bollard_backend::BollardBackend;
use counterspell_judge::sandbox::config::{OciRuntime, SandboxConfig};
use counterspell_judge::sandbox::daemon::connect;
use counterspell_judge::sandbox::preflight::{self, PreflightState};
use counterspell_judge::sandbox::sandbox_executor::SandboxExecutor;
use counterspell_judge::sandbox::spec::SessionSpec;
use counterspell_judge::sandbox::testkit::DirProblemTests;
use counterspell_judge::testkit::RecordingContext;

/// The path the probe report is written to for the verification report (S25).
const PROBE_REPORT: &str = ".kiro/specs/judge-sandbox/probe-report.json";

/// Runs `source` on `add` through an executor wired to `backend` — for cases
/// where the backend itself is the thing under test. A failing preflight is
/// honest: the gate then answers `Infrastructure` for every run.
async fn execute_with_backend(
    cfg: &SandboxConfig,
    catalog: &counterspell_judge::languages::LanguageCatalog,
    backend: BollardBackend,
    preflights: Vec<Arc<PreflightState>>,
    source: &str,
) -> sbx::RunObservation {
    let executor = SandboxExecutor::new(
        catalog.clone(),
        DirProblemTests::new(sbx::fixture_dir()),
        backend,
        cfg.clone(),
        preflights,
        "sbx-scenario",
    );
    let (ctx, log, _stop) = RecordingContext::with_stop(1);
    let run = counterspell_judge::executor::RunInput {
        run_id: counterspell_common::ids::RunId::new(),
        problem_slug: "add".to_owned(),
        lang: "python".to_owned(),
        mode: RunMode::Submit,
        code: source.to_owned(),
        attempt: 1,
    };
    let started = Instant::now();
    let outcome = executor.execute(run, ctx).await;
    sbx::RunObservation {
        outcome,
        events: log.events(),
        run_wall: started.elapsed(),
    }
}

/// S21 — a wrong socket and an unresponsive daemon both end in
/// `Infrastructure`, and neither can ever produce a verdict.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s21_wrong_socket_and_dead_daemon_give_infrastructure() {
    let (_real_docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let source = sbx::solution("add_correct.py");

    // (a) A socket path where nothing listens: `connect` refuses to build a
    // client at all — the run cannot even reach a backend, which is the
    // infrastructure shape the worker reports.
    let missing = cfg
        .docker_socket
        .with_file_name("counterspell-no-such-daemon.sock");
    let attempted = connect(&missing, Duration::from_secs(5));
    assert!(
        attempted.is_err(),
        "a missing socket fails at connect: {attempted:?}"
    );

    // (b) A listener that accepts and never answers — the stopped-daemon
    // shape. Every call must end inside the API timeout, not hang.
    let language = catalog.get("python").expect("python is known");
    let spec = preflight::probe_spec(language, cfg.run_wall_cap, cfg.runtime).expect("probe spec");
    let dir = tempfile::tempdir().expect("a socket dir");
    let path = dir.path().join("deaf.sock");
    let listener = tokio::net::UnixListener::bind(&path).expect("the stub socket binds");
    tokio::spawn(async move {
        while let Ok((_stream, _addr)) = listener.accept().await {
            // Accept and drop: the daemon "answers" with nothing.
        }
    });
    let docker = connect(&path, Duration::from_secs(5)).expect("the client builds");
    let backend = BollardBackend::new(docker, Duration::from_secs(5));
    let state = Arc::new(PreflightState::new());
    let started = Instant::now();
    let probe = preflight::refresh(&state, &backend, &spec).await;
    assert!(
        probe.is_err() && !state.passed(),
        "a daemon that never answers fails the probe closed"
    );
    assert!(
        started.elapsed() < Duration::from_secs(30),
        "the dead daemon was bounded by the API timeout, not a hang"
    );
    let observation = execute_with_backend(&cfg, &catalog, backend, vec![state], &source).await;
    assert!(
        matches!(observation.outcome, Err(ExecError::Infrastructure(_))),
        "a run against a dead daemon is infrastructure, never a verdict: {:?}",
        observation.outcome
    );
}

/// S22 — an image whose pinned digest is not on the daemon ends in
/// `InfraError` at create time, and no image is pulled to satisfy it.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s22_unknown_digest_is_infrastructure_never_a_pull() {
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let backend = BollardBackend::new(docker.clone(), cfg.api_timeout);
    let language = catalog.get("python").expect("python is known");
    let run_id = counterspell_common::ids::RunId::new();
    let mut spec = SessionSpec::new(
        language,
        cfg.run_wall_cap,
        cfg.runtime,
        run_id.as_uuid(),
        "sbx-scenario",
        1,
        // Expiry far in the future is fine: the create must fail before start.
        4_102_444_800,
    )
    .expect("the spec builds");
    // A syntactically valid pinned digest that is not on the daemon.
    spec.policy.image = PinnedImage::parse(
        "python:3.13-slim@sha256:0000000000000000000000000000000000000000000000000000000000000000",
    )
    .expect("the bogus digest parses");

    let images_before: Vec<_> = docker
        .list_images(None::<bollard::query_parameters::ListImagesOptions>)
        .await
        .expect("the daemon lists images")
        .into_iter()
        .map(|image| image.id)
        .collect();
    let started = Instant::now();
    let created = counterspell_judge::sandbox::backend::Backend::create(&backend, &spec).await;
    let images_after: Vec<_> = docker
        .list_images(None::<bollard::query_parameters::ListImagesOptions>)
        .await
        .expect("the daemon lists images")
        .into_iter()
        .map(|image| image.id)
        .collect();
    assert!(
        created.is_err(),
        "an unknown pinned digest must not create a container: {}",
        created
            .err()
            .map_or_else(|| "created".to_owned(), |error| error.to_string())
    );
    assert_eq!(
        images_before, images_after,
        "the daemon's image list is unchanged — nothing was pulled"
    );
    assert!(
        started.elapsed() < cfg.api_timeout,
        "the refusal was immediate, not a pull attempt"
    );
}

/// S23 — `SANDBOX_RUNTIME=runsc`: when the runtime is configured, the probe
/// runs under it and the result is recorded as TESTED with its outcome;
/// otherwise the scenario records NOT TESTED. It never assumes.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s23_gvisor_status_is_recorded_never_assumed() {
    let (docker, cfg) = sbx::sandbox().await;
    let configured = std::env::var("SANDBOX_RUNTIME").ok();
    if configured.as_deref() != Some("runsc") {
        println!("S23: NOT TESTED — SANDBOX_RUNTIME is not runsc");
        return;
    }
    let catalog = sbx::catalogue(&cfg);
    let backend = BollardBackend::new(docker, cfg.api_timeout);
    let language = catalog.get("python").expect("python is known");
    let spec = preflight::probe_spec(language, cfg.run_wall_cap, OciRuntime::Runsc)
        .expect("the runsc probe spec builds");
    let state = Arc::new(PreflightState::new());
    match preflight::refresh(&state, &backend, &spec).await {
        Ok(()) => println!("S23: TESTED — the probe passed under runsc"),
        Err(error) => println!("S23: TESTED — the probe failed under runsc: {error}"),
    }
}

/// S25 — the real probe's report is written as JSON for the verification
/// report, listing every check with its expected and observed value.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s25_the_probe_report_is_written_for_the_verification_report() {
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let backend = BollardBackend::new(docker, cfg.api_timeout);
    let language = catalog.get("python").expect("python is known");
    let spec = preflight::probe_spec(language, cfg.run_wall_cap, cfg.runtime).expect("probe spec");
    let state = Arc::new(PreflightState::new());
    preflight::refresh(&state, &backend, &spec)
        .await
        .expect("the probe runs");
    let report = state.latest().expect("the probe leaves a report");
    assert!(report.passed(), "the production policy passes its probe");
    let json = serde_json::to_string_pretty(&report).expect("the report serialises");
    assert!(
        json.contains("\"expected\"") && json.contains("\"observed\""),
        "every check records expected and observed"
    );
    let path = sbx::repo_root().join(PROBE_REPORT);
    std::fs::write(&path, format!("{json}\n")).expect("the report file is written");
    let written = std::fs::read_to_string(&path).expect("the report file reads back");
    assert_eq!(written.trim_end(), json, "the file holds the probe report");
    println!("S25: probe report written to {}", path.display());
}
