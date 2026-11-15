//! Tests for the run event stream (run-lifecycle T29, design Path 9, requirements R12 and R13).
//!
//! Run with `DATABASE_URL=<url> cargo test -p counterspell-server --test sse -- --ignored pg_`.

mod support;

use counterspell_common::{
    ids::{RunId, UserId},
    status::RunMode,
};
use counterspell_server::{
    config::RunLimits,
    runs::{
        enqueue::{enqueue, NewRun},
        sse::replay_events_from,
    },
    testkit::ensure_user,
};
use sqlx::PgPool;
use support::publish_two_sum;
use uuid::Uuid;

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

fn owner(number: u128) -> UserId {
    UserId::from_uuid(Uuid::from_u128(number))
}

/// Saves a queued run for `user` and returns its id.
async fn queued_run(pool: &PgPool, user: UserId) -> RunId {
    ensure_user(pool, user).await.expect("user fixture");
    publish_two_sum(pool).await;
    let run = NewRun::new(
        RunId::new(),
        user,
        "two-sum",
        "python",
        RunMode::Submit,
        "print(1)\n",
        None,
    );
    enqueue(pool, &run, &limits()).await.expect("enqueue");
    run.run_id
}

/// Adds `count` non-terminal events to a run, numbered from 1, the way the judge writes them.
async fn add_events(pool: &PgPool, run: RunId, count: i32) {
    sqlx::query(
        "INSERT INTO run_events (submission_id, seq, attempt, type, payload) \
         SELECT $1, g, 1, 'test', '{}'::jsonb FROM generate_series(1, $2) AS g",
    )
    .bind(run.as_uuid())
    .bind(count)
    .execute(pool)
    .await
    .expect("insert events");
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_replay_is_owner_scoped(pool: PgPool) {
    let run = queued_run(&pool, owner(1)).await;
    add_events(&pool, run, 3).await;

    let as_owner = replay_events_from(&pool, owner(1), run, 0)
        .await
        .expect("owner replay");
    let as_stranger = replay_events_from(&pool, owner(2), run, 0)
        .await
        .expect("stranger replay");

    assert_eq!(
        as_owner.iter().map(|row| row.seq).collect::<Vec<_>>(),
        vec![1, 2, 3]
    );
    assert!(
        as_stranger.is_empty(),
        "another user must see no events of this run"
    );
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_replay_pages_in_order_from_the_cursor(pool: PgPool) {
    let run = queued_run(&pool, owner(1)).await;
    add_events(&pool, run, 450).await;

    let first = replay_events_from(&pool, owner(1), run, 0)
        .await
        .expect("first page");
    let second = replay_events_from(&pool, owner(1), run, 200)
        .await
        .expect("second page");
    let last = replay_events_from(&pool, owner(1), run, 450)
        .await
        .expect("empty page");

    assert_eq!(first.len(), 200, "a page holds at most 200 events");
    assert_eq!(first.first().map(|row| row.seq), Some(1));
    assert_eq!(first.last().map(|row| row.seq), Some(200));
    assert_eq!(second.first().map(|row| row.seq), Some(201));
    assert_eq!(second.last().map(|row| row.seq), Some(400));
    assert!(last.is_empty(), "nothing after the last event");
}
