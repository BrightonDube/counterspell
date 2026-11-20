// SPDX-License-Identifier: MIT
//! Generated-input driver properties for every published pack (language-drivers
//! T10, R43 to R49).
//!
//! Each published pack supplies a known-correct fixture per enabled language
//! under `packs/<slug>/fixtures/<lang>/`. This suite runs every fixture on the
//! real rootless daemon through `SandboxOracle`: generated schema-valid inputs
//! pass through the driver protocol, the fixture's answer must agree with the
//! Python reference under the pack comparator, a repeated call must produce the
//! same answer, and each answer must survive a canonical JSON round trip.
//! Fixtures are test-only programs — the Python reference stays the only
//! expected-output authority. Ignored by default.
//!
//! `cargo test -p counterspell-judge --features sandbox --test sbx_pack_props -- --ignored`
#![cfg(feature = "sandbox")]

mod sbx;

use std::path::{Path, PathBuf};
use std::time::Duration;

use counterspell_catalogue_cli::load::signature_descriptor;
use counterspell_engine::generate::{CandidateStream, GeneratorConfig};
use counterspell_engine::{Observation, Oracle, OracleError, ProgramResult};
use counterspell_grimoire::meta::Meta;
use counterspell_grimoire::record;
use counterspell_judge::languages::{LanguageCatalog, LanguageDefinition};
use counterspell_judge::sandbox::bollard_backend::BollardBackend;
use counterspell_judge::sandbox::oracle::SandboxOracle;
use serde_json::Value;

/// The enabled languages and their fixture file extensions.
const FIXTURE_LANGS: [(&str, &str); 2] = [("python", "py"), ("javascript", "js")];
/// Per-call deadline handed to the oracle, matching the sbx scenario bound.
const CALL_LIMIT: Duration = Duration::from_secs(2);

/// The shipped packs directory (`packs/`).
fn packs_root() -> PathBuf {
    sbx::repo_root().join("packs")
}

/// Reads one pack file as text.
fn read_pack_file(path: &Path) -> String {
    std::fs::read_to_string(path)
        .unwrap_or_else(|error| panic!("reading {}: {error}", path.display()))
}

/// Parses a pack's `meta.toml`.
fn pack_meta(pack_dir: &Path) -> Meta {
    toml::from_str(&read_pack_file(&pack_dir.join("meta.toml")))
        .unwrap_or_else(|error| panic!("{}: meta.toml: {error}", pack_dir.display()))
}

/// The published pack directories, sorted by path. `_`-prefixed directories and
/// drafts are skipped, exactly as the catalogue import treats them.
fn published_packs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    for entry in std::fs::read_dir(packs_root()).expect("packs/ is readable") {
        let path = entry.expect("a packs/ entry").path();
        let Some(slug) = path.file_name().and_then(|name| name.to_str()) else {
            continue;
        };
        if !path.is_dir() || slug.starts_with('_') {
            continue;
        }
        if pack_meta(&path).status == "published" {
            dirs.push(path);
        }
    }
    dirs.sort();
    dirs
}

/// Deterministic inputs for a pack: the seeded stream yields the public samples
/// first, then `hidden_count` schema-valid generated values.
fn generated_inputs(meta: &Meta) -> Vec<Value> {
    let samples = meta.samples.iter().map(|sample| sample.input.clone());
    let seed = u64::try_from(meta.generator.seed).expect("the seed is in range");
    let take = usize::try_from(meta.generator.hidden_count).expect("hidden_count is in range");
    CandidateStream::new(
        meta.input_schema.clone(),
        seed,
        Vec::new(),
        samples.collect(),
        GeneratorConfig::default(),
    )
    .take(take)
    .map(|candidate| candidate.value)
    .collect()
}

/// Runs `evaluate` on a blocking thread, as the engine does: the oracle moves in
/// and comes back out with the result.
async fn evaluate(
    oracle: SandboxOracle<BollardBackend>,
    input: &Value,
) -> (
    SandboxOracle<BollardBackend>,
    Result<Observation, OracleError>,
) {
    let mut oracle = oracle;
    let input = input.clone();
    tokio::task::spawn_blocking(move || {
        let result = oracle.evaluate(&input, CALL_LIMIT);
        (oracle, result)
    })
    .await
    .expect("the blocking evaluate joins")
}

