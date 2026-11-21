//! Database tests for the enqueue pin (problem-catalogue C29, requirements CAT-L3, CAT-L5 and P9).
//!
//! The submission row takes its version from the published problem in the same statement that
//! inserts it. A run queued before an unpublish keeps its version. An unpublished problem queues
//! nothing.
//!
//! Run with `DATABASE_URL=<throwaway postgres 17> cargo test -p counterspell-server --test pg_catalogue_pin -- --ignored pg_`.

use counterspell_common::{
    ids::{RunId, UserId},
    status::RunMode,
};
use counterspell_judge::catalogue::testkit::{insert_published_fixture, insert_static_fixture};
use counterspell_server::{
    config::RunLimits,
    runs::enqueue::{enqueue, EnqueueOutcome, NewRun},
    testkit::ensure_user,
};
use sqlx::{postgres::PgPoolOptions, PgPool};

fn limits() -> RunLimits {
    RunLimits {
        run_rate_user: 30,
        run_rate_ip: 100,
        run_active_cap: 3,
        queue_depth_max: 200,
        max_source_bytes: 65_536,
        request_body_limit_bytes: 131_072,
    }
}

fn new_run(user: UserId, key: Option<&str>) -> NewRun {
    NewRun::new(
        RunId::new(),
        user,
        "two-sum",
        "python",
        RunMode::Submit,
        "print(1)\n",
        key.map(str::to_owned),
    )
}

async fn user_in(pool: &PgPool) -> UserId {
    let user = UserId::new();
    ensure_user(pool, user).await.expect("user fixture");
    user
}

async fn count(pool: &PgPool, sql: &str, user: UserId) -> i64 {
    sqlx::query_scalar(sql)
        .bind(user.as_uuid())
        .fetch_one(pool)
        .await
        .expect("count query")
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_pin_keeps_the_version_after_unpublish(pool: PgPool) {
    let user = user_in(&pool).await;
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("published fixture");
    let server_pool = PgPoolOptions::new()
        .max_connections(1)
        .after_connect(|connection, _| {
            Box::pin(async move {
                sqlx::query("SET ROLE counterspell_server")
                    .execute(connection)
                    .await?;
                Ok(())
            })
        })
        .connect_with(pool.connect_options().as_ref().clone())
        .await
        .expect("restricted server pool");
    let run = new_run(user, None);
    let outcome = enqueue(&server_pool, &run, &limits())
        .await
        .expect("enqueue as the server role");
    assert_eq!(outcome, EnqueueOutcome::Created { run_id: run.run_id });

    sqlx::query(
        "UPDATE problems SET publication_state = 'unpublished', unpublished_at = now() WHERE slug = 'two-sum'",
    )
    .execute(&pool)
    .await
    .expect("unpublish");

    let (version,): (Option<i32>,) =
        sqlx::query_as("SELECT problem_version FROM submissions WHERE id = $1")
            .bind(run.run_id.as_uuid())
            .fetch_one(&pool)
            .await
            .expect("submission row");
    assert_eq!(version, Some(1));
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_pin_refuses_an_unpublished_problem_and_queues_nothing(pool: PgPool) {
    let user = user_in(&pool).await;
    insert_static_fixture(&pool, "two-sum", 1)
        .await
        .expect("static fixture");
    let run = new_run(user, None);

    let outcome = enqueue(&pool, &run, &limits()).await.expect("enqueue");

    assert_eq!(outcome, EnqueueOutcome::NotPublished);
    let submissions = count(
        &pool,
        "SELECT count(*) FROM submissions WHERE user_id = $1",
        user,
    )
    .await;
    let jobs = count(
        &pool,
        "SELECT count(*) FROM jobs j JOIN submissions s ON s.id = j.submission_id WHERE s.user_id = $1",
        user,
    )
    .await;
    assert_eq!((submissions, jobs), (0, 0));
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_catalogue_pin_replay_still_returns_the_same_run(pool: PgPool) {
    let user = user_in(&pool).await;
    insert_published_fixture(&pool, "two-sum", 1)
        .await
        .expect("published fixture");
    let first = new_run(user, Some("pin-key"));
    let again = new_run(user, Some("pin-key"));
    enqueue(&pool, &first, &limits())
        .await
        .expect("first enqueue");

    let replay = enqueue(&pool, &again, &limits()).await.expect("replay");

    assert_eq!(
        replay,
        EnqueueOutcome::Replayed {
            run_id: first.run_id
        }
    );
}
