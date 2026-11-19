// SPDX-License-Identifier: MIT
//! Model tests for `PackSandboxRunner::bench` (efficiency-familiar T25, R13.1 and R13.2).
//!
//! The fake backend stands in for the daemon, so these tests see every call the runner
//! wrote. Pack validation must time the reference the way a learner's run is timed: two
//! warm-ups and five timed calls per size through the same bench collector, not one plain
//! call per size.
#![cfg(feature = "sandbox")]

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::Duration;

use counterspell_catalogue_cli::pack_runner::{PackSandboxRunner, REFERENCE_SOURCE};
use counterspell_engine::ProgramResult;
use counterspell_grimoire::runner::{PackRunner, RunnerError, Target};
use counterspell_judge::languages::LanguageCatalog;
use counterspell_judge::sandbox::bench::{TIMED_REPS, WARM_UPS};
use counterspell_judge::sandbox::config::{OciRuntime, SandboxConfig};
use counterspell_judge::sandbox::testkit::{Behaviour, FakeBackend};
use serde_json::json;
use tokio::runtime::Runtime;

const LIMIT: Duration = Duration::from_secs(4);

fn languages_file() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../languages/languages.toml")
}

fn config() -> SandboxConfig {
    SandboxConfig {
        docker_socket: PathBuf::from("/run/user/0/sandbox.sock"),
        runtime: OciRuntime::Runc,
        max_containers: 4,
        reaper_interval: Duration::from_secs(10),
        orphan_age: Duration::from_secs(40),
        kill_slack: Duration::from_millis(500),
        run_wall_cap: Duration::from_secs(20),
        max_consecutive_abnormal: 3,
        cleanup_budget: Duration::from_secs(5),
        api_timeout: Duration::from_secs(30),
        languages_file: languages_file(),
    }
}

fn answer(id: u64, out: i64) -> Behaviour {
    Behaviour::Answer(format!("{{\"t\":{id},\"out\":{out}}}\n").into_bytes())
}

fn runner(runtime: &Runtime, backend: &FakeBackend) -> PackSandboxRunner<FakeBackend> {
    let catalog = LanguageCatalog::load(&languages_file()).expect("the catalogue loads");
    let python = catalog.get("python").expect("python is enabled").clone();
    let mut sources = BTreeMap::new();
    sources.insert(
        REFERENCE_SOURCE.to_owned(),
        "def add_one(value):\n    return value + 1\n".to_owned(),
    );
    let signature = json!({
        "function": "add_one",
        "params": [{ "name": "value", "type": { "kind": "int" } }],
        "returns": { "kind": "int" }
    });
    PackSandboxRunner::new(
        runtime.handle().clone(),
        backend.clone(),
        python,
        &signature,
        sources,
        &config(),
    )
    .expect("the runner builds")
}

#[test]
fn the_bench_times_each_size_with_warm_ups_and_repeats_then_runs_still_work() {
    let runtime = Runtime::new().expect("a runtime");
    let backend = FakeBackend::new();
    let per_size = WARM_UPS + TIMED_REPS;
    // Bench calls are numbered 0.. by the collector; the fresh session after the bench
    // numbers its ordinary calls from 1 again.
    let mut script: Vec<Behaviour> = (0..(2 * per_size) as u64).map(|id| answer(id, 6)).collect();
    script.push(answer(1, 6));
    backend.script_program(script);
    let mut runner = runner(&runtime, &backend);

    let timings = runner
        .bench(&[(4, json!([5])), (16, json!([7]))], LIMIT)
        .expect("the bench runs");

    assert_eq!(
        timings.iter().map(|(n, _)| *n).collect::<Vec<_>>(),
        vec![4, 16],
        "one median per size, in order"
    );
    assert!(timings.iter().all(|(_, ns)| *ns > 0), "{timings:?}");
    let ledger = backend.ledger();
    assert_eq!(
        ledger.inputs().len(),
        2 * per_size,
        "warm-ups and repeats per size"
    );
    assert!(String::from_utf8_lossy(&ledger.inputs()[0]).contains("[5]"));
    assert!(String::from_utf8_lossy(&ledger.inputs()[per_size]).contains("[7]"));

    // The bench left a bench-mode driver behind; the next ordinary call gets a fresh session.
    let result = runner
        .run(Target::Reference, &json!([5]), LIMIT)
        .expect("the run after the bench works");
    assert_eq!(result, ProgramResult::Returned(json!(6)));
    assert_eq!(
        backend.ledger().created(),
        2,
        "the bench session was not reused for runs"
    );
    drop(runner);
}

#[test]
fn a_reference_that_dies_during_the_bench_is_an_infrastructure_error() {
    let runtime = Runtime::new().expect("a runtime");
    let backend = FakeBackend::new();
    backend.script_program([
        answer(0, 6),
        Behaviour::Exit {
            code: 1,
            oom_kill: 0,
        },
    ]);
    let mut runner = runner(&runtime, &backend);

    let outcome = runner.bench(&[(4, json!([5]))], LIMIT);

    assert!(
        matches!(outcome, Err(RunnerError::Infrastructure(ref reason)) if reason.contains("bench")),
        "{outcome:?}"
    );
    drop(runner);
}
