//! Sandbox checks through `PackRunner` (SBX-01 to SBX-04).
//!
//! These checks run only after every static check passes, so a broken pack never
//! reaches a runner. The runner is the only way pack code runs. This module asks it
//! for results and compares them with the authored data.
//!
//! Hidden test inputs are generated in memory and only the shape of the reference's
//! result is checked. Their expected values are never written out, and no hidden
//! input appears in a diagnostic.

use crate::codes;
use crate::ghosts::{self, SandboxGhost};
use crate::meta::Meta;
use crate::pack::Pack;
use crate::report::{Diagnostic, Report};
use crate::runner::{PackRunner, RunnerError, Target};
use crate::samples::accepts_return;
use crate::signature::LaunchType;
use counterspell_engine::generate::{CandidateStream, GeneratorConfig};
use counterspell_engine::shrink::{shrink, Probe};
use counterspell_engine::{
    classify, compare_size, Budget, Classification, FailureKind, FailurePredicate, Observation,
    ProgramResult,
};
use serde_json::Value;
use std::cmp::Ordering;
use std::io;
use std::time::Duration;

/// Why the sandbox stage could not finish. A pack error is never one of these.
#[derive(Debug)]
pub enum SandboxFault {
    /// A pack file could not be read.
    Io(io::Error),
    /// The runner failed. This is infrastructure, not a pack error.
    Runner(RunnerError),
}

/// Runs SBX-01 to SBX-04 against `pack`. Stops early only on a runner fault.
///
/// Callers must run the static checks first. The time limit comes from
/// `meta.limits`, which the static checks have already range-checked.
///
/// # Errors
///
/// Returns the first [`RunnerError`] the runner reports.
pub fn check(
    meta: &Meta,
    pack: &Pack,
    runner: &mut dyn PackRunner,
    report: &mut Report,
) -> Result<(), RunnerError> {
    let limit = Duration::from_millis(u64::try_from(meta.limits.time_ms).unwrap_or_default());
    check_samples(meta, runner, limit, report)?;
    check_hidden(meta, runner, limit, report)?;
    let ghosts = pack
        .text("known_ghosts.toml")
        .map(ghosts::sandbox_ghosts)
        .unwrap_or_default();
    for (index, ghost) in ghosts.iter().enumerate() {
        check_ghost(index, ghost, meta, runner, limit, report)?;
    }
    Ok(())
}

/// SBX-01: the reference must return each authored sample's expected value.
fn check_samples(
    meta: &Meta,
    runner: &mut dyn PackRunner,
    limit: Duration,
    report: &mut Report,
) -> Result<(), RunnerError> {
    let comparator = meta.engine_comparator();
    for (index, sample) in meta.samples.iter().enumerate() {
        let result = runner.run(Target::Reference, &sample.input, limit)?;
        let agrees = matches!(
            &result,
            ProgramResult::Returned(got) if comparator.equal(&sample.expected, got)
        );
        if !agrees {
            report.push(Diagnostic::new(
                codes::SBX_01,
                "meta.toml",
                Some(&format!("/samples/{index}/expected")),
                "the reference does not return this sample's expected value",
            ));
        }
    }
    Ok(())
}

/// SBX-02: the reference must return a value of the declared shape on every hidden input.
///
/// Hidden inputs come from the engine's seeded stream, in memory. The check looks
/// at the shape of each result only.
fn check_hidden(
    meta: &Meta,
    runner: &mut dyn PackRunner,
    limit: Duration,
    report: &mut Report,
) -> Result<(), RunnerError> {
    let (Ok(seed), Ok(count)) = (
        u64::try_from(meta.generator.seed),
        usize::try_from(meta.generator.hidden_count),
    ) else {
        return Ok(());
    };
    let return_kind = LaunchType::parse(&meta.returns);
    let stream = CandidateStream::new(
        meta.input_schema.clone(),
        seed,
        Vec::new(),
        Vec::new(),
        GeneratorConfig::default(),
    );
    for (index, candidate) in stream.take(count).enumerate() {
        let result = runner.run(Target::Reference, &candidate.value, limit)?;
        let shaped = matches!(
            &result,
            ProgramResult::Returned(value)
                if return_kind.is_some_and(|kind| accepts_return(kind, value))
        );
        if !shaped {
            report.push(Diagnostic::new(
                codes::SBX_02,
                "meta.toml",
                Some("/generator"),
                format!(
                    "the reference crashes, times out or returns the wrong shape on hidden test {}",
                    index + 1
                ),
            ));
        }
    }
    Ok(())
}

/// SBX-03 and SBX-04 for one known ghost.
fn check_ghost(
    index: usize,
    ghost: &SandboxGhost,
    meta: &Meta,
    runner: &mut dyn PackRunner,
    limit: Duration,
    report: &mut Report,
) -> Result<(), RunnerError> {
    let known = &ghost.known;
    let comparator = meta.engine_comparator();
    let observation = observe(runner, &ghost.program, &known.input, limit)?;
    let predicate = match classify(&observation, &comparator) {
        Classification::Fail(predicate)
            if known.matches(&known.input, &predicate, &observation.user) =>
        {
            predicate
        }
        _ => {
            report.push(Diagnostic::new(
                codes::SBX_03,
                "known_ghosts.toml",
                Some(&format!("/ghost/{index}")),
                "the known ghost does not reproduce its recorded failure",
            ));
            return Ok(());
        }
    };
    // The engine does not shrink timeouts, so there is nothing to minimise.
    if known.kind == FailureKind::Timeout {
        return Ok(());
    }
    check_minimal(index, ghost, &predicate, meta, runner, limit, report)
}

/// SBX-04: a ghost is not minimal when the shrinker finds a smaller input that fails the same way.
fn check_minimal(
    index: usize,
    ghost: &SandboxGhost,
    predicate: &FailurePredicate,
    meta: &Meta,
    runner: &mut dyn PackRunner,
    limit: Duration,
    report: &mut Report,
) -> Result<(), RunnerError> {
    let comparator = meta.engine_comparator();
    let mut fault = None;
    let outcome = shrink(
        &meta.input_schema,
        ghost.known.input.clone(),
        Budget::default().max_shrink_evals,
        &mut |candidate: &Value| match observe(runner, &ghost.program, candidate, limit) {
            Ok(observation)
                if classify(&observation, &comparator)
                    == Classification::Fail(predicate.clone()) =>
            {
                Probe::Keep
            }
            Ok(_) => Probe::Reject,
            Err(error) => {
                fault = Some(error);
                Probe::Stop
            }
        },
    );
    if let Some(error) = fault {
        return Err(error);
    }
    if compare_size(&outcome.value, &ghost.known.input) == Ordering::Less {
        report.push(Diagnostic::new(
            codes::SBX_04,
            "known_ghosts.toml",
            Some(&format!("/ghost/{index}/input")),
            "a smaller input fails the same way, so this known ghost is not minimal",
        ));
    }
    Ok(())
}

/// Runs the user's program and the reference on one input, each within `limit`.
fn observe(
    runner: &mut dyn PackRunner,
    program: &str,
    input: &Value,
    limit: Duration,
) -> Result<Observation, RunnerError> {
    let user = runner.run(Target::Program(program), input, limit)?;
    let reference = runner.run(Target::Reference, input, limit)?;
    Ok(Observation { user, reference })
}
