// SPDX-License-Identifier: MIT
//! Real-daemon bench-mode handoff (language-drivers T08): a JavaScript solution
//! runs a correctness-sized session, then the bench driver starts in the same
//! container, warm-ups and timed calls land medians per size, and a spent
//! budget reports `Incomplete` rather than a verdict.

#![cfg(feature = "sandbox")]

mod sbx;

use std::time::Duration;

use counterspell_common::status::{RunMode, Verdict};
use counterspell_engine::Schema;
use counterspell_judge::executor::RunOutcome;
use counterspell_judge::familiar::{BenchCollector, BenchOutcome, BenchPlan};
use counterspell_judge::languages::{LanguageCatalog, LanguageDefinition};
use counterspell_judge::sandbox::backend::BackendSession;
use counterspell_judge::sandbox::bench::SandboxBenchCollector;
use counterspell_judge::sandbox::bollard_backend::BollardBackend;
use counterspell_judge::sandbox::config::SandboxConfig;
use counterspell_judge::sandbox::session::{open_session, Opened};
use counterspell_judge::sandbox::spec::{build_tar, driver_file_name, SandboxFiles, SessionSpec};
use counterspell_judge::testkit::RecordingContext;
use serde_json::Value;
use tokio::sync::watch;
use uuid::Uuid;

/// A correct JavaScript two-sum (`CommonJS` export, per protocol.md).
const JS_TWO_SUM: &str = "module.exports = function two_sum(nums, target) {\n    const seen = new Map();\n    for (let i = 0; i < nums.length; i++) {\n        if (seen.has(target - nums[i])) return [seen.get(target - nums[i]), i];\n        seen.set(nums[i], i);\n    }\n    return [];\n};\n";

/// The signature from the `two-sum` fixture: `nums` (list, index 0) grows.
fn schema() -> Schema {
    Schema::Tuple {
        items: vec![
            Schema::IntArray {
                min_len: 0,
                max_len: 16,
                min: 0,
                max: 10_000,
                sorted: false,
                unique: false,
            },
            Schema::Int {
                min: 0,
                max: 20_000,
            },
        ],
    }
}

/// The `two-sum` public signature, as the archive carries it.
fn signature() -> Vec<u8> {
    let text = std::fs::read_to_string(sbx::fixture_dir().join("two-sum.json"))
        .expect("the fixture reads");
    let spec: Value = serde_json::from_str(&text).expect("the fixture parses");
    serde_json::to_vec(&spec["signature"]).expect("the signature encodes")
}

/// The real JavaScript driver file (paths in the catalogue sit under `languages/`).
fn js_driver(cfg: &SandboxConfig, lang: &LanguageDefinition) -> Vec<u8> {
    let root = cfg
        .languages_file
        .parent()
        .expect("the catalogue file has a parent");
    std::fs::read(root.join(lang.driver().as_str())).expect("driver file reads")
}

/// An archive with the real JavaScript driver and the given source.
fn js_archive(lang: &LanguageDefinition, driver: Vec<u8>, source: &str) -> Vec<u8> {
    let files = SandboxFiles::new(
        lang.file().clone(),
        driver_file_name(lang).expect("driver name"),
        source.as_bytes().to_vec(),
        driver,
        signature(),
    )
    .expect("three files");
    build_tar(&files).expect("archive builds")
}

async fn bench_session(
    backend: &BollardBackend,
    catalog: &LanguageCatalog,
    cfg: &SandboxConfig,
    archive: &[u8],
) -> Opened<impl BackendSession> {
    let lang = catalog.get("javascript").expect("javascript is enabled");
    let spec = SessionSpec::new(
        lang,
        cfg.run_wall_cap,
        cfg.runtime,
        Uuid::new_v4(),
        "sbx-bench",
        1,
        i64::MAX / 2,
    )
    .expect("spec builds");
    open_session(backend, &spec, lang, archive)
        .await
        .expect("the session opens")
}

fn plan(mode: RunMode, sizes: &[u64], budget: Duration) -> BenchPlan {
    BenchPlan {
        mode,
        sizes: sizes.to_vec(),
        size_param: 0,
        problem_slug: "two-sum".to_owned(),
        problem_version: 1,
        budget,
    }
}

