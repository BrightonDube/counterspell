//! Sandbox checks through `PackRunner` (SBX-01 to SBX-04, SBX-09).
//!
//! These checks run only after every static check passes, so a broken pack never
//! reaches a runner. The runner is the only way pack code runs. This module asks it
//! for results and compares them with the authored data.
//!
//! [`run_checks`] is the one implementation of the code-running pack checks. The
//! validator ([`check`]) and the catalogue import both call it, so the hidden set a
//! pack is validated against is the hidden set the import stores. Hidden inputs are
//! generated in memory. Their expected values are never written out by this module,
//! and no hidden input appears in a finding or a diagnostic.

use crate::codes;
use crate::ghosts::{self, SandboxGhost};
use crate::hidden;
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
use counterspell_engine::generate::{CandidateStream, GeneratorConfig, Origin};
use counterspell_engine::shrink::{shrink, Probe};
use counterspell_engine::{
    classify, compare_size, Budget, Classification, Comparator, FailureKind, FailurePredicate,
    Observation, ProgramResult, Schema,
};
use serde_json::Value;
use std::cmp::Ordering;
use std::collections::HashSet;
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

/// Says whether a generated case is retired, from its input and the reference's answer.
pub type RetiredCheck<'a> = &'a dyn Fn(&Value, &Value) -> bool;

/// The seeded, generated part of the hidden set.
pub struct Generated<'a> {
    /// The candidate-stream seed.
    pub seed: u64,
    /// How many generated hidden cases to produce.
    pub count: usize,
    /// `Some` after a recorded rotation. The seed-independent enumerated prefix is then
    /// skipped, and a generated case the predicate marks retired (by input and the
    /// reference's answer) is drawn again.
    pub retired: Option<RetiredCheck<'a>>,
}

/// Everything the code-running checks need for one pack version, already parsed.
pub struct SandboxInputs<'a> {
    /// The pack comparator.
    pub comparator: Comparator,
    /// The launch return-type name (`int`, `list[int]`, …) for the shape check.
    pub returns: &'a str,
    /// The per-call time limit from `limits.time_ms`.
    pub limit: Duration,
    /// The engine input schema: the candidates' domain and the shrinker's.
    pub schema: &'a Schema,
    /// The public samples `(input, expected)`.
    pub samples: &'a [(Value, Value)],
    /// The curated hidden cases `(input, authored expected)` from `hidden.toml`.
    pub curated: &'a [(Value, Value)],
    /// The generated hidden cases, or `None` when the pack declares no generator.
    pub generated: Option<Generated<'a>>,
    /// The known ghosts to reproduce.
    pub ghosts: &'a [SandboxGhost],
}

/// How far [`run_checks`] goes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Scope {
    /// Stop at the first error and skip the warning-only minimality check (the import).
    FirstError,
    /// Run every check, SBX-04 included, and report every finding (the validator).
    Everything,
}

/// Which hidden case a finding or a produced case belongs to.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HiddenSource {
    /// A curated case from `hidden.toml`.
    Curated,
    /// A case generated from the seeded candidate stream.
    Generated,
}

/// One hidden case and the reference's own answer for it.
#[derive(Clone, Debug, PartialEq)]
pub struct HiddenCase {
    /// Where the input came from.
    pub source: HiddenSource,
    /// The case input.
    pub input: Value,
    /// The reference's answer. For a curated case it equals the authored value under the
    /// comparator, or the case is a finding instead.
    pub expected: Value,
}

/// One pack problem found by [`run_checks`]. Findings carry indices and ids only, never a
/// hidden value.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Finding {
    /// SBX-01: the reference does not return this sample's expected value.
    SampleDisagrees {
        /// The sample's index.
        index: usize,
    },
    /// SBX-09: the reference does not return this curated case's expected value.
    CuratedDisagrees {
        /// The curated case's index in `hidden.toml`.
        index: usize,
    },
    /// SBX-02: the reference crashes, times out or returns the wrong shape on a hidden input.
    ReferenceFailed {
        /// Curated or generated.
        source: HiddenSource,
        /// The case's index inside its own list.
        index: usize,
    },
    /// The candidate stream holds fewer unique inputs than `hidden_count` asks for.
    TooFewGenerated {
        /// The requested number of generated cases.
        wanted: usize,
        /// How many unique inputs the stream produced.
        got: usize,
    },
    /// SBX-03: a known ghost does not reproduce its recorded failure.
    GhostNotReproduced {
        /// The ghost's index in `known_ghosts.toml`.
        index: usize,
        /// The ghost's id.
        id: String,
    },
    /// SBX-04 (warning): a smaller input fails the same way as this known ghost.
    GhostNotMinimal {
        /// The ghost's index in `known_ghosts.toml`.
        index: usize,
        /// The ghost's id.
        id: String,
    },
}

