//! Publish/unpublish transitions through `catalogue::publish` (C21, CAT-P1..P5).
//!
//! The CLI is a thin shell over these functions; the database trigger is the
//! authority on the gate, so the tests assert the CLI-facing API and the
//! trigger agree: what publishes through `publish` publishes through a bare
//! UPDATE, and refusals carry the trigger's own check text.
//!
//! Run with `cargo test -p counterspell-judge --test pg_catalogue_publish -- --ignored`.

mod catalogue_support;

use catalogue_support as s;
use counterspell_judge::catalogue::publish::{publish, unpublish};
use counterspell_judge::catalogue::publish::{PublishOutcome, UnpublishOutcome};
use sqlx::{PgPool, Row};

/// Builds `two-sum` with `version` carrying 2 examples, 4 hints, one concept
/// tag and optionally a sandbox stamp — the full gate surface.
async fn build(pool: &PgPool, version: i32, sandboxed: bool) {
    s::track(pool, "arrays").await.expect("track");
    s::problem(pool, "two-sum", "arrays")
        .await
        .expect("problem");
    s::version(pool, "two-sum", version).await.expect("version");
    s::tag_concept(pool, "two-sum", "hash-maps")
        .await
        .expect("concept");
    s::examples(pool, "two-sum", version, 2)
        .await
        .expect("examples");
    s::hints(pool, "two-sum", version, 4).await.expect("hints");
    if sandboxed {
        s::hidden_test(pool, "two-sum", version, 1)
            .await
            .expect("hidden");
        s::sandbox_fill(pool, "two-sum", version)
            .await
            .expect("fill");
    }
}

async fn problem_row(pool: &PgPool, slug: &str) -> (String, Option<i32>) {
    sqlx::query("SELECT publication_state, published_version FROM problems WHERE slug = $1")
        .bind(slug)
        .fetch_one(pool)
        .await
        .map(|row| (row.get(0), row.get(1)))
        .expect("problem row")
}

async fn event_count(pool: &PgPool, slug: &str, action: &str) -> i64 {
    sqlx::query_scalar(
        "SELECT count(*) FROM catalogue_events WHERE problem_slug = $1 AND action = $2",
    )
    .bind(slug)
    .bind(action)
    .fetch_one(pool)
    .await
    .expect("event count")
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_publish_defaults_to_latest_validated(pool: PgPool) {
    build(&pool, 1, true).await;
    s::version(&pool, "two-sum", 2).await.expect("version 2");
    s::examples(&pool, "two-sum", 2, 2).await.expect("examples");
    s::hints(&pool, "two-sum", 2, 4).await.expect("hints");
    s::hidden_test(&pool, "two-sum", 2, 1)
        .await
        .expect("hidden");
    s::sandbox_fill(&pool, "two-sum", 2).await.expect("fill");

    let outcome = publish(&pool, "two-sum", None, "tester")
        .await
        .expect("publish");
    assert_eq!(outcome, PublishOutcome::Published { version: 2 });
    assert_eq!(
        problem_row(&pool, "two-sum").await,
        ("published".to_string(), Some(2))
    );
    assert_eq!(event_count(&pool, "two-sum", "publish").await, 1);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_publish_explicit_version_and_trigger_agree(pool: PgPool) {
    build(&pool, 1, true).await;

    let outcome = publish(&pool, "two-sum", Some(1), "tester")
        .await
        .expect("publish");
    assert_eq!(outcome, PublishOutcome::Published { version: 1 });
    // CLI and trigger agree: the same UPDATE the helper just ran is what the
    // raw statement asserts in pg_catalogue_publish_gate.
    assert_eq!(
        problem_row(&pool, "two-sum").await,
        ("published".to_string(), Some(1))
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_publish_reports_the_failed_check_text(pool: PgPool) {
    build(&pool, 1, false).await;

    let outcome = publish(&pool, "two-sum", Some(1), "tester")
        .await
        .expect("publish");
    let PublishOutcome::GateRefused { check } = outcome else {
        panic!("an unvalidated version must hit the gate, got {outcome:?}");
    };
    assert!(
        check.contains("sandbox-validated"),
        "the trigger's own text is reported: {check}"
    );
    assert_eq!(event_count(&pool, "two-sum", "publish").await, 0);
    assert_eq!(
        problem_row(&pool, "two-sum").await,
        ("unpublished".to_string(), None),
        "a refused publish writes nothing"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_publish_without_validated_version_or_slug_refuses(pool: PgPool) {
    build(&pool, 1, false).await;

    assert_eq!(
        publish(&pool, "two-sum", None, "tester")
            .await
            .expect("publish"),
        PublishOutcome::Refused {
            code: "no_validated_version"
        }
    );
    assert_eq!(
        publish(&pool, "ghost-problem", None, "tester")
            .await
            .expect("publish"),
        PublishOutcome::Refused {
            code: "no_validated_version"
        }
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_unpublish_keeps_history_and_is_idempotent(pool: PgPool) {
    build(&pool, 1, true).await;
    publish(&pool, "two-sum", Some(1), "tester")
        .await
        .expect("publish");

    assert_eq!(
        unpublish(&pool, "two-sum", "tester")
            .await
            .expect("unpublish"),
        UnpublishOutcome::Unpublished
    );
    let (state, pinned) = problem_row(&pool, "two-sum").await;
    assert_eq!(state, "unpublished");
    assert_eq!(pinned, Some(1), "published_version is kept as history");
    let row = sqlx::query(
        "SELECT unpublished_at IS NOT NULL, \
         (SELECT count(*) FROM problem_versions WHERE problem_slug = 'two-sum') \
         FROM problems WHERE slug = 'two-sum'",
    )
    .fetch_one(&pool)
    .await
    .expect("row");
    assert!(row.get::<bool, _>(0), "unpublished_at is stamped");
    assert_eq!(row.get::<i64, _>(1), 1, "the version row is untouched");
    assert_eq!(event_count(&pool, "two-sum", "unpublish").await, 1);

    // A second unpublish is a no-op: no state change and no new event.
    assert_eq!(
        unpublish(&pool, "two-sum", "tester")
            .await
            .expect("unpublish"),
        UnpublishOutcome::Refused {
            code: "already_unpublished"
        }
    );
    assert_eq!(event_count(&pool, "two-sum", "unpublish").await, 1);
    assert_eq!(
        unpublish(&pool, "ghost-problem", "tester")
            .await
            .expect("unpublish"),
        UnpublishOutcome::Refused {
            code: "not_in_catalogue"
        }
    );
}
