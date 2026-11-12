//! Ghost hunt orchestration: the request, the report, and the search phase.
//!
//! A hunt searches candidate inputs for the first failure against the trusted
//! reference, then shrinks it (added in a later step). The engine performs no
//! I/O. Every program call goes through the [`Oracle`] it is given.

use crate::autopsy::{build_autopsy, Autopsy, KnownGhost};
use crate::budget::Budget;
use crate::classify::{classify, Classification, FailureKind, FailurePredicate, ProgramResult};
use crate::compare::Comparator;
use crate::generate::{CandidateStream, GeneratorConfig, Origin};
use crate::oracle::{Clock, Oracle, OracleError};
use crate::rng::derive_seed;
use crate::schema::Schema;
use crate::shrink::{shrink, Probe, ShrinkReport, ShrinkStep, ShrinkStop};
use crate::size::{size_of, SizeMetric};
use crate::{ENGINE_VERSION, GENERATOR_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{self, Display, Formatter};
use std::time::Duration;

/// The longest identity string the hunt accepts, in bytes.
const MAX_IDENTITY_BYTES: usize = 256;
/// Reference failures in one hunt before the reference is treated as unreliable.
const MAX_REFERENCE_FAILURES: u32 = 3;

/// Identifies the submission and problem a hunt is for.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HuntIdentity {
    /// The problem's stable slug.
    pub problem_slug: String,
    /// The SHA-256 of the submitted code, as hex.
    pub code_sha256: String,
    /// The submission's language.
    pub language: String,
    /// The problem version the hunt ran against.
    pub problem_version: u32,
    /// The runtime image digest the Oracle ran in.
    pub image_digest: String,
}

/// Everything one hunt needs. Build it with [`HuntRequest::new`] and set fields.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HuntRequest {
    /// The input shape.
    pub schema: Schema,
    /// How user and reference results are compared.
    pub comparator: Comparator,
    /// The submission and problem identity.
    pub identity: HuntIdentity,
    /// Replaces the derived seed, for replay and tooling.
    pub seed_override: Option<u64>,
    /// The failure to look for. `None` takes the first wrong answer or crash.
    pub target: Option<FailurePredicate>,
    /// Public sample inputs from the pack.
    pub samples: Vec<Value>,
    /// Authored known ghosts from the pack.
    pub known_ghosts: Vec<KnownGhost>,
    /// Concept tags copied into the autopsy.
    pub concept_tags: Vec<String>,
    /// The limits for this hunt.
    pub budget: Budget,
}

impl HuntRequest {
    /// Creates a request with no samples, known ghosts or target, and the default budget.
    #[must_use]
    pub fn new(schema: Schema, comparator: Comparator, identity: HuntIdentity) -> Self {
        Self {
            schema,
            comparator,
            identity,
            seed_override: None,
            target: None,
            samples: Vec::new(),
            known_ghosts: Vec::new(),
            concept_tags: Vec::new(),
            budget: Budget::default(),
        }
    }
}

/// The inputs that make a hunt's result reproducible.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Provenance {
    /// The seed the candidate stream used.
    pub seed: u64,
    /// The candidate-generation version.
    pub generator_version: String,
    /// The engine version.
    pub engine_version: String,
    /// The problem version echoed from the request.
    pub problem_version: u32,
    /// The image digest echoed from the request.
    pub image_digest: String,
}

/// Counts gathered during one hunt.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HuntStats {
    /// Oracle calls made by the search, shrink and verification stages.
    pub oracle_calls: u32,
    /// Candidates the search stage evaluated.
    pub candidates_evaluated: u32,
    /// Known ghosts and samples skipped because they broke the schema.
    pub invalid_seeds_skipped: u32,
    /// Timeouts seen during search that were not the target.
    pub timeouts_seen: u32,
    /// Candidates whose reference run did not return a value.
    pub reference_failures: u32,
}

/// A failure the hunt has found and may report.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Ghost {
    /// The failure this ghost reproduces.
    pub predicate: FailurePredicate,
    /// The input, shrunk when shrinking ran.
    pub input: Value,
    /// The reference value, when the reference returned one.
    pub expected: Option<Value>,
    /// The user's raw result on `input`.
    pub got: ProgramResult,
    /// The stage that first produced the failing input.
    pub found_via: Origin,
    /// The size metric of `input`.
    pub size: SizeMetric,
    /// The shrink trace.
    pub shrink: ShrinkReport,
    /// Whether shrinking stopped on a budget.
    pub budget_exhausted: bool,
    /// How many times the final input was confirmed.
    pub verification_runs: u32,
    /// The Bug Autopsy for this ghost.
    pub autopsy: Autopsy,
}

impl Ghost {
    /// Reports whether the input was shrunk, as opposed to a timeout ghost left as found.
    #[must_use]
    pub fn is_minimized(&self) -> bool {
        self.shrink.stop != ShrinkStop::NotAttempted
    }
}

