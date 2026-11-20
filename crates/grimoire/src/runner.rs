//! `PackRunner` trait and the engine Oracle adapter.
//!
//! The judge implements [`PackRunner`] through its rootless sandbox. This crate
//! never implements it on the host. A program's failure is a
//! [`ProgramResult`], and only an infrastructure fault is a [`RunnerError`].

use counterspell_engine::{Observation, Oracle, OracleError};
use serde_json::Value;
use std::time::Duration;

pub use counterspell_engine::runner::{PackRunner, RunnerError, Target};

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
