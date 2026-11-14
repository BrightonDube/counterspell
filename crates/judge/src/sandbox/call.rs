//! One call to the driver: write one line, read one answer line, all under a deadline.
//!
//! The call reports what it observed and nothing more. It never compares an answer with
//! an expected value, and it never decides a verdict. The run code and the comparator do
//! that. Any abnormal end (deadline, stop, stdout over budget, the driver exiting) kills
//! and removes the container before the call returns, so the caller starts the next test
//! in a fresh container.

use std::cmp::min;
use std::collections::VecDeque;
use std::time::Duration;

use serde_json::Value;
use tokio::sync::watch;
use tokio::time::{sleep_until, Instant};

use super::backend::{BackendSession, DriverChannel, DriverEvent, InfraError};
use super::driver_protocol::encode_call;
use super::session::{Opened, Session};
use super::stream::{LineReader, StderrBuffer};

/// Stdout one call may produce, in bytes (requirement R6.2). Also the longest allowed line.
pub const STDOUT_BUDGET: usize = 1024 * 1024;

/// The time limits of one call.
#[derive(Debug, Clone, Copy)]
pub struct CallLimits {
    /// The per-test time limit.
    pub time: Duration,
    /// The run's absolute deadline. The call never runs past it.
    pub run_deadline: Instant,
    /// How long the kill may take before the call reports `KillFailed`.
    pub kill_slack: Duration,
}

/// What the call saw. Each variant is a fact, not a verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CallFacts {
    /// The program answered with this line (without its newline) in time.
    Answer {
        /// The answer line.
        line: Vec<u8>,
        /// Time from the start of the call to the answer.
        elapsed: Duration,
    },
    /// The deadline passed first. The container has been killed.
    Deadline,
    /// The stop signal fired first. The container has been killed.
    Stopped,
    /// The program wrote more than the stdout budget. The container has been killed.
    OutputLimit,
    /// The driver process exited before answering. The container has been removed.
    DriverExited {
        /// The exit code.
        code: i64,
        /// The kernel's OOM-kill count for the container.
        oom_kill: u64,
        /// The last complete stderr line, kept even when stderr was truncated.
        last_stderr_line: Option<Vec<u8>>,
    },
}

/// The per-session I/O state that outlives one call: the stdout reader, answer lines
/// that arrived early, and the stderr buffer.
#[derive(Debug)]
pub struct CallIo {
    stdout: LineReader,
    pending: VecDeque<Vec<u8>>,
    stderr: StderrBuffer,
}

impl CallIo {
    /// Fresh I/O state for a new driver.
    #[must_use]
    pub fn new() -> Self {
        Self {
            stdout: LineReader::with_budget(STDOUT_BUDGET, STDOUT_BUDGET),
            pending: VecDeque::new(),
            stderr: StderrBuffer::new(),
        }
    }
}

impl Default for CallIo {
    fn default() -> Self {
        Self::new()
    }
}

/// Resolves when the stop signal fires, or when its sender is gone.
async fn stopped(stop: &mut watch::Receiver<bool>) {
    loop {
        if *stop.borrow() {
            return;
        }
        if stop.changed().await.is_err() {
            return;
        }
    }
}

/// Runs one call: writes `args` as call `id`, then reads one answer line.
///
/// Each wait is a biased `select!`. The deadline is checked first, then the stop signal,
/// then the I/O, so a late answer can never beat the deadline by luck.
///
/// # Errors
///
/// Returns an [`InfraError`] when the daemon fails or the kill cannot be confirmed. A
/// daemon error is never a verdict. The caller closes the session when it sees one.
pub async fn run_call<S: BackendSession + 'static>(
    opened: &mut Opened<S>,
    io: &mut CallIo,
    id: u64,
    args: &[Value],
    limits: CallLimits,
    stop: &mut watch::Receiver<bool>,
) -> Result<CallFacts, InfraError> {
    let started = Instant::now();
    let Opened { session, driver } = opened;

    if let Some(line) = io.pending.pop_front() {
        return Ok(answer(line, started));
    }

    let deadline = min(started + limits.time, limits.run_deadline);
    let line = encode_call(id, args)
        .map_err(|error| InfraError::Unexpected(format!("encoding call {id}: {error:?}")))?;
    io.stdout.reset_budget(STDOUT_BUDGET);

    tokio::select! {
        biased;
        () = sleep_until(deadline) => {
            return abnormal(session, limits, CallFacts::Deadline).await;
        }
        () = stopped(stop) => {
            return abnormal(session, limits, CallFacts::Stopped).await;
        }
        written = driver.write_line(&line) => written?,
    }

    loop {
        tokio::select! {
            biased;
            () = sleep_until(deadline) => {
                return abnormal(session, limits, CallFacts::Deadline).await;
            }
            () = stopped(stop) => {
                return abnormal(session, limits, CallFacts::Stopped).await;
            }
            event = driver.next_event() => match event? {
                DriverEvent::Line(bytes) | DriverEvent::Chunk(bytes) => {
                    let Ok(lines) = io.stdout.push(&bytes) else {
                        return abnormal(session, limits, CallFacts::OutputLimit).await;
                    };
                    io.pending.extend(lines);
                    if let Some(line) = io.pending.pop_front() {
                        return Ok(answer(line, started));
                    }
                }
                DriverEvent::Stderr(bytes) => io.stderr.push(&bytes),
                DriverEvent::Exited { code, oom_kill } => {
                    let facts = CallFacts::DriverExited {
                        code,
                        oom_kill,
                        last_stderr_line: io.stderr.last_line().map(<[u8]>::to_vec),
                    };
                    return abnormal(session, limits, facts).await;
                }
            },
        }
    }
}

fn answer(line: Vec<u8>, started: Instant) -> CallFacts {
    CallFacts::Answer {
        line,
        elapsed: started.elapsed(),
    }
}

/// Kills and removes the container, confirms it is gone within the kill slack, and then
/// returns the facts that ended the call.
async fn abnormal<S: BackendSession + 'static>(
    session: &mut Session<S>,
    limits: CallLimits,
    facts: CallFacts,
) -> Result<CallFacts, InfraError> {
    session.abort(limits.kill_slack).await?;
    Ok(facts)
}