/// T10: every published pack, every enabled language — generated schema-valid
/// inputs round-trip through the driver protocol and each fixture agrees with
/// the Python reference under the pack comparator. A repeated first call must
/// return the identical answer.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_published_packs_fixtures_agree_with_reference() {
    let _guard = sbx::ONE_RUN_AT_A_TIME.lock().await;
    let Some(private_root) = sbx::private_packs_root() else {
        eprintln!("skipping: no counterspell-packs-private checkout is available");
        return;
    };
    let packs = published_packs();
    assert!(
        packs.len() >= 3,
        "the property suite must cover every published pack — found {packs:?}"
    );
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let python = catalog
        .get("python")
        .filter(|lang| lang.is_enabled())
        .expect("python is enabled")
        .clone();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("the oracle runtime builds");

    for pack_dir in &packs {
        check_pack(
            pack_dir,
            &private_root,
            &docker,
            &cfg,
            &catalog,
            &python,
            runtime.handle(),
        )
        .await;
    }
    // Session drops spawn kill+remove on the test runtime; wait for the work to
    // land and reap any corpses so the daemon is left clean.
    sbx::drain_dead(&docker, &cfg, Duration::from_secs(30)).await;
    runtime.shutdown_background();
}

/// Runs the fixture-vs-reference agreement check for one pack in both fixture
/// languages.
#[allow(
    clippy::too_many_arguments,
    reason = "the daemon context travels together; a context struct would hide nothing"
)]
async fn check_pack(
    pack_dir: &Path,
    private_root: &Path,
    docker: &bollard::Docker,
    cfg: &counterspell_judge::sandbox::config::SandboxConfig,
    catalog: &LanguageCatalog,
    python: &LanguageDefinition,
    handle: &tokio::runtime::Handle,
) {
    let meta = pack_meta(pack_dir);
    let slug = meta.slug.clone();
    let private_dir = private_root.join("packs").join(&slug);
    let record = record::build_pair(pack_dir, &private_dir)
        .unwrap_or_else(|error| panic!("{slug}: the pack record builds: {error:?}"));
    let signature =
        signature_descriptor(&record).expect("the signature maps to the protocol shape");
    let reference = read_pack_file(&private_dir.join("reference/solution.py"));
    let inputs = generated_inputs(&meta);
    assert!(!inputs.is_empty(), "{slug}: the generator yields inputs");

    for (lang, ext) in FIXTURE_LANGS {
        let lang_def = catalog
            .get(lang)
            .filter(|lang| lang.is_enabled())
            .unwrap_or_else(|| panic!("{lang} is enabled"))
            .clone();
        let fixture =
            read_pack_file(&pack_dir.join(format!("fixtures/{lang}/known_correct.{ext}")));
        let mut oracle = SandboxOracle::new(
            handle.clone(),
            BollardBackend::new(docker.clone(), cfg.api_timeout),
            BollardBackend::new(docker.clone(), cfg.api_timeout),
            lang_def,
            python.clone(),
            &signature,
            &fixture,
            &reference,
            cfg,
        )
        .expect("the oracle builds");

        let mut first_answer: Option<Value> = None;
        for (index, input) in inputs.iter().enumerate() {
            let (back, result) = evaluate(oracle, input).await;
            oracle = back;
            let observation = result.unwrap_or_else(|error| {
                panic!(
                    "{slug}/{lang} input #{index} (seed {}): {error}",
                    meta.generator.seed
                )
            });
            let (ProgramResult::Returned(got), ProgramResult::Returned(expected)) =
                (&observation.user, &observation.reference)
            else {
                panic!(
                    "{slug}/{lang} seed {} input #{index} {input}: \
                     user = {:?}, reference = {:?}",
                    meta.generator.seed, observation.user, observation.reference
                );
            };
            // The answer is a canonical JSON value: decoding, re-encoding and
            // re-decoding preserves it exactly.
            let reparsed: Value =
                serde_json::from_str(&serde_json::to_string(got).expect("the answer re-encodes"))
                    .expect("the answer re-decodes");
            assert_eq!(
                &reparsed, got,
                "{slug}/{lang} seed {} input #{index}: the answer is not canonical JSON",
                meta.generator.seed
            );
            assert!(
                record.comparator.equal(expected, got),
                "{slug}/{lang} seed {} input #{index} {input}: fixture {got} \
                 disagrees with reference {expected}",
                meta.generator.seed
            );
            if index == 0 {
                first_answer = Some(got.clone());
            }
        }

        // Repeatability: the same input through the same session returns the
        // identical answer.
        let (back, result) = evaluate(oracle, &inputs[0]).await;
        oracle = back;
        let repeated = result.expect("the repeat call answers");
        assert_eq!(
            repeated.user,
            ProgramResult::Returned(first_answer.expect("a first answer")),
            "{slug}/{lang}: a repeated call must be deterministic"
        );
        drop(oracle);
    }
}
