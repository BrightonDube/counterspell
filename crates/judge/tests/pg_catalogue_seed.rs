//! `seed-dev`/`seed-demo` through the real binary (C22, CAT-D1..D3).
//!
//! With every shipped pack still `status = "draft"`, both seeds find no
//! eligible packs, import nothing, write nothing and exit 0 — the seeds reach
//! their "no eligible packs" answer before demanding a database write or a
//! sandbox runner. `list` prints the catalogue's slug, state and version.
//!
//! `seed-demo`'s publish step is exercised by `pg_catalogue_publish`; it is
//! not run here because it needs a sandbox runner (the spec keeps it out of
//! CI).
//!
//! Run with `cargo test -p counterspell-judge --test pg_catalogue_seed -- --ignored`.

use std::path::PathBuf;
use std::process::Command;

use sqlx::postgres::PgConnectOptions;

/// The compiled binary under test.
const BIN: &str = env!("CARGO_BIN_EXE_counterspell-catalogue");

/// The repository's shipped `packs/` tree.
fn shipped_packs() -> String {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../packs")
        .canonicalize()
        .expect("packs dir")
        .to_str()
        .expect("utf-8")
        .to_string()
}

/// The pool's URL as `CATALOGUE_DATABASE_URL` (the value is never printed by
/// the binary, but the child still needs it to connect).
fn catalogue_url(options: &PgConnectOptions) -> String {
    format!(
        "postgres://{}@{}:{}/{}",
        options.get_username(),
        options.get_host(),
        options.get_port(),
        options.get_database().expect("a test database name")
    )
}

/// Spawns the binary with an isolated environment.
#[allow(clippy::disallowed_methods)] // a test harness may spawn the CLI under test
fn run(args: &[&str], url: Option<String>) -> std::process::Output {
    let mut command = Command::new(BIN);
    command
        .args(args)
        .env_remove("CATALOGUE_DATABASE_URL")
        .env_remove("DATABASE_URL");
    if let Some(url) = url {
        command.env("CATALOGUE_DATABASE_URL", url);
    }
    command.output().expect("binary runs")
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_seed_dev_static_only_imports_no_drafts(pool: sqlx::PgPool) {
    let private = tempfile::tempdir().unwrap();
    let output = run(
        &[
            "seed-dev",
            "--static-only",
            "--packs",
            &shipped_packs(),
            "--private-root",
            private.path().to_str().unwrap(),
        ],
        Some(catalogue_url(&pool.connect_options())),
    );
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("no eligible packs"),
        "all three starter packs are draft, so nothing is eligible: {stdout}"
    );
    let problems: i64 = sqlx::query_scalar("SELECT count(*) FROM problems")
        .fetch_one(&pool)
        .await
        .expect("count");
    assert_eq!(problems, 0, "a seed over drafts writes no problems");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_seed_demo_imports_no_drafts(pool: sqlx::PgPool) {
    let private = tempfile::tempdir().unwrap();
    let output = run(
        &[
            "seed-demo",
            "--packs",
            &shipped_packs(),
            "--private-root",
            private.path().to_str().unwrap(),
        ],
        Some(catalogue_url(&pool.connect_options())),
    );
    assert_eq!(
        output.status.code(),
        Some(0),
        "drafts make the seed a no-op before the sandbox runner is even demanded: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("no eligible packs"));
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_seed_requires_private_root(_pool: sqlx::PgPool) {
    let output = run(&["seed-dev", "--static-only"], None);
    assert_eq!(output.status.code(), Some(2), "--private-root is required");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_list_prints_slug_state_and_version(pool: sqlx::PgPool) {
    sqlx::query("INSERT INTO tracks (slug, name) VALUES ('arrays', 'Arrays')")
        .execute(&pool)
        .await
        .expect("track");
    sqlx::query(
        "INSERT INTO problems (slug, title, track_slug, difficulty) \
         VALUES ('count-pairs', 'Count Pairs', 'arrays', 'easy'), \
                ('two-sum', 'Two Sum', 'arrays', 'easy')",
    )
    .execute(&pool)
    .await
    .expect("problems");
    sqlx::query(
        "INSERT INTO problem_versions (problem_slug, version, content_sha256, manifest, title, \
         track_slug, difficulty, statement_markdown, constraints, signature, input_schema, \
         comparator, limits, engine_version, generator_version, hidden_count) \
         VALUES ('count-pairs', 3, decode(repeat('07', 32), 'hex'), '{}', 'Count Pairs', \
         'arrays', 'easy', 'S', '[]', '{}', '{}', 'exact', '{}', 'e', 'g', 5)",
    )
    .execute(&pool)
    .await
    .expect("version");
    // Unpublished but with a kept published_version: the post-unpublish history
    // shape (CAT-P5), so `list` must show the version column.
    sqlx::query("UPDATE problems SET published_version = 3 WHERE slug = 'count-pairs'")
        .execute(&pool)
        .await
        .expect("pin history");

    let output = run(&["list"], Some(catalogue_url(&pool.connect_options())));
    assert_eq!(output.status.code(), Some(0));
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("count-pairs unpublished 3"),
        "got: {stdout}"
    );
    assert!(stdout.contains("two-sum unpublished -"), "got: {stdout}");
}
