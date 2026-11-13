//! The executor seam (design Path 7). The worker hands a claimed run to a
//! `RunExecutor`, and the executor reports progress and returns an outcome.
//!
//! The executor can only emit public events through `RunContext` and return an
//! outcome. It has no way to write a verdict, a status or a job state. The worker
//! turns the outcome into the one terminal write.

use crate::queue::emit::emit;
use crate::queue::finish::FinalResult;
use crate::queue::{ClaimToken, QueueError};
use counterspell_common::events::PublicEvent;
use counterspell_common::ids::RunId;
use counterspell_common::status::{FailureReason, RunMode};
use sqlx::PgPool;
use tokio::sync::watch;

/// What an executor needs to run one claimed attempt. It has no `Debug` impl, so the source code is not logged.
pub struct RunInput {
    /// The run being executed.
    pub run_id: RunId,
    /// The problem to load.
    pub problem_slug: String,
    /// The language of the source.
    pub lang: String,
    /// Sample run or hidden-test submission.
    pub mode: RunMode,
    /// The user's source text.
    pub code: String,
    /// The attempt number, starting at 1.
    pub attempt: u16,
}

/// How an executor ended a run that it could run to the end.
#[derive(Debug)]
pub enum RunOutcome {
    /// The program ran and the judge produced a result.
    Completed(FinalResult),
    /// The run cannot be judged for a reason outside the program.
    Failed(FailureReason),
}

/// Why an executor did not return an outcome.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecError {
    /// Something outside the program failed, such as the sandbox or its image. Retryable.
    Infrastructure(String),
    /// The stop signal fired (cancel, lost lease, shutdown or the hard deadline).
    Stopped,
}

/// Runs one claimed attempt. Generic code uses this trait, not `dyn`, so no future is boxed.
pub trait RunExecutor: Send + Sync + 'static {
    /// Runs `run` and reports progress through `ctx`.
    ///
    /// # Errors
    ///
    /// Returns `ExecError::Stopped` when the stop signal fired, or
    /// `ExecError::Infrastructure` when something outside the program failed.
    fn execute(
        &self,
        run: RunInput,
        ctx: RunContext,
    ) -> impl std::future::Future<Output = Result<RunOutcome, ExecError>> + Send;
}

/// Where a context sends its events.
enum Sink {
    /// The real queue. Each event is a fenced append.
    // The worker builds this variant in run-lifecycle T17.
    #[allow(dead_code)]
    Queue { pool: PgPool, token: ClaimToken },
    /// An in-memory list for unit tests.
    #[cfg(feature = "testkit")]
    Recording(std::sync::Arc<std::sync::Mutex<Vec<PublicEvent>>>),
}

/// The only things an executor may do to its run: emit a public event and watch the stop signal.
pub struct RunContext {
    sink: Sink,
    run: RunId,
    attempt: u16,
    stop: watch::Receiver<bool>,
}

impl RunContext {
    /// Builds the context the worker gives an executor. Only the worker creates one.
    // The worker calls this in run-lifecycle T17.
    #[allow(dead_code)]
    pub(crate) fn for_queue(
        pool: PgPool,
        token: ClaimToken,
        attempt: u16,
        stop: watch::Receiver<bool>,
    ) -> Self {
        let run = token.run_id();
        Self {
            sink: Sink::Queue { pool, token },
            run,
            attempt,
            stop,
        }
    }

    /// Builds a context that stores events in memory, for tests.
    #[cfg(feature = "testkit")]
    pub(crate) fn recording(
        log: std::sync::Arc<std::sync::Mutex<Vec<PublicEvent>>>,
        run: RunId,
        attempt: u16,
        stop: watch::Receiver<bool>,
    ) -> Self {
        Self {
            sink: Sink::Recording(log),
            run,
            attempt,
            stop,
        }
    }

    /// The run this context belongs to.
    #[must_use]
    pub const fn run_id(&self) -> RunId {
        self.run
    }

    /// The attempt number, starting at 1.
    #[must_use]
    pub const fn attempt(&self) -> u16 {
        self.attempt
    }

    /// Stores one public event for the run.
    ///
    /// A lost lease is reported as `Stopped`, so the executor must stop and write nothing more.
    ///
    /// # Errors
    ///
    /// Returns `ExecError::Stopped` when the claim is no longer held, or
    /// `ExecError::Infrastructure` when the event cannot be stored.
    pub async fn emit(&self, event: &PublicEvent) -> Result<(), ExecError> {
        match &self.sink {
            Sink::Queue { pool, token } => match emit(pool, token, event).await {
                Ok(_) => Ok(()),
                Err(QueueError::LeaseLost) => Err(ExecError::Stopped),
                Err(_) => Err(ExecError::Infrastructure(
                    "event could not be stored".to_owned(),
                )),
            },
            #[cfg(feature = "testkit")]
            Sink::Recording(log) => {
                if let Ok(mut events) = log.lock() {
                    events.push(event.clone());
                }
                Ok(())
            }
        }
    }

    /// True when the stop signal has fired. The executor should kill its program and return `Stopped`.
    #[must_use]
    pub fn stop_requested(&self) -> bool {
        *self.stop.borrow()
    }

    /// Waits until the stop signal fires. A dropped sender also counts as a stop.
    pub async fn stopped(&mut self) {
        loop {
            if *self.stop.borrow() {
                return;
            }
            if self.stop.changed().await.is_err() {
                return;
            }
        }
    }
}

/// The executor used until the sandbox runner exists. It never produces a verdict.
///
/// Every run it receives fails as an infrastructure error. The worker retries it, and after
/// the last attempt the run ends as `internal_error`, never as a wrong answer.
#[derive(Debug, Clone, Copy, Default)]
pub struct NoExecutor;

impl RunExecutor for NoExecutor {
    fn execute(
        &self,
        _run: RunInput,
        _ctx: RunContext,
    ) -> impl std::future::Future<Output = Result<RunOutcome, ExecError>> + Send {
        std::future::ready(Err(ExecError::Infrastructure(
            "no executor configured".to_owned(),
        )))
    }
}