impl Finding {
    /// Whether this finding fails the pack. Only SBX-04 is a warning.
    #[must_use]
    pub fn is_error(&self) -> bool {
        !matches!(self, Self::GhostNotMinimal { .. })
    }
}

/// What [`run_checks`] found and the hidden cases the reference answered.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct SandboxOutcome {
    /// Every finding, in check order: samples, curated, generated, ghosts.
    pub findings: Vec<Finding>,
    /// The hidden set in write order: curated first, then generated.
    pub hidden: Vec<HiddenCase>,
}

/// Runs the code-running pack checks through `runner`.
///
/// The checks run in a fixed order: samples (SBX-01), curated cases (SBX-02, SBX-09),
/// generated cases (SBX-02 and the shortfall), then known ghosts (SBX-03, and SBX-04
/// under [`Scope::Everything`]). A generated input that repeats a curated input is drawn
/// again, so no hidden case repeats.
///
/// # Errors
///
/// Returns the first [`RunnerError`]. A runner fault is infrastructure, never a finding.
pub fn run_checks(
    inputs: &SandboxInputs<'_>,
    scope: Scope,
    runner: &mut dyn PackRunner,
) -> Result<SandboxOutcome, RunnerError> {
    let mut run = Run {
        inputs,
        scope,
        runner,
        return_kind: LaunchType::parse(inputs.returns),
        seen: HashSet::new(),
        outcome: SandboxOutcome::default(),
    };
    let _stopped = run.samples()? || run.curated()? || run.generated()? || run.ghosts()?;
    Ok(run.outcome)
}

/// One [`run_checks`] call in progress. Each stage returns `true` when the scope says stop.
struct Run<'i, 'r> {
    inputs: &'i SandboxInputs<'i>,
    scope: Scope,
    runner: &'r mut dyn PackRunner,
    return_kind: Option<LaunchType>,
    /// Hidden inputs already in the set, as compact JSON text. `serde_json` keeps object
    /// keys sorted, so equal values have equal text.
    seen: HashSet<String>,
    outcome: SandboxOutcome,
}

impl Run<'_, '_> {
    /// Records a finding; `true` when the scope stops at it.
    fn found(&mut self, finding: Finding) -> bool {
        let stop = self.scope == Scope::FirstError && finding.is_error();
        self.outcome.findings.push(finding);
        stop
    }

    /// Runs the reference on `input` within the pack limit.
    fn reference(&mut self, input: &Value) -> Result<ProgramResult, RunnerError> {
        self.runner.run(Target::Reference, input, self.inputs.limit)
    }

    /// The reference's answer when it is a well-shaped return for the declared type.
    fn shaped(&self, result: ProgramResult) -> Option<Value> {
        match result {
            ProgramResult::Returned(value)
                if self
                    .return_kind
                    .is_some_and(|kind| accepts_return(kind, &value)) =>
            {
                Some(value)
            }
            _ => None,
        }
    }

    /// SBX-01: the reference must return each sample's expected value.
    fn samples(&mut self) -> Result<bool, RunnerError> {
        for (index, (input, expected)) in self.inputs.samples.iter().enumerate() {
            let result = self.reference(input)?;
            let agrees = matches!(
                &result,
                ProgramResult::Returned(got) if self.inputs.comparator.equal(expected, got)
            );
            if !agrees && self.found(Finding::SampleDisagrees { index }) {
                return Ok(true);
            }
        }
        Ok(false)
    }

