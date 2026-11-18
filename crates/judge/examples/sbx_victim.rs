// SPDX-License-Identifier: MIT
//! Trusted helper binary for the S16 recovery scenario.
//!
//! It runs `tests/hostile/sleep_forever.py` through the real executor on the
//! sandbox daemon, prints its run id, then waits inside `execute` until the
//! run ends or the process is `SIGKILL`ed by the scenario test. The test then
//! proves that nothing labelled `counterspell.managed` remains sixty seconds
//! after the container was created — once via self-expiry plus the sweeper,
//! and once with a PID 1 sleep that outlives the orphan age so only the
//! sweeper can remove it.
//!
//! Environment:
//! - `SANDBOX_DOCKER_SOCKET` (required): the rootless sandbox daemon socket.
//! - `SBX_VICTIM_RUN_CAP_SECS` (optional): overrides the run cap. The
//!   container's PID 1 `sleep` is the cap plus five seconds, so a large value
//!   keeps the container alive past the orphan age.
//! - `LANGUAGES_FILE` (optional): the catalogue path.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use counterspell_common::ids::RunId;
use counterspell_common::status::RunMode;
use counterspell_judge::executor::{RunExecutor, RunInput};
use counterspell_judge::languages::LanguageCatalog;
use counterspell_judge::sandbox::bollard_backend::BollardBackend;
use counterspell_judge::sandbox::config::{OciRuntime, SandboxConfig};
use counterspell_judge::sandbox::daemon::{connect, verify};
use counterspell_judge::sandbox::preflight::{self, PreflightState};
use counterspell_judge::sandbox::sandbox_executor::SandboxExecutor;
use counterspell_judge::sandbox::testkit::DirProblemTests;
use counterspell_judge::testkit::RecordingContext;

#[tokio::main]
async fn main() {
    let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let socket = std::env::var("SANDBOX_DOCKER_SOCKET")
        .expect("SANDBOX_DOCKER_SOCKET must name the rootless sandbox daemon socket");
    let run_cap = std::env::var("SBX_VICTIM_RUN_CAP_SECS")
        .ok()
        .and_then(|text| text.parse::<u64>().ok())
        .map_or_else(|| Duration::from_secs(20), Duration::from_secs);
    let cfg = SandboxConfig {
        docker_socket: socket.into(),
        runtime: OciRuntime::Runc,
        max_containers: 4,
        reaper_interval: Duration::from_secs(10),
        orphan_age: Duration::from_secs(40),
        kill_slack: Duration::from_millis(500),
        run_wall_cap: run_cap,
        max_consecutive_abnormal: 3,
        cleanup_budget: Duration::from_secs(5),
        api_timeout: Duration::from_secs(30),
        languages_file: std::env::var("LANGUAGES_FILE").map_or_else(
            |_| manifest.join("../../languages/languages.toml"),
            PathBuf::from,
        ),
    };
    let docker = connect(&cfg.docker_socket, cfg.api_timeout).expect("the socket path is accepted");
    verify(&docker, cfg.runtime, cfg.api_timeout)
        .await
        .expect("the daemon passes the rootless identity check");
    let backend = BollardBackend::new(docker, cfg.api_timeout);
    let catalog = LanguageCatalog::load(&cfg.languages_file).expect("the shipped catalogue loads");
    let state = Arc::new(PreflightState::new());
    let language = catalog
        .get("python")
        .filter(|lang| lang.is_enabled())
        .expect("python is enabled");
    let probe =
        preflight::probe_spec(language, cfg.run_wall_cap, cfg.runtime).expect("the probe spec");
    preflight::refresh(&state, &backend, &probe)
        .await
        .expect("the preflight probe runs");
    assert!(state.passed(), "sandbox preflight must pass");
    let executor = SandboxExecutor::new(
        catalog,
        DirProblemTests::new(manifest.join("tests/fixtures/problems")),
        backend,
        cfg,
        vec![state],
        "sbx-victim",
    );
    // The sender must stay alive for the run or `stopped()` would fire at once.
    let (ctx, _log, _stop) = RecordingContext::with_stop(1);
    let run = RunInput {
        run_id: RunId::new(),
        problem_slug: "add".to_owned(),
        problem_version: 1,
        lang: "python".to_owned(),
        mode: RunMode::Submit,
        ghost_input: None,
        code: std::fs::read_to_string(manifest.join("tests/hostile/sleep_forever.py"))
            .expect("the hostile program is readable data"),
        attempt: 1,
    };
    println!("VICTIM-RUN:{}", run.run_id);
    let outcome = executor.execute(run, ctx).await;
    println!("VICTIM-DONE:{outcome:?}");
}
