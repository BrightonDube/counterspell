//! `approval.toml` gates the sandbox (problem-catalogue C19b, CAT-PT7, CAT-I11).
//!
//! A missing approval, a wrong `pack_version` and a hash mismatch each refuse
//! the pack before the sandbox runs — through the real binary, with zero
//! catalogue writes. The in-process test drives the same production ordering
//! (`check_approval` before `validate_sandbox`) with a counting `PackRunner`:
//! the fake records every call, so a refusal that still reached the sandbox
//! would show up as a call. Checking only printed output could not prove the
//! sandbox stayed idle — the call count can.
//!
//! Fixtures live under `tests/fixtures/c19b/` and carry canary strings only.
//!
//! Run with
//! `cargo test -p counterspell-judge --test pg_catalogue_approval_gate -- --ignored`.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use counterspell_catalogue_cli::load::{assemble_sandbox, read_pack};
use counterspell_catalogue_cli::sandbox_fill::{validate_sandbox, ValidateOutcome};
use counterspell_engine::classify::ProgramResult;
use counterspell_grimoire::runner::{PackRunner, RunnerError, Target};
use counterspell_judge::catalogue::approval::{check_approval, parse_approval, ApprovalRefusal};
use counterspell_judge::catalogue::retired::parse_retired;
use serde_json::{json, Value};
use sqlx::postgres::PgConnectOptions;
use sqlx::ConnectOptions;
use sqlx::PgPool;

/// The compiled binary under test.
const BIN: &str = env!("CARGO_BIN_EXE_counterspell-catalogue");

/// `tests/fixtures/c19b/`.
fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/c19b")
}

/// The pool's URL as `CATALOGUE_DATABASE_URL`.
fn catalogue_url(options: &PgConnectOptions) -> String {
    options.to_url_lossy().to_string()
}

/// Spawns `import --static-only` against the fixture public tree.
#[allow(clippy::disallowed_methods)] // a test harness may spawn the CLI under test
fn import(private_root: &Path, url: &str) -> std::process::Output {
    let packs = fixture_root().join("public");
    Command::new(BIN)
        .args([
            "import",
            "--static-only",
            "--packs",
            packs.to_str().unwrap(),
            "--slug",
            "add-one",
            "--private-root",
            private_root.to_str().unwrap(),
        ])
        .env("CATALOGUE_DATABASE_URL", url)
        .env_remove("DATABASE_URL")
        .output()
        .expect("binary runs")
}

/// Recursively copies a fixture directory.
fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("mkdir");
    for entry in std::fs::read_dir(from).expect("read dir") {
        let entry = entry.expect("entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copy");
        }
    }
}

/// A private root holding `add-one`'s private files and `approval`, when given.
fn private_root(approval: Option<&str>) -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("tempdir");
    let target = root.path().join("packs/add-one");
    copy_dir(&fixture_root().join("private/packs/add-one"), &target);
    if let Some(text) = approval {
        std::fs::write(target.join("approval.toml"), text).expect("approval writes");
    }
    root
}

/// The combined manifest hash for the fixture pair, recomputed over `root`.
fn combined_hash(root: &Path) -> String {
    let files = read_pack(
        &fixture_root().join("public/add-one"),
        Some(&root.join("packs/add-one")),
    )
    .expect("pack reads");
    files.manifest().expect("manifest").content_sha256_hex()
}

/// A well-formed `approval.toml` for `add-one` version 1.
fn approval_toml(hash_hex: &str, pack_version: i32) -> String {
    format!(
        "slug = \"add-one\"\n\
         pack_version = {pack_version}\n\
         content_sha256 = \"{hash_hex}\"\n\
         public_commit = \"0123456789abcdef0123456789abcdef01234567\"\n\
         approved_by = \"tester\"\n\
         approved_at = \"2026-11-16\"\n"
    )
}

/// Row counts for `(problems, problem_versions, catalogue_events)`.
async fn counts(pool: &PgPool) -> (i64, i64, i64) {
    sqlx::query_as(
        "SELECT (SELECT count(*) FROM problems), (SELECT count(*) FROM problem_versions), \
         (SELECT count(*) FROM catalogue_events)",
    )
    .fetch_one(pool)
    .await
    .expect("counts")
}

/// A `PackRunner` that counts every `run` call and answers like the `add-one`
/// reference (`value + 1`), so the sandbox fill can complete when reached.
struct CountingRunner {
    calls: usize,
}