/// Re-checks a ghost with a fresh Oracle run.
///
/// The ghost holds only if its input still fits the schema and the Oracle
/// reproduces the same failure, expected value and actual result each run.
#[must_use]
pub fn verify_ghost(
    req: &HuntRequest,
    ghost: &Ghost,
    oracle: &mut dyn Oracle,
    clock: &dyn Clock,
) -> bool {
    if req.schema.contains(&ghost.input).is_err() {
        return false;
    }
    let deadline = clock.now().saturating_add(req.budget.total_wall);
    let mut stats = HuntStats::default();
    match check_input(
        req,
        &ghost.input,
        &ghost.predicate,
        oracle,
        clock,
        deadline,
        &mut stats,
    ) {
        Ok(Check::Confirmed(run)) => {
            run.expected == ghost.expected && run.got.sanitised() == ghost.got
        }
        _ => false,
    }
}

/// Why a hunt found no ghost. None of these says the solution is correct.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum NotFoundReason {
    /// The candidate budget ran out first.
    CandidateBudget,
    /// The time budget ran out first.
    WallTime,
    /// The generator had no more distinct inputs to try.
    CandidatesExhausted,
    /// A failure was seen, but it did not fail again when re-run.
    Unreproducible,
}

impl Display for NotFoundReason {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        let text = match self {
            Self::CandidateBudget => "No failing input found within the search budget.",
            Self::WallTime => "No failing input found before the time budget ran out.",
            Self::CandidatesExhausted => {
                "No failing input found among the inputs the generator could produce."
            }
            Self::Unreproducible => "A failing input was seen but did not fail again when re-run.",
        };
        formatter.write_str(text)
    }
}

/// Why a hunt stopped with an infrastructure or request problem.
///
/// The judge maps every variant to an internal error, never to a wrong answer.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum AbortReason {
    /// The Oracle reported an infrastructure failure.
    OracleFailure(String),
    /// The reference kept failing, so its expected values cannot be trusted.
    ReferenceUnreliable,
    /// The request was invalid. The message says what was wrong.
    InvalidRequest(String),
}

/// The three possible ends of a hunt.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum HuntOutcome {
    /// A verified ghost.
    Found(Box<Ghost>),
    /// No ghost, with the reason.
    NotFound(NotFoundReason),
    /// The hunt could not finish safely.
    Aborted(AbortReason),
}

/// The full result of a hunt. Every hunt returns one, with provenance.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct HuntReport {
    /// What the hunt ended with.
    pub outcome: HuntOutcome,
    /// The inputs that make the result reproducible.
    pub provenance: Provenance,
    /// The budget the hunt ran under.
    pub budget: Budget,
    /// Counts from the hunt.
    pub stats: HuntStats,
}

/// The first failing input found by the search stage, before shrinking.
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct FirstFailure {
    /// The failing input.
    pub input: Value,
    /// The failure the input shows.
    pub predicate: FailurePredicate,
    /// The stage that produced the input.
    pub origin: Origin,
}

/// The result of the search stage: a failure to shrink, or a final outcome.
pub(crate) enum Search {
    /// A failure was found.
    Found(FirstFailure),
    /// The search ended without a failure to shrink.
    Ended(HuntOutcome),
}

/// Runs a hunt and returns a report.
///
/// The search finds the first failure the request targets. A found failure is
/// then shrunk, confirmed by re-running it, and reported as a ghost only when
/// it reproduces. Anything that cannot be confirmed is reported as not found.
pub fn hunt(req: &HuntRequest, oracle: &mut dyn Oracle, clock: &dyn Clock) -> HuntReport {
    let hunt_start = clock.now();
    let seed = req.seed_override.unwrap_or_else(|| {
        derive_seed(
            &req.identity.problem_slug,
            &req.identity.code_sha256,
            &req.identity.language,
        )
    });
    let provenance = Provenance {
        seed,
        generator_version: GENERATOR_VERSION.to_owned(),
        engine_version: ENGINE_VERSION.to_owned(),
        problem_version: req.identity.problem_version,
        image_digest: req.identity.image_digest.clone(),
    };
    let mut stats = HuntStats::default();
    let outcome = if let Some(reason) = invalid_reason(req) {
        HuntOutcome::Aborted(AbortReason::InvalidRequest(reason))
    } else {
        match search(req, seed, oracle, clock, &mut stats) {
            Search::Ended(outcome) => outcome,
            Search::Found(first) => finish(req, first, oracle, clock, hunt_start, &mut stats),
        }
    };
    HuntReport {
        outcome,
        provenance,
        budget: req.budget,
        stats,
    }
}

