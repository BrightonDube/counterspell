//! Public records of a run: what the browser, the database and the logs may see.
//!
//! The output visibility rule (amendment A1) lives here. Sample tests may show their
//! input, expected value, answer and stderr. Once the first hidden test has run, every
//! later record carries only the ordinal, verdict and time, whatever the program printed.
//! Hidden tests never carry any output.

use serde::Serialize;

use super::compare::TestVerdict;
use super::sanitize::{sanitize_display, DETAIL_CAP};
use super::verdict::FinalVerdict;

/// The verdict of one test, without any data attached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum VerdictKind {
    /// The answer was correct.
    Pass,
    /// The answer was wrong, or the protocol was broken.
    WrongAnswer,
    /// The program failed before answering.
    RuntimeError,
    /// The test missed its deadline.
    TimeLimit,
    /// The program ran out of memory.
    MemoryLimit,
    /// The program wrote too much output.
    OutputLimit,
}

impl VerdictKind {
    /// The kind of a test verdict.
    #[must_use]
    pub fn of(verdict: &TestVerdict) -> Self {
        match verdict {
            TestVerdict::Pass(_) => Self::Pass,
            TestVerdict::WrongAnswer(_) => Self::WrongAnswer,
            TestVerdict::RuntimeError => Self::RuntimeError,
            TestVerdict::TimeLimit => Self::TimeLimit,
            TestVerdict::MemoryLimit => Self::MemoryLimit,
            TestVerdict::OutputLimit => Self::OutputLimit,
        }
    }
}

/// The raw text of one sample test. It is never kept for a hidden test.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SampleDetail {
    /// The input given to the program.
    pub input: Vec<u8>,
    /// The expected output.
    pub expected: Vec<u8>,
    /// The raw answer the program printed.
    pub got: Vec<u8>,
    /// The program's stderr, possibly truncated.
    pub stderr: Vec<u8>,
}

/// One executed test, as the run code observed it, before redaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestRun {
    /// The test's position in the problem.
    pub ordinal: u32,
    /// Whether this is a visible sample test.
    pub sample: bool,
    /// Time the test took, in milliseconds.
    pub time_ms: u64,
    /// The test's verdict.
    pub verdict: VerdictKind,
    /// Raw output. Only a sample test keeps it; a hidden test's detail is ignored.
    pub detail: Option<SampleDetail>,
}

/// A test as it may be shown to users and stored. Hidden tests carry no output at all.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum PublicTest {
    /// A visible sample test, with its sanitised and bounded detail.
    Sample {
        /// The test's position.
        ordinal: u32,
        /// The verdict.
        verdict: VerdictKind,
        /// Time in milliseconds.
        time_ms: u64,
        /// The input, as plain text, at most 4 KiB.
        input: String,
        /// The expected output, as plain text, at most 4 KiB.
        expected: String,
        /// The answer the program printed, as plain text, at most 4 KiB.
        got: String,
        /// The stderr, as plain text, at most 4 KiB.
        stderr: String,
    },
    /// A hidden test, or any test after the first hidden test. Verdict and time only.
    Hidden {
        /// The test's position.
        ordinal: u32,
        /// The verdict.
        verdict: VerdictKind,
        /// Time in milliseconds.
        time_ms: u64,
    },
}

/// Turns executed tests into public records, in the order they ran.
///
/// The builder remembers whether a hidden test has run. From then on it never shows
/// output, so a program cannot reveal a hidden test through a later visible test.
#[derive(Debug, Default)]
pub struct RecordBuilder {
    hidden_phase: bool,
}

impl RecordBuilder {
    /// A builder that has not yet seen a hidden test.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Redacts one executed test according to the visibility rule.
    pub fn record(&mut self, run: TestRun) -> PublicTest {
        if !run.sample {
            self.hidden_phase = true;
        }
        match run.detail {
            Some(detail) if run.sample && !self.hidden_phase => PublicTest::Sample {
                ordinal: run.ordinal,
                verdict: run.verdict,
                time_ms: run.time_ms,
                input: bounded(&detail.input),
                expected: bounded(&detail.expected),
                got: bounded(&detail.got),
                stderr: bounded(&detail.stderr),
            },
            _ => PublicTest::Hidden {
                ordinal: run.ordinal,
                verdict: run.verdict,
                time_ms: run.time_ms,
            },
        }
    }
}

fn bounded(bytes: &[u8]) -> String {
    sanitize_display(bytes, DETAIL_CAP).text
}

/// The public result of a run.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct FinalResult {
    /// The final verdict for the run.
    pub verdict: FinalVerdict,
    /// How many executed tests passed.
    pub passed: u32,
    /// How many tests the run planned.
    pub total: u32,
    /// The public record of each executed test, in run order.
    pub tests: Vec<PublicTest>,
}

/// Builds the public result from the verdict and the executed tests.
#[must_use]
pub fn build_result(verdict: FinalVerdict, total: u32, runs: Vec<TestRun>) -> FinalResult {
    let mut builder = RecordBuilder::new();
    let passed = runs
        .iter()
        .filter(|run| run.verdict == VerdictKind::Pass)
        .count();
    let tests = runs.into_iter().map(|run| builder.record(run)).collect();
    FinalResult {
        verdict,
        passed: u32::try_from(passed).unwrap_or(u32::MAX),
        total,
        tests,
    }
}