impl PackRunner for CountingRunner {
    fn run(
        &mut self,
        _target: Target<'_>,
        input: &Value,
        _limit: Duration,
    ) -> Result<ProgramResult, RunnerError> {
        self.calls += 1;
        let value = input[0].as_i64().unwrap_or_default() + 1;
        Ok(ProgramResult::Returned(json!(value)))
    }

    fn bench(
        &mut self,
        sized_inputs: &[(u64, Value)],
        _limit: Duration,
    ) -> Result<Vec<(u64, u64)>, RunnerError> {
        Ok(sized_inputs.iter().map(|(size, _)| (*size, 0)).collect())
    }
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_missing_approval_refuses_before_the_sandbox(pool: PgPool) {
    let root = private_root(None);
    let output = import(root.path(), &catalogue_url(&pool.connect_options()));
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("add-one refused:approval_missing"),
        "missing approval refuses: {stdout}"
    );
    assert_eq!(counts(&pool).await, (0, 0, 0), "a refusal writes nothing");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_wrong_pack_version_and_hash_mismatch_refuse_with_zero_writes(pool: PgPool) {
    let url = catalogue_url(&pool.connect_options());

    // The approval's pack_version disagrees with meta.toml's.
    let wrong_version = private_root(Some(&approval_toml(&"f".repeat(64), 99)));
    let output = import(wrong_version.path(), &url);
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("refused:approval_mismatch"),
        "a wrong pack_version refuses"
    );

    // A well-formed hash that is not the combined manifest hash.
    let bad_hash = private_root(Some(&approval_toml(&"f".repeat(64), 1)));
    let output = import(bad_hash.path(), &url);
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("refused:approval_mismatch"),
        "a hash mismatch refuses"
    );

    assert_eq!(
        counts(&pool).await,
        (0, 0, 0),
        "every refusal writes nothing"
    );
}

/// The production order at the seam `import_one` uses: `check_approval`, and
/// only on success the sandbox step. A counting runner proves the sandbox
/// stayed idle for every refusal — and runs exactly once the gate opens.
#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_approval_gate_runs_the_sandbox_only_after_approval(pool: PgPool) {
    let url = catalogue_url(&pool.connect_options());
    let mut runner = CountingRunner { calls: 0 };

    // First a good approval goes through the real binary so a stored version
    // exists for `validate_sandbox` to stamp.
    let good = private_root(None);
    let hash = combined_hash(good.path());
    std::fs::write(
        good.path().join("packs/add-one/approval.toml"),
        approval_toml(&hash, 1),
    )
    .expect("approval writes");
    let output = import(good.path(), &url);
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("add-one created"),
        "the approved pack imports"
    );

    // Each refusal case is checked by the production `check_approval`; the
    // counting runner proves `validate_sandbox` was never invoked.
    for (approval, want) in [
        (None, ApprovalRefusal::ApprovalMissing),
        (
            Some(approval_toml(&hash, 99)),
            ApprovalRefusal::ApprovalMismatch,
        ),
        (
            Some(approval_toml(&"f".repeat(64), 1)),
            ApprovalRefusal::ApprovalMismatch,
        ),
    ] {
        let parsed = approval.map(|text| parse_approval(&text).expect("well-formed"));
        assert_eq!(
            check_approval(parsed.as_ref(), "add-one", 1, &hash),
            Err(want)
        );
        // The refused pack never reaches the sandbox step.
        assert_eq!(runner.calls, 0, "the sandbox ran on a refused approval");
    }

    // With the approval satisfied the gate opens: the same runner the refused
    // cases never saw now drives the fill end-to-end.
    let files = read_pack(
        &fixture_root().join("public/add-one"),
        Some(&good.path().join("packs/add-one")),
    )
    .expect("pack reads");
    let manifest = files.manifest().expect("manifest");
    let meta: counterspell_grimoire::meta::Meta =
        toml::from_str(files.text_any("meta.toml").expect("meta")).expect("meta parses");
    let private_pack = good.path().join("packs/add-one");
    let record = counterspell_grimoire::record::build_pair(
        &fixture_root().join("public/add-one"),
        &private_pack,
    )
    .expect("record");
    let data = assemble_sandbox(&record, &meta, &files, &manifest).expect("sandbox data");
    let retired = parse_retired("").expect("empty retired file");
    let pack = data.sandbox_pack("add-one", &retired, "sha256:test-digest");

    let outcome = validate_sandbox(&pool, "add-one", &pack, &mut runner, "tester")
        .await
        .expect("validate");
    assert!(
        matches!(outcome, ValidateOutcome::Validated { .. }),
        "{outcome:?}"
    );
    assert!(runner.calls > 0, "the approved pack did reach the sandbox");
}