    /// SBX-02 and SBX-09: each curated case must get a well-shaped answer equal to the
    /// authored one. The stored expected value is the reference's own answer.
    fn curated(&mut self) -> Result<bool, RunnerError> {
        for (index, (input, authored)) in self.inputs.curated.iter().enumerate() {
            self.seen.insert(input.to_string());
            let result = self.reference(input)?;
            let Some(expected) = self.shaped(result) else {
                let finding = Finding::ReferenceFailed {
                    source: HiddenSource::Curated,
                    index,
                };
                if self.found(finding) {
                    return Ok(true);
                }
                continue;
            };
            if !self.inputs.comparator.equal(authored, &expected) {
                if self.found(Finding::CuratedDisagrees { index }) {
                    return Ok(true);
                }
                continue;
            }
            self.outcome.hidden.push(HiddenCase {
                source: HiddenSource::Curated,
                input: input.clone(),
                expected,
            });
        }
        Ok(false)
    }

    /// SBX-02 on `count` unique generated inputs, then the shortfall check.
    fn generated(&mut self) -> Result<bool, RunnerError> {
        let Some(generated) = &self.inputs.generated else {
            return Ok(false);
        };
        let rotated = generated.retired.is_some();
        let stream = CandidateStream::new(
            self.inputs.schema.clone(),
            generated.seed,
            Vec::new(),
            Vec::new(),
            GeneratorConfig::default(),
        );
        // The enumerated prefix is independent of the seed. After a recorded rotation,
        // consume it without selecting it, so the seeded tail also excludes those inputs.
        let candidates =
            stream.filter(|candidate| !rotated || candidate.origin == Origin::Generated);
        // The stream yields unique candidates; a tiny domain could otherwise loop on
        // duplicates forever, so the attempt count is bounded before the shortfall check.
        let attempts = generated.count.saturating_mul(8).max(16);
        let mut tried = 0_usize;
        for candidate in candidates.take(attempts) {
            if tried == generated.count {
                break;
            }
            let input = candidate.value;
            if !self.seen.insert(input.to_string()) {
                continue;
            }
            let result = self.reference(&input)?;
            let Some(expected) = self.shaped(result) else {
                let finding = Finding::ReferenceFailed {
                    source: HiddenSource::Generated,
                    index: tried,
                };
                tried += 1;
                if self.found(finding) {
                    return Ok(true);
                }
                continue;
            };
            if generated
                .retired
                .is_some_and(|retired| retired(&input, &expected))
            {
                continue;
            }
            tried += 1;
            self.outcome.hidden.push(HiddenCase {
                source: HiddenSource::Generated,
                input,
                expected,
            });
        }
        if tried < generated.count {
            let finding = Finding::TooFewGenerated {
                wanted: generated.count,
                got: tried,
            };
            return Ok(self.found(finding));
        }
        Ok(false)
    }

    /// SBX-03 for each known ghost, and SBX-04 under [`Scope::Everything`].
    fn ghosts(&mut self) -> Result<bool, RunnerError> {
        for (index, ghost) in self.inputs.ghosts.iter().enumerate() {
            let known = &ghost.known;
            let observation = self.observe(&ghost.program, &known.input)?;
            let predicate = match classify(&observation, &self.inputs.comparator) {
                Classification::Fail(predicate)
                    if known.matches(&known.input, &predicate, &observation.user) =>
                {
                    predicate
                }
                _ => {
                    let id = known.id.clone();
                    if self.found(Finding::GhostNotReproduced { index, id }) {
                        return Ok(true);
                    }
                    continue;
                }
            };
            // The engine does not shrink timeouts, so there is nothing to minimise.
            if self.scope == Scope::Everything
                && known.kind != FailureKind::Timeout
                && !self.minimal(ghost, &predicate)?
            {
                let id = known.id.clone();
                self.found(Finding::GhostNotMinimal { index, id });
            }
        }
        Ok(false)
    }

