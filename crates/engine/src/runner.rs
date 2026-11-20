//! The seam between pack tooling and whatever executes pack programs.
//!
//! [`PackRunner`] lives in the engine so the judge can implement it without depending on
//! Grimoire, and Grimoire and `counterspell-mcp` can both use it. The engine only declares the
//! contract. A program's failure is a [`ProgramResult`]; only an infrastructure fault is a
//! [`RunnerError`]. Nothing here runs code.

use crate::ProgramResult;
use serde_json::Value;
use std::time::Duration;

/// What to run: the trusted reference, or one incorrect program by its path in the pack.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Target<'a> {
    /// The trusted Python reference.
    Reference,
    /// A program under `incorrect/`, named by its path inside the pack.
    Program(&'a str),
}

/// A fault in the runner itself, such as an unreachable sandbox daemon.
///
/// A user or program failure is never a `RunnerError`. It is a [`ProgramResult`].
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RunnerError {
    /// The sandbox or reference infrastructure failed.
    Infrastructure(String),
}

/// Runs pack programs under the sandbox. The judge is the only real implementation.
pub trait PackRunner {
    /// Runs the reference or one incorrect program on one input, within `limit`.
    ///
    /// # Errors
    ///
    /// Returns [`RunnerError`] only for an infrastructure fault.
    fn run(
        &mut self,
        target: Target<'_>,
        input: &Value,
        limit: Duration,
    ) -> Result<ProgramResult, RunnerError>;

    /// Times the reference on inputs of growing size. Returns `(size, median_ns)` pairs.
    ///
    /// # Errors
    ///
    /// Returns [`RunnerError`] only for an infrastructure fault.
    fn bench(
        &mut self,
        sized_inputs: &[(u64, Value)],
        limit: Duration,
    ) -> Result<Vec<(u64, u64)>, RunnerError>;
}
