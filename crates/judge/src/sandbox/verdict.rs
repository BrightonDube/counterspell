//! The final verdict for one run, decided from the per-test verdicts and the run facts.
//!
//! This module is pure. It does not know how a test ran; it only applies the precedence
//! in `design.md` section 9:
//!
//! 1. A stop signal gives `Stopped`.
//! 2. A failed compile gives `CompileError`.
//! 3. The first non-passing test, in ordinal order, gives its own verdict.
//! 4. A run cut short by the run cap gives `TimeLimit`.
//! 5. A complete run where every test passed gives `Accepted`. This is the only branch
//!    that can return `Accepted`.

use super::compare::TestVerdict;

/// The verdict for a whole run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum FinalVerdict {
    /// Every planned test ran and passed.
    Accepted,
    /// The program did not compile (or failed the syntax check).
    CompileError,
    /// The first failing test gave a wrong answer or a broken protocol line.
    WrongAnswer,
    /// The first failing test ended with an error.
    RuntimeError,
    /// The first failing test missed its deadline, or the run hit the run cap.
    TimeLimit,
    /// The first failing test ran out of memory.
    MemoryLimit,
    /// The first failing test wrote too much output.
    OutputLimit,
    /// The run was stopped on request. It has no verdict to report.
    Stopped,
    /// Fewer tests ran than planned, and nothing explains why. This is an internal fault.
    Incomplete,
}

/// The result of one test, as the run code observed it.
#[derive(Debug)]
pub struct ExecutedTest {
    ordinal: u32,
    verdict: TestVerdict,
}

impl ExecutedTest {
    /// Records the verdict of the test with this ordinal.
    #[must_use]
    pub fn new(ordinal: u32, verdict: TestVerdict) -> Self {
        Self { ordinal, verdict }
    }

    /// The test's position in the problem.
    #[must_use]
    pub fn ordinal(&self) -> u32 {
        self.ordinal
    }

    /// The verdict for this test.
    #[must_use]
    pub fn verdict(&self) -> &TestVerdict {
        &self.verdict
    }
}

/// Facts about the whole run, observed by the worker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RunFacts {
    /// Whether the compile step (or the syntax check) succeeded.
    pub compiled: bool,
    /// How many tests the run planned to execute.
    pub planned: u32,
    /// Whether the run cap stopped the run before every test finished.
    pub cut_by_cap: bool,
    /// Whether a stop signal ended the run.
    pub stopped: bool,
}

/// Decides the final verdict for a run. Pure: the same inputs always give the same output.
#[must_use]
pub fn decide(tests: &[ExecutedTest], facts: &RunFacts) -> FinalVerdict {
    if facts.stopped {
        return FinalVerdict::Stopped;
    }
    if !facts.compiled {
        return FinalVerdict::CompileError;
    }

    let first_failure = tests
        .iter()
        .filter_map(|test| failure_kind(test.verdict()).map(|kind| (test.ordinal(), kind)))
        .min_by_key(|(ordinal, _)| *ordinal);
    if let Some((_, kind)) = first_failure {
        return kind;
    }

    if facts.cut_by_cap {
        return FinalVerdict::TimeLimit;
    }

    let complete =
        facts.planned > 0 && u32::try_from(tests.len()).is_ok_and(|ran| ran == facts.planned);
    if complete {
        FinalVerdict::Accepted
    } else {
        FinalVerdict::Incomplete
    }
}

/// The final verdict a failing test implies, or `None` for a passing test.
fn failure_kind(verdict: &TestVerdict) -> Option<FinalVerdict> {
    match verdict {
        TestVerdict::Pass(_) => None,
        TestVerdict::WrongAnswer(_) => Some(FinalVerdict::WrongAnswer),
        TestVerdict::RuntimeError => Some(FinalVerdict::RuntimeError),
        TestVerdict::TimeLimit => Some(FinalVerdict::TimeLimit),
        TestVerdict::MemoryLimit => Some(FinalVerdict::MemoryLimit),
        TestVerdict::OutputLimit => Some(FinalVerdict::OutputLimit),
    }
}
