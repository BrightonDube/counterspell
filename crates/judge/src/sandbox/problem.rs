// SPDX-License-Identifier: MIT
//! The problem-tests seam (judge-sandbox T24, design section 14): how the executor
//! loads a problem's tests without knowing where problems live.
//!
//! The seam keeps expected values inside the judge. `Ok(None)` means the problem is not
//! usable — unpublished, unknown, or carrying something the judge cannot support — and
//! the run fails as `problem_unavailable`. A loader never guesses: data it cannot use
//! is an absent problem, and a broken store is a [`ProblemError`].

use std::future::Future;

use counterspell_common::familiar::class::ClassId;
use counterspell_common::status::RunMode;
use counterspell_engine::{Comparator, Schema};
use serde_json::Value;
use thiserror::Error;

/// A problem's bench declaration: the sizes to time and which parameter grows.
#[derive(Debug, Clone, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct BenchDecl {
    /// The input sizes the bench runs, in increasing order.
    pub sizes: Vec<u64>,
    /// The index of the signature parameter that grows with `n`.
    pub size_param: usize,
}

/// One test of a problem, as a loader reports it. `expected` never leaves the judge.
#[derive(Debug, Clone, PartialEq)]
pub struct TestCase {
    /// The test's position in the suite, starting at 1.
    pub ordinal: u32,
    /// Whether the user may see this test's values.
    pub is_sample: bool,
    /// The call arguments, in the signature's order.
    pub args: Vec<Value>,
    /// The expected output value.
    pub expected: Value,
}

/// A problem as the executor needs it: metadata, the comparator and the tests.
#[derive(Debug, Clone, PartialEq)]
pub struct ProblemSpec {
    /// The problem's slug.
    pub slug: String,
    /// The published version the tests belong to.
    pub version: u32,
    /// Public signature metadata, written to `signature.json` inside the sandbox.
    pub signature: Value,
    /// How answers are compared with the expected value.
    pub comparator: Comparator,
    /// A per-test limit that overrides the language default, when the problem sets one.
    pub time_limit_ms: Option<u32>,
    /// The tests for the requested mode, in ordinal order.
    pub tests: Vec<TestCase>,
    /// The reference Python source. Only `SandboxOracle` uses it; it is never sent to
    /// the user's container.
    pub reference_python: Option<String>,
    /// The declared bench, when the problem publishes one.
    pub bench: Option<BenchDecl>,
    /// The generator schema the bench sizes inputs with. Required with `bench`.
    pub generator_schema: Option<Schema>,
    /// The complexity class the problem author declared. Required with `bench`.
    pub reference_complexity: Option<ClassId>,
}

/// Why a problem could not be loaded. Always infrastructure, never the user's fault.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProblemError {
    /// The store failed, or its data could not be read.
    #[error("problem store failed: {0}")]
    Store(String),
}

/// Loads a problem's tests for one run.
///
/// `version` is the version the submission was pinned to when it was enqueued — a
/// submission keeps its pinned version even after the problem is unpublished (P9).
/// `mode` picks the slice: `Run` returns only the sample tests, `Submit` the whole
/// suite.
pub trait ProblemTests: Send + Sync + 'static {
    /// Loads `slug` at `version` for `mode`.
    ///
    /// # Errors
    ///
    /// Returns [`ProblemError`] when the store itself failed. A problem that is simply
    /// not there is `Ok(None)`, not an error.
    fn load(
        &self,
        slug: &str,
        version: u32,
        mode: RunMode,
    ) -> impl Future<Output = Result<Option<ProblemSpec>, ProblemError>> + Send;
}

/// Filters a loaded spec to the tests the mode runs. Mode `Run` keeps only samples.
#[must_use]
pub fn tests_for_mode(spec: &ProblemSpec, mode: RunMode) -> ProblemSpec {
    let mut spec = spec.clone();
    if mode == RunMode::Run {
        spec.tests.retain(|test| test.is_sample);
    }
    spec
}

/// The production loader until the problem catalogue exists: every problem is absent,
/// so every run fails as `problem_unavailable` rather than producing a verdict.
#[derive(Debug, Clone, Copy, Default)]
pub struct EmptyProblemTests;

impl ProblemTests for EmptyProblemTests {
    fn load(
        &self,
        _slug: &str,
        _version: u32,
        _mode: RunMode,
    ) -> impl Future<Output = Result<Option<ProblemSpec>, ProblemError>> + Send {
        std::future::ready(Ok(None))
    }
}
