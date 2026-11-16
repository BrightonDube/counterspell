// SPDX-License-Identifier: MIT
//! `validate-sandbox` runs (problem-catalogue C18, CAT-I16, CAT-PT10).
//!
//! `validate_sandbox` gates on the latest stored version, runs the C16 fill
//! through a scripted `PackRunner` — not sandbox evidence — and writes through
//! the C17 transaction. A run whose recomputed hash disagrees with the stored
//! `content_sha256` refuses `private_tree_changed` before the sandbox runs.
//!
//! Run with `cargo test -p counterspell-judge --test pg_catalogue_validate_sandbox -- --ignored`.

use std::collections::VecDeque;
use std::time::Duration;

use counterspell_engine::{Comparator, ProgramResult, Schema};
use counterspell_grimoire::runner::{PackRunner, RunnerError, Target};
use counterspell_judge::catalogue::retired::RetiredFile;
use counterspell_judge::catalogue::sandbox_fill::{
    validate_sandbox, FillRefusal, SandboxCheck, SandboxPack, ValidateOutcome,
};
use counterspell_judge::catalogue::testkit::static_import as import;
use counterspell_judge::catalogue::write::write_static;
use serde_json::{json, Value};
use sqlx::PgPool;

/// A runner that returns scripted reference results in order, then times out.
struct Scripted {
    reference: VecDeque<ProgramResult>,
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

/// Inserts the track, the concept and one static version of `two-sum`.
async fn seed_version(pool: &PgPool) {
    sqlx::query("INSERT INTO tracks (slug, name) VALUES ('arrays', 'Arrays')")
        .execute(pool)
        .await
        .expect("track");
    sqlx::query("INSERT INTO concepts (id, name) VALUES ('hash-maps', 'Hash maps')")
        .execute(pool)
        .await
        .expect("concept");
    write_static(pool, &import(1, 0xaa, "Find two numbers."), "operator")
        .await
        .expect("static import");
}

/// A pack snapshot whose hash matches the stored version. `examples` and
/// `curated` stay owned by the caller so the check can borrow them.
fn pack<'a>(
    examples: &'a [(Value, Value)],
    curated: &'a [(Value, Value)],
    retired: &'a RetiredFile,
    content_sha256: [u8; 32],
) -> SandboxPack<'a> {
    SandboxPack {
        content_sha256,
        check: SandboxCheck {
            slug: "two-sum",
            pack_version: 1,
            comparator: Comparator::Exact,
            returns: "int",
            limit: Duration::from_millis(500),
            examples,
            curated,
            seed: 1729,
            hidden_count: 2,
            schema: Schema::Int { min: 0, max: 9 },
            ghosts: &[],
            retired,
            image_digest: "sha256:sandbox-image",
        },
        generator_version: "gen-1".to_string(),
        sources: vec![("private/two-sum/hidden.toml".to_string(), [0x11; 32])],
    }
}

/// A runner that agrees with `expected` on every call, for however many calls
/// the fill makes (examples + curated + generated).
fn agreeing(expected: &[(Value, Value)], generated: usize) -> Scripted {
    Scripted {
        reference: examples_plus_curated(expected, generated),
    }
}

/// `expected` results followed by `generated` well-shaped `int` returns.
fn examples_plus_curated(expected: &[(Value, Value)], generated: usize) -> VecDeque<ProgramResult> {
    expected
        .iter()
        .map(|(_, value)| value.clone())
        .chain((0..generated).map(|index| json!(index)))
        .map(ProgramResult::Returned)
        .collect()
}