#[tokio::test]
#[ignore = "needs the rootless sandbox daemon (SANDBOX_DOCKER_SOCKET)"]
async fn sbx_bench_records_timed_points_in_the_same_container() {
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let backend = BollardBackend::new(docker.clone(), cfg.api_timeout);
    let lang = catalog.get("javascript").expect("javascript is enabled");
    let archive = js_archive(lang, js_driver(&cfg, lang), JS_TWO_SUM);
    let mut opened = bench_session(&backend, &catalog, &cfg, &archive).await;
    let (_tx, mut stop) = watch::channel(false);
    let (ctx, _log) = RecordingContext::without_stop(1);

    let outcome = {
        let mut collector = SandboxBenchCollector::new(
            &mut opened,
            lang,
            schema(),
            Duration::from_secs(2),
            &ctx,
            &mut stop,
        );
        collector
            .collect(&plan(
                RunMode::Submit,
                &[16, 64, 256, 1024, 2048, 4096],
                Duration::from_secs(4),
            ))
            .await
            .expect("the bench completes")
    };

    let BenchOutcome::Complete(points) = outcome else {
        panic!("a good program completes the bench: {outcome:?}");
    };
    assert_eq!(
        points.iter().map(|p| p.n).collect::<Vec<_>>(),
        vec![16, 64, 256, 1024, 2048, 4096],
        "one median point per size"
    );
    for point in &points {
        assert!(
            point.median_ns > 0,
            "a timed median is measured, not zero: {point:?}"
        );
    }
    eprintln!("bench points: {points:?}");

    opened.session.close().await.expect("the session closes");
    sbx::drain_dead(&docker, &cfg, Duration::from_secs(60)).await;
}

#[tokio::test]
#[ignore = "needs the rootless sandbox daemon (SANDBOX_DOCKER_SOCKET)"]
async fn sbx_bench_budget_cut_is_incomplete_and_the_container_is_clean() {
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let backend = BollardBackend::new(docker.clone(), cfg.api_timeout);
    let lang = catalog.get("javascript").expect("javascript is enabled");
    let archive = js_archive(lang, js_driver(&cfg, lang), JS_TWO_SUM);
    let mut opened = bench_session(&backend, &catalog, &cfg, &archive).await;
    let (_tx, mut stop) = watch::channel(false);
    let (ctx, _log) = RecordingContext::without_stop(1);

    let started = std::time::Instant::now();
    let outcome = {
        let mut collector = SandboxBenchCollector::new(
            &mut opened,
            lang,
            schema(),
            Duration::from_secs(2),
            &ctx,
            &mut stop,
        );
        collector
            .collect(&plan(
                RunMode::Run,
                &[16, 64, 256],
                Duration::from_millis(1),
            ))
            .await
            .expect("the bench completes")
    };
    let wall = started.elapsed();

    assert_eq!(
        outcome,
        BenchOutcome::Incomplete(vec![]),
        "a spent budget stops the bench before any point: {outcome:?}"
    );
    assert!(
        wall < Duration::from_secs(5),
        "the budget bounds the bench wall, not the run cap: {wall:?}"
    );

    opened.session.close().await.expect("the session closes");
    sbx::drain_dead(&docker, &cfg, Duration::from_secs(60)).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_bench_run_publishes_points_and_records_timings() {
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);

    // The full executor path: correctness, then the bench in the same container.
    let observation = sbx::run_source(
        &docker,
        &cfg,
        &catalog,
        "javascript",
        "two-sum",
        JS_TWO_SUM,
        RunMode::Submit,
    )
    .await;

    let outcome = observation.outcome.as_ref().expect("the run completes");
    let RunOutcome::Completed(result) = outcome else {
        panic!("the run must complete: {outcome:?}");
    };
    assert_eq!(result.verdict, Verdict::Accepted);

    // three tests, then bench_start, one point per declared size, the fit, the mood.
    let names = observation.event_names();
    assert_eq!(
        names[..11],
        [
            "compiling",
            "test",
            "test",
            "test",
            "bench_start",
            "bench_point",
            "bench_point",
            "bench_point",
            "bench_point",
            "bench_point",
            "bench_point"
        ],
        "the bench tail follows the test events in order"
    );
    // The fit on real timings usually settles a class; when it does it precedes the
    // mood, which always ends the run.
    let tail = &names[11..];
    assert!(
        tail == ["familiar"] || tail == ["complexity", "familiar"],
        "the mood always ends the run: {tail:?}"
    );
    assert_eq!(
        observation.timings.len(),
        30,
        "six sizes, five timed repeats each"
    );
    assert!(
        observation.timings.iter().all(|row| row.ns > 0),
        "every recorded sample is positive"
    );
    assert!(
        result.measured.is_some(),
        "the measured columns ride the terminal result"
    );
    assert!(
        sbx::cleared_within(&docker, &cfg, Duration::from_secs(15))
            .await
            .is_empty(),
        "no container survives the run"
    );
}
