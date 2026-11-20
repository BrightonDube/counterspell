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
use crate::meta::{Bench, Meta};
use crate::pack::Pack;
use crate::report::{Diagnostic, Report};
use crate::runner::{PackRunner, RunnerError, Target};
use crate::samples::accepts_return;
use crate::signature::LaunchType;
use counterspell_engine::complexity::{
    bench_seed, fit, sized_input, Class as EngineClass, FitConfig, Mode as EngineMode,
    Point as EnginePoint,
};
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

impl SandboxFault {
    /// The SBX-06 diagnostic for this fault. It is an infrastructure failure (exit 2), never a pack error.
    #[must_use]
    pub fn diagnostic(&self) -> Diagnostic {
        let message = match self {
            Self::Io(error) => error.to_string(),
            Self::Runner(RunnerError::Infrastructure(message)) => message.clone(),
        };
        Diagnostic::new(codes::SBX_06, "sandbox", None, message)
    }
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
    if let Some(bench) = &meta.bench {
        check_bench(meta, bench, runner, limit, report)?;
    }
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

/// Times the reference on the declared bench sizes and checks the measured class (BEN-01, SBX-05).
///
/// # Errors
///
/// Returns the runner's fault. A fault is infrastructure, so the caller exits 2.
fn check_bench(
    meta: &Meta,
    bench: &Bench,
    runner: &mut dyn PackRunner,
    limit: Duration,
    report: &mut Report,
) -> Result<(), RunnerError> {
    let inputs = match bench_inputs(meta, bench) {
        Ok(inputs) => inputs,
        Err(diagnostic) => {
            report.push(diagnostic);
            return Ok(());
        }
    };
    if let Some(diagnostic) = narrow_span(&inputs) {
        report.push(diagnostic);
    }
    let points = runner.bench(&inputs, limit)?;
    if let Some(declared) = &meta.reference_complexity {
        if let Some(diagnostic) = compare_with_declared(declared, &points) {
            report.push(diagnostic);
        }
    }
    Ok(())
}

/// Builds one input per declared bench size with the engine's seeded generator (BEN-01 on failure).
fn bench_inputs(meta: &Meta, bench: &Bench) -> Result<Vec<(u64, Value)>, Diagnostic> {
    let Some(index) = meta
        .params
        .iter()
        .position(|param| param.name == bench.size_param)
    else {
        return Err(ben_01(format!(
            "size parameter `{}` is not a parameter of the function",
            bench.size_param
        )));
    };
    let version = u32::try_from(meta.pack_version)
        .map_err(|_| ben_01("pack_version must be a positive 32-bit number"))?;
    let mut inputs = Vec::with_capacity(bench.sizes.len());
    for &size in &bench.sizes {
        let n = u64::try_from(size).map_err(|_| ben_01(format!("size {size} is negative")))?;
        let seed = bench_seed(&meta.slug, version, n);
        let value = sized_input(&meta.input_schema, index, n, seed)
            .map_err(|error| ben_01(format!("size {n}: {error}")))?;
        inputs.push((n, value));
    }
    Ok(inputs)
}

/// How many times the largest bench size must exceed the lowest for the fit to reach `high`
/// confidence (R13.4).
const MIN_BENCH_SPAN: u64 = 32;

/// A BEN-02 warning when the largest bench size is less than 32 times the lowest.
fn narrow_span(inputs: &[(u64, Value)]) -> Option<Diagnostic> {
    let lowest = inputs.iter().map(|(n, _)| *n).min()?;
    let largest = inputs.iter().map(|(n, _)| *n).max()?;
    if lowest > 0 && largest / lowest >= MIN_BENCH_SPAN {
        return None;
    }
    let span = largest.checked_div(lowest).unwrap_or(0);
    Some(Diagnostic::new(
        codes::BEN_02,
        "meta.toml",
        Some("/bench/sizes"),
        format!(
            "the bench sizes span {span}x; use at least {MIN_BENCH_SPAN}x so the fit can reach high confidence"
        ),
    ))
}

/// A BEN-01 diagnostic for a bench size that cannot become an input.
fn ben_01(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(codes::BEN_01, "meta.toml", Some("/bench/sizes"), message)
}

/// Fits the timings and compares the class with the declared one (SBX-05). `None` when it is confirmed.
fn compare_with_declared(declared: &str, points: &[(u64, u64)]) -> Option<Diagnostic> {
    let expected = declared_class(declared)?;
    let fit_points: Vec<EnginePoint> = points
        .iter()
        .map(|&(n, median_ns)| EnginePoint {
            n,
            median_ns,
            spread: None,
        })
        .collect();
    let report = match fit(&fit_points, EngineMode::Full, &FitConfig::default()) {
        Ok(report) => report,
        Err(error) => {
            return Some(sbx_05(format!(
                "the bench timings cannot be fitted: {error}"
            )))
        }
    };
    let plausible = report
        .plausible
        .iter()
        .map(|class| class.id())
        .collect::<Vec<_>>()
        .join(", ");
    match report.class {
        Some(class) if class == expected => None,
        Some(class) => Some(sbx_05(format!(
            "the measured class {} differs from reference_complexity {declared} (plausible: {plausible})",
            class.id()
        ))),
        None => Some(sbx_05(format!(
            "the measured class is not confirmed, so {declared} is not checked (plausible: {plausible})"
        ))),
    }
}

/// An SBX-05 warning about the bench's measured class.
fn sbx_05(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new(
        codes::SBX_05,
        "meta.toml",
        Some("/reference_complexity"),
        message,
    )
}

/// The engine class named by a `reference_complexity` value, or `None` for an unknown name.
fn declared_class(text: &str) -> Option<EngineClass> {
    match text {
        "1" => Some(EngineClass::Constant),
        "log n" => Some(EngineClass::Logarithmic),
        "n" => Some(EngineClass::Linear),
        "n log n" => Some(EngineClass::Linearithmic),
        "n^2" => Some(EngineClass::Quadratic),
        "n^3" => Some(EngineClass::Cubic),
        "2^n" => Some(EngineClass::Exponential),
        _ => None,
    }
}

#[cfg(test)]
mod bench_tests {
    use super::*;
    use crate::fixtures;
    use counterspell_engine::ProgramResult;

