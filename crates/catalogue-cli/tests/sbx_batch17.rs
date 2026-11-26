// SPDX-License-Identifier: MIT
//! Batch-17 evidence: the ten new catalogue packs through the real rootless sandbox daemon.
//!
//! For each batch-17 pack this runs the production S10 fill (`sandbox_fill`:
//! SBX-01 samples and curated hidden cases, SBX-02 generated candidates,
//! SBX-03 known ghosts, CAT-PT8 retired-hash refusal) and the S11 ghost legs
//! (`hunt` per `incorrect/` program must find a counterexample; the same hunt
//! on the reference must return not-found) through the judge's own
//! `PackRunnerFactory`/`SandboxOracle`. The pinned image digest is printed per
//! pack for the state record. Ignored by default.
//!
//! `SANDBOX_DOCKER_SOCKET=/run/user/1000/docker.sock cargo test -p counterspell-catalogue-cli --features sandbox --test sbx_batch17 -- --ignored --test-threads=1`
#![cfg(feature = "sandbox")]

mod sbx;

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::Duration;

use counterspell_catalogue_cli::load::{assemble_sandbox, read_pack, signature_descriptor};
use counterspell_catalogue_cli::sandbox_fill::{sandbox_fill, FillOutcome};
use counterspell_engine::{hunt, HuntIdentity, HuntOutcome, HuntRequest};
use counterspell_engine::{KnownGhost, SystemClock};
use counterspell_grimoire::meta::Meta;
use counterspell_grimoire::record;
use counterspell_judge::catalogue::retired::parse_retired;
use counterspell_judge::sandbox::bollard_backend::BollardBackend;
use counterspell_judge::sandbox::oracle::SandboxOracle;
use counterspell_judge::sandbox::pack_runner::{PackRunnerFactory, REFERENCE_SOURCE};
use sha2::{Digest, Sha256};

/// The batch-17 set this run covers.
const SLUGS: [&str; 10] = [
    "row-join",
    "null-marks",
    "key-count",
    "key-last",
    "group-tally",
    "join-span",
    "depend-rite",
    "null-join",
    "serial-check",
    "sched-rite",
];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn sha256_hex(bytes: &[u8]) -> String {
    Sha256::digest(bytes)
        .iter()
        .fold(String::new(), |mut hex, byte| {
            let _ = write!(hex, "{byte:02x}");
            hex
        })
}

/// The runner sources the job archive needs: the reference plus each
/// `incorrect/` program, private-side only.
fn runner_sources(private_dir: &Path) -> BTreeMap<String, String> {
    let mut sources = BTreeMap::new();
    sources.insert(
        REFERENCE_SOURCE.to_owned(),
        std::fs::read_to_string(private_dir.join("reference/solution.py"))
            .expect("reference/solution.py reads"),
    );
    for entry in std::fs::read_dir(private_dir.join("incorrect")).expect("incorrect/ reads") {
        let entry = entry.expect("entry");
        let name = format!("incorrect/{}", entry.file_name().to_str().expect("utf-8"));
        sources.insert(
            name.clone(),
            std::fs::read_to_string(entry.path()).unwrap_or_else(|e| panic!("{name}: {e}")),
        );
    }
    sources
}

