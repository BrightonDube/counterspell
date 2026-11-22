//! The trusted comparator: the only code that can make a passing test.
//!
//! The program prints a raw answer. This module parses that answer, checks its shape
//! and compares its value with the expected value. Only [`judge_answer`] can produce
//! [`TestVerdict::Pass`], because the proof value [`Verified`] has a private field.
//!
//! A compile-time guarantee, checked by a doc test:
//!
//! ```compile_fail
//! use counterspell_judge::sandbox::compare::{TestVerdict, Verified};
//! // `Verified` has a private field, so code outside `compare` cannot build it.
//! let _ = TestVerdict::Pass(Verified(()));
//! ```

use std::fmt;

use counterspell_engine::Comparator;
use serde_json::Value;

use super::driver_protocol::{parse_answer_detail, ProtocolViolation};

/// A hidden or visible expected output. It is never printed: `Debug` shows only a
/// placeholder, so an expected value cannot reach a log or an error message.
pub struct Expected {
    value: Value,
}

impl Expected {
    /// Wraps an expected output.
    #[must_use]
    pub fn new(value: Value) -> Self {
        Self { value }
    }
}

impl fmt::Debug for Expected {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("Expected(<redacted>)")
    }
}

/// Proof that the comparator judged an answer equal. The field is private, so only
/// this module can create the value.
#[derive(Debug, PartialEq, Eq)]
pub struct Verified(());

/// Why an answer is wrong.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WrongReason {
    /// The line broke the protocol (not JSON, wrong id, extra key, and so on).
    Protocol(ProtocolViolation),
    /// The line was valid, and its value differs from the expected value.
    Different,
}

/// The verdict for one test. Only [`judge_answer`] builds `Pass`.
#[derive(Debug, PartialEq, Eq)]
pub enum TestVerdict {
    /// The answer is valid and equal to the expected value.
    Pass(Verified),
    /// The answer is wrong.
    WrongAnswer(WrongReason),
    /// The program stopped with an error before answering.
    RuntimeError,
    /// The per-test deadline passed.
    TimeLimit,
    /// The program ran out of memory.
    MemoryLimit,
    /// The program wrote more output than allowed.
    OutputLimit,
}

/// The comparator's full read of one answer: its verdict and the stderr embedded
/// in the same protocol line. Embedding the text in the answer makes it impossible
/// for delayed process stderr from an earlier call to be attributed to the next one.
#[derive(Debug)]
pub struct JudgedAnswer {
    /// The test verdict.
    pub verdict: TestVerdict,
    /// The bounded diagnostics the driver captured during this call.
    pub stderr: Option<Vec<u8>>,
}

/// Judges one raw answer line against the expected output and keeps its stderr.
#[must_use]
pub fn judge_answer_detail(
    expected: &Expected,
    call_id: u64,
    line: &[u8],
    comparator: Comparator,
) -> JudgedAnswer {
    match parse_answer_detail(line, call_id) {
        Err(violation) => JudgedAnswer {
            verdict: TestVerdict::WrongAnswer(WrongReason::Protocol(violation)),
            stderr: None,
        },
        Ok(answer) => {
            let verdict = if comparator.equal(&expected.value, &answer.out) {
                TestVerdict::Pass(Verified(()))
            } else {
                TestVerdict::WrongAnswer(WrongReason::Different)
            };
            JudgedAnswer {
                verdict,
                stderr: answer.stderr.map(String::into_bytes),
            }
        }
    }
}

/// Judges one raw answer line against the expected output.
///
/// The line passes only if it is one valid answer for `call_id` and its value is equal to
/// `expected` under `comparator`. Anything else is a wrong answer, however it is worded.
#[must_use]
pub fn judge_answer(
    expected: &Expected,
    call_id: u64,
    line: &[u8],
    comparator: Comparator,
) -> TestVerdict {
    judge_answer_detail(expected, call_id, line, comparator).verdict
}