    /// A runner whose bench returns the scripted result. Its `run` is never expected to be used.
    struct ScriptedBench(Result<Vec<(u64, u64)>, RunnerError>);

    impl PackRunner for ScriptedBench {
        fn run(
            &mut self,
            _target: Target<'_>,
            _input: &Value,
            _limit: Duration,
        ) -> Result<ProgramResult, RunnerError> {
            Err(RunnerError::Infrastructure(
                "run is not scripted".to_owned(),
            ))
        }

        fn bench(
            &mut self,
            _sized_inputs: &[(u64, Value)],
            _limit: Duration,
        ) -> Result<Vec<(u64, u64)>, RunnerError> {
            self.0.clone()
        }
    }

    /// The valid pack, declared as `reference_complexity = "n"`, with a bench over `values`.
    fn benched_meta(size_param: &str, sizes: &str) -> Meta {
        let text = fixtures::valid_text().replace(
            "comparator = \"exact\"\n",
            "comparator = \"exact\"\nreference_complexity = \"n\"\n",
        ) + &format!("\n[bench]\nsize_param = \"{size_param}\"\nsizes = {sizes}\n");
        fixtures::parse(&text)
    }

    /// Eight sizes spanning 128 times, so a clean curve can reach `Medium`.
    const EIGHT_SIZES: &str = "[8, 16, 32, 64, 128, 256, 512, 1024]";

    fn codes(report: &Report) -> Vec<&str> {
        report.diagnostics().iter().map(|d| d.code).collect()
    }

    fn limit() -> Duration {
        Duration::from_millis(2000)
    }

    #[test]
    fn a_bench_on_an_unknown_parameter_is_ben01_and_the_runner_is_not_called() {
        let meta = benched_meta("missing", EIGHT_SIZES);
        let bench = meta.bench.clone().expect("the fixture declares a bench");
        let mut runner = ScriptedBench(Err(RunnerError::Infrastructure("down".to_owned())));
        let mut report = Report::new("count-pairs");

        let outcome = check_bench(&meta, &bench, &mut runner, limit(), &mut report);

        assert_eq!(outcome, Ok(()));
        assert_eq!(codes(&report), vec!["BEN-01"]);
    }

    #[test]
    fn a_unique_bench_size_the_range_cannot_hold_is_ben01() {
        let text = fixtures::valid_text()
            .replace(
                "comparator = \"exact\"\n",
                "comparator = \"exact\"\nreference_complexity = \"n\"\n",
            )
            .replace(
                "{ type = \"int_array\", min_len = 0, max_len = 30, min = -50, max = 50 }",
                "{ type = \"int_array\", min_len = 0, max_len = 30, min = 0, max = 3, unique = true }",
            )
            + "\n[bench]\nsize_param = \"values\"\nsizes = [8, 16, 32]\n";
        let meta = fixtures::parse(&text);
        let bench = meta.bench.clone().expect("the fixture declares a bench");
        let mut runner = ScriptedBench(Err(RunnerError::Infrastructure("down".to_owned())));
        let mut report = Report::new("count-pairs");

        let outcome = check_bench(&meta, &bench, &mut runner, limit(), &mut report);

        assert_eq!(outcome, Ok(()));
        assert_eq!(codes(&report), vec!["BEN-01"]);
    }

