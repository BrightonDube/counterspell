//! `PgProblemTests` reads against real Postgres (problem-catalogue C26, X1).
//!
//! The loader reads the pinned version through the judge's `SELECT`-only path:
//! Run sees examples only, Submit adds hidden rows, the reference is joined in,
//! and a version the store does not hold — including version 0, the unpinned
//! marker the worker sends — comes back `None`, which the executor reports as
//! `problem_unavailable`.
//!
//! Run with `cargo test -p counterspell-judge --test pg_problem_tests -- --ignored`.

use counterspell_common::status::RunMode;
use counterspell_judge::catalogue::testkit::insert_published_fixture;
use counterspell_judge::problem_store::PgProblemTests;
use counterspell_judge::sandbox::problem::ProblemTests;
use sqlx::PgPool;

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_problem_tests_run_has_examples_no_hidden(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("fixture");
    let tests = PgProblemTests::new(pool);

    let spec = tests
        .load("two-sum", 1, RunMode::Run)
        .await
        .expect("run load")
        .expect("the pinned version exists");

    assert_eq!(spec.slug, "two-sum");
    assert_eq!(spec.version, 1);
    assert_eq!(spec.tests.len(), 2, "Run sees the two public examples");
    assert!(spec.tests.iter().all(|case| case.is_sample));
    assert!(
        spec.reference_python.is_some(),
        "the reference source is joined for every mode"
    );
    assert_eq!(
        spec.signature["protocol_version"], 1,
        "the stored signature is the driver-protocol descriptor"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_problem_tests_submit_includes_hidden(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("fixture");
    let tests = PgProblemTests::new(pool);

    let spec = tests
        .load("two-sum", 1, RunMode::Submit)
        .await
        .expect("submit load")
        .expect("the pinned version exists");

    let samples = spec.tests.iter().filter(|case| case.is_sample).count();
    let hidden = spec.tests.iter().filter(|case| !case.is_sample).count();
    assert_eq!(
        (samples, hidden),
        (2, 5),
        "Submit sees samples then hidden rows"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_problem_tests_unpinned_and_missing_versions_are_none(pool: PgPool) {
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("fixture");
    let tests = PgProblemTests::new(pool);

    // The worker maps a submission with no pinned version to 0; no store can
    // satisfy it, so the run reports problem_unavailable rather than guessing.
    for version in [0, 2] {
        assert!(
            tests
                .load("two-sum", version, RunMode::Submit)
                .await
                .expect("an absent version is not an error")
                .is_none(),
            "version {version} is absent"
        );
    }
}
