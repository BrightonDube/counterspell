//! Database test for the judge's placeholder executor (run-lifecycle T19).
//!
//! Until the sandbox runner exists, the judge runs `NoExecutor`. A run it receives must
//! never end with a verdict, and it must go back to the queue as an infrastructure error.
//!
//! Run with `cargo test -p counterspell-judge --test service -- --ignored pg_`.

use counterspell_judge::config::JudgeConfig;
use counterspell_judge::executor::NoExecutor;
use counterspell_judge::testkit::{assert_invariants, seed_run};
use counterspell_judge::worker::Worker;
use sqlx::PgPool;

#[ignore = "needs DATABASE_URL"]
#[sqlx::test(migrations = "../../migrations")]
async fn pg_placeholder_executor_never_completes_a_run(pool: PgPool) {
    let run = seed_run(&pool).await.unwrap();
    let cfg = JudgeConfig::from_lookup(|_| None).expect("defaults are valid");
    let worker = Worker::new(pool.clone(), cfg, NoExecutor, "judge-a".to_owned());

    assert!(worker.process_next().await.unwrap());

    let (status, job_state, attempts): (String, String, i16) = sqlx::query_as(
        "SELECT s.status, j.state, j.attempts FROM submissions s \
         JOIN jobs j ON j.submission_id = s.id WHERE s.id = $1",
    )
    .bind(run.as_uuid())
    .fetch_one(&pool)
    .await
    .unwrap();
    // The attempt is released for retry, with no verdict and no terminal event.
    assert_eq!(
        (status.as_str(), job_state.as_str(), attempts),
        ("queued", "queued", 1)
    );
    let verdict: Option<String> =
        sqlx::query_scalar("SELECT verdict FROM submissions WHERE id = $1")
            .bind(run.as_uuid())
            .fetch_one(&pool)
            .await
            .unwrap();
    assert_eq!(verdict, None);
    assert_invariants(&pool).await.unwrap();
}
