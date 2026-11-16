//! Run orchestration: compile, then run the tests in plan order, with a fresh container
//! after every abnormal end, fail-fast, the run cap and the stop signal.
//!
//! Sample tests run before any hidden test (amendment A1). Only this module and
//! [`compare`](super::compare) see expected values. The per-test verdicts it returns feed
//! [`verdict::decide`](super::verdict::decide), which is the only way to get a run verdict.

use std::time::Duration;

use serde_json::Value;
use tokio::sync::watch;
use tokio::time::{timeout_at, Instant};

use super::backend::{Backend, BackendSession, InfraError};
use super::call::{run_call, CallFacts, CallIo, CallLimits};
use super::compare::WrongReason;
use super::compare::{judge_answer, Expected, TestVerdict};
use super::driver_protocol::ProtocolViolation;
use super::session::{open_session, Opened, SessionError};
use super::spec::SessionSpec;
use super::verdict::{decide, ExecutedTest, FinalVerdict, RunFacts};
use crate::languages::LanguageDefinition;
use counterspell_engine::Comparator;

/// One test of the plan. The expected value lives only here and in `compare`.
pub struct PlannedTest {
    /// The test's position in the problem.
    pub ordinal: u32,
    /// Whether this is a visible sample test.
    pub sample: bool,
    /// The arguments for the call, in the signature's order.
    pub args: Vec<Value>,
    /// The expected output.
    pub expected: Expected,
    /// How the answer is compared with `expected`.
    pub comparator: Comparator,
    /// The per-test time limit.
    pub time_limit: Duration,
}

/// Limits and the stop signal for one run.
#[derive(Debug, Clone, Copy)]
pub struct RunSettings {
    /// The run's absolute deadline (the run cap).
    pub run_deadline: Instant,
    /// How long a kill may take before it counts as a failure.
    pub kill_slack: Duration,
    /// After this many abnormal tests in a row, the rest of the plan is not run.
    pub max_consecutive_abnormal: u32,
}

/// When one test ran, for the public records.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TestTiming {
    /// The test's ordinal.
    pub ordinal: u32,
    /// Whether it was a sample test.
    pub sample: bool,
    /// Time from the start of the call to its verdict.
    pub elapsed: Duration,
}

/// What one run observed. Turn it into a verdict with [`RunResult::verdict`].
#[derive(Debug)]
pub struct RunResult {
    /// The verdict of each test that ran, in the order they ran.
    pub executed: Vec<ExecutedTest>,
    /// Timing of each test that ran.
    pub timings: Vec<TestTiming>,
    /// The facts the verdict needs.
    pub facts: RunFacts,
}

impl RunResult {
    /// The final verdict for the run.
    #[must_use]
    pub fn verdict(&self) -> FinalVerdict {
        decide(&self.executed, &self.facts)
    }
}

/// Sorts the plan into run order: every sample test first, then every hidden test, each
/// group by ordinal. This is amendment A1: no visible test runs after a hidden one.
#[must_use]
pub fn schedule(mut tests: Vec<PlannedTest>) -> Vec<PlannedTest> {
    tests.sort_by_key(|test| (!test.sample, test.ordinal));
    tests
}

/// True when the stop signal has fired, or its sender has gone. The call runner treats a
/// dropped sender the same way.
fn stop_requested(stop: &mut watch::Receiver<bool>) -> bool {
    *stop.borrow() || stop.has_changed().is_err()
}

/// Mutable state of one run, kept together so the loop's early exits stay readable.
#[derive(Debug)]
struct Progress {
    executed: Vec<ExecutedTest>,
    timings: Vec<TestTiming>,
    compiled: bool,
    cut_by_cap: bool,
    stopped: bool,
    consecutive_abnormal: u32,
    sessions_opened: u32,
}

