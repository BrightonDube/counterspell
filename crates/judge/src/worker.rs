//! The worker (design Path 7). It claims a run, runs it through the executor, and
//! turns the outcome into one terminal write.
//!
//! The worker never decides a verdict. It only passes on what the executor returned:
//! a completed result goes to `finish`, a failure reason to `finish_failed`, and an
//! infrastructure problem to `release_for_retry`. A heartbeat keeps the lease alive
//! and watches for a cancel, a lost lease or shutdown.

use crate::config::JudgeConfig;
use crate::executor::{ExecError, RunContext, RunExecutor, RunInput, RunOutcome};
use crate::queue::claim::{claim_next, ClaimedRun};
use crate::queue::finish::{finish, finish_failed};
use crate::queue::lease::{release_for_retry, renew};
use crate::queue::{ClaimToken, QueueError};
use counterspell_common::status::FailureReason;
use sqlx::PgPool;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::watch;

/// Time kept back from the maximum claim age, so the worker gives up before the reaper does.
const DEADLINE_MARGIN: Duration = Duration::from_secs(5);

/// Why the executor was asked to stop. The first reason written wins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum StopReason {
    /// Nothing has asked the executor to stop.
    Running,
    /// The user cancelled the run.
    Cancel,
    /// The worker is shutting down and the grace period has passed.
    Shutdown,
    /// Another worker took the lease, or renewal failed for a full lease period.
    LeaseLost,
}

/// Claims and runs queued runs through one executor.
pub struct Worker<E: RunExecutor> {
    pool: PgPool,
    cfg: JudgeConfig,
    executor: Arc<E>,
    id: String,
}

impl<E: RunExecutor> Worker<E> {
    /// A worker that identifies itself as `id` on every claim.
    #[must_use]
    pub fn new(pool: PgPool, cfg: JudgeConfig, executor: E, id: String) -> Self {
        Self {
            pool,
            cfg,
            executor: Arc::new(executor),
            id,
        }
    }

    /// Claims runs while a slot is free, and runs each one in its own task.
    ///
    /// Stops claiming when `shutdown` is set, then waits for the runs in flight to finish.
    /// Runs still going after the grace period are stopped by their heartbeat.
    pub async fn run(self: Arc<Self>, mut shutdown: watch::Receiver<bool>) {
        let slots = Arc::new(tokio::sync::Semaphore::new(self.cfg.concurrency));
        let poll = self.cfg.claim_poll;
        loop {
            if *shutdown.borrow() {
                break;
            }
            let permit = tokio::select! {
                acquired = Arc::clone(&slots).acquire_owned() => match acquired {
                    Ok(permit) => permit,
                    Err(_closed) => break,
                },
                _ = shutdown.changed() => continue,
            };
            match claim_next(&self.pool, &self.id, self.cfg.lease).await {
                Ok(Some(claimed)) => {
                    let worker = Arc::clone(&self);
                    let running_shutdown = shutdown.clone();
                    tokio::spawn(async move {
                        worker.execute(claimed, running_shutdown).await;
                        drop(permit);
                    });
                }
                Ok(None) => {
                    drop(permit);
                    tokio::select! {
                        () = tokio::time::sleep(poll) => {}
                        _ = shutdown.changed() => {}
                    }
                }
                Err(error) => {
                    drop(permit);
                    tracing::error!(%error, "claim failed; retrying");
                    tokio::select! {
                        () = tokio::time::sleep(poll) => {}
                        _ = shutdown.changed() => {}
                    }
                }
            }
        }
        // Wait until every slot is free, which means every run in flight has ended.
        let all = u32::try_from(self.cfg.concurrency).unwrap_or(u32::MAX);
        let _drained = slots.acquire_many(all).await;
    }

    /// Claims one ready run and runs it to its end.
    ///
    /// Returns `false` when no run was ready. A failure inside the run is recorded on the run
    /// and is not returned here; only a failed claim is an error.
    ///
    /// # Errors
    ///
    /// Returns a `QueueError` when the claim itself fails.
    pub async fn process_next(&self) -> Result<bool, QueueError> {
        let Some(claimed) = claim_next(&self.pool, &self.id, self.cfg.lease).await? else {
            return Ok(false);
        };
        // Keep the sender alive for the whole run, so the shutdown receiver never reports a stop by accident.
        let (_keep_running, shutdown) = watch::channel(false);
        self.execute(claimed, shutdown).await;
        Ok(true)
    }