/// `(sandbox_validated_at IS NOT NULL, hidden row count)` for version 1.
async fn stamped_and_hidden(pool: &PgPool) -> (bool, i64) {
    sqlx::query_as(
        "SELECT sandbox_validated_at IS NOT NULL, \
         (SELECT count(*) FROM hidden_tests WHERE problem_slug = 'two-sum' AND version = 1) \
         FROM problem_versions WHERE problem_slug = 'two-sum' AND version = 1",
    )
    .fetch_one(pool)
    .await
    .expect("stamp")
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_validate_sandbox_stamps_and_publishes(pool: PgPool) {
    seed_version(&pool).await;
    // The fixture carries two samples; one curated case joins them before the
    // two generated candidates.
    let examples = [(json!([1]), json!(1)), (json!([2]), json!(2))];
    let curated = [(json!(7), json!(7))];
    let retired = RetiredFile::default();
    let mut runner = agreeing(
        &examples.iter().chain(&curated).cloned().collect::<Vec<_>>(),
        2,
    );

    let outcome = validate_sandbox(
        &pool,
        "two-sum",
        &pack(&examples, &curated, &retired, [0xaa; 32]),
        &mut runner,
        "operator",
    )
    .await
    .expect("validate ran");
    assert_eq!(outcome, ValidateOutcome::Validated { version: 1 });
    assert_eq!(
        stamped_and_hidden(&pool).await,
        (true, 3),
        "one curated + two generated cases, stamped in one transaction"
    );

    // The publish gate sees a complete version: stamped, 2-5 examples, four
    // hints and a concept tag — the update succeeds.
    sqlx::query(
        "UPDATE problems SET publication_state = 'published', published_version = 1, \
         published_at = now() WHERE slug = 'two-sum'",
    )
    .execute(&pool)
    .await
    .expect("the stamped version passes the publish gate");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_validate_sandbox_refuses_a_second_run(pool: PgPool) {
    seed_version(&pool).await;
    let examples = [(json!([1]), json!(1))];
    let curated = [(json!(7), json!(7))];
    let retired = RetiredFile::default();
    let mut runner = agreeing(
        &examples.iter().chain(&curated).cloned().collect::<Vec<_>>(),
        2,
    );
    validate_sandbox(
        &pool,
        "two-sum",
        &pack(&examples, &curated, &retired, [0xaa; 32]),
        &mut runner,
        "operator",
    )
    .await
    .expect("first validate");

    let mut unused = Scripted {
        reference: VecDeque::new(),
    };
    let second = validate_sandbox(
        &pool,
        "two-sum",
        &pack(&examples, &curated, &retired, [0xaa; 32]),
        &mut unused,
        "operator",
    )
    .await
    .expect("second validate");
    assert_eq!(
        second,
        ValidateOutcome::AlreadyValidated { version: 1 },
        "the gate refuses before the runner is called"
    );
    assert_eq!(stamped_and_hidden(&pool).await.1, 3, "no rows were added");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_validate_sandbox_refuses_a_changed_tree(pool: PgPool) {
    seed_version(&pool).await;
    let examples = [(json!([1]), json!(1))];
    let retired = RetiredFile::default();
    // The stored hash is 0xaa; a private tree that recomputes to 0xbb no longer
    // matches the imported content (CAT-PT10) and is refused before any run.
    let mut unused = Scripted {
        reference: VecDeque::new(),
    };
    let outcome = validate_sandbox(
        &pool,
        "two-sum",
        &pack(&examples, &[], &retired, [0xbb; 32]),
        &mut unused,
        "operator",
    )
    .await
    .expect("gate ran");
    assert_eq!(outcome, ValidateOutcome::PrivateTreeChanged { version: 1 });
    assert_eq!(
        stamped_and_hidden(&pool).await,
        (false, 0),
        "nothing was written"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_validate_sandbox_refusal_writes_nothing(pool: PgPool) {
    seed_version(&pool).await;
    let examples = [(json!([1]), json!(1))];
    let retired = RetiredFile::default();
    // The sample passes; the first generated input times out (SBX-02).
    let mut runner = Scripted {
        reference: [json!(1)]
            .into_iter()
            .map(ProgramResult::Returned)
            .collect(),
    };
    let outcome = validate_sandbox(
        &pool,
        "two-sum",
        &pack(&examples, &[], &retired, [0xaa; 32]),
        &mut runner,
        "operator",
    )
    .await
    .expect("validate ran");
    assert_eq!(
        outcome,
        ValidateOutcome::Refused(FillRefusal::ReferenceFailed {
            source: counterspell_judge::catalogue::sandbox_fill::CaseSource::Generated,
            index: 0,
        }),
        "the fill refusal reaches the report, not a panic"
    );
    assert_eq!(
        stamped_and_hidden(&pool).await,
        (false, 0),
        "a refused fill leaves zero hidden rows and no stamp"
    );
}