    #[test]
    fn a_runner_fault_during_the_bench_is_returned_not_reported() {
        let meta = benched_meta("values", EIGHT_SIZES);
        let bench = meta.bench.clone().expect("the fixture declares a bench");
        let mut runner = ScriptedBench(Err(RunnerError::Infrastructure("down".to_owned())));
        let mut report = Report::new("count-pairs");

        let outcome = check_bench(&meta, &bench, &mut runner, limit(), &mut report);

        assert_eq!(outcome, Err(RunnerError::Infrastructure("down".to_owned())));
        assert_eq!(codes(&report), Vec::<&str>::new());
    }

    #[test]
    fn a_linear_bench_against_a_linear_reference_is_not_reported() {
        let meta = benched_meta("values", EIGHT_SIZES);
        let bench = meta.bench.clone().expect("the fixture declares a bench");
        let points: Vec<(u64, u64)> = [8_u64, 16, 32, 64, 128, 256, 512, 1024]
            .iter()
            .map(|&n| (n, 1000 * n))
            .collect();
        let mut runner = ScriptedBench(Ok(points));
        let mut report = Report::new("count-pairs");

        check_bench(&meta, &bench, &mut runner, limit(), &mut report).expect("no fault");

        assert!(report.diagnostics().is_empty(), "{:?}", codes(&report));
    }

    #[test]
    fn a_quadratic_bench_against_a_linear_reference_is_sbx05() {
        let meta = benched_meta("values", EIGHT_SIZES);
        let bench = meta.bench.clone().expect("the fixture declares a bench");
        let points: Vec<(u64, u64)> = [8_u64, 16, 32, 64, 128, 256, 512, 1024]
            .iter()
            .map(|&n| (n, 3 * n * n + 500))
            .collect();
        let mut runner = ScriptedBench(Ok(points));
        let mut report = Report::new("count-pairs");

        check_bench(&meta, &bench, &mut runner, limit(), &mut report).expect("no fault");

        assert_eq!(codes(&report), vec!["SBX-05"]);
        assert!(report.diagnostics()[0].message.contains("n^2"));
    }

    /// Linear timings for the given sizes, so only the span decides what is reported.
    fn linear(sizes: &[u64]) -> Vec<(u64, u64)> {
        sizes.iter().map(|&n| (n, 1000 * n)).collect()
    }

    /// R13.4: sizes spanning only 8 times cannot reach `high` confidence, so the validator warns
    /// (BEN-02) and still times the bench.
    #[test]
    fn bench_sizes_spanning_less_than_32_times_warn_ben02() {
        let meta = benched_meta("values", "[64, 128, 256, 512]");
        let bench = meta.bench.clone().expect("the fixture declares a bench");
        let mut runner = ScriptedBench(Ok(linear(&[64, 128, 256, 512])));
        let mut report = Report::new("count-pairs");

        check_bench(&meta, &bench, &mut runner, limit(), &mut report).expect("no fault");

        assert_eq!(codes(&report)[0], "BEN-02");
        assert!(report.diagnostics()[0].message.contains("8x"));
    }

    /// R13.4: a span of exactly 32 times is enough, so nothing is reported for clean linear data.
    #[test]
    fn bench_sizes_spanning_exactly_32_times_do_not_warn() {
        let sizes = [32_u64, 64, 128, 256, 512, 1024];
        let meta = benched_meta("values", "[32, 64, 128, 256, 512, 1024]");
        let bench = meta.bench.clone().expect("the fixture declares a bench");
        let mut runner = ScriptedBench(Ok(linear(&sizes)));
        let mut report = Report::new("count-pairs");

        check_bench(&meta, &bench, &mut runner, limit(), &mut report).expect("no fault");

        assert!(report.diagnostics().is_empty(), "{:?}", codes(&report));
    }

    #[test]
    fn flat_timings_are_reported_as_not_confirmed() {
        let meta = benched_meta("values", EIGHT_SIZES);
        let bench = meta.bench.clone().expect("the fixture declares a bench");
        let points: Vec<(u64, u64)> = [8_u64, 16, 32, 64, 128, 256, 512, 1024]
            .iter()
            .map(|&n| (n, 700))
            .collect();
        let mut runner = ScriptedBench(Ok(points));
        let mut report = Report::new("count-pairs");

        check_bench(&meta, &bench, &mut runner, limit(), &mut report).expect("no fault");

        assert_eq!(codes(&report), vec!["SBX-05"]);
        assert!(report.diagnostics()[0].message.contains("not confirmed"));
    }
}