/// Shrinks a found failure, confirms the result, and builds the ghost.
fn finish(
    req: &HuntRequest,
    first: FirstFailure,
    oracle: &mut dyn Oracle,
    clock: &dyn Clock,
    hunt_start: Duration,
    stats: &mut HuntStats,
) -> HuntOutcome {
    let total_deadline = hunt_start.saturating_add(req.budget.total_wall);
    let shrunk = if first.predicate.kind == FailureKind::Timeout {
        not_shrunk(&first.input)
    } else {
        match shrink_failure(req, &first, oracle, clock, hunt_start, stats) {
            Ok(report) => report,
            Err(error) => {
                return HuntOutcome::Aborted(AbortReason::OracleFailure(error.message().to_owned()))
            }
        }
    };

    // Newest trace entry first: the final input, then the earlier ones it came from.
    let candidates: Vec<Value> = shrunk
        .steps
        .iter()
        .rev()
        .map(|step| step.input.clone())
        .collect();
    let mut confirmed = None;
    for candidate in candidates {
        match check_input(
            req,
            &candidate,
            &first.predicate,
            oracle,
            clock,
            total_deadline,
            stats,
        ) {
            Ok(Check::Confirmed(run)) => {
                confirmed = Some((candidate, run));
                break;
            }
            Ok(Check::Unreproduced) => {}
            Ok(Check::OutOfTime) => return HuntOutcome::NotFound(NotFoundReason::WallTime),
            Err(error) => {
                return HuntOutcome::Aborted(AbortReason::OracleFailure(error.message().to_owned()))
            }
        }
    }
    let Some((input, run)) = confirmed else {
        return HuntOutcome::NotFound(NotFoundReason::Unreproducible);
    };

    let budget_exhausted = matches!(shrunk.stop, ShrinkStop::EvalBudget | ShrinkStop::WallTime);
    let got = run.got.sanitised();
    let autopsy = build_autopsy(
        &input,
        &first.predicate,
        run.expected.as_ref(),
        &got,
        req.budget.eval_limit,
        &req.known_ghosts,
        &req.concept_tags,
    );
    let size = size_of(&input);
    HuntOutcome::Found(Box::new(Ghost {
        predicate: first.predicate,
        input,
        expected: run.expected,
        got,
        found_via: first.origin,
        size,
        shrink: shrunk,
        budget_exhausted,
        verification_runs: req.budget.verify_runs,
        autopsy,
    }))
}

/// Shrinks the first failure, asking the Oracle about each new proposal.
///
/// The shrink stops at its own deadline. An Oracle error ends the shrink early
/// and is returned, so it is never mistaken for a smaller input.
fn shrink_failure(
    req: &HuntRequest,
    first: &FirstFailure,
    oracle: &mut dyn Oracle,
    clock: &dyn Clock,
    hunt_start: Duration,
    stats: &mut HuntStats,
) -> Result<ShrinkReport, OracleError> {
    let elapsed = clock.now().saturating_sub(hunt_start);
    let deadline = hunt_start.saturating_add(req.budget.shrink_deadline(elapsed));
    let mut failure: Option<OracleError> = None;
    let outcome = {
        let mut probe = |candidate: &Value| -> Probe {
            let now = clock.now();
            if now >= deadline {
                return Probe::Stop;
            }
            let limit = req.budget.eval_limit.min(deadline.saturating_sub(now));
            stats.oracle_calls += 1;
            match oracle.evaluate(candidate, limit) {
                Ok(observation)
                    if classify(&observation, &req.comparator)
                        == Classification::Fail(first.predicate.clone()) =>
                {
                    Probe::Keep
                }
                Ok(_) => Probe::Reject,
                Err(error) => {
                    failure = Some(error);
                    Probe::Stop
                }
            }
        };
        shrink(
            &req.schema,
            first.input.clone(),
            req.budget.max_shrink_evals,
            &mut probe,
        )
    };
    match failure {
        Some(error) => Err(error),
        None => Ok(outcome.report),
    }
}

/// The trace for a timeout ghost. It is reported as found and never shrunk.
fn not_shrunk(input: &Value) -> ShrinkReport {
    ShrinkReport {
        steps: vec![ShrinkStep {
            index: 0,
            input: input.clone(),
            size: size_of(input),
            evaluations: 0,
        }],
        total_steps: 0,
        omitted_steps: 0,
        evaluations: 0,
        stop: ShrinkStop::NotAttempted,
    }
}

/// What re-running one input established.
#[derive(Clone, Debug, PartialEq)]
struct Confirmed {
    /// The reference value, when the reference returned one.
    expected: Option<Value>,
    /// The user's raw result.
    got: ProgramResult,
}

/// The outcome of checking one input by re-running it.
enum Check {
    /// Every run failed the same way with the same values.
    Confirmed(Confirmed),
    /// A run failed differently, or did not fail at all.
    Unreproduced,
    /// The deadline passed before the checks finished.
    OutOfTime,
}

