//! The private tree is not optional (problem-catalogue C19b, CAT-PT2, CAT-PT3,
//! CAT-PT15, CAT-I11).
//!
//! A run with no `--private-root` exits 2 and leaves every catalogue table at
//! zero rows — the preflight fires before any database connection. A run whose
//! private root lacks one slug's part refuses that slug and imports the others;
//! the refused slug writes nothing.
//!
//! Fixtures live under `tests/fixtures/c19b/` and carry canary strings only.
//!
//! Run with
//! `cargo test -p counterspell-judge --test pg_catalogue_private_tree_required -- --ignored`.

use std::path::{Path, PathBuf};
use std::process::Command;

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

/// Spawns the binary with `CATALOGUE_DATABASE_URL` pointing at the test pool.
#[allow(clippy::disallowed_methods)] // a test harness may spawn the CLI under test
fn import(args: &[&str], url: &str) -> std::process::Output {
    Command::new(BIN)
        .args(args)
        .env("CATALOGUE_DATABASE_URL", url)
        .env_remove("DATABASE_URL")
        .output()
        .expect("binary runs")
}

/// Copies the fixture private tree into a tempdir and writes `approval.toml`
/// for `add-one` with the real combined manifest hash.
fn private_root_with_approval() -> tempfile::TempDir {
    let root = tempfile::tempdir().expect("tempdir");
    let source = fixture_root().join("private/packs/add-one");
    let target = root.path().join("packs/add-one");
    copy_dir(&source, &target);
    let files = counterspell_catalogue_cli::load::read_pack(
        &fixture_root().join("public/add-one"),
        Some(&target),
    )
    .expect("pack reads");
    let manifest = files.manifest().expect("manifest");
    std::fs::write(
        target.join("approval.toml"),
        approval_toml(&manifest.content_sha256_hex()),
    )
    .expect("approval writes");
    root
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

/// A well-formed `approval.toml` naming `add-one` version 1 and `hash_hex`.
fn approval_toml(hash_hex: &str) -> String {
    format!(
        "slug = \"add-one\"\n\
         pack_version = 1\n\
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

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_import_without_private_root_exits_2_and_writes_nothing(pool: PgPool) {
    let packs = fixture_root().join("public");
    let output = import(
        &[
            "import",
            "--static-only",
            "--packs",
            packs.to_str().unwrap(),
        ],
        &catalogue_url(&pool.connect_options()),
    );

    assert_eq!(output.status.code(), Some(2), "--private-root is required");
    assert_eq!(
        counts(&pool).await,
        (0, 0, 0),
        "the preflight refuses before any database write"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_missing_private_part_refuses_one_slug_and_imports_the_other(pool: PgPool) {
    let private = private_root_with_approval();
    let packs = fixture_root().join("public");
    let output = import(
        &[
            "import",
            "--static-only",
            "--packs",
            packs.to_str().unwrap(),
            "--private-root",
            private.path().to_str().unwrap(),
        ],
        &catalogue_url(&pool.connect_options()),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(
        stdout.contains("lonesome refused:private_part_missing"),
        "the slug without a private part is refused: {stdout}"
    );
    assert!(
        stdout.contains("add-one created"),
        "the complete pack still imports: {stdout}"
    );
    assert_eq!(counts(&pool).await, (1, 1, 1));
    let lonesome_rows: i64 =
        sqlx::query_scalar("SELECT count(*) FROM catalogue_events WHERE problem_slug = 'lonesome'")
            .fetch_one(&pool)
            .await
            .expect("count");
    assert_eq!(lonesome_rows, 0, "the refused slug writes nothing");
}
