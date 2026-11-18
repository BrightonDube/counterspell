//! One call to the driver: write one line, read one answer line, all under a deadline.
//!
//! The call reports what it observed and nothing more. It never compares an answer with
//! an expected value, and it never decides a verdict. The run code and the comparator do
//! that. Any abnormal end (deadline, stop, stdout over budget, the driver exiting) kills
//! and removes the container before the call returns, so the caller starts the next test
//! in a fresh container.

use std::cmp::min;
use std::time::Duration;

use serde_json::Value;
use tokio::sync::watch;
use tokio::time::{sleep_until, Instant};

use super::backend::{BackendSession, DriverChannel, DriverEvent, InfraError};
use super::driver_protocol::encode_call;
use super::session::Opened;
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
    /// The program wrote more than one line in a single read, or data after its answer.
    /// The protocol allows exactly one answer line per call (R8.4). The container has been killed.
    ExtraOutput,
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

/// The per-driver I/O state: the stdout reader and the stderr buffer. A new driver gets a new
/// `CallIo`, so nothing carries over from a container that has ended.
#[derive(Debug)]
pub struct CallIo {
    stdout: LineReader,
    stderr: StderrBuffer,
}

impl CallIo {
    /// Fresh I/O state for a new driver.
    #[must_use]
    pub fn new() -> Self {
        Self {
            stdout: LineReader::with_budget(STDOUT_BUDGET, STDOUT_BUDGET),
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
/// An infrastructure error ends the container before this returns, so an error path never
/// leaves a program running. The caller still gets the original error.
///
/// # Errors
///
/// Returns an [`InfraError`] when the daemon fails or the kill cannot be confirmed. A
/// daemon error is never a verdict.
pub async fn run_call<S: BackendSession + 'static>(
    opened: &mut Opened<S>,
    io: &mut CallIo,
    id: u64,
    args: &[Value],
    limits: CallLimits,
    stop: &mut watch::Receiver<bool>,
) -> Result<CallFacts, InfraError> {
    let result = call_once(opened, io, id, args, limits, stop).await;
    if result.is_err() {
        // Best effort: the daemon may be the thing that failed. The sweeper is the backstop.
        // The attach stream closes first — an open one stalls the daemon's removal.
        drop(opened.driver.take());
        let _ = opened.session.abort(limits.kill_slack).await;
    }
    result
}

async fn call_once<S: BackendSession + 'static>(
    opened: &mut Opened<S>,
    io: &mut CallIo,
    id: u64,
    args: &[Value],
    limits: CallLimits,
    stop: &mut watch::Receiver<bool>,
) -> Result<CallFacts, InfraError> {
    let started = Instant::now();
    let deadline = min(started + limits.time, limits.run_deadline);
    let line = encode_call(id, args)
        .map_err(|error| InfraError::Unexpected(format!("encoding call {id}: {error:?}")))?;
    io.stdout.reset_budget(STDOUT_BUDGET);

    tokio::select! {
        biased;
        () = sleep_until(deadline) => {
            return abnormal(opened, limits, CallFacts::Deadline).await;
        }
        () = stopped(stop) => {
            return abnormal(opened, limits, CallFacts::Stopped).await;
        }
        written = write_call(opened, &line) => written?,
    }

    loop {
        tokio::select! {
            biased;
            () = sleep_until(deadline) => {
                return abnormal(opened, limits, CallFacts::Deadline).await;
            }
            () = stopped(stop) => {
                return abnormal(opened, limits, CallFacts::Stopped).await;
            }
            event = next_event(opened) => match event? {
                DriverEvent::Line(bytes) | DriverEvent::Chunk(bytes) => {
                    let Ok(lines) = io.stdout.push(&bytes) else {
                        return abnormal(opened, limits, CallFacts::OutputLimit).await;
                    };
                    // One answer line and nothing after it. Any further line, or a partial
                    // line after the answer, is data the protocol does not allow.
                    match lines.as_slice() {
                        [] => {}
                        [line] if io.stdout.held() == 0 => {
                            return Ok(answer(line.clone(), started));
                        }
                        _ => return abnormal(opened, limits, CallFacts::ExtraOutput).await,
                    }
                }
                DriverEvent::Stderr(bytes) => io.stderr.push(&bytes),
                DriverEvent::Exited { code, oom_kill } => {
                    let facts = CallFacts::DriverExited {
                        code,
                        oom_kill,
                        last_stderr_line: io.stderr.last_line().map(<[u8]>::to_vec),
                    };
                    return abnormal(opened, limits, facts).await;
                }
            },
        }
    }
}

/// Borrows the live driver for one operation, so the borrow ends with the select arm
/// and `abnormal` can take the whole `Opened`.
async fn write_call<S: BackendSession + 'static>(
    opened: &mut Opened<S>,
    line: &[u8],
) -> Result<(), InfraError> {
    opened
        .driver
        .as_mut()
        .ok_or_else(|| InfraError::Unexpected("call on a session with no driver".to_owned()))?
        .write_line(line)
        .await
}

/// The driver's next event. Same borrow scoping as [`write_call`].
async fn next_event<S: BackendSession + 'static>(
    opened: &mut Opened<S>,
) -> Result<DriverEvent, InfraError> {
    opened
        .driver
        .as_mut()
        .ok_or_else(|| InfraError::Unexpected("call on a session with no driver".to_owned()))?
        .next_event()
        .await
}

fn answer(line: Vec<u8>, started: Instant) -> CallFacts {
    CallFacts::Answer {
        line,
        elapsed: started.elapsed(),
    }
}

/// Kills and removes the container, confirms it is gone within the kill slack, and then
/// returns the facts that ended the call. The driver — the open attach stream — is
/// dropped first: a daemon's removal waits on stream teardown, so a program still
/// flooding it would stall the kill past the slack.
async fn abnormal<S: BackendSession + 'static>(
    opened: &mut Opened<S>,
    limits: CallLimits,
    facts: CallFacts,
) -> Result<CallFacts, InfraError> {
    drop(opened.driver.take());
    opened.session.abort(limits.kill_slack).await?;
    Ok(facts)
}
