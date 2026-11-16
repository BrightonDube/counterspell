//! Database tests for the dashboard indexes (dashboard-fsrs T02, DASH-7).
//!
//! The dashboard reads completed runs by owner and finish time, and accepted runs by owner and
//! problem. The migration must add those two access paths and must be safe to run again.
//!
//! Run with `DATABASE_URL=<throwaway postgres 17> cargo test -p counterspell-server --test pg_dashboard_indexes -- --ignored pg_`.

use sqlx::PgPool;

const MIGRATION: &str = include_str!("../../../migrations/20261115140000_dashboard_indexes.sql");

/// Returns the names of every index on `submissions`, sorted.
async fn submission_index_names(pool: &PgPool) -> sqlx::Result<Vec<String>> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT indexname FROM pg_indexes WHERE tablename = 'submissions' ORDER BY indexname",
    )
    .fetch_all(pool)
    .await?;
    Ok(rows.into_iter().map(|(name,)| name).collect())
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_dashboard_indexes_exist_after_migrations(pool: PgPool) {
    let names = submission_index_names(&pool)
        .await
        .expect("list submission indexes");
    assert!(
        names.contains(&"submissions_dashboard_user_finished".to_owned()),
        "missing completed-run index, have {names:?}"
    );
    assert!(
        names.contains(&"submissions_dashboard_user_accepted".to_owned()),
        "missing accepted-run index, have {names:?}"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_dashboard_migration_runs_again_without_error_or_change(pool: PgPool) {
    let before = submission_index_names(&pool).await.expect("before");
    sqlx::raw_sql(MIGRATION)
        .execute(&pool)
        .await
        .expect("second run of the migration");
    sqlx::raw_sql(MIGRATION)
        .execute(&pool)
        .await
        .expect("third run of the migration");
    let after = submission_index_names(&pool).await.expect("after");
    assert_eq!(
        after, before,
        "re-running the migration must not add or drop indexes"
    );
}
