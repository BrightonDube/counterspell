//! Ghost hunt orchestration: the request, the report, and the search phase.
//!
//! A hunt searches candidate inputs for the first failure against the trusted
//! reference, then shrinks it (added in a later step). The engine performs no
//! I/O. Every program call goes through the [`Oracle`] it is given.

use crate::autopsy::{Autopsy, KnownGhost};
use crate::budget::Budget;
use crate::classify::{classify, Classification, FailureKind, FailurePredicate, ProgramResult};
use crate::compare::Comparator;
use crate::generate::{CandidateStream, GeneratorConfig, Origin};
use crate::oracle::{Clock, Oracle};
use crate::rng::derive_seed;
use crate::schema::Schema;
use crate::shrink::ShrinkReport;
use crate::size::SizeMetric;
use crate::{ENGINE_VERSION, GENERATOR_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::{self, Display, Formatter};

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
/// A found failure is not reported yet. Shrinking and verification arrive in
/// the next step, so until then the hunt refuses to report an unverified ghost.
pub fn hunt(req: &HuntRequest, oracle: &mut dyn Oracle, clock: &dyn Clock) -> HuntReport {
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
            Search::Found(first) => HuntOutcome::Aborted(AbortReason::InvalidRequest(format!(
                "unverified {:?} failure on {}; shrinking is not wired yet",
                first.predicate.kind, first.input
            ))),
        }
    };
    HuntReport {
        outcome,
        provenance,
        budget: req.budget,
        stats,
    }
}

/// Describes the first request problem, or returns `None` for a valid request.
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
    use serde_json::json;
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
