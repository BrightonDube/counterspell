//! Runs the judge's background tasks together and stops them together (design Paths 6 and 7).
//!
//! The reaper and one worker run until the shutdown signal is set. `serve` returns only
//! after both have finished, so the caller never exits in the middle of a write.

use crate::config::JudgeConfig;
use crate::executor::RunExecutor;
use crate::queue::reap::run_reaper;
use crate::worker::Worker;
use sqlx::PgPool;
use std::sync::Arc;
use tokio::sync::watch;
use uuid::Uuid;

/// Runs the reaper and a worker until `shutdown` is set, then waits for both to finish.
///
/// The worker id ends in a random part, so two judge processes never share an id.
pub async fn serve<E: RunExecutor>(
    pool: PgPool,
    cfg: JudgeConfig,
    executor: E,
    shutdown: watch::Receiver<bool>,
) {
    let worker_id = format!("judge-{}", Uuid::new_v4());
    let worker = Arc::new(Worker::new(pool.clone(), cfg.clone(), executor, worker_id));
    tokio::join!(
        run_reaper(&pool, &cfg, shutdown.clone()),
        Arc::clone(&worker).run(shutdown),
    );
}
