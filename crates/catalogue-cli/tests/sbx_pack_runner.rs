// SPDX-License-Identifier: MIT
//! The production `PackRunner` against the real rootless daemon (grimoire T20).
//!
//! `counterspell-catalogue import --sandbox`, `validate-sandbox` and
//! `seed-demo` all drive packs through `PackSandboxRunner`. This suite proves
//! the reference returns through the driver protocol, an `incorrect/` program
//! gets its own session, a timed-out call is a `ProgramResult` rather than an
//! infrastructure error, and `bench` returns one timing per sized input.
//! Ignored by default.
//!
//! `cargo test -p counterspell-judge --features sandbox --test sbx_pack_runner -- --ignored`
#![cfg(feature = "sandbox")]

mod sbx;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use bollard::Docker;
use counterspell_catalogue_cli::load::signature_descriptor;
use counterspell_catalogue_cli::pack_runner::{PackSandboxRunner, REFERENCE_SOURCE};
use counterspell_engine::ProgramResult;
use counterspell_grimoire::record;
use counterspell_grimoire::runner::{PackRunner, RunnerError, Target};
use counterspell_judge::sandbox::bollard_backend::BollardBackend;
use counterspell_judge::sandbox::config::SandboxConfig;
use serde_json::{json, Value};
use tokio::runtime::Runtime;

/// Per-call deadline handed to the runner, matching the fixture's `time_ms`.
const LIMIT: Duration = Duration::from_secs(4);

/// The synthetic canary pack's public tree — `add_one(value) -> int`, no
/// hidden data. Its `reference/` lives in the fixture's private tree.
fn pack_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/c19b/public/add-one")
}

/// The canary pack's private tree.
fn pack_private_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/c19b/private/packs/add-one")
}

/// The pack's sources as the catalogue binary assembles them: the reference
/// under `REFERENCE_SOURCE`, each `incorrect/` file under its own path.
fn sources(_pack: &Path) -> BTreeMap<String, String> {
    let mut sources = BTreeMap::new();
    sources.insert(
        REFERENCE_SOURCE.to_string(),
        std::fs::read_to_string(pack_private_dir().join("reference/solution.py"))
            .expect("the fixture reference reads"),
    );
    sources.insert(
        "incorrect/always-zero.py".to_string(),
        "def add_one(value):\n    return 0\n".to_string(),
    );
    sources.insert(
        "incorrect/hangs.py".to_string(),
        "def add_one(value):\n    while True:\n        pass\n".to_string(),
    );
    sources
}

/// The driver-wire signature `signature_descriptor` produces for the pack.
fn signature(public_dir: &Path, private_dir: &Path) -> Value {
    let record = record::build_pair(public_dir, private_dir).expect("the fixture pack records");
    signature_descriptor(&record).expect("the wire signature serialises")
}

/// Reads a real pack's runner sources from its private tree:
/// `reference/solution.py` under [`REFERENCE_SOURCE`] and every `incorrect/`
/// file under its path.
fn real_sources(private_dir: &Path) -> BTreeMap<String, String> {
    let mut sources = BTreeMap::new();
    sources.insert(
        REFERENCE_SOURCE.to_string(),
        std::fs::read_to_string(private_dir.join("reference/solution.py"))
            .expect("the pack reference reads"),
    );
    for entry in std::fs::read_dir(private_dir.join("incorrect")).expect("incorrect/ reads") {
        let entry = entry.expect("an incorrect/ entry");
        let name = entry.file_name();
        let Some(name) = name.to_str() else { continue };
        if !entry.file_type().is_ok_and(|kind| kind.is_file()) {
            continue;
        }
        sources.insert(
            format!("incorrect/{name}"),
            std::fs::read_to_string(entry.path()).expect("an incorrect program reads"),
        );
    }
    sources
}

/// A daemon client, test config and a runtime to hold the runner's handle.
struct Daemon {
    runtime: Runtime,
    docker: Docker,
    cfg: SandboxConfig,
}

impl Daemon {
    fn connect() -> Self {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .expect("the runner runtime builds");
        let (docker, cfg) = runtime.block_on(sbx::sandbox());
        Self {
            runtime,
            docker,
            cfg,
        }
    }

    fn runner(
        &self,
        sources: BTreeMap<String, String>,
        signature: &Value,
    ) -> PackSandboxRunner<BollardBackend> {
        let catalogue = sbx::catalogue(&self.cfg);
        let lang = catalogue
            .get("python")
            .filter(|lang| lang.is_enabled())
            .expect("python is enabled in the shipped catalogue")
            .clone();
        PackSandboxRunner::new(
            self.runtime.handle().clone(),
            BollardBackend::new(self.docker.clone(), self.cfg.api_timeout),
            lang,
            signature,
            sources,
            &self.cfg,
        )
        .expect("the pack runner builds")
    }

    /// Drops with the runtime alive so session teardown can schedule
    /// kill-and-remove work, then waits for every managed container to go.
    fn drain(self) {
        let Self {
            runtime,
            docker,
            cfg,
        } = self;
        runtime.block_on(sbx::drain_dead(&docker, &cfg, Duration::from_secs(30)));
        runtime.shutdown_background();
    }
}

