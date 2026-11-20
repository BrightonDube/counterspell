// SPDX-License-Identifier: MIT
//! The `baseline` admission on the real rootless daemon (efficiency-familiar T22, R8, R13.2).
//!
//! A fixture runs through `FixtureCollector` and the same bench driver a learner's Submit
//! uses. A quadratic fixture declared quadratic is stored; the same fixture declared linear is
//! refused and nothing is stored. Quadratic is used because its growth separates cleanly from the
//! other classes; noisy linear and n log n data are mostly `Low` by design (see
//! `docs/kiro-log/efficiency-familiar-calibration.md`). Ignored by default.
//!
//! `cargo test -p counterspell-judge --features sandbox --test sbx_baseline -- --ignored`
#![cfg(feature = "sandbox")]

mod sbx;

use std::sync::Mutex;
use std::time::Duration;

use counterspell_common::familiar::ClassId;
use counterspell_common::status::RunMode;
use counterspell_engine::Schema;
use counterspell_judge::familiar::baseline_cmd::{
    fixture_inputs, run_baseline, BaselineOutcome, BaselineRequest, BaselineStore,
    FixtureCollector, Refusal,
};
use counterspell_judge::familiar::store::{BaselineRow, StoreError};
use counterspell_judge::familiar::BenchPlan;
use counterspell_judge::sandbox::bollard_backend::BollardBackend;
use counterspell_judge::sandbox::oracle::{side, Role};
use serde_json::json;

/// Benches must not compete for CPU: production runs one bench at a time, so the two tests in
/// this file take turns instead of timing each other.
static ONE_BENCH_AT_A_TIME: tokio::sync::Mutex<()> = tokio::sync::Mutex::const_new(());

/// Keeps accepted rows in memory so the test needs no database.
#[derive(Default)]
struct MemoryStore {
    rows: Mutex<Vec<BaselineRow>>,
}

impl BaselineStore for MemoryStore {
    fn insert(
        &self,
        rows: &[BaselineRow],
    ) -> impl std::future::Future<Output = Result<u64, StoreError>> + Send {
        let mut held = self.rows.lock().expect("the lock is not poisoned");
        held.extend_from_slice(rows);
        std::future::ready(Ok(rows.len() as u64))
    }
}

/// `f(values)` takes a list of integers and returns an integer.
fn signature() -> Vec<u8> {
    serde_json::to_vec(&json!({
        "protocol_version": 1,
        "function": "f",
        "parameters": [{"name": "values", "type": {"kind": "list", "items": {"kind": "int"}}}],
        "returns": {"kind": "int"}
    }))
    .expect("the signature encodes")
}

const QUADRATIC: &str = "def f(values):\n    count = 0\n    for i in range(len(values)):\n        for j in range(i + 1, len(values)):\n            count += 1\n    return count\n";

fn schema() -> Schema {
    Schema::Tuple {
        items: vec![Schema::IntArray {
            min_len: 0,
            max_len: 16,
            min: 0,
            max: 1000,
            sorted: false,
            unique: false,
        }],
    }
}

/// Runs `source` as the fixture at `sizes`, declared as `declared`, against an in-memory store.
async fn baseline_of(
    source: &str,
    sizes: &[u64],
    declared: ClassId,
) -> (BaselineOutcome, MemoryStore, String) {
    let _turn = ONE_BENCH_AT_A_TIME.lock().await;
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let python = catalog.get("python").expect("python is enabled").clone();
    let digest = python.image().as_str().to_owned();
    let driver_root = cfg
        .languages_file
        .parent()
        .expect("the catalogue file has a parent")
        .to_path_buf();
    let fixture_side = side(
        BollardBackend::new(docker, cfg.api_timeout),
        python,
        &driver_root,
        &signature(),
        source,
        &cfg,
        Role::Reference,
    )
    .expect("the side builds");
    let inputs = fixture_inputs(&schema(), 0, "sbx-baseline", 1, sizes).expect("inputs build");
    let (_stop, stop_rx) = tokio::sync::watch::channel(false);
    let mut collector =
        FixtureCollector::new(fixture_side, inputs, Duration::from_secs(10), stop_rx);
    let store = MemoryStore::default();
    let request = BaselineRequest {
        problem_slug: "sbx-baseline".to_owned(),
        problem_version: 1,
        lang: "python".to_owned(),
        image_digest: digest.clone(),
        reference: declared,
        plan: BenchPlan {
            mode: RunMode::Submit,
            sizes: sizes.to_vec(),
            size_param: 0,
            problem_slug: "sbx-baseline".to_owned(),
            problem_version: 1,
            // A generous budget: this test is about the class, not the Submit time limit.
            budget: Duration::from_secs(60),
        },
    };
    let outcome = run_baseline(&mut collector, &store, &request)
        .await
        .expect("the bench itself does not fail");
    (outcome, store, digest)
}

#[tokio::test]
#[ignore = "needs the rootless sandbox daemon (SANDBOX_DOCKER_SOCKET)"]
async fn sbx_baseline_stores_a_quadratic_fixture_declared_quadratic() {
    let sizes = [256, 512, 1024, 2048, 4096, 8192];
    let (outcome, store, digest) = baseline_of(QUADRATIC, &sizes, ClassId::Quadratic).await;

    assert_eq!(outcome, BaselineOutcome::Stored { inserted: 6 });
    let rows = store.rows.lock().expect("lock");
    let stored: Vec<u64> = rows.iter().map(|row| row.n).collect();
    assert_eq!(stored, sizes.to_vec());
    assert!(rows
        .iter()
        .all(|row| row.lang == "python" && row.image_digest == digest && row.median_ns > 0));
}

#[tokio::test]
#[ignore = "needs the rootless sandbox daemon (SANDBOX_DOCKER_SOCKET)"]
async fn sbx_baseline_refuses_a_quadratic_fixture_declared_linear() {
    let sizes = [256, 512, 1024, 2048, 4096, 8192];
    let (outcome, store, _digest) = baseline_of(QUADRATIC, &sizes, ClassId::Linear).await;

    // The bench finished (not incomplete, not failed) and the fit disagreed with the declaration.
    match outcome {
        BaselineOutcome::Refused(Refusal::ClassMismatch { observed, .. }) => {
            assert_eq!(observed, ClassId::Quadratic);
        }
        BaselineOutcome::Refused(Refusal::NotConfirmed { .. }) => {}
        other => panic!("a quadratic fixture must be refused on the fit, got {other:?}"),
    }
    assert!(store.rows.lock().expect("lock").is_empty());
}