    /// SBX-04: `false` when the shrinker finds a smaller input that fails the same way.
    fn minimal(
        &mut self,
        ghost: &SandboxGhost,
        predicate: &FailurePredicate,
    ) -> Result<bool, RunnerError> {
        let comparator = self.inputs.comparator;
        let mut fault = None;
        let outcome = shrink(
            self.inputs.schema,
            ghost.known.input.clone(),
            Budget::default().max_shrink_evals,
            &mut |candidate: &Value| match self.observe(&ghost.program, candidate) {
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
        Ok(compare_size(&outcome.value, &ghost.known.input) != Ordering::Less)
    }

    /// Runs the program and the reference on one input, each within the pack limit.
    fn observe(&mut self, program: &str, input: &Value) -> Result<Observation, RunnerError> {
        let limit = self.inputs.limit;
        let user = self.runner.run(Target::Program(program), input, limit)?;
        let reference = self.runner.run(Target::Reference, input, limit)?;
        Ok(Observation { user, reference })
    }
}

/// Runs the sandbox checks on `pack` for the validator and pushes their diagnostics.
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
    let samples: Vec<(Value, Value)> = meta
        .samples
        .iter()
        .map(|sample| (sample.input.clone(), sample.expected.clone()))
        .collect();
    let curated = match pack.text(hidden::HIDDEN_FILE).map(hidden::curated_cases) {
        None => Vec::new(),
        Some(Ok(cases)) => cases,
        Some(Err(_)) => {
            // The parser's message can quote private case values, so it is not reported.
            report.push(Diagnostic::new(
                codes::SBX_09,
                hidden::HIDDEN_FILE,
                None,
                "hidden.toml does not parse, so its cases cannot be checked",
            ));
            Vec::new()
        }
    };
    let ghosts = pack
        .text("known_ghosts.toml")
        .map(ghosts::sandbox_ghosts)
        .unwrap_or_default();
    let generated = meta.generator.as_ref().and_then(|generator| {
        Some(Generated {
            seed: u64::try_from(generator.seed).ok()?,
            count: usize::try_from(generator.hidden_count).ok()?,
            retired: None,
        })
    });
    let inputs = SandboxInputs {
        comparator: meta.engine_comparator(),
        returns: &meta.returns,
        limit,
        schema: &meta.input_schema,
        samples: &samples,
        curated: &curated,
        generated,
        ghosts: &ghosts,
    };
    let outcome = run_checks(&inputs, Scope::Everything, runner)?;
    for finding in &outcome.findings {
        report.push(diagnostic(finding));
    }
    if let Some(bench) = &meta.bench {
        check_bench(meta, bench, runner, limit, report)?;
    }
    Ok(())
}

/// The validator diagnostic for one finding.
fn diagnostic(finding: &Finding) -> Diagnostic {
    match finding {
        Finding::SampleDisagrees { index } => Diagnostic::new(
            codes::SBX_01,
            "meta.toml",
            Some(&format!("/samples/{index}/expected")),
            "the reference does not return this sample's expected value",
        ),
        Finding::CuratedDisagrees { index } => Diagnostic::new(
            codes::SBX_09,
            hidden::HIDDEN_FILE,
            Some(&format!("/case/{index}/expected")),
            "the reference does not return this curated case's expected value",
        ),
        Finding::ReferenceFailed {
            source: HiddenSource::Curated,
            index,
        } => Diagnostic::new(
            codes::SBX_02,
            hidden::HIDDEN_FILE,
            Some(&format!("/case/{index}")),
            format!(
                "the reference crashes, times out or returns the wrong shape on curated case {}",
                index + 1
            ),
        ),
        Finding::ReferenceFailed {
            source: HiddenSource::Generated,
            index,
        } => Diagnostic::new(
            codes::SBX_02,
            "meta.toml",
            Some("/generator"),
            format!(
                "the reference crashes, times out or returns the wrong shape on hidden test {}",
                index + 1
            ),
        ),
        Finding::TooFewGenerated { wanted, got } => Diagnostic::new(
            codes::GEN_01,
            "generator.toml",
            Some("/generator/hidden_count"),
            format!("the generator yields only {got} unique hidden inputs, but hidden_count asks for {wanted}"),
        ),
        Finding::GhostNotReproduced { index, .. } => Diagnostic::new(
            codes::SBX_03,
            "known_ghosts.toml",
            Some(&format!("/ghost/{index}")),
            "the known ghost does not reproduce its recorded failure",
        ),
        Finding::GhostNotMinimal { index, .. } => Diagnostic::new(
            codes::SBX_04,
            "known_ghosts.toml",
            Some(&format!("/ghost/{index}/input")),
            "a smaller input fails the same way, so this known ghost is not minimal",
        ),
    }
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