    /// Runs a claimed run and writes its one outcome.
    pub(crate) async fn execute(&self, claimed: ClaimedRun, shutdown: watch::Receiver<bool>) {
        let token = claimed.token.clone();
        if claimed.cancel_requested {
            // The user cancelled before this claim, so the program never runs.
            record(
                "finish cancelled run",
                finish_failed(&self.pool, &token, FailureReason::Cancelled).await,
            );
            return;
        }

        let reason = Arc::new(Mutex::new(StopReason::Running));
        let (stop_tx, stop_rx) = watch::channel(false);
        let heartbeat = tokio::spawn(heartbeat(
            self.pool.clone(),
            self.cfg.clone(),
            token.clone(),
            stop_tx,
            Arc::clone(&reason),
            shutdown,
        ));

        let attempt = u16::try_from(token.attempts().saturating_add(1)).unwrap_or(u16::MAX);
        let input = RunInput {
            run_id: token.run_id(),
            problem_slug: claimed.problem_slug,
            lang: claimed.lang,
            mode: claimed.mode,
            code: claimed.code,
            attempt,
        };
        let ctx = RunContext::for_queue(self.pool.clone(), token.clone(), attempt, stop_rx);
        let executor = Arc::clone(&self.executor);
        // Running the executor in its own task means a panic becomes a join error here.
        let run = tokio::spawn(async move { executor.execute(input, ctx).await });
        let abort = run.abort_handle();
        let joined = tokio::time::timeout(hard_deadline(&self.cfg), run).await;
        heartbeat.abort();
        let why = reason.lock().map_or(StopReason::Running, |guard| *guard);

        match joined {
            Err(_elapsed) => {
                abort.abort();
                self.release(&token, "hard deadline reached").await;
            }
            Ok(Err(_join_error)) => self.release(&token, "executor panicked").await,
            Ok(Ok(outcome)) => self.settle(&token, outcome, why).await,
        }
    }

    /// Maps the executor's answer to the one write that belongs to it.
    async fn settle(
        &self,
        token: &ClaimToken,
        outcome: Result<RunOutcome, ExecError>,
        why: StopReason,
    ) {
        match outcome {
            Ok(RunOutcome::Completed(result)) => {
                record("finish run", finish(&self.pool, token, &result).await);
            }
            Ok(RunOutcome::Failed(reason)) => {
                record(
                    "finish failed run",
                    finish_failed(&self.pool, token, reason).await,
                );
            }
            Err(ExecError::Infrastructure(message)) => self.release(token, &message).await,
            Err(ExecError::Stopped) => match why {
                StopReason::Cancel => record(
                    "finish cancelled run",
                    finish_failed(&self.pool, token, FailureReason::Cancelled).await,
                ),
                // The lease is gone, so this worker must write nothing.
                StopReason::LeaseLost => {}
                StopReason::Shutdown => self.release(token, "worker shutting down").await,
                StopReason::Running => self.release(token, "executor stopped unexpectedly").await,
            },
        }
    }

    /// Gives the attempt back so another worker can retry it.
    async fn release(&self, token: &ClaimToken, cause: &str) {
        record(
            "release run for retry",
            release_for_retry(&self.pool, token, cause, &self.cfg)
                .await
                .map(|_| ()),
        );
    }
}

/// Keeps the lease alive and turns a cancel, a lost lease or a shutdown into a stop signal.
///
/// It renews every `renew` interval. A renewal that fails for a whole lease period counts
/// as a lost lease, because the reaper will take the job back by then.
async fn heartbeat(
    pool: PgPool,
    cfg: JudgeConfig,
    token: ClaimToken,
    stop: watch::Sender<bool>,
    reason: Arc<Mutex<StopReason>>,
    mut shutdown: watch::Receiver<bool>,
) {
    let mut ticker = tokio::time::interval(cfg.renew);
    let mut last_renewed = Instant::now();
    loop {
        tokio::select! {
            _ = ticker.tick() => match renew(&pool, &token, cfg.lease).await {
                Ok(renewal) => {
                    last_renewed = Instant::now();
                    if renewal.cancel_requested {
                        stop_with(&stop, &reason, StopReason::Cancel);
                        return;
                    }
                }
                Err(QueueError::LeaseLost) => {
                    stop_with(&stop, &reason, StopReason::LeaseLost);
                    return;
                }
                Err(error) => {
                    tracing::warn!(%error, "lease renewal failed");
                    if last_renewed.elapsed() >= cfg.lease {
                        stop_with(&stop, &reason, StopReason::LeaseLost);
                        return;
                    }
                }
            },
            changed = shutdown.changed() => {
                if changed.is_err() {
                    // No shutdown sender is left, so nothing can ask this run to stop. Stop renewing.
                    return;
                }
                if *shutdown.borrow() {
                    // Give the executor the grace period to finish on its own first.
                    tokio::time::sleep(cfg.shutdown_grace).await;
                    stop_with(&stop, &reason, StopReason::Shutdown);
                    return;
                }
            }
        }
    }
}

/// Records why the executor must stop, then signals it. The first reason is kept.
fn stop_with(stop: &watch::Sender<bool>, reason: &Mutex<StopReason>, why: StopReason) {
    if let Ok(mut current) = reason.lock() {
        if *current == StopReason::Running {
            *current = why;
        }
    }
    // A send only fails when the executor has already finished, which is fine.
    let _ = stop.send(true);
}

/// The longest one run may hold its executor: the maximum claim age, less a safety margin.
fn hard_deadline(cfg: &JudgeConfig) -> Duration {
    cfg.max_claim_age.saturating_sub(DEADLINE_MARGIN)
}

/// Logs the outcome of a terminal write. A lost lease is expected and logged quietly.
fn record(action: &str, result: Result<(), QueueError>) {
    match result {
        Ok(()) => {}
        Err(QueueError::LeaseLost) => {
            tracing::info!(action, "lease lost before writing; nothing recorded");
        }
        Err(error) => tracing::error!(action, %error, "could not record the outcome"),
    }
}
