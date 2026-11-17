// SPDX-License-Identifier: MIT
//! Sandbox import writes (problem-catalogue C17, CAT-I14, CAT-S2, CAT-S3).
//!
//! `write_sandbox` stores the hidden data a `sandbox_fill` produced — the hidden
//! tests, the generation record and the private-file hashes — and stamps the
//! version in one transaction. The runner below is scripted, so these tests prove
//! the write path, not the sandbox.
//!
//! Run with `cargo test -p counterspell-judge --test pg_catalogue_sandbox_import -- --ignored`.

use std::time::Duration;

use counterspell_catalogue_cli::sandbox_fill::{
    sandbox_fill, CaseSource, FillOutcome, HiddenFill, SandboxCheck, SandboxFill,
};
use counterspell_engine::{Comparator, ProgramResult, Schema};
use counterspell_grimoire::runner::{PackRunner, RunnerError, Target};
use counterspell_judge::catalogue::retired::RetiredFile;
use counterspell_judge::catalogue::testkit::static_import as import;
use counterspell_judge::catalogue::write::{
    write_sandbox, write_static, SandboxImport, SandboxOutcome,
};
use serde_json::{json, Value};
use sqlx::PgPool;

/// A runner that returns scripted reference results in order, then times out.
struct Scripted {
    reference: std::collections::VecDeque<ProgramResult>,
}

impl PackRunner for Scripted {
    fn run(
        &mut self,
        target: Target<'_>,
        _input: &Value,
        _limit: Duration,
    ) -> Result<ProgramResult, RunnerError> {
        match target {
            Target::Reference => Ok(self
                .reference
                .pop_front()
                .unwrap_or(ProgramResult::TimedOut)),
            Target::Program(path) => panic!("unscripted program {path}"),
        }
    }

    fn bench(
        &mut self,
        _sized_inputs: &[(u64, Value)],
        _limit: Duration,
    ) -> Result<Vec<(u64, u64)>, RunnerError> {
        Err(RunnerError::Infrastructure(
            "bench is not scripted".to_owned(),
        ))
    }
}

/// Inserts the track and concept that the test packs refer to.
async fn seed_taxonomy(pool: &PgPool) {
    sqlx::query("INSERT INTO tracks (slug, name) VALUES ('arrays', 'Arrays')")
        .execute(pool)
        .await
        .expect("track");
    sqlx::query("INSERT INTO concepts (id, name) VALUES ('hash-maps', 'Hash maps')")
        .execute(pool)
        .await
        .expect("concept");
}

/// Writes one static version so a sandbox fill has a version row to stamp.
async fn seed_version(pool: &PgPool) {
    seed_taxonomy(pool).await;
    write_static(pool, &import(1, 0xaa, "Find two numbers."), "operator")
        .await
        .expect("static import");
}

/// A fill with one curated and two generated cases, plus two private sources.
fn sandbox_import() -> SandboxImport {
    SandboxImport {
        slug: "two-sum".to_string(),
        version: 1,
        fill: SandboxFill {
            hidden: vec![
                HiddenFill {
                    source: CaseSource::Curated,
                    input: json!([[1, 2], 3]),
                    expected: json!([0, 1]),
                },
                HiddenFill {
                    source: CaseSource::Generated,
                    input: json!([[4, 5], 9]),
                    expected: json!([0, 1]),
                },
                HiddenFill {
                    source: CaseSource::Generated,
                    input: json!([[7, 8], 15]),
                    expected: json!([0, 1]),
                },
            ],
            digest: "sha256:sandbox-image".to_string(),
        },
        generator_seed: 1729,
        generator_version: "gen-1".to_string(),
        sources: vec![
            ("private/two-sum/hidden.toml".to_string(), [0x11; 32]),
            ("private/two-sum/reference.py".to_string(), [0x22; 32]),
        ],
    }
}