/// Runs `input` `verify_runs` times and requires the same failure and values each time.
fn check_input(
    req: &HuntRequest,
    input: &Value,
    predicate: &FailurePredicate,
    oracle: &mut dyn Oracle,
    clock: &dyn Clock,
    deadline: Duration,
    stats: &mut HuntStats,
) -> Result<Check, OracleError> {
    let mut first_run: Option<Confirmed> = None;
    for _ in 0..req.budget.verify_runs {
        let now = clock.now();
        if now >= deadline {
            return Ok(Check::OutOfTime);
        }
        let limit = req.budget.eval_limit.min(deadline.saturating_sub(now));
        stats.oracle_calls += 1;
        let observation = oracle.evaluate(input, limit)?;
        if classify(&observation, &req.comparator) != Classification::Fail(predicate.clone()) {
            return Ok(Check::Unreproduced);
        }
        let expected = match &observation.reference {
            ProgramResult::Returned(value) => Some(value.clone()),
            _ => None,
        };
        let run = Confirmed {
            expected,
            got: observation.user,
        };
        match first_run.take() {
            Some(previous) if previous != run => return Ok(Check::Unreproduced),
            _ => first_run = Some(run),
        }
    }
    Ok(first_run.map_or(Check::Unreproduced, Check::Confirmed))
}

fn invalid_reason(req: &HuntRequest) -> Option<String> {
    if let Err(error) = req.schema.check_definition() {
        return Some(error.to_string());
    }
    if let Err(reason) = req.budget.validate() {
        return Some(reason.to_owned());
    }
    let identity = &req.identity;
    if identity.image_digest.is_empty() {
        return Some("image digest is empty".to_owned());
    }
    let longest = [
        &identity.problem_slug,
        &identity.code_sha256,
        &identity.language,
        &identity.image_digest,
    ]
    .iter()
    .map(|field| field.len())
    .max()
    .unwrap_or(0);
    if longest > MAX_IDENTITY_BYTES {
        return Some("an identity field is too long".to_owned());
    }
    None
}

/// Searches candidates for the first failure the request targets.
pub(crate) fn search(
    req: &HuntRequest,
    seed: u64,
    oracle: &mut dyn Oracle,
    clock: &dyn Clock,
    stats: &mut HuntStats,
) -> Search {
    let known_inputs: Vec<Value> = req
        .known_ghosts
        .iter()
        .map(|ghost| ghost.input.clone())
        .collect();
    let mut stream = CandidateStream::new(
        req.schema.clone(),
        seed,
        known_inputs,
        req.samples.clone(),
        GeneratorConfig::default(),
    );
    if req
        .target
        .as_ref()
        .is_some_and(|target| target.kind == FailureKind::Timeout)
    {
        stream = stream.timeout_target();
    }
    stats.invalid_seeds_skipped = stream.invalid_seeds_skipped();
    let deadline = clock.now().saturating_add(req.budget.search_deadline());

    loop {
        let Some(candidate) = stream.next() else {
            return Search::Ended(HuntOutcome::NotFound(NotFoundReason::CandidatesExhausted));
        };
        if stats.candidates_evaluated >= req.budget.max_candidates {
            return Search::Ended(HuntOutcome::NotFound(NotFoundReason::CandidateBudget));
        }
        let now = clock.now();
        if now >= deadline {
            return Search::Ended(HuntOutcome::NotFound(NotFoundReason::WallTime));
        }
        let limit = req.budget.eval_limit.min(deadline.saturating_sub(now));
        stats.candidates_evaluated += 1;
        stats.oracle_calls += 1;
        let observation = match oracle.evaluate(&candidate.value, limit) {
            Ok(observation) => observation,
            Err(error) => {
                return Search::Ended(HuntOutcome::Aborted(AbortReason::OracleFailure(
                    error.message().to_owned(),
                )))
            }
        };
        match classify(&observation, &req.comparator) {
            Classification::Pass => {}
            Classification::ReferenceFailed => {
                stats.reference_failures += 1;
                if stats.reference_failures >= MAX_REFERENCE_FAILURES {
                    return Search::Ended(HuntOutcome::Aborted(AbortReason::ReferenceUnreliable));
                }
            }
            Classification::Fail(predicate) => {
                if is_target(req, &predicate) {
                    return Search::Found(FirstFailure {
                        input: candidate.value,
                        predicate,
                        origin: candidate.origin,
                    });
                }
                if predicate.kind == FailureKind::Timeout {
                    stats.timeouts_seen += 1;
                }
            }
        }
    }
}

