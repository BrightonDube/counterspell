//! The runner seam lives in the engine so the judge can implement it without depending on
//! Grimoire (counterspell-mcp-power T05, Brighton's 19 Nov decision).
//!
//! A runner written against the engine's trait must drive Grimoire's `ProgramOracle`, and
//! Grimoire's re-exported names must be the very same types. If the two ever became separate
//! copies, this file would stop compiling.

use counterspell_engine::runner::{PackRunner, RunnerError, Target};
use counterspell_engine::{Oracle, ProgramResult};
use counterspell_grimoire::runner::ProgramOracle;
use serde_json::{json, Value};
use std::time::Duration;

/// A runner whose incorrect program crashes and whose reference returns 2.
struct CrashingProgram;

impl PackRunner for CrashingProgram {
    fn run(
        &mut self,
        target: Target<'_>,
        _input: &Value,
        _limit: Duration,
    ) -> Result<ProgramResult, RunnerError> {
        match target {
            Target::Reference => Ok(ProgramResult::Returned(json!(2))),
            Target::Program(_) => Ok(ProgramResult::Crashed {
                exception_class: "IndexError".to_owned(),
                message: "list index out of range".to_owned(),
            }),
        }
    }

    fn bench(
        &mut self,
        _sized_inputs: &[(u64, Value)],
        _limit: Duration,
    ) -> Result<Vec<(u64, u64)>, RunnerError> {
        Err(RunnerError::Infrastructure("no bench here".to_owned()))
    }
}

/// A crash is a fact about the program, so it reaches the engine as an observation, while a
/// runner fault is an oracle error and never a verdict.
#[test]
fn an_engine_runner_drives_the_grimoire_oracle() {
    let mut runner = CrashingProgram;
    let mut oracle = ProgramOracle::new(&mut runner, "incorrect/crash.py");

    let observation = oracle
        .evaluate(&json!([1, 2]), Duration::from_secs(1))
        .expect("a crash is an observation, not an error");

    assert_eq!(observation.reference, ProgramResult::Returned(json!(2)));
    assert_eq!(
        observation.user,
        ProgramResult::Crashed {
            exception_class: "IndexError".to_owned(),
            message: "list index out of range".to_owned(),
        }
    );
}

/// Grimoire's `Target` and `RunnerError` are the engine's own types, not look-alikes.
#[test]
fn grimoire_re_exports_the_engine_types() {
    let target: counterspell_grimoire::runner::Target<'_> = Target::Program("a.py");
    let fault: counterspell_grimoire::runner::RunnerError =
        RunnerError::Infrastructure("daemon down".to_owned());
    assert_eq!(target, Target::Program("a.py"));
    assert_eq!(fault, RunnerError::Infrastructure("daemon down".to_owned()));
}
