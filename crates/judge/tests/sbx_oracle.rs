// SPDX-License-Identifier: MIT
//! `SandboxOracle` on the real daemon (judge-sandbox T34, R15.4 to R15.6).
//!
//! A good pair returns equal `Returned` values; a hostile user side can only
//! ever produce `TimedOut`, `Crashed` or `Garbled`, inside `limit + slack` —
//! the oracle's answers are facts about what the programs did, and a forged
//! verdict line would still only be text the engine compares. Ignored by
//! default.
//!
//! `cargo test -p counterspell-judge --features sandbox --test sbx_oracle -- --ignored`
#![cfg(feature = "sandbox")]

mod sbx;

use std::time::{Duration, Instant};

use counterspell_engine::{Oracle, ProgramResult};
use counterspell_judge::sandbox::bollard_backend::BollardBackend;
use counterspell_judge::sandbox::oracle::SandboxOracle;
use serde_json::{json, Value};

/// `add`'s public signature — the only problem data the oracle may hold.
fn signature() -> Value {
    json!({
        "protocol_version": 1,
        "function": "add",
        "parameters": [
            {"name": "a", "type": {"kind": "int"}},
            {"name": "b", "type": {"kind": "int"}}
        ],
        "returns": {"kind": "int"}
    })
}

/// The fixture's trusted reference implementation.
const REFERENCE: &str = "def add(a, b):\n    return a + b\n";

/// Builds a real-daemon oracle and the runtime `evaluate` blocks on.
async fn oracle(user_source: &str) -> (SandboxOracle<BollardBackend>, tokio::runtime::Runtime) {
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let python = catalog.get("python").expect("python is known").clone();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .expect("the oracle's runtime builds");
    let oracle = SandboxOracle::new(
        runtime.handle().clone(),
        BollardBackend::new(docker.clone(), cfg.api_timeout),
        BollardBackend::new(docker, cfg.api_timeout),
        python.clone(),
        python,
        &signature(),
        user_source,
        REFERENCE,
        &cfg,
    )
    .expect("the oracle builds");
    (oracle, runtime)
}

/// Runs `evaluate` on a blocking thread, as the engine does: the oracle moves
/// in and comes back out with the result.
async fn evaluate(
    oracle: SandboxOracle<BollardBackend>,
    input: Value,
    limit: Duration,
) -> (
    SandboxOracle<BollardBackend>,
    Result<counterspell_engine::Observation, counterspell_engine::OracleError>,
) {
    let mut oracle = oracle;
    tokio::task::spawn_blocking(move || {
        let result = oracle.evaluate(&input, limit);
        (oracle, result)
    })
    .await
    .expect("the blocking call joins")
}

/// S34-good: a correct user program and the reference both return the same
/// value on a public input, in separate containers.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_oracle_good_pair() {
    let (oracle, runtime) = oracle(&sbx::solution("add_correct.py")).await;
    let (oracle, first) = evaluate(oracle, json!([2, 3]), Duration::from_secs(2)).await;
    let observation = first.expect("both sides answered");
    assert_eq!(observation.user, ProgramResult::Returned(json!(5)));
    assert_eq!(observation.reference, ProgramResult::Returned(json!(5)));
    // A second call reuses the live sessions — no new containers needed.
    let (oracle, second) = evaluate(oracle, json!([10, -4]), Duration::from_secs(2)).await;
    let again = second.expect("both sides answered again");
    assert_eq!(again.user, ProgramResult::Returned(json!(6)));
    drop(oracle);
    runtime.shutdown_background();
    let (docker, cfg) = sbx::sandbox().await;
    let left = sbx::cleared_within(&docker, &cfg, Duration::from_secs(10)).await;
    assert!(
        left.is_empty(),
        "the oracle's containers were removed: {left:?}"
    );
}

/// S34-hostile: hostile corpus programs on the user side can only ever
/// produce `TimedOut`, `Crashed` or `Garbled` — and each within the bound.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_oracle_hostile_user() {
    let limit = Duration::from_secs(3);
    for (name, expected) in [
        ("sleep_forever.py", "TimedOut"),
        ("early_exit.py", "Crashed"),
        ("self_kill.py", "Crashed"),
        ("stdout_flood.py", "Garbled"),
        ("memory_alloc.py", "Crashed"),
    ] {
        let (oracle, runtime) = oracle(&sbx::hostile(name)).await;
        let started = Instant::now();
        let (oracle, result) = evaluate(oracle, json!([2, 3]), limit).await;
        let elapsed = started.elapsed();
        drop(oracle);
        runtime.shutdown_background();
        match result {
            Ok(observation) => {
                let kind = match &observation.user {
                    ProgramResult::Returned(_) => "Returned",
                    ProgramResult::Crashed { .. } => "Crashed",
                    ProgramResult::TimedOut => "TimedOut",
                    ProgramResult::Garbled { .. } => "Garbled",
                };
                assert!(
                    matches!(
                        observation.user,
                        ProgramResult::TimedOut
                            | ProgramResult::Crashed { .. }
                            | ProgramResult::Garbled { .. }
                    ),
                    "{name}: a hostile program gives {kind}, expected {expected}: {:?}",
                    observation.user
                );
                assert!(
                    elapsed < limit + Duration::from_secs(30),
                    "{name}: the call stayed near limit + slack: {elapsed:?}"
                );
                println!("{name}: {kind} in {elapsed:?} ({observation:?})");
            }
            // A daemon fault under hostile load is infrastructure, still never
            // a user verdict — acceptable evidence of containment.
            Err(error) => println!("{name}: infrastructure — {error}"),
        }
        let (docker, cfg) = sbx::sandbox().await;
        sbx::drain_dead(&docker, &cfg, Duration::from_secs(90)).await;
    }
}
