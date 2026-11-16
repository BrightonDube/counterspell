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
use counterspell_judge::familiar::store::TimingRow;
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
pub fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// The fixture problems directory (`tests/fixtures/problems`).
pub fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/problems")
}

/// The trusted probe scripts directory (`tests/probes`).
pub fn probes_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/probes")
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

/// Polls `list_managed` until it is empty or `within` passes, then returns the last
/// list. Abnormal-end containers are killed and removed on the call path, but a removal
/// can lag the verdict by a moment — the sweep-free bound is seconds, not the orphan age.
pub async fn cleared_within(
    docker: &Docker,
    cfg: &SandboxConfig,
    within: Duration,
) -> Vec<ManagedContainer> {
    let backend = BollardBackend::new(docker.clone(), cfg.api_timeout);
    let deadline = tokio::time::Instant::now() + within;
    let mut last = managed_containers(&backend).await;
    while !last.is_empty() && tokio::time::Instant::now() < deadline {
        tokio::time::sleep(Duration::from_millis(400)).await;
        last = managed_containers(&backend).await;
    }
    last
}

/// Asserts no managed container is still *running* within `within`, then reaps
/// any corpses — the role the sweeper plays in production (no sweeper runs
/// during scenario tests). Docker's removal of a killed container can lag the
/// verdict by tens of seconds under load, so absence is not the bound; "not
/// running" is. Returns how many corpses were reaped.
pub async fn drain_dead(docker: &Docker, cfg: &SandboxConfig, within: Duration) -> usize {
    let backend = BollardBackend::new(docker.clone(), cfg.api_timeout);
    let deadline = tokio::time::Instant::now() + within;
    let mut leftovers = managed_containers(&backend).await;
    loop {
        let mut survivors = Vec::new();
        for container in &leftovers {
            let inspected = docker
                .inspect_container(&container.id, None)
                .await
                .expect("the daemon inspects a managed container");
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

/// What one real run produced: the executor's outcome and every public event it
/// emitted, in order.
pub struct RunObservation {
    /// The outcome the worker would turn into the terminal write.
    pub outcome: Result<RunOutcome, ExecError>,
    /// The public events the executor emitted, in emission order.
    pub events: Vec<PublicEvent>,
    /// The raw bench samples the run recorded for `run_timings`.
    pub timings: Vec<TimingRow>,
    /// Wall time of `executor.execute` alone — the preflight probe before it is not
    /// part of the run cap.
    pub run_wall: Duration,
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
        problem_version: 1,
        lang: lang.to_owned(),
        mode,
        code: source.to_owned(),
        attempt: 1,
    };
    let started = std::time::Instant::now();
    let outcome = executor.execute(run, ctx).await;
    RunObservation {
        outcome,
        events: log.events(),
        timings: log.timings(),
        run_wall: started.elapsed(),
    }
}

/// One executor shared by several runs — concurrency scenarios need a single
/// container semaphore, so two `run_source` calls (two executors) would not do.
/// Preflight runs once for every enabled language before the executor is built.
pub async fn shared_executor(
    docker: &Docker,
    cfg: &SandboxConfig,
    catalog: &LanguageCatalog,
) -> Arc<SandboxExecutor<DirProblemTests, BollardBackend>> {
    let backend = BollardBackend::new(docker.clone(), cfg.api_timeout);
    let mut states = Vec::new();
    for name in ["python", "javascript"] {
        if let Some(lang_def) = catalog.get(name).filter(|lang| lang.is_enabled()) {
            let state = Arc::new(PreflightState::new());
            let spec =
                preflight::probe_spec(lang_def, cfg.run_wall_cap, cfg.runtime).expect("probe spec");
            preflight::refresh(&state, &backend, &spec)
                .await
                .expect("the preflight probe runs");
            assert!(state.passed(), "preflight must pass for {name}");
            states.push(state);
        }
    }
    Arc::new(SandboxExecutor::new(
        catalog.clone(),
        DirProblemTests::new(fixture_dir()),
        backend,
        cfg.clone(),
        states,
        "sbx-scenario",
    ))
}

/// Runs `source` through an already-built executor — for S18/S19 where the
/// container semaphore must be shared between concurrent runs.
pub async fn execute_on(
    executor: &SandboxExecutor<DirProblemTests, BollardBackend>,
    lang: &str,
    slug: &str,
    source: &str,
    mode: RunMode,
) -> RunObservation {
    let (ctx, log, _stop) = RecordingContext::with_stop(1);
    let run = RunInput {
        run_id: RunId::new(),
        problem_slug: slug.to_owned(),
        problem_version: 1,
        lang: lang.to_owned(),
        mode,
        code: source.to_owned(),
        attempt: 1,
    };
    let started = std::time::Instant::now();
    let outcome = executor.execute(run, ctx).await;
    RunObservation {
        outcome,
        events: log.events(),
        timings: log.timings(),
        run_wall: started.elapsed(),
    }
}

/// Peak container counts the sampler observed.
#[derive(Debug, Clone, Copy)]
pub struct ManagedPeak {
    /// The most managed containers the daemon ever listed in one sample —
    /// dead corpses included, since Docker's removal can lag a kill.
    pub listed: usize,
    /// The most containers still `running` in one sample — the count the
    /// container semaphore is actually bound to.
    pub running: usize,
}

/// Samples the daemon's managed-container list every `every` until `stop`
/// fires, reporting the largest listed and running counts seen. Sampling is
/// out-of-band evidence: it watches the daemon, not anything the runs report.
pub async fn sample_managed_peak(
    docker: Docker,
    cfg: SandboxConfig,
    every: Duration,
    mut stop: tokio::sync::watch::Receiver<bool>,
) -> ManagedPeak {
    let backend = BollardBackend::new(docker.clone(), cfg.api_timeout);
    let mut peak = ManagedPeak {
        listed: 0,
        running: 0,
    };
    loop {
        let containers = managed_containers(&backend).await;
        peak.listed = peak.listed.max(containers.len());
        let mut running = 0_usize;
        for container in &containers {
            // A container removed between the list and the inspect no longer
            // counts — it cannot be running any more.
            let inspected = docker.inspect_container(&container.id, None).await;
            let running_state = inspected
                .ok()
                .and_then(|container| container.state)
                .and_then(|state| state.running)
                .unwrap_or(false);
            if running_state {
                running += 1;
            }
        }
        peak.running = peak.running.max(running);
        tokio::select! {
            () = tokio::time::sleep(every) => {}
            _ = stop.changed() => return peak,
        }
    }
}

/// Starts [`run_source`] on a spawned task and returns its stop sender with the
/// task handle. Firing `stop` mid-call is how S17's cancel path and S24's stop
/// scenario reach a live call — the daemon's container list is the out-of-band
/// proof that the run was mid-call when the signal landed.
pub fn spawn_source(
    docker: &Docker,
    cfg: &SandboxConfig,
    catalog: &LanguageCatalog,
    lang: &str,
    slug: &str,
    source: &str,
    mode: RunMode,
) -> (
    tokio::task::JoinHandle<RunObservation>,
    tokio::sync::watch::Sender<bool>,
    String,
) {
    let backend = BollardBackend::new(docker.clone(), cfg.api_timeout);
    let state = Arc::new(PreflightState::new());
    let probe = catalog
        .get(lang)
        .filter(|lang| lang.is_enabled())
        .map(|lang_def| {
            preflight::probe_spec(lang_def, cfg.run_wall_cap, cfg.runtime).expect("probe spec")
        });
    let executor = SandboxExecutor::new(
        catalog.clone(),
        DirProblemTests::new(fixture_dir()),
        backend.clone(),
        cfg.clone(),
        vec![state.clone()],
        "sbx-scenario",
    );
    let (ctx, log, stop) = RecordingContext::with_stop(1);
    let run_id = RunId::new();
    let run_id_text = run_id.to_string();
    let run = RunInput {
        run_id,
        problem_slug: slug.to_owned(),
        problem_version: 1,
        lang: lang.to_owned(),
        mode,
        code: source.to_owned(),
        attempt: 1,
    };
    let handle = tokio::spawn(async move {
        if let Some(spec) = probe {
            preflight::refresh(&state, &backend, &spec)
                .await
                .expect("the preflight probe runs");
            assert!(
                state.passed(),
                "sandbox preflight must pass before a scenario run"
            );
        }
        let started = std::time::Instant::now();
        let outcome = executor.execute(run, ctx).await;
        RunObservation {
            outcome,
            events: log.events(),
            timings: log.timings(),
            run_wall: started.elapsed(),
        }
    });
    (handle, stop, run_id_text)
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

/// The host's primary LAN IP, found by asking the OS which interface a packet
/// to a public address would leave on. A rootless bridge container reaches the
/// host through this address — the bridge gateway and `host.docker.internal`
/// answer locally but do not forward.
pub fn host_ip() -> std::net::IpAddr {
    let socket = std::net::UdpSocket::bind("0.0.0.0:0").expect("a UDP socket binds");
    socket
        .connect("192.0.2.1:53")
        .expect("connect only picks a route");
    socket
        .local_addr()
        .expect("the socket has a local address")
        .ip()
}

/// Binds a TCP and a UDP socket on every interface and records every connection
/// or datagram until the returned set is dropped. `0.0.0.0` is required: the
/// sandbox reaches this host through its LAN IP, not its loopback.
pub async fn listeners() -> ListenerSet {
    let attempts: Arc<Mutex<Vec<String>>> = Arc::new(Mutex::new(Vec::new()));
    let tcp = TcpListener::bind(SocketAddr::from(([0, 0, 0, 0], 0)))
        .await
        .expect("the TCP listener binds");
    let udp = UdpSocket::bind(SocketAddr::from(([0, 0, 0, 0], 0)))
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