#[test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
fn pack_runner_reference_returns() {
    let _guard = sbx::ONE_RUN_AT_A_TIME.blocking_lock();
    let daemon = Daemon::connect();
    let mut runner = daemon.runner(
        sources(&pack_dir()),
        &signature(&pack_dir(), &pack_private_dir()),
    );
    let result = runner
        .run(Target::Reference, &json!([41]), LIMIT)
        .expect("the reference call runs");
    assert!(
        matches!(&result, ProgramResult::Returned(value) if *value == json!(42)),
        "expected Returned(42), got {result:?}"
    );
    drop(runner);
    daemon.drain();
}

#[test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
fn pack_runner_program_gets_own_session() {
    let _guard = sbx::ONE_RUN_AT_A_TIME.blocking_lock();
    let daemon = Daemon::connect();
    let mut runner = daemon.runner(
        sources(&pack_dir()),
        &signature(&pack_dir(), &pack_private_dir()),
    );
    let result = runner
        .run(
            Target::Program("incorrect/always-zero.py"),
            &json!([41]),
            LIMIT,
        )
        .expect("the incorrect program call runs");
    assert!(
        matches!(&result, ProgramResult::Returned(value) if *value == json!(0)),
        "expected Returned(0), got {result:?}"
    );
    drop(runner);
    daemon.drain();
}

#[test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
fn pack_runner_timeout_is_a_program_result() {
    let _guard = sbx::ONE_RUN_AT_A_TIME.blocking_lock();
    let daemon = Daemon::connect();
    let mut runner = daemon.runner(
        sources(&pack_dir()),
        &signature(&pack_dir(), &pack_private_dir()),
    );
    let result = runner
        .run(Target::Program("incorrect/hangs.py"), &json!([1]), LIMIT)
        .expect("a hanging program is a result, not infrastructure");
    assert!(
        matches!(result, ProgramResult::TimedOut),
        "expected TimedOut, got {result:?}"
    );
    drop(runner);
    daemon.drain();
}

#[test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
fn pack_runner_bench_returns_sized_pairs() {
    let _guard = sbx::ONE_RUN_AT_A_TIME.blocking_lock();
    let daemon = Daemon::connect();
    let mut runner = daemon.runner(
        sources(&pack_dir()),
        &signature(&pack_dir(), &pack_private_dir()),
    );
    let timings = runner
        .bench(&[(1, json!([5])), (2, json!([6])), (3, json!([7]))], LIMIT)
        .expect("the bench runs");
    assert_eq!(timings.len(), 3, "one timing per sized input");
    for (size, nanos) in &timings {
        assert!(nanos > &0, "size {size} produced a zero timing");
    }
    assert_eq!(
        timings.iter().map(|(size, _)| *size).collect::<Vec<_>>(),
        vec![1, 2, 3],
        "the sizes pass through in order"
    );
    drop(runner);
    daemon.drain();
}

/// T20's verify: the same checks `sandbox_fake.rs` proves with a model runner
/// must pass against the real Python files through the real sandbox — every
/// starter pack clean under `grimoire::validate_sandbox`.
#[test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
fn starter_packs_pass_sandbox_checks_with_the_real_runner() {
    let _guard = sbx::ONE_RUN_AT_A_TIME.blocking_lock();
    let Some(private_root) = sbx::private_packs_root() else {
        eprintln!("skipping: no counterspell-packs-private checkout is available");
        return;
    };
    let daemon = Daemon::connect();
    for slug in ["count-pairs", "diagonal-total", "longest-run-length"] {
        let public_dir = sbx::repo_root().join("packs").join(slug);
        let private_dir = private_root.join("packs").join(slug);
        let mut runner = daemon.runner(
            real_sources(&private_dir),
            &signature(&public_dir, &private_dir),
        );
        let report =
            counterspell_grimoire::validate_sandbox_pair(&public_dir, &private_dir, &mut runner)
                .unwrap_or_else(|fault| panic!("{slug}: sandbox fault: {fault:?}"));
        assert!(
            !report.has_errors(),
            "{slug}: sandbox checks reported errors: {report:?}"
        );
    }
    daemon.drain();
}

#[test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
fn pack_runner_unknown_program_is_infrastructure() {
    let _guard = sbx::ONE_RUN_AT_A_TIME.blocking_lock();
    let daemon = Daemon::connect();
    let mut runner = daemon.runner(
        sources(&pack_dir()),
        &signature(&pack_dir(), &pack_private_dir()),
    );
    let Err(RunnerError::Infrastructure(message)) =
        runner.run(Target::Program("incorrect/missing.py"), &json!([1]), LIMIT)
    else {
        panic!("an unknown program is an infrastructure fault");
    };
    assert!(
        message.contains("incorrect/missing.py"),
        "the fault names the missing program: {message}"
    );
    drop(runner);
    daemon.drain();
}