/// `(sandbox_validated_at IS NOT NULL, sandbox_image_digest)` for the version row.
async fn stamp(pool: &PgPool) -> (bool, Option<String>) {
    sqlx::query_as(
        "SELECT sandbox_validated_at IS NOT NULL, sandbox_image_digest \
         FROM problem_versions WHERE problem_slug = 'two-sum' AND version = 1",
    )
    .fetch_one(pool)
    .await
    .expect("stamp")
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_sandbox_import_writes_hidden(pool: PgPool) {
    seed_version(&pool).await;

    let outcome = write_sandbox(&pool, &sandbox_import(), "operator")
        .await
        .expect("sandbox write");
    assert_eq!(outcome, SandboxOutcome::Stamped { version: 1 });

    // The hidden rows carry their source and continue the ordinals after the
    // two samples the static import wrote.
    let rows: Vec<(i32, String)> = sqlx::query_as(
        "SELECT ordinal, source FROM hidden_tests \
         WHERE problem_slug = 'two-sum' AND version = 1 ORDER BY ordinal",
    )
    .fetch_all(&pool)
    .await
    .expect("hidden rows");
    assert_eq!(
        rows,
        vec![
            (3, "curated".to_string()),
            (4, "generated".to_string()),
            (5, "generated".to_string()),
        ],
        "samples hold ordinals 1 and 2; the fill continues from 3"
    );

    let generation: (i64, String) = sqlx::query_as(
        "SELECT generator_seed, generator_version FROM hidden_generation \
         WHERE problem_slug = 'two-sum' AND version = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("generation row");
    assert_eq!(generation, (1729, "gen-1".to_string()));

    let sources: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM hidden_sources WHERE problem_slug = 'two-sum' AND version = 1",
    )
    .fetch_one(&pool)
    .await
    .expect("sources count");
    assert_eq!(sources, 2);

    assert_eq!(
        stamp(&pool).await,
        (true, Some("sha256:sandbox-image".to_string())),
        "the version is stamped in the same transaction"
    );

    // The event records counts only — never a hidden value (CAT-S6).
    let (action, detail): (String, Value) = sqlx::query_as(
        "SELECT action, detail FROM catalogue_events \
         WHERE problem_slug = 'two-sum' AND action = 'sandbox_validate'",
    )
    .fetch_one(&pool)
    .await
    .expect("event");
    assert_eq!(action, "sandbox_validate");
    assert_eq!(
        detail,
        json!({"hidden_curated": 1, "hidden_generated": 2, "hidden_sources": 2}),
        "counts only, no inputs or expected values"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_sandbox_import_is_one_time(pool: PgPool) {
    seed_version(&pool).await;
    write_sandbox(&pool, &sandbox_import(), "operator")
        .await
        .expect("first fill");

    let second = write_sandbox(&pool, &sandbox_import(), "operator")
        .await
        .expect("second fill");
    assert_eq!(
        second,
        SandboxOutcome::Unchanged { version: 1 },
        "a stamped version refuses a second fill"
    );

    let hidden: i64 =
        sqlx::query_scalar("SELECT count(*) FROM hidden_tests WHERE problem_slug = 'two-sum'")
            .fetch_one(&pool)
            .await
            .expect("hidden count");
    assert_eq!(hidden, 3, "no rows were added");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_sandbox_import_missing_version(pool: PgPool) {
    seed_version(&pool).await;
    let mut import = sandbox_import();
    import.version = 9;

    let outcome = write_sandbox(&pool, &import, "operator")
        .await
        .expect("missing version");
    assert_eq!(outcome, SandboxOutcome::Missing { version: 9 });

    let hidden: i64 = sqlx::query_scalar("SELECT count(*) FROM hidden_tests")
        .fetch_one(&pool)
        .await
        .expect("hidden count");
    assert_eq!(hidden, 0, "nothing was written");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_sandbox_refusal_writes_nothing(pool: PgPool) {
    seed_version(&pool).await;

    // A reference timeout on a generated input refuses the fill (SBX-02). The
    // caller then never reaches write_sandbox, and the version keeps no rows.
    let examples = [(json!([1]), json!(1))];
    let curated = [(json!([[1, 2], 3]), json!([0, 1]))];
    let retired = RetiredFile::default();
    let check = SandboxCheck {
        slug: "two-sum",
        pack_version: 1,
        comparator: Comparator::Exact,
        returns: "list[int]",
        limit: Duration::from_millis(500),
        examples: &examples,
        curated: &curated,
        seed: 1729,
        hidden_count: 3,
        schema: Schema::Int { min: 0, max: 9 },
        ghosts: &[],
        retired: &retired,
        image_digest: "sha256:sandbox-image",
    };
    // The sample and the curated case pass; the first generated input times out.
    let mut runner = Scripted {
        reference: [json!(1), json!([0, 1])]
            .into_iter()
            .map(ProgramResult::Returned)
            .collect(),
    };
    let outcome = sandbox_fill(&check, &mut runner).expect("fill ran");
    assert!(
        matches!(outcome, FillOutcome::Refused(_)),
        "a timing-out reference refuses the fill"
    );

    let hidden: i64 = sqlx::query_scalar("SELECT count(*) FROM hidden_tests")
        .fetch_one(&pool)
        .await
        .expect("hidden count");
    assert_eq!(hidden, 0, "a refused fill leaves zero hidden rows");
    assert_eq!(stamp(&pool).await, (false, None), "the stamp stays NULL");
}
