//! Tests for `RunWaker` (run-lifecycle T28, design Path 9 and requirements R12.5, R13.3, R13.4).
//!
//! The waker is a hint. These tests check that it carries only run ids, that it tells streams to read
//! again after the listener reconnects, and that a slow receiver lags harmlessly.
//!
//! Run with `DATABASE_URL=<url> cargo test -p counterspell-server --test wake -- --ignored pg_`.

use std::time::Duration;

use counterspell_common::ids::RunId;
use counterspell_server::runs::wake::{RunWaker, Wake};
use sqlx::PgPool;
use tokio::sync::broadcast::{error::RecvError, Receiver};

/// How long a test waits for one wake-up before it counts as missing.
const WAIT: Duration = Duration::from_secs(10);

/// Waits for the next wake-up that `keep` accepts. Lagged receivers are skipped, as streams do.
async fn next_matching(
    rx: &mut Receiver<Wake>,
    keep: impl Fn(Wake) -> bool,
) -> Result<Wake, String> {
    let deadline = tokio::time::Instant::now() + WAIT;
    loop {
        match tokio::time::timeout_at(deadline, rx.recv()).await {
            Err(_) => return Err("no wake-up within the wait".to_owned()),
            Ok(Ok(wake)) if keep(wake) => return Ok(wake),
            Ok(Ok(_) | Err(RecvError::Lagged(_))) => {}
            Ok(Err(RecvError::Closed)) => return Err("the waker channel closed".to_owned()),
        }
    }
}

/// Subscribes and waits until the listener has connected. The first `Wake::All` marks that point.
async fn ready_waker(pool: &PgPool) -> (RunWaker, Receiver<Wake>) {
    let waker = RunWaker::start(pool.clone());
    let mut rx = waker.subscribe();
    next_matching(&mut rx, |wake| wake == Wake::All)
        .await
        .expect("listener connects and sends the first wake-all");
    (waker, rx)
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_waker_delivers_run_id_only(pool: PgPool) {
    let (_waker, mut rx) = ready_waker(&pool).await;
    let run = RunId::new();

    sqlx::query("SELECT pg_notify('run_events', $1)")
        .bind(run.to_string())
        .execute(&pool)
        .await
        .expect("notify");

    let wake = next_matching(&mut rx, |wake| matches!(wake, Wake::Run(_)))
        .await
        .expect("the run id arrives");
    assert_eq!(wake, Wake::Run(run));
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_waker_ignores_payloads_that_are_not_run_ids(pool: PgPool) {
    let (_waker, mut rx) = ready_waker(&pool).await;
    let run = RunId::new();

    // Garbage first, then a real run id. Only the real one may come through.
    sqlx::query("SELECT pg_notify('run_events', 'not-a-run'), pg_notify('run_events', $1)")
        .bind(run.to_string())
        .execute(&pool)
        .await
        .expect("notify");

    let wake = next_matching(&mut rx, |wake| matches!(wake, Wake::Run(_)))
        .await
        .expect("the real run id arrives");
    assert_eq!(wake, Wake::Run(run));
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_waker_reconnect_sends_wake_all(pool: PgPool) {
    let (_waker, mut rx) = ready_waker(&pool).await;

    // Drop the listener's connection from the server side. The listener must reconnect and say so.
    let killed: i64 = sqlx::query_scalar(
        "SELECT count(*) FROM (SELECT pg_terminate_backend(pid) FROM pg_stat_activity \
         WHERE datname = current_database() AND pid <> pg_backend_pid() \
         AND query ILIKE 'LISTEN%') AS killed",
    )
    .fetch_one(&pool)
    .await
    .expect("terminate listener");
    assert!(killed >= 1, "expected to end the listener connection");

    let wake = next_matching(&mut rx, |wake| wake == Wake::All)
        .await
        .expect("a wake-all after reconnect");
    assert_eq!(wake, Wake::All);
}

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_waker_lag_is_harmless(pool: PgPool) {
    let (_waker, mut rx) = ready_waker(&pool).await;

    // Far more distinct wake-ups than the channel holds, while nothing reads. Distinct payloads keep
    // Postgres from merging them into one notification.
    sqlx::query(
        "SELECT pg_notify('run_events', gen_random_uuid()::text) FROM generate_series(1, 1500)",
    )
    .execute(&pool)
    .await
    .expect("flood notifications");

    // The receiver has lagged. It must still deliver later wake-ups normally.
    let run = RunId::new();
    sqlx::query("SELECT pg_notify('run_events', $1)")
        .bind(run.to_string())
        .execute(&pool)
        .await
        .expect("notify after the flood");

    let mut lagged = false;
    let deadline = tokio::time::Instant::now() + WAIT;
    loop {
        match tokio::time::timeout_at(deadline, rx.recv()).await {
            Err(elapsed) => panic!("the final wake-up never arrived after lagging: {elapsed}"),
            Ok(Err(RecvError::Lagged(_))) => lagged = true,
            Ok(Err(RecvError::Closed)) => panic!("the waker channel closed"),
            Ok(Ok(Wake::Run(id))) if id == run => break,
            Ok(Ok(_)) => {}
        }
    }
    assert!(lagged, "the flood should have caused a lag");
}

#[test]
fn wake_all_concerns_every_run_and_run_wake_only_its_own() {
    let mine = RunId::new();
    let other = RunId::new();

    assert!(Wake::All.concerns(mine));
    assert!(Wake::Run(mine).concerns(mine));
    assert!(!Wake::Run(other).concerns(mine));
}
