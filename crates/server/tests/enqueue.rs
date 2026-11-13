//! Database tests for the enqueue transaction (run-lifecycle T24, design Path 1 step 9).
//!
//! A run and its queue ticket are saved together or not at all. The per-user caps are checked under a
//! lock, so parallel requests from one user cannot slip past them.
//!
//! Run with `DATABASE_URL=<throwaway postgres 17> cargo test -p counterspell-server --test enqueue -- --ignored pg_`.

use std::time::Duration;

use counterspell_common::{
    ids::{RunId, UserId},
    status::RunMode,
};
use counterspell_server::{
    config::RunLimits,
    runs::enqueue::{enqueue, EnqueueOutcome, NewRun},
};
use sqlx::{postgres::PgListener, PgPool};
use tokio::task::JoinSet;

fn limits(active_cap: u64, queue_depth_max: u64) -> RunLimits {
    RunLimits {
        run_rate_user: 30,
        run_rate_ip: 100,
        run_active_cap: active_cap,
        queue_depth_max,
        max_source_bytes: 65_536,
        request_body_limit_bytes: 131_072,
    }
}

fn new_run(user: UserId, key: Option<&str>, code: &str) -> NewRun {
    NewRun::new(
        RunId::new(),
        user,
        "two-sum",
        "python",
        RunMode::Submit,
        code,
        key.map(str::to_owned),
    )
}

async fn submissions_for(pool: &PgPool, user: UserId) -> i64 {
    sqlx::query_scalar("SELECT count(*) FROM submissions WHERE user_id = $1")
        .bind(user.as_uuid())
        .fetch_one(pool)
        .await
        .unwrap()
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_enqueue_creates_the_run_and_its_job_together(pool: PgPool) {
    let user = UserId::new();
    let run = new_run(user, Some("first"), "print(1)\n");

    let outcome = enqueue(&pool, &run, &limits(3, 200)).await.unwrap();

    assert_eq!(outcome, EnqueueOutcome::Created { run_id: run.run_id });
    let (status, job_state): (String, String) = sqlx::query_as(
        "SELECT s.status, j.state FROM submissions s JOIN jobs j ON j.submission_id = s.id WHERE s.id = $1",
    )
    .bind(run.run_id.as_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    assert_eq!((status.as_str(), job_state.as_str()), ("queued", "queued"));
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_enqueue_replays_the_same_key_and_content(pool: PgPool) {
    let user = UserId::new();
    let first = new_run(user, Some("same-key"), "print(1)\n");
    enqueue(&pool, &first, &limits(3, 200)).await.unwrap();

    // A retry gets a new run id from the client side, but the same key and content.
    let retry = new_run(user, Some("same-key"), "print(1)\n");
    let outcome = enqueue(&pool, &retry, &limits(3, 200)).await.unwrap();

    assert_eq!(
        outcome,
        EnqueueOutcome::Replayed {
            run_id: first.run_id
        }
    );
    assert_eq!(
        submissions_for(&pool, user).await,
        1,
        "a replay must not add a run"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_enqueue_refuses_a_reused_key_with_different_code(pool: PgPool) {
    let user = UserId::new();
    enqueue(
        &pool,
        &new_run(user, Some("reused"), "print(1)\n"),
        &limits(3, 200),
    )
    .await
    .unwrap();

    let outcome = enqueue(
        &pool,
        &new_run(user, Some("reused"), "print(2)\n"),
        &limits(3, 200),
    )
    .await
    .unwrap();

    assert_eq!(outcome, EnqueueOutcome::Reuse);
    assert_eq!(submissions_for(&pool, user).await, 1);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_enqueue_refuses_past_the_active_cap(pool: PgPool) {
    let user = UserId::new();
    let caps = limits(2, 200);

    let a = enqueue(&pool, &new_run(user, Some("a"), "x = 1\n"), &caps)
        .await
        .unwrap();
    let b = enqueue(&pool, &new_run(user, Some("b"), "x = 2\n"), &caps)
        .await
        .unwrap();
    let c = enqueue(&pool, &new_run(user, Some("c"), "x = 3\n"), &caps)
        .await
        .unwrap();

    assert!(matches!(a, EnqueueOutcome::Created { .. }));
    assert!(matches!(b, EnqueueOutcome::Created { .. }));
    assert_eq!(c, EnqueueOutcome::CapReached);
    assert_eq!(submissions_for(&pool, user).await, 2);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_finished_runs_do_not_count_toward_the_cap(pool: PgPool) {
    let user = UserId::new();
    let caps = limits(1, 200);
    let first = new_run(user, Some("old"), "x = 1\n");
    enqueue(&pool, &first, &caps).await.unwrap();
    sqlx::query(
        "UPDATE submissions SET status = 'failed', failure_reason = 'cancelled', finished_at = now() WHERE id = $1",
    )
    .bind(first.run_id.as_uuid())
    .execute(&pool)
    .await
    .unwrap();
    sqlx::query("UPDATE jobs SET state = 'failed' WHERE submission_id = $1")
        .bind(first.run_id.as_uuid())
        .execute(&pool)
        .await
        .unwrap();

    let outcome = enqueue(&pool, &new_run(user, Some("new"), "x = 2\n"), &caps)
        .await
        .unwrap();

    assert!(
        matches!(outcome, EnqueueOutcome::Created { .. }),
        "{outcome:?}"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_enqueue_refuses_when_the_whole_queue_is_full(pool: PgPool) {
    let caps = limits(3, 1);
    let first_user = UserId::new();
    enqueue(&pool, &new_run(first_user, Some("one"), "x = 1\n"), &caps)
        .await
        .unwrap();

    let second_user = UserId::new();
    let outcome = enqueue(&pool, &new_run(second_user, Some("two"), "x = 1\n"), &caps)
        .await
        .unwrap();

    assert_eq!(outcome, EnqueueOutcome::QueueFull);
    assert_eq!(submissions_for(&pool, second_user).await, 0);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_enqueue_notifies_the_judge(pool: PgPool) {
    let mut listener = PgListener::connect_with(&pool).await.unwrap();
    listener.listen("run_jobs").await.unwrap();

    enqueue(
        &pool,
        &new_run(UserId::new(), None, "x = 1\n"),
        &limits(3, 200),
    )
    .await
    .unwrap();

    let received = tokio::time::timeout(Duration::from_secs(5), listener.recv()).await;
    assert!(
        matches!(received, Ok(Ok(_))),
        "no run_jobs notification arrived"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_parallel_enqueues_by_one_user_respect_the_cap(pool: PgPool) {
    let user = UserId::new();
    let caps = limits(3, 200);
    let mut tasks = JoinSet::new();
    for index in 0..8 {
        let pool = pool.clone();
        let caps = caps.clone();
        let run = new_run(user, Some(&format!("parallel-{index}")), "x = 1\n");
        tasks.spawn(async move { enqueue(&pool, &run, &caps).await.unwrap() });
    }

    let mut created = 0;
    let mut refused = 0;
    while let Some(result) = tasks.join_next().await {
        match result.unwrap() {
            EnqueueOutcome::Created { .. } => created += 1,
            EnqueueOutcome::CapReached => refused += 1,
            other => panic!("unexpected outcome {other:?}"),
        }
    }

    assert_eq!((created, refused), (3, 5));
    assert_eq!(submissions_for(&pool, user).await, 3);
}
