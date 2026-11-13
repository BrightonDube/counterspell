//! `PackRunner` trait and the engine Oracle adapter.
//!
//! The judge implements [`PackRunner`] through its rootless sandbox. This crate
//! never implements it on the host. A program's failure is a
//! [`ProgramResult`], and only an infrastructure fault is a [`RunnerError`].

use counterspell_engine::{Observation, Oracle, OracleError, ProgramResult};
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

/// Adapts a [`PackRunner`] and one incorrect program to the engine's [`Oracle`],
/// so the engine's shrinker and hunt can reuse the same runner.
pub struct ProgramOracle<'a> {
    runner: &'a mut dyn PackRunner,
    program: &'a str,
}

impl<'a> ProgramOracle<'a> {
    /// Builds an oracle that compares `program` with the reference.
    #[must_use]
    pub fn new(runner: &'a mut dyn PackRunner, program: &'a str) -> Self {
        Self { runner, program }
    }
}

impl Oracle for ProgramOracle<'_> {
    fn evaluate(&mut self, input: &Value, limit: Duration) -> Result<Observation, OracleError> {
        let user = self
            .runner
            .run(Target::Program(self.program), input, limit)
            .map_err(into_oracle_error)?;
        let reference = self
            .runner
            .run(Target::Reference, input, limit)
            .map_err(into_oracle_error)?;
        Ok(Observation { user, reference })
    }
}

/// Converts a runner fault into the engine's infrastructure error.
fn into_oracle_error(error: RunnerError) -> OracleError {
    match error {
        RunnerError::Infrastructure(message) => OracleError::new(message),
    }
}
