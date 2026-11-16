// SPDX-License-Identifier: MIT
//! Shared harness for the real-daemon scenario tests (judge-sandbox T25, R16.2, R16.3).
//!
//! Every helper here runs against the rootless sandbox daemon that
//! `SANDBOX_DOCKER_SOCKET` names — after `daemon::verify` has proven the identity
//! checks, never against the main Docker socket. Scenario tests are `#[ignore]`d by
//! default because they need that daemon.
//!
//! Corpus files under `tests/hostile/` and `tests/fixtures/` are data: they are read as
//! text and delivered into a container as submitted source. Nothing on the host runs
//! them.

#![cfg(feature = "sandbox")]
#![allow(dead_code)]

use std::net::SocketAddr;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use bollard::Docker;
use counterspell_common::events::PublicEvent;
use counterspell_common::ids::RunId;
use counterspell_common::status::RunMode;
use counterspell_judge::executor::{ExecError, RunExecutor, RunInput, RunOutcome};
use counterspell_judge::languages::LanguageCatalog;
use counterspell_judge::sandbox::backend::{Backend, ManagedContainer};
use counterspell_judge::sandbox::bollard_backend::BollardBackend;
use counterspell_judge::sandbox::config::{OciRuntime, SandboxConfig};
use counterspell_judge::sandbox::daemon::{connect, verify};
use counterspell_judge::sandbox::preflight::{self, PreflightState};
use counterspell_judge::sandbox::sandbox_executor::SandboxExecutor;
use counterspell_judge::sandbox::testkit::DirProblemTests;
use counterspell_judge::testkit::RecordingContext;
use tokio::net::{TcpListener, UdpSocket};

/// The longest a single daemon call may take in scenario tests.
pub const API_TIMEOUT: Duration = Duration::from_secs(30);
/// How long a scenario's runs may take before the harness itself gives up.
pub const RUN_CAP: Duration = Duration::from_secs(20);

/// The repository root, from this crate's manifest directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The fixture problems directory (`tests/fixtures/problems`).
pub fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/problems")
}

/// Reads one fixture solution file (`tests/fixtures/solutions/<name>`) as text.
pub fn solution(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures/solutions")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("reading {}: {error}", path.display()))
}

/// Reads one hostile corpus file (`tests/hostile/<name>`) as text. It is only ever data
/// for the sandbox.
pub fn hostile(name: &str) -> String {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/hostile")
        .join(name);
    std::fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("reading {}: {error}", path.display()))
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
pub async fn managed_containers(backend: &BollardBackend) -> Vec<ManagedContainer> {
    backend
        .list_managed()
        .await
        .expect("the daemon lists containers")
}

/// What one real run produced: the executor's outcome and every public event it
/// emitted, in order.
pub struct RunObservation {
    /// The outcome the worker would turn into the terminal write.
    pub outcome: Result<RunOutcome, ExecError>,
    /// The public events the executor emitted, in emission order.
    pub events: Vec<PublicEvent>,
}

impl RunObservation {
    /// The event names in order, for shape assertions like `["compiling", "test", …]`.
    pub fn event_names(&self) -> Vec<&'static str> {
        self.events.iter().map(PublicEvent::event_type).collect()
    }
}

/// Runs `source` against the fixture problem `slug` in `mode`, on the real daemon.
///
/// The helper probes the language first: the executor's preflight gate sees the real
/// report, not a stub. When the language is disabled the probe is skipped — the
/// language check fires before the preflight gate anyway.
pub async fn run_source(
    docker: &Docker,
    cfg: &SandboxConfig,
    catalog: &LanguageCatalog,
    lang: &str,
    slug: &str,
    source: &str,
    mode: RunMode,
) -> RunObservation {
    let backend = BollardBackend::new(docker.clone(), cfg.api_timeout);
    let state = Arc::new(PreflightState::new());
    if let Some(lang_def) = catalog.get(lang).filter(|lang| lang.is_enabled()) {
        let spec =
            preflight::probe_spec(lang_def, cfg.run_wall_cap, cfg.runtime).expect("probe spec");
        preflight::refresh(&state, &backend, &spec)
            .await
            .expect("the preflight probe runs");
        assert!(
            state.passed(),
            "sandbox preflight must pass before a scenario run"
        );
    }
    let executor = SandboxExecutor::new(
        catalog.clone(),
        DirProblemTests::new(fixture_dir()),
        backend,
        cfg.clone(),
        vec![state],
        "sbx-scenario",
    );
    let (ctx, log, _stop) = RecordingContext::with_stop(1);
    let run = RunInput {
        run_id: RunId::new(),
        problem_slug: slug.to_owned(),
        lang: lang.to_owned(),
        mode,
        code: source.to_owned(),
        attempt: 1,
    };
    let outcome = executor.execute(run, ctx).await;
    RunObservation {
        outcome,
        events: log.events(),
    }
}

/// A host-side listener set for the network scenario (S12): one TCP and one UDP socket
/// on the loopback, recording every peer that reaches them.
pub struct ListenerSet {
    /// The TCP port the hostile program may try.
    pub tcp_port: u16,
    /// The UDP port it may try.
    pub udp_port: u16,
    attempts: Arc<Mutex<Vec<String>>>,
}

impl ListenerSet {
    /// The peers that reached the listeners so far, as `"<proto> <addr>"` strings.
    pub fn attempts(&self) -> Vec<String> {
        self.attempts.lock().expect("attempts lock").clone()
    }
}

/// Binds a TCP and a UDP socket on the loopback and records every connection or
/// datagram until the returned set is dropped.
pub async fn listeners() -> ListenerSet {
    let attempts: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let tcp = TcpListener::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .expect("the TCP listener binds");
    let udp = UdpSocket::bind(SocketAddr::from(([127, 0, 0, 1], 0)))
        .await
        .expect("the UDP socket binds");
    let tcp_port = tcp.local_addr().expect("TCP address").port();
    let udp_port = udp.local_addr().expect("UDP address").port();
    let tcp_attempts = attempts.clone();
    tokio::spawn(async move {
        while let Ok((stream, peer)) = tcp.accept().await {
            tcp_attempts
                .lock()
                .expect("attempts lock")
                .push(format!("tcp {peer}"));
            drop(stream);
        }
    });
    let udp_attempts = attempts.clone();
    tokio::spawn(async move {
        let mut buffer = vec![0_u8; 2048];
        while let Ok((_len, peer)) = udp.recv_from(&mut buffer).await {
            udp_attempts
                .lock()
                .expect("attempts lock")
                .push(format!("udp {peer}"));
        }
    });
    ListenerSet {
        tcp_port,
        udp_port,
        attempts,
    }
}
