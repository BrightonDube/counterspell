//! The `PackRunner` seam and its `Oracle` adapter (T17).
//!
//! A fake runner stands in for the judge's sandbox. The tests prove how the adapter
//! routes calls and how it treats infrastructure errors, not how a sandbox runs code.

use counterspell_engine::{Observation, Oracle, ProgramResult};
use counterspell_grimoire::runner::{PackRunner, ProgramOracle, RunnerError, Target};
use serde_json::{json, Value};
use std::time::Duration;

/// A runner that answers from fixed values and records which target it was asked.
struct FakeRunner {
    user: Result<ProgramResult, RunnerError>,
    seen: Vec<String>,
}

impl PackRunner for FakeRunner {
    fn run(
        &mut self,
        target: Target<'_>,
        _input: &Value,
        _limit: Duration,
    ) -> Result<ProgramResult, RunnerError> {
        match target {
            Target::Reference => {
                self.seen.push("reference".to_owned());
                Ok(ProgramResult::Returned(json!(2)))
            }
            Target::Program(path) => {
                self.seen.push(path.to_owned());
                self.user.clone()
            }
        }
    }

    fn bench(
        &mut self,
        _sized_inputs: &[(u64, Value)],
        _limit: Duration,
    ) -> Result<Vec<(u64, u64)>, RunnerError> {
        Ok(Vec::new())
    }
}

#[test]
fn adapter_runs_the_program_and_the_reference_on_the_same_input() {
    let mut runner = FakeRunner {
        user: Ok(ProgramResult::Returned(json!(1))),
        seen: Vec::new(),
    };
    let mut oracle = ProgramOracle::new(&mut runner, "incorrect/same-index.py");

    let observation: Observation = oracle
        .evaluate(&json!([[0], 0]), Duration::from_secs(1))
        .expect("the fake runner is healthy");

    assert_eq!(observation.user, ProgramResult::Returned(json!(1)));
    assert_eq!(observation.reference, ProgramResult::Returned(json!(2)));
    assert_eq!(runner.seen, ["incorrect/same-index.py", "reference"]);
}

#[test]
fn a_crash_is_a_program_result_not_an_oracle_error() {
    let crash = ProgramResult::Crashed {
        exception_class: "ValueError".to_owned(),
        message: "empty".to_owned(),
    };
    let mut runner = FakeRunner {
        user: Ok(crash.clone()),
        seen: Vec::new(),
    };
    let mut oracle = ProgramOracle::new(&mut runner, "incorrect/empty-min.py");

    let observation = oracle
        .evaluate(&json!([[], 0]), Duration::from_secs(1))
        .expect("a crash is data, not infrastructure");

    assert_eq!(observation.user, crash);
}

#[test]
fn infra_fault_is_oracle_error_not_user_result() {
    let mut runner = FakeRunner {
        user: Err(RunnerError::Infrastructure("daemon unreachable".to_owned())),
        seen: Vec::new(),
    };
    let mut oracle = ProgramOracle::new(&mut runner, "incorrect/same-index.py");

    let outcome = oracle.evaluate(&json!([[0], 0]), Duration::from_secs(1));

    let error = outcome.expect_err("an infrastructure fault must not look like a wrong answer");
    assert_eq!(error.message(), "daemon unreachable");
}
