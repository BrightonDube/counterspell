// A test double for `PackRunner`. It returns recorded outputs for the fixture
// programs and NEVER talks to a sandbox. Passing a dry-run harness on this
// runner is NOT evidence that the sandbox works - it only proves the static
// pipeline stages and state transitions behave (tdd-and-quality: the double
// must say what it is not).
#![allow(dead_code)]

use counterspell_engine::classify::ProgramResult;
use counterspell_engine::runner::{PackRunner, RunnerError, Target};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::time::Duration;

/// Canned answers keyed by `(target name, canonical input)`.
pub struct FakeRunner {
    /// Program name -> the canned result for any input.
    pub programs: HashMap<String, ProgramResult>,
    /// The canned reference result for any input.
    pub reference: ProgramResult,
    /// How many times `run` was called - the dry run asserts sandbox stages
    /// (S10/S11) never execute, so this stays zero for those stages.
    pub calls: usize,
}

impl FakeRunner {
    pub fn new(reference: Value) -> Self {
        Self {
            programs: HashMap::new(),
            reference: ProgramResult::Returned(reference),
            calls: 0,
        }
    }

    pub fn with_program(mut self, name: &str, result: ProgramResult) -> Self {
        self.programs.insert(name.to_owned(), result);
        self
    }
}

impl PackRunner for FakeRunner {
    fn run(
        &mut self,
        target: Target<'_>,
        _input: &Value,
        _limit: Duration,
    ) -> Result<ProgramResult, RunnerError> {
        self.calls += 1;
        let result = match target {
            Target::Reference => self.reference.clone(),
            Target::Program(name) => self
                .programs
                .get(name)
                .cloned()
                .unwrap_or(ProgramResult::Returned(json!(null))),
        };
        Ok(result)
    }

    fn bench(
        &mut self,
        _sized_inputs: &[(u64, Value)],
        _limit: Duration,
    ) -> Result<Vec<(u64, u64)>, RunnerError> {
        Ok(Vec::new())
    }
}
