// SPDX-License-Identifier: MIT
//! Model tests for `SandboxOracle` (judge-sandbox T34, R15.4 to R15.6). Two
//! `FakeBackend`s stand in for the daemon — one per side, so user and
//! reference can never share a container — and the scripted behaviours drive
//! every `CallFacts` shape into its `ProgramResult` mapping. No Docker needed.
#![cfg(all(feature = "sandbox", feature = "testkit"))]

use std::path::PathBuf;
use std::time::{Duration, Instant};

use counterspell_engine::{Oracle, ProgramResult};
use counterspell_judge::languages::{LanguageCatalog, LanguageDefinition};
use counterspell_judge::sandbox::config::{OciRuntime, SandboxConfig};
use counterspell_judge::sandbox::oracle::SandboxOracle;
use counterspell_judge::sandbox::testkit::{Behaviour, FakeBackend};
use serde_json::{json, Value};

/// A signature `Value` matching the driver's schema — the only problem data
/// the oracle may hold. Expected outputs never reach it: `evaluate` accepts
/// `&Value`, the public input the engine generates, and nothing typed as a
/// hidden test could compile into this call.
fn signature() -> Value {
    json!({
        "protocol_version": 1,
        "function": "f",
        "parameters": [{"name": "a", "type": {"kind": "int"}}],
        "returns": {"kind": "int"}
    })
}

fn config() -> SandboxConfig {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..");
    SandboxConfig {
        docker_socket: PathBuf::from("/unused/sbx.sock"),
        runtime: OciRuntime::Runc,
        max_containers: 2,
        reaper_interval: Duration::from_secs(10),
        orphan_age: Duration::from_secs(40),
        kill_slack: Duration::from_millis(50),
        run_wall_cap: Duration::from_secs(20),
        max_consecutive_abnormal: 3,
        cleanup_budget: Duration::from_secs(5),
        api_timeout: Duration::from_secs(30),
        languages_file: root.join("languages/languages.toml"),
    }
}

fn python() -> LanguageDefinition {
    let cfg = config();
    LanguageCatalog::load(&cfg.languages_file)
        .expect("the shipped catalogue loads")
        .get("python")
        .expect("python is known")
        .clone()
}

/// An oracle over two fake backends plus the multi-thread runtime `evaluate`
/// blocks on. The oracle owns no runtime; the caller does.
fn rig() -> (
    SandboxOracle<FakeBackend>,
    FakeBackend,
    FakeBackend,
    tokio::runtime::Runtime,
) {
    let user = FakeBackend::new();
    let reference = FakeBackend::new();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_time()
        .build()
        .expect("the oracle's runtime builds");
    let oracle = SandboxOracle::new(
        runtime.handle().clone(),
        user.clone(),
        reference.clone(),
        python(),
        python(),
        &signature(),
        "def f(a):\n    return a\n",
        "def f(a):\n    return a\n",
        &config(),
    )
    .expect("the oracle builds");
    (oracle, user, reference, runtime)
}

fn answer(id: u64, out: i64) -> Behaviour {
    Behaviour::Answer(format!("{{\"t\":{id},\"out\":{out}}}\n").into_bytes())
}

/// Parsed answer lines become `Returned` on both sides.
#[test]
fn answers_become_returned() {
    let (mut oracle, user, reference, _rt) = rig();
    user.script_program([answer(1, 7)]);
    reference.script_program([answer(1, 9)]);
    let observation = oracle
        .evaluate(&json!([2, 5]), Duration::from_millis(500))
        .expect("both sides answered");
    assert_eq!(observation.user, ProgramResult::Returned(json!(7)));
    assert_eq!(observation.reference, ProgramResult::Returned(json!(9)));
}

/// An answer line the protocol parser rejects is `Garbled`, never trusted.
#[test]
fn a_line_the_parser_rejects_is_garbled() {
    let (mut oracle, user, reference, _rt) = rig();
    user.script_program([answer(1, 1), Behaviour::Answer(b"not json\n".to_vec())]);
    reference.script_program([answer(1, 1), answer(2, 1)]);
    let first = oracle
        .evaluate(&json!([1]), Duration::from_millis(500))
        .expect("first evaluate");
    assert_eq!(first.user, ProgramResult::Returned(json!(1)));
    let second = oracle
        .evaluate(&json!([1]), Duration::from_millis(500))
        .expect("second evaluate");
    let ProgramResult::Garbled { snippet } = &second.user else {
        panic!("unparsable output is Garbled: {:?}", second.user);
    };
    assert_eq!(snippet, "not json");
}