/// S10 + S11 for every batch-17 pack, in one session so the daemon is touched once.
#[allow(
    clippy::too_many_lines,
    reason = "the ten packs walk one shared daemon session; splitting per leg would hide the setup"
)]
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn batch17_packs_sandbox_and_ghost_checks() {
    let _guard = sbx::ONE_RUN_AT_A_TIME.lock().await;
    let Some(private_root) = sbx::private_packs_root() else {
        eprintln!("skipping: no counterspell-packs-private checkout is available");
        return;
    };
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let python = catalog
        .get("python")
        .filter(|lang| lang.is_enabled())
        .expect("python is enabled")
        .clone();
    let digest = python.image().as_str().to_owned();
    let factory =
        PackRunnerFactory::from_config(cfg.clone()).expect("the pack runner factory builds");
    let retired = parse_retired(
        &std::fs::read_to_string(private_root.join("retired-values.toml")).unwrap_or_default(),
    )
    .expect("retired-values.toml parses");
    // Oracle and hunt work run on their own multi-thread runtime: the test's
    // current-thread runtime cannot drive their blocking daemon calls.
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("the worker runtime builds");

    for slug in SLUGS {
        let public_dir = repo_root().join("packs").join(slug);
        let private_dir = private_root.join("packs").join(slug);
        assert!(private_dir.is_dir(), "{slug}: private part missing");
        let files = read_pack(&public_dir, Some(&private_dir)).expect("pack reads");
        let record =
            record::build_pair(&public_dir, &private_dir).expect("static pair validation passes");
        let meta: Meta = toml::from_str(files.text_any("meta.toml").expect("meta.toml present"))
            .expect("meta parses");
        let manifest = files.manifest().expect("manifest");
        let data =
            assemble_sandbox(&record, &meta, &files, &manifest).expect("sandbox data assembles");
        let pack = data.sandbox_pack(slug, &retired, &digest);
        let signature = signature_descriptor(&record).expect("signature encodes");
        let sources = runner_sources(&private_dir);
        let mut runner = factory
            .build(sources.clone(), &signature)
            .expect("runner builds");

        // S10: the production fill — SBX-01/02/03, curated + generated hidden, retired check.
        let outcome = sandbox_fill(&pack.check, &mut runner).expect("the runner answers");
        let FillOutcome::Filled(fill) = outcome else {
            panic!("{slug}: S10 fill refused: {outcome:?}");
        };
        assert_eq!(
            fill.digest, digest,
            "{slug}: stamped digest must be the pinned image"
        );
        println!("{slug}: S10 filled; sandbox image digest {digest}");

        // S11: every incorrect program yields a counterexample; the reference finds none.
        let backend = BollardBackend::new(docker.clone(), cfg.api_timeout);
        let reference_text = sources
            .get(REFERENCE_SOURCE)
            .expect("reference source")
            .clone();
        let known: Vec<KnownGhost> = data
            .ghosts
            .iter()
            .map(|ghost| ghost.known.clone())
            .collect();
        let mut targets: Vec<(String, String)> = sources
            .iter()
            .filter(|(path, _)| path.starts_with("incorrect/"))
            .map(|(path, text)| (path.clone(), text.clone()))
            .collect();
        targets.push((REFERENCE_SOURCE.to_owned(), reference_text.clone()));
        for (program, user_text) in targets {
            let is_reference = program == REFERENCE_SOURCE;
            let mut oracle = SandboxOracle::new(
                runtime.handle().clone(),
                backend.clone(),
                backend.clone(),
                python.clone(),
                python.clone(),
                &signature,
                &user_text,
                &reference_text,
                &cfg,
            )
            .expect("oracle builds");
            let mut request = HuntRequest::new(
                data.schema.clone(),
                data.comparator,
                HuntIdentity {
                    problem_slug: slug.to_owned(),
                    code_sha256: sha256_hex(user_text.as_bytes()),
                    language: "python".to_owned(),
                    problem_version: u32::try_from(data.pack_version).unwrap_or(0),
                    image_digest: digest.clone(),
                },
            );
            request.samples = data
                .examples
                .iter()
                .map(|(input, _)| input.clone())
                .collect();
            request.known_ghosts = known.clone();
            let report = runtime
                .spawn_blocking(move || hunt(&request, &mut oracle, &SystemClock::new()))
                .await
                .expect("the hunt joins");
            match (&report.outcome, is_reference) {
                (HuntOutcome::Found(_), false) => {
                    println!("{slug}: {program} yields a counterexample (S11 pass)");
                }
                (HuntOutcome::NotFound(_), true) => {
                    println!("{slug}: reference hunt is not-found within budget (S11 pass)");
                }
                (other, _) => panic!(
                    "{slug}: {program} hunt outcome unexpected (reference={is_reference}): {other:?}"
                ),
            }
        }
    }
    // drain_dead asserts nothing is still running and reaps the corpses.
    let _ = sbx::drain_dead(&docker, &cfg, Duration::from_secs(30)).await;
    runtime.shutdown_background();
}