/// Reports whether a failure is the one the hunt is looking for.
///
/// Without a target, the first wrong answer or crash counts. Timeouts count
/// only when the target asks for them.
fn is_target(req: &HuntRequest, predicate: &FailurePredicate) -> bool {
    match &req.target {
        Some(target) => target == predicate,
        None => predicate.kind != FailureKind::Timeout,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::classify::{FailureKind, Observation, ProgramResult};
    use crate::oracle::{ManualClock, OracleError};
    use crate::shrink::ShrinkStop;
    use serde_json::json;
    use std::cell::Cell;
    use std::time::Duration;

    /// Wraps a closure so tests can script the Oracle inline.
    struct Scripted<F>(F);

    impl<F> Oracle for Scripted<F>
    where
        F: FnMut(&Value, Duration) -> Result<Observation, OracleError>,
    {
        fn evaluate(&mut self, input: &Value, limit: Duration) -> Result<Observation, OracleError> {
            (self.0)(input, limit)
        }
    }

    fn identity() -> HuntIdentity {
        HuntIdentity {
            problem_slug: "pair-count".to_owned(),
            code_sha256: "ab12".to_owned(),
            language: "python".to_owned(),
            problem_version: 1,
            image_digest: "sha256:test".to_owned(),
        }
    }

    fn request(schema: Schema) -> HuntRequest {
        HuntRequest::new(schema, Comparator::Exact, identity())
    }

    fn pass(input: &Value) -> Observation {
        Observation {
            user: ProgramResult::Returned(input.clone()),
            reference: ProgramResult::Returned(input.clone()),
        }
    }

    fn int_range(min: i64, max: i64) -> Schema {
        Schema::Int { min, max }
    }

    fn index_error() -> ProgramResult {
        ProgramResult::Crashed {
            exception_class: "IndexError".to_owned(),
            message: String::new(),
        }
    }

    fn run_search(
        req: &HuntRequest,
        oracle: &mut dyn Oracle,
        clock: &dyn Clock,
    ) -> (Search, HuntStats) {
        let mut stats = HuntStats::default();
        let seed = derive_seed("pair-count", "ab12", "python");
        let result = search(req, seed, oracle, clock, &mut stats);
        (result, stats)
    }

    #[test]
    fn hunt_candidate_budget_stops_a_passing_program() {
        let mut req = request(int_range(0, 100));
        req.budget.max_candidates = 5;
        let clock = ManualClock::new();
        let mut oracle = Scripted(|input: &Value, _: Duration| Ok(pass(input)));
        let (result, stats) = run_search(&req, &mut oracle, &clock);
        assert!(matches!(
            result,
            Search::Ended(HuntOutcome::NotFound(NotFoundReason::CandidateBudget))
        ));
        assert_eq!(stats.candidates_evaluated, 5);
    }

    #[test]
    fn hunt_small_space_ends_as_exhausted() {
        let req = request(int_range(0, 0));
        let clock = ManualClock::new();
        let mut oracle = Scripted(|input: &Value, _: Duration| Ok(pass(input)));
        let (result, stats) = run_search(&req, &mut oracle, &clock);
        assert!(matches!(
            result,
            Search::Ended(HuntOutcome::NotFound(NotFoundReason::CandidatesExhausted))
        ));
        assert_eq!(stats.candidates_evaluated, 1);
    }

    #[test]
    fn hunt_wall_time_stops_a_slow_search() {
        let mut req = request(int_range(0, 100));
        req.budget.search_wall = Duration::from_secs(1);
        let clock = ManualClock::new();
        let mut oracle = Scripted(|input: &Value, _: Duration| {
            clock.advance(Duration::from_millis(300));
            Ok(pass(input))
        });
        let (result, _) = run_search(&req, &mut oracle, &clock);
        assert!(matches!(
            result,
            Search::Ended(HuntOutcome::NotFound(NotFoundReason::WallTime))
        ));
    }

    #[test]
    fn hunt_bad_reference_aborts_after_three() {
        let req = request(int_range(0, 100));
        let clock = ManualClock::new();
        let mut oracle = Scripted(|input: &Value, _: Duration| {
            Ok(Observation {
                user: ProgramResult::Returned(input.clone()),
                reference: index_error(),
            })
        });
        let (result, stats) = run_search(&req, &mut oracle, &clock);
        assert!(matches!(
            result,
            Search::Ended(HuntOutcome::Aborted(AbortReason::ReferenceUnreliable))
        ));
        assert_eq!(stats.reference_failures, 3);
    }

    #[test]
    fn hunt_oracle_error_aborts_with_its_message() {
        let req = request(int_range(0, 100));
        let clock = ManualClock::new();
        let mut oracle = Scripted(|_: &Value, _: Duration| Err(OracleError::new("sandbox gone")));
        let (result, _) = run_search(&req, &mut oracle, &clock);
        assert!(matches!(
            result,
            Search::Ended(HuntOutcome::Aborted(AbortReason::OracleFailure(message)))
                if message == "sandbox gone"
        ));
    }

    #[test]
    fn hunt_target_filter_skips_other_failures() {
        let mut req = request(int_range(0, 1));
        req.target = Some(FailurePredicate {
            kind: FailureKind::WrongAnswer,
            exception_class: None,
        });
        let clock = ManualClock::new();
        let mut oracle = Scripted(|input: &Value, _: Duration| {
            if input == &json!(0) {
                Ok(Observation {
                    user: index_error(),
                    reference: ProgramResult::Returned(json!(0)),
                })
            } else {
                Ok(Observation {
                    user: ProgramResult::Returned(json!(99)),
                    reference: ProgramResult::Returned(json!(0)),
                })
            }
        });
        let (result, _) = run_search(&req, &mut oracle, &clock);
        let Search::Found(first) = result else {
            panic!("expected a wrong answer to be found");
        };
        assert_eq!(first.input, json!(1));
        assert_eq!(first.predicate.kind, FailureKind::WrongAnswer);
    }

    #[test]
    fn hunt_without_target_takes_the_first_crash() {
        let req = request(int_range(0, 1));
        let clock = ManualClock::new();
        let mut oracle = Scripted(|input: &Value, _: Duration| {
            Ok(Observation {
                user: index_error(),
                reference: ProgramResult::Returned(input.clone()),
            })
        });
        let (result, _) = run_search(&req, &mut oracle, &clock);
        let Search::Found(first) = result else {
            panic!("expected a crash to be found");
        };
        assert_eq!(first.input, json!(0));
        assert_eq!(
            first.predicate.exception_class.as_deref(),
            Some("IndexError")
        );
    }

    #[test]
    fn hunt_timeouts_counted_not_reported_by_default() {
        let req = request(int_range(0, 1));
        let clock = ManualClock::new();
        let mut oracle = Scripted(|input: &Value, _: Duration| {
            if input == &json!(0) {
                Ok(Observation {
                    user: ProgramResult::TimedOut,
                    reference: ProgramResult::Returned(json!(0)),
                })
            } else {
                Ok(Observation {
                    user: ProgramResult::Returned(json!(99)),
                    reference: ProgramResult::Returned(json!(0)),
                })
            }
        });
        let (result, stats) = run_search(&req, &mut oracle, &clock);
        let Search::Found(first) = result else {
            panic!("expected the wrong answer to be found");
        };
        assert_eq!(first.input, json!(1));
        assert_eq!(stats.timeouts_seen, 1);
    }

    #[test]
    fn hunt_timeout_target_reports_the_timeout() {
        let mut req = request(int_range(0, 1));
        req.target = Some(FailurePredicate {
            kind: FailureKind::Timeout,
            exception_class: None,
        });
        let clock = ManualClock::new();
        let mut oracle = Scripted(|input: &Value, _: Duration| {
            Ok(Observation {
                user: ProgramResult::TimedOut,
                reference: ProgramResult::Returned(input.clone()),
            })
        });
        let (result, _) = run_search(&req, &mut oracle, &clock);
        let Search::Found(first) = result else {
            panic!("expected the timeout to be found");
        };
        assert_eq!(first.input, json!(0));
        assert_eq!(first.predicate.kind, FailureKind::Timeout);
    }

    #[test]
    fn hunt_invalid_budget_is_an_invalid_request() {
        let mut req = request(int_range(0, 1));
        req.budget.total_wall = Duration::from_secs(9);
        let clock = ManualClock::new();
        let mut oracle = Scripted(|input: &Value, _: Duration| Ok(pass(input)));
        let report = hunt(&req, &mut oracle, &clock);
        assert!(matches!(
            report.outcome,
            HuntOutcome::Aborted(AbortReason::InvalidRequest(_))
        ));
    }

    #[test]
    fn hunt_empty_image_digest_is_an_invalid_request() {
        let mut req = request(int_range(0, 1));
        req.identity.image_digest = String::new();
        let clock = ManualClock::new();
        let mut oracle = Scripted(|input: &Value, _: Duration| Ok(pass(input)));
        let report = hunt(&req, &mut oracle, &clock);
        assert!(matches!(
            report.outcome,
            HuntOutcome::Aborted(AbortReason::InvalidRequest(_))
        ));
    }

    #[test]
    fn hunt_report_provenance_for_a_passing_program() {
        let mut req = request(int_range(0, 100));
        req.budget.max_candidates = 2;
        let clock = ManualClock::new();
        let mut oracle = Scripted(|input: &Value, _: Duration| Ok(pass(input)));
        let report = hunt(&req, &mut oracle, &clock);
        assert_eq!(
            report.outcome,
            HuntOutcome::NotFound(NotFoundReason::CandidateBudget)
        );
        assert_eq!(
            report.provenance.seed,
            derive_seed("pair-count", "ab12", "python")
        );
        assert_eq!(report.provenance.generator_version, GENERATOR_VERSION);
        assert_eq!(report.provenance.image_digest, "sha256:test");
    }

    /// The sum of the integers in an array input, for the synthetic oracles.
    fn sum_of(input: &Value) -> i64 {
        input
            .as_array()
            .map_or(0, |items| items.iter().filter_map(Value::as_i64).sum())
    }

    /// Reference returns the sum. The user adds one when any item is negative.
    fn negative_bug_oracle(
    ) -> Scripted<impl FnMut(&Value, Duration) -> Result<Observation, OracleError>> {
        Scripted(|input: &Value, _: Duration| {
            let expected = sum_of(input);
            let has_negative = input.as_array().is_some_and(|items| {
                items
                    .iter()
                    .any(|item| item.as_i64().is_some_and(|n| n < 0))
            });
            let user = if has_negative { expected + 1 } else { expected };
            Ok(Observation {
                user: ProgramResult::Returned(json!(user)),
                reference: ProgramResult::Returned(json!(expected)),
            })
        })
    }

    fn sum_array_schema() -> Schema {
        Schema::IntArray {
            min_len: 0,
            max_len: 6,
            min: -9,
            max: 9,
            sorted: false,
            unique: false,
        }
    }

    fn found(outcome: HuntOutcome) -> Ghost {
        match outcome {
            HuntOutcome::Found(ghost) => *ghost,
            other => panic!("expected a ghost, got {other:?}"),
        }
    }

    #[test]
    fn hunt_finds_and_shrinks_a_wrong_answer() {
        let req = request(sum_array_schema());
        let clock = ManualClock::new();
        let mut oracle = negative_bug_oracle();
        let report = hunt(&req, &mut oracle, &clock);
        let ghost = found(report.outcome);
        assert_eq!(ghost.input, json!([-1]));
        assert_eq!(ghost.expected, Some(json!(-1)));
        assert_eq!(ghost.got, ProgramResult::Returned(json!(0)));
        assert_eq!(ghost.predicate.kind, FailureKind::WrongAnswer);
        assert_eq!(ghost.shrink.stop, ShrinkStop::LocalMinimum);
        assert!(!ghost.budget_exhausted);
        assert_eq!(ghost.verification_runs, 2);
        assert!(ghost.is_minimized());
    }

    #[test]
    fn hunt_crash_shrinks_to_the_empty_list() {
        let req = request(sum_array_schema());
        let clock = ManualClock::new();
        let mut oracle = Scripted(|input: &Value, _: Duration| {
            let reference = ProgramResult::Returned(json!(sum_of(input)));
            let user = if input == &json!([]) {
                ProgramResult::Crashed {
                    exception_class: "IndexError".to_owned(),
                    message: "list index out of range".to_owned(),
                }
            } else {
                reference.clone()
            };
            Ok(Observation { user, reference })
        });
        let ghost = found(hunt(&req, &mut oracle, &clock).outcome);
        assert_eq!(ghost.input, json!([]));
        assert_eq!(ghost.predicate.kind, FailureKind::RuntimeError);
        assert_eq!(
            ghost.predicate.exception_class.as_deref(),
            Some("IndexError")
        );
        assert_eq!(ghost.expected, Some(json!(0)));
    }

    #[test]
    fn hunt_flaky_final_input_walks_back_to_a_reproduced_entry() {
        let req = request(Schema::Int { min: -9, max: 9 });
        let clock = ManualClock::new();
        let minus_two_calls = Cell::new(0_u32);
        let mut oracle = Scripted(|input: &Value, _: Duration| {
            let value = input.as_i64().unwrap_or(0);
            let repeat_of_minus_two = value == -2 && {
                let before = minus_two_calls.get();
                minus_two_calls.set(before + 1);
                before > 0
            };
            let user = if value <= -2 && !repeat_of_minus_two {
                value + 1
            } else {
                value
            };
            Ok(Observation {
                user: ProgramResult::Returned(json!(user)),
                reference: ProgramResult::Returned(json!(value)),
            })
        });
        let ghost = found(hunt(&req, &mut oracle, &clock).outcome);
        assert_eq!(ghost.input, json!(-4));
        assert_eq!(ghost.expected, Some(json!(-4)));
        assert_eq!(ghost.got, ProgramResult::Returned(json!(-3)));
    }

    #[test]
    fn hunt_always_flaky_is_unreproducible() {
        let req = request(Schema::Int { min: -9, max: 9 });
        let clock = ManualClock::new();
        let calls = Cell::new(0_u32);
        let mut oracle = Scripted(|input: &Value, _: Duration| {
            let value = input.as_i64().unwrap_or(0);
            let first = calls.get() == 0;
            calls.set(calls.get() + 1);
            let user = if first { value + 1 } else { value };
            Ok(Observation {
                user: ProgramResult::Returned(json!(user)),
                reference: ProgramResult::Returned(json!(value)),
            })
        });
        let report = hunt(&req, &mut oracle, &clock);
        assert_eq!(
            report.outcome,
            HuntOutcome::NotFound(NotFoundReason::Unreproducible)
        );
    }

    #[test]
    fn hunt_timeout_ghost_is_not_shrunk() {
        let mut req = request(Schema::Int { min: -9, max: 9 });
        req.target = Some(FailurePredicate {
            kind: FailureKind::Timeout,
            exception_class: None,
        });
        let clock = ManualClock::new();
        let mut oracle = Scripted(|input: &Value, _: Duration| {
            let value = input.as_i64().unwrap_or(0);
            let user = if value < 0 {
                ProgramResult::TimedOut
            } else {
                ProgramResult::Returned(json!(value))
            };
            Ok(Observation {
                user,
                reference: ProgramResult::Returned(json!(value)),
            })
        });
        let ghost = found(hunt(&req, &mut oracle, &clock).outcome);
        assert_eq!(ghost.input, json!(-1));
        assert_eq!(ghost.shrink.stop, ShrinkStop::NotAttempted);
        assert!(!ghost.is_minimized());
        assert_eq!(ghost.got, ProgramResult::TimedOut);
        assert_eq!(ghost.verification_runs, 2);
    }

    #[test]
    fn hunt_shrink_budget_marks_the_ghost_exhausted() {
        let mut req = request(sum_array_schema());
        req.budget.max_shrink_evals = 1;
        let clock = ManualClock::new();
        let mut oracle = negative_bug_oracle();
        let ghost = found(hunt(&req, &mut oracle, &clock).outcome);
        assert_eq!(ghost.shrink.stop, ShrinkStop::EvalBudget);
        assert!(ghost.budget_exhausted);
        assert_eq!(ghost.input, json!([-1]));
    }

    #[test]
    fn hunt_oracle_error_mid_shrink_aborts() {
        let req = request(sum_array_schema());
        let clock = ManualClock::new();
        let zero_calls = Cell::new(0_u32);
        let mut oracle = Scripted(|input: &Value, _: Duration| {
            if input == &json!([0]) {
                let before = zero_calls.get();
                zero_calls.set(before + 1);
                if before > 0 {
                    return Err(OracleError::new("boom"));
                }
            }
            let expected = sum_of(input);
            let user = if input == &json!([-1]) {
                expected + 1
            } else {
                expected
            };
            Ok(Observation {
                user: ProgramResult::Returned(json!(user)),
                reference: ProgramResult::Returned(json!(expected)),
            })
        });
        let report = hunt(&req, &mut oracle, &clock);
        assert_eq!(
            report.outcome,
            HuntOutcome::Aborted(AbortReason::OracleFailure("boom".to_owned()))
        );
    }

    #[test]
    fn hunt_known_ghost_gives_the_authored_mistake() {
        let mut req = request(sum_array_schema());
        req.known_ghosts = vec![KnownGhost {
            id: "negatives".to_owned(),
            input: json!([-1]),
            kind: FailureKind::WrongAnswer,
            exception_class: None,
            got: Some(json!(0)),
            explanation: "Negative items are off by one.".to_owned(),
            concept_tags: Vec::new(),
        }];
        let clock = ManualClock::new();
        let mut oracle = negative_bug_oracle();
        let ghost = found(hunt(&req, &mut oracle, &clock).outcome);
        assert_eq!(
            ghost
                .autopsy
                .authored_mistake
                .map(|mistake| mistake.known_ghost_id),
            Some("negatives".to_owned())
        );
    }

    #[test]
    fn verify_ghost_accepts_a_reproduced_ghost() {
        let req = request(sum_array_schema());
        let clock = ManualClock::new();
        let mut oracle = negative_bug_oracle();
        let ghost = found(hunt(&req, &mut oracle, &clock).outcome);
        let mut fresh = negative_bug_oracle();
        assert!(verify_ghost(&req, &ghost, &mut fresh, &clock));
    }

    #[test]
    fn verify_ghost_rejects_a_changed_expected_value() {
        let req = request(sum_array_schema());
        let clock = ManualClock::new();
        let mut oracle = negative_bug_oracle();
        let mut ghost = found(hunt(&req, &mut oracle, &clock).outcome);
        ghost.expected = Some(json!(5));
        let mut fresh = negative_bug_oracle();
        assert!(!verify_ghost(&req, &ghost, &mut fresh, &clock));
    }

    #[test]
    fn hunt_not_found_texts_do_not_claim_correctness() {
        for reason in [
            NotFoundReason::CandidateBudget,
            NotFoundReason::WallTime,
            NotFoundReason::CandidatesExhausted,
            NotFoundReason::Unreproducible,
        ] {
            assert!(!reason.to_string().to_lowercase().contains("correct"));
        }
    }
}