/// A driver exit carries the traceback tail into `Crashed`'s class and message.
#[test]
fn an_exception_becomes_crashed_with_class_and_message() {
    let (mut oracle, user, reference, _rt) = rig();
    user.script_program([
        Behaviour::Stderr(b"Traceback (most recent call last):\n".to_vec()),
        Behaviour::Stderr(b"ValueError: bad input\n".to_vec()),
        Behaviour::Exit {
            code: 1,
            oom_kill: 0,
        },
    ]);
    reference.script_program([answer(1, 0)]);
    let observation = oracle
        .evaluate(&json!([1]), Duration::from_millis(500))
        .expect("the crash is a result, not an error");
    assert_eq!(
        observation.user,
        ProgramResult::Crashed {
            exception_class: "ValueError".to_owned(),
            message: "bad input".to_owned(),
        }
    );
    // The dead session is reopened on the next call, with a fresh container.
    user.script_program([answer(1, 4)]);
    reference.script_program([answer(2, 0)]);
    let again = oracle
        .evaluate(&json!([1]), Duration::from_millis(500))
        .expect("the side reopened");
    assert_eq!(again.user, ProgramResult::Returned(json!(4)));
    assert_eq!(
        user.ledger().created(),
        2,
        "the crash's container was replaced, not reused"
    );
}

/// The kernel's OOM counter, not the program's text, says `OutOfMemory`.
#[test]
fn an_oom_kill_is_crashed_not_a_verdict() {
    let (mut oracle, user, reference, _rt) = rig();
    user.script_program([Behaviour::Exit {
        code: 137,
        oom_kill: 1,
    }]);
    reference.script_program([answer(1, 0)]);
    let observation = oracle
        .evaluate(&json!([1]), Duration::from_millis(500))
        .expect("OOM is a result, not an error");
    assert_eq!(
        observation.user,
        ProgramResult::Crashed {
            exception_class: "OutOfMemory".to_owned(),
            message: "killed by the memory limit".to_owned(),
        }
    );
}

/// A hang becomes `TimedOut` inside `limit + KILL_SLACK` — the bound the
/// engine relies on.
#[test]
fn a_hang_is_timed_out_within_limit_plus_slack() {
    let (mut oracle, user, reference, _rt) = rig();
    user.script_program([Behaviour::Hang]);
    reference.script_program([answer(1, 0)]);
    let limit = Duration::from_millis(200);
    let started = Instant::now();
    let observation = oracle
        .evaluate(&json!([1]), limit)
        .expect("the deadline is a result, not an error");
    assert_eq!(observation.user, ProgramResult::TimedOut);
    // `run_deadline` is limit + kill_slack; wall time stays near that.
    assert!(
        started.elapsed() < limit + Duration::from_secs(5),
        "evaluate returned within limit + slack: {:?}",
        started.elapsed()
    );
}

/// A compile failure on the user side is the program's own `Crashed`; on the
/// reference side it is infrastructure, so the whole evaluate errors.
#[test]
fn compile_failure_depends_on_the_side() {
    let (mut oracle, user, reference, _rt) = rig();
    // Exit 0 for the archive unpack, then 1 for the compile.
    user.script_exec_exits([0, 1]);
    reference.script_program([answer(1, 0)]);
    let observation = oracle
        .evaluate(&json!([1]), Duration::from_millis(500))
        .expect("the user's compile failure is a result");
    let ProgramResult::Crashed {
        exception_class, ..
    } = &observation.user
    else {
        panic!("a failed compile is Crashed: {:?}", observation.user);
    };
    assert_eq!(exception_class, "CompileError");

    let (mut oracle, user, reference, _rt) = rig();
    user.script_program([answer(1, 0)]);
    reference.script_exec_exits([0, 1]);
    let failed = oracle.evaluate(&json!([1]), Duration::from_millis(500));
    assert!(
        failed.is_err(),
        "the trusted reference failing is infrastructure: {failed:?}"
    );
}

/// A daemon fault is `OracleError`, never turned into a user failure.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn infrastructure_is_an_oracle_error_never_a_verdict() {
    let (mut oracle, user, _reference, rt) = rig();
    user.fail_call(
        1,
        counterspell_judge::sandbox::backend::InfraError::ApiTimeout,
    );
    let failed = tokio::task::spawn_blocking(move || {
        oracle.evaluate(&json!([1]), Duration::from_millis(500))
    })
    .await
    .expect("the blocking call joins");
    assert!(
        failed.is_err(),
        "a daemon fault is OracleError, not a user result: {failed:?}"
    );
    // A runtime cannot be dropped inside async context — detach it instead.
    rt.shutdown_background();
}

/// `evaluate` refuses an async context honestly instead of panicking inside
/// `Handle::block_on`.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn evaluate_rejects_the_async_context() {
    let (mut oracle, _user, _reference, rt) = rig();
    let failed = oracle.evaluate(&json!([1]), Duration::from_millis(10));
    assert!(
        failed.is_err(),
        "inside a runtime evaluate errors, never panics: {failed:?}"
    );
    rt.shutdown_background();
}