/// Runs a plan against a backend.
///
/// The first container is opened with the compile step. A compile failure there gives
/// `compiled = false` and no test runs. Every abnormal test ends its container, and the
/// next test opens a fresh one. Stop is checked before each test and during each call.
///
/// # Errors
///
/// Returns an [`InfraError`] when the sandbox fails. An infrastructure error is never a
/// verdict on the user's code.
pub async fn execute_run<B: Backend>(
    backend: &B,
    lang: &LanguageDefinition,
    archive: &[u8],
    session_spec: impl Fn(u32) -> Result<SessionSpec, InfraError>,
    plan: Vec<PlannedTest>,
    settings: RunSettings,
    stop: &mut watch::Receiver<bool>,
) -> Result<RunResult, InfraError> {
    let (result, session) =
        execute_run_and_keep(backend, lang, archive, session_spec, plan, settings, stop).await?;
    if let Some(open) = session {
        if let Err(error) = open.session.close().await {
            tracing::warn!(%error, "closing the run's container did not confirm removal");
        }
    }
    Ok(result)
}

/// Runs a plan like [`execute_run`], but hands the still-open session back to the
/// caller instead of closing it. The bench collector (T08) uses this to run the
/// bench driver in the same container the correctness calls ran in; the caller
/// becomes responsible for closing it, whatever the outcome.
///
/// # Errors
///
/// Returns an [`InfraError`] when the sandbox fails. An infrastructure error is never a
/// verdict on the user's code.
#[allow(clippy::too_many_arguments)]
pub async fn execute_run_and_keep<B: Backend>(
    backend: &B,
    lang: &LanguageDefinition,
    archive: &[u8],
    session_spec: impl Fn(u32) -> Result<SessionSpec, InfraError>,
    plan: Vec<PlannedTest>,
    settings: RunSettings,
    stop: &mut watch::Receiver<bool>,
) -> Result<(RunResult, Option<Opened<B::Session>>), InfraError> {
    if settings.max_consecutive_abnormal == 0 {
        return Err(InfraError::Unexpected(
            "max_consecutive_abnormal must be at least 1".to_owned(),
        ));
    }
    let planned = u32::try_from(plan.len()).unwrap_or(u32::MAX);
    let mut progress = Progress {
        executed: Vec::new(),
        timings: Vec::new(),
        compiled: true,
        cut_by_cap: false,
        stopped: false,
        consecutive_abnormal: 0,
        sessions_opened: 0,
    };
    let mut current: Option<Opened<B::Session>> = None;
    let mut io = CallIo::new();

    for test in schedule(plan) {
        if stop_requested(stop) {
            progress.stopped = true;
            break;
        }
        if Instant::now() >= settings.run_deadline {
            progress.cut_by_cap = true;
            break;
        }
        if progress.consecutive_abnormal >= settings.max_consecutive_abnormal {
            break;
        }

        if current.is_none() {
            // The open is bounded by the run cap, so a stalled daemon cannot hold the slot.
            let opening = open_next(
                backend,
                lang,
                archive,
                &session_spec,
                &mut progress.sessions_opened,
            );
            match timeout_at(settings.run_deadline, opening).await {
                Ok(Ok(opened)) => {
                    current = Some(opened);
                    io = CallIo::new();
                }
                Ok(Err(SessionError::CompileFailed(_))) if progress.sessions_opened == 1 => {
                    progress.compiled = false;
                    break;
                }
                Ok(Err(error)) => return Err(into_infra(error)),
                Err(_elapsed) => return Err(InfraError::ApiTimeout),
            }
        }
        let Some(open) = current.as_mut() else {
            return Err(InfraError::Unexpected(
                "no open container for a test".to_owned(),
            ));
        };

        let Some((verdict, elapsed, container_alive, answer)) =
            run_test(open, &mut io, &test, &settings, stop)
                .await?
                .into_ran()
        else {
            progress.stopped = true;
            break;
        };

        // Any end other than an answer has already removed the container. Drop the handle and
        // open a fresh one for the next test.
        if container_alive {
            progress.consecutive_abnormal = 0;
        } else {
            current = None;
            progress.consecutive_abnormal += 1;
        }
        progress.timings.push(TestTiming {
            ordinal: test.ordinal,
            sample: test.sample,
            elapsed,
        });
        progress.executed.push(match answer {
            Some(line) => ExecutedTest::with_answer(test.ordinal, verdict, line),
            None => ExecutedTest::new(test.ordinal, verdict),
        });
    }

    Ok((
        RunResult {
            executed: progress.executed,
            timings: progress.timings,
            facts: RunFacts {
                compiled: progress.compiled,
                planned,
                cut_by_cap: progress.cut_by_cap,
                stopped: progress.stopped,
            },
        },
        current,
    ))
}

/// What one test's call produced.
enum Step {
    /// The test ran to a verdict. `container_alive` is false when the call ended the container.
    Ran {
        verdict: TestVerdict,
        elapsed: Duration,
        container_alive: bool,
        /// The answer line the program printed, when it answered. Kept for a sample
        /// test's public record.
        answer: Option<Vec<u8>>,
    },
    /// The stop signal ended the call. The test has no verdict.
    Stopped,
}

impl Step {
    /// The parts of a ran test, or `None` when the run was stopped.
    fn into_ran(self) -> Option<(TestVerdict, Duration, bool, Option<Vec<u8>>)> {
        match self {
            Self::Ran {
                verdict,
                elapsed,
                container_alive,
                answer,
            } => Some((verdict, elapsed, container_alive, answer)),
            Self::Stopped => None,
        }
    }
}

/// Makes one call and turns what it observed into the test's verdict.
async fn run_test<S: BackendSession + 'static>(
    open: &mut Opened<S>,
    io: &mut CallIo,
    test: &PlannedTest,
    settings: &RunSettings,
    stop: &mut watch::Receiver<bool>,
) -> Result<Step, InfraError> {
    let limits = CallLimits {
        time: test.time_limit,
        run_deadline: settings.run_deadline,
        kill_slack: settings.kill_slack,
    };
    let started = Instant::now();
    let facts = run_call(open, io, u64::from(test.ordinal), &test.args, limits, stop).await?;
    let elapsed = started.elapsed();
    let container_alive = matches!(facts, CallFacts::Answer { .. });
    let (verdict, answer) = match facts {
        CallFacts::Answer { line, .. } => (
            judge_answer(
                &test.expected,
                u64::from(test.ordinal),
                &line,
                test.comparator,
            ),
            Some(line),
        ),
        CallFacts::Deadline => (TestVerdict::TimeLimit, None),
        CallFacts::OutputLimit => (TestVerdict::OutputLimit, None),
        CallFacts::ExtraOutput => (
            TestVerdict::WrongAnswer(WrongReason::Protocol(ProtocolViolation::ExtraOutput)),
            None,
        ),
        CallFacts::DriverExited { oom_kill, .. } if oom_kill > 0 => {
            (TestVerdict::MemoryLimit, None)
        }
        CallFacts::DriverExited { .. } => (TestVerdict::RuntimeError, None),
        CallFacts::Stopped => return Ok(Step::Stopped),
    };
    Ok(Step::Ran {
        verdict,
        elapsed,
        container_alive,
        answer,
    })
}

/// Opens the next container, numbering it so its name is unique within the run.
async fn open_next<B: Backend>(
    backend: &B,
    lang: &LanguageDefinition,
    archive: &[u8],
    session_spec: &impl Fn(u32) -> Result<SessionSpec, InfraError>,
    sessions_opened: &mut u32,
) -> Result<Opened<B::Session>, SessionError> {
    *sessions_opened += 1;
    let spec = session_spec(*sessions_opened).map_err(SessionError::Infrastructure)?;
    open_session(backend, &spec, lang, archive).await
}

/// Maps a session error to the infrastructure error the run returns. A compile failure
/// after the first container is an internal fault, because the same source compiled before.
fn into_infra(error: SessionError) -> InfraError {
    match error {
        SessionError::Infrastructure(error) => error,
        SessionError::PolicyViolations(violations) => {
            InfraError::PolicyRejected(format!("{violations:?}"))
        }
        SessionError::CompileFailed(failure) => InfraError::Unexpected(format!(
            "compile failed when a container was restarted: {failure:?}"
        )),
    }
}
