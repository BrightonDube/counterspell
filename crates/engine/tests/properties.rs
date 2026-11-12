//! Hunt-level property tests with synthetic oracles and a manual clock.
//!
//! Each property names the break it catches. The synthetic oracle is a test
//! double: its reference counts JSON nodes, and a bug family decides when the
//! user's program is wrong or crashes.

mod support;

use counterspell_engine::generate::{CandidateStream, GeneratorConfig};
use counterspell_engine::{
    build_autopsy, classify, hunt, verify_ghost, Budget, Classification, Clock, Comparator,
    FailureKind, FailurePredicate, HuntIdentity, HuntOutcome, HuntReport, HuntRequest, ManualClock,
    NotFoundReason, Oracle, ProgramResult, Schema, ENGINE_VERSION, GENERATOR_VERSION, GHOST_LABEL,
};
use proptest::prelude::*;
use serde_json::{json, Value};
use std::collections::HashSet;
use std::time::Duration;
use support::{Bug, SyntheticOracle};

/// The clock time the synthetic oracle charges for one ordinary call.
const COST_PER_CALL: Duration = Duration::from_millis(5);

fn arb_schema() -> impl Strategy<Value = Schema> {
    let leaf = prop_oneof![
        (-20_i64..=20, 0_i64..=20).prop_map(|(min, span)| Schema::Int {
            min,
            max: min + span,
        }),
        (
            0_usize..=4,
            0_usize..=4,
            -9_i64..=9,
            0_i64..=9,
            any::<bool>(),
            any::<bool>()
        )
            .prop_map(|(min_len, extra, min, span, sorted, unique)| {
                Schema::IntArray {
                    min_len,
                    max_len: min_len + extra,
                    min,
                    max: min + span,
                    sorted,
                    unique,
                }
            }),
        (
            prop::sample::select(vec!["ab", "xyz", "q"]),
            0_usize..=3,
            0_usize..=3
        )
            .prop_map(|(alphabet, min_len, extra)| Schema::Str {
                alphabet: alphabet.to_owned(),
                min_len,
                max_len: min_len + extra,
            }),
        (
            0_usize..=3,
            0_usize..=3,
            0_usize..=2,
            0_usize..=2,
            -9_i64..=9,
            0_i64..=9
        )
            .prop_map(|(min_rows, extra_rows, min_cols, extra_cols, min, span)| {
                Schema::Matrix {
                    min_rows,
                    max_rows: min_rows + extra_rows,
                    min_cols,
                    max_cols: min_cols + extra_cols,
                    min,
                    max: min + span,
                }
            }),
    ]
    .boxed();
    prop_oneof![
        3 => leaf.clone(),
        1 => prop::collection::vec(leaf, 1..=2).prop_map(|items| Schema::Tuple { items }),
    ]
    .prop_filter("a valid schema definition", |schema| {
        schema.check_definition().is_ok()
    })
}

fn arb_bug() -> impl Strategy<Value = Bug> {
    prop_oneof![
        Just(Bug::Always),
        Just(Bug::Never),
        Just(Bug::ReferenceCrashes),
        (0_usize..=8).prop_map(Bug::WrongWhenLarge),
        (0_usize..=8).prop_map(Bug::CrashWhenLarge),
        (-3_i64..=3).prop_map(Bug::WrongWhenContains),
    ]
}

fn arb_cost() -> impl Strategy<Value = Duration> {
    prop_oneof![
        Just(Duration::from_millis(1)),
        Just(COST_PER_CALL),
        Just(Duration::from_millis(300)),
        Just(Duration::from_secs(2)),
    ]
}

fn arb_small_sample() -> impl Strategy<Value = Value> {
    prop::collection::vec(-3_i64..3, 0..3).prop_map(|items| json!(items))
}

fn identity() -> HuntIdentity {
    HuntIdentity {
        problem_slug: "synthetic".to_owned(),
        code_sha256: "abc123".to_owned(),
        language: "python".to_owned(),
        problem_version: 1,
        image_digest: "sha256:test".to_owned(),
    }
}

fn request(schema: Schema, samples: Vec<Value>, seed: u64) -> HuntRequest {
    let mut req = HuntRequest::new(schema, Comparator::Exact, identity());
    req.samples = samples;
    req.seed_override = Some(seed);
    req
}

/// Runs one hunt on a fresh manual clock and returns the report, the Oracle
/// call count and the clock time the hunt consumed.
fn run_hunt(req: &HuntRequest, bug: &Bug, cost: Duration) -> (HuntReport, u32, Duration) {
    let clock = ManualClock::new();
    let mut oracle = SyntheticOracle::new(bug.clone(), &clock, cost);
    let report = hunt(req, &mut oracle, &clock);
    (report, oracle.calls(), clock.now())
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Break: the stream yields a value that breaks the schema, or the same
    /// value twice. Samples that break the schema must be skipped.
    #[test]
    fn p1_candidate_stream_yields_valid_distinct_values(
        schema in arb_schema(),
        seed in any::<u64>(),
        samples in prop::collection::vec(arb_small_sample(), 0..3),
    ) {
        let stream = CandidateStream::new(
            schema.clone(),
            seed,
            Vec::new(),
            samples,
            GeneratorConfig::default(),
        );
        let mut seen = HashSet::new();
        for candidate in stream.take(120) {
            prop_assert!(schema.contains(&candidate.value).is_ok());
            prop_assert!(seen.insert(candidate.value.to_string()));
        }
    }

    /// Break: a shrink step keeps a value that no longer fails the same way.
    /// Every trace entry must reproduce the ghost's predicate on a fresh Oracle.
    #[test]
    fn p6_every_recorded_shrink_step_reproduces_the_predicate(
        schema in arb_schema(),
        bug in arb_bug(),
        seed in any::<u64>(),
    ) {
        let req = request(schema, Vec::new(), seed);
        let (report, _, _) = run_hunt(&req, &bug, COST_PER_CALL);
        if let HuntOutcome::Found(ghost) = &report.outcome {
            for step in &ghost.shrink.steps {
                let clock = ManualClock::new();
                let mut oracle = SyntheticOracle::new(bug.clone(), &clock, COST_PER_CALL);
                let observation = oracle
                    .evaluate(&step.input, req.budget.eval_limit)
                    .expect("the synthetic oracle never fails");
                prop_assert_eq!(
                    classify(&observation, &req.comparator),
                    Classification::Fail(ghost.predicate.clone())
                );
            }
        }
    }

    /// Break: a ghost is reported that does not reproduce, or whose expected
    /// value or actual result differs from a fresh run.
    #[test]
    fn p8_found_ghosts_reproduce_on_a_fresh_oracle(
        schema in arb_schema(),
        bug in arb_bug(),
        cost in arb_cost(),
        seed in any::<u64>(),
    ) {
        let req = request(schema, Vec::new(), seed);
        let (report, _, _) = run_hunt(&req, &bug, cost);
        if let HuntOutcome::Found(ghost) = &report.outcome {
            prop_assert!(req.schema.contains(&ghost.input).is_ok());

            let clock = ManualClock::new();
            let mut oracle = SyntheticOracle::new(bug.clone(), &clock, cost);
            let observation = oracle
                .evaluate(&ghost.input, req.budget.eval_limit)
                .expect("the synthetic oracle never fails");
            prop_assert_eq!(
                classify(&observation, &req.comparator),
                Classification::Fail(ghost.predicate.clone())
            );
            prop_assert_eq!(&observation.user, &ghost.got);
            match &observation.reference {
                ProgramResult::Returned(expected) => {
                    prop_assert_eq!(&ghost.expected, &Some(expected.clone()));
                }
                other => prop_assert!(false, "reference should return a value, got {other:?}"),
            }

            let check_clock = ManualClock::new();
            let mut check_oracle = SyntheticOracle::new(bug.clone(), &check_clock, cost);
            prop_assert!(verify_ghost(&req, ghost, &mut check_oracle, &check_clock));
        }
    }

    /// Break: equal requests and equal Oracle behaviour give different reports.
    #[test]
    fn p9_equal_requests_give_equal_outcomes_provenance_and_stats(
        schema in arb_schema(),
        bug in arb_bug(),
        samples in prop::collection::vec(arb_small_sample(), 0..3),
        seed in any::<u64>(),
    ) {
        let req = request(schema, samples, seed);
        let (first, _, _) = run_hunt(&req, &bug, COST_PER_CALL);
        let (second, _, _) = run_hunt(&req, &bug, COST_PER_CALL);
        prop_assert_eq!(&first.outcome, &second.outcome);
        prop_assert_eq!(&first.provenance, &second.provenance);
        prop_assert_eq!(&first.stats, &second.stats);
    }

    /// Break: a hunt makes more Oracle calls than R8.7 allows, or spends more
    /// manual-clock time than the total wall budget, for any Oracle behaviour.
    #[test]
    fn p10_hunts_terminate_within_their_budgets(
        schema in arb_schema(),
        bug in arb_bug(),
        cost in arb_cost(),
        seed in any::<u64>(),
    ) {
        let req = request(schema, Vec::new(), seed);
        let (_, calls, elapsed) = run_hunt(&req, &bug, cost);
        let budget = req.budget;
        let call_limit = budget.max_candidates + budget.max_shrink_evals + budget.verify_runs * 21;
        prop_assert!(calls <= call_limit, "{calls} calls exceed {call_limit}");
        prop_assert!(elapsed <= budget.total_wall, "{elapsed:?} exceeds the total budget");
    }
}

#[test]
fn p9_fixed_example_is_deterministic_on_a_known_bug() {
    let req = request(Schema::Int { min: -10, max: 10 }, Vec::new(), 7);
    let (first, _, _) = run_hunt(&req, &Bug::WrongWhenLarge(1), COST_PER_CALL);
    let (second, _, _) = run_hunt(&req, &Bug::WrongWhenLarge(1), COST_PER_CALL);
    assert_eq!(first, second);
    assert!(matches!(first.outcome, HuntOutcome::Found(_)));
}

/// Break: the P6 and P8 properties pass vacuously if no hunt ever finds a
/// ghost. An oracle that is always wrong on a small range must produce one.
#[test]
fn p8_non_vacuity_an_always_wrong_oracle_yields_a_ghost() {
    let req = request(Schema::Int { min: 0, max: 5 }, Vec::new(), 11);
    let (report, _, _) = run_hunt(&req, &Bug::Always, COST_PER_CALL);
    match report.outcome {
        HuntOutcome::Found(ghost) => assert_eq!(
            ghost.shrink.stop,
            counterspell_engine::shrink::ShrinkStop::LocalMinimum
        ),
        other => panic!("expected a ghost, got {other:?}"),
    }
}

/// Phrases the engine must never put in a result (R9.6), plus the stricter list
/// the design sets for templates.
const FORBIDDEN_PHRASES: [&str; 6] = [
    "smallest possible",
    "global",
    "proven",
    "optimal",
    "the minimum",
    "smallest",
];

/// Panics if `text` contains a forbidden phrase, ignoring case.
fn assert_no_forbidden_phrase(text: &str) {
    let lower = text.to_lowercase();
    for phrase in FORBIDDEN_PHRASES {
        assert!(
            !lower.contains(phrase),
            "{text:?} contains the forbidden phrase {phrase:?}"
        );
    }
}

/// Integer-only inputs. The wording rule covers the engine's own templates, not
/// data a learner's program echoes back, so this generator avoids text values.
fn arb_integer_input() -> impl Strategy<Value = Value> {
    prop::collection::vec(-100_i64..100, 0..6).prop_map(|items| json!([items, 3]))
}

#[test]
fn wording_not_found_reasons_and_the_ghost_label_avoid_forbidden_phrases() {
    for reason in [
        NotFoundReason::CandidateBudget,
        NotFoundReason::WallTime,
        NotFoundReason::CandidatesExhausted,
        NotFoundReason::Unreproducible,
    ] {
        assert_no_forbidden_phrase(&reason.to_string());
    }
    assert_no_forbidden_phrase(GHOST_LABEL);
}

#[test]
fn wording_json_contract_uses_the_ghosts_table_names() {
    let req = request(Schema::Int { min: 0, max: 5 }, Vec::new(), 3);
    let (report, _, _) = run_hunt(&req, &Bug::CrashWhenLarge(1), COST_PER_CALL);
    let value = serde_json::to_value(&report).expect("a report serialises");

    let ghost = &value["outcome"]["Found"];
    assert_eq!(ghost["predicate"]["kind"], "crash");
    assert_eq!(ghost["predicate"]["exception_class"], "ValueError");
    assert!(ghost["shrink"]["steps"].is_array());
    assert!(ghost["budget_exhausted"].is_boolean());
}

#[test]
fn wording_json_contract_provenance_fields_match_the_engine() {
    let req = request(Schema::Int { min: 0, max: 5 }, Vec::new(), 3);
    let (report, _, _) = run_hunt(&req, &Bug::CrashWhenLarge(1), COST_PER_CALL);
    let value = serde_json::to_value(&report).expect("a report serialises");

    assert_eq!(value["provenance"]["generator_version"], GENERATOR_VERSION);
    assert_eq!(value["provenance"]["engine_version"], ENGINE_VERSION);
    assert_eq!(value["provenance"]["problem_version"], 1);
    assert_eq!(value["provenance"]["image_digest"], "sha256:test");
    assert!(value["provenance"]["seed"].is_u64());
}

#[test]
fn wording_kind_names_match_the_ghosts_table() {
    assert_eq!(
        serde_json::to_value(FailureKind::WrongAnswer).expect("serialises"),
        "wrong"
    );
    assert_eq!(
        serde_json::to_value(FailureKind::RuntimeError).expect("serialises"),
        "crash"
    );
    assert_eq!(
        serde_json::to_value(FailureKind::Timeout).expect("serialises"),
        "timeout"
    );
}

#[test]
fn wording_json_contract_counts_and_magnitude_have_the_right_types() {
    let req = request(Schema::Int { min: 0, max: 5 }, Vec::new(), 3);
    let (report, _, _) = run_hunt(&req, &Bug::CrashWhenLarge(1), COST_PER_CALL);
    let value = serde_json::to_value(&report).expect("a report serialises");
    let ghost = &value["outcome"]["Found"];
    assert!(ghost["verification_runs"].is_u64());
    assert!(
        ghost["size"]["magnitude"].is_string(),
        "magnitude is a decimal string"
    );
}

#[test]
fn wording_fixed_reports_survive_a_json_round_trip() {
    let found_request = request(Schema::Int { min: 0, max: 5 }, Vec::new(), 3);
    let (found, _, _) = run_hunt(&found_request, &Bug::Always, COST_PER_CALL);
    let (not_found, _, _) = run_hunt(&found_request, &Bug::Never, COST_PER_CALL);
    let mut aborted_request = found_request.clone();
    aborted_request.identity.image_digest = String::new();
    let (aborted, _, _) = run_hunt(&aborted_request, &Bug::Always, COST_PER_CALL);

    for report in [found, not_found, aborted] {
        let text = serde_json::to_string(&report).expect("a report serialises");
        let back: HuntReport = serde_json::from_str(&text).expect("a report deserialises");
        assert_eq!(back, report);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Break: a result type loses data when stored as JSON and read back.
    #[test]
    fn wording_p14_results_survive_a_json_round_trip(
        schema in arb_schema(),
        bug in arb_bug(),
        seed in any::<u64>(),
    ) {
        let req = request(schema.clone(), Vec::new(), seed);
        let (report, _, _) = run_hunt(&req, &bug, COST_PER_CALL);

        let report_text = serde_json::to_string(&report).expect("a report serialises");
        let report_back: HuntReport = serde_json::from_str(&report_text).expect("a report reads back");
        prop_assert_eq!(report_back, report);

        let schema_text = serde_json::to_string(&schema).expect("a schema serialises");
        let schema_back: Schema = serde_json::from_str(&schema_text).expect("a schema reads back");
        prop_assert_eq!(schema_back, schema);

        let budget = Budget::default();
        let budget_text = serde_json::to_string(&budget).expect("a budget serialises");
        let budget_back: Budget = serde_json::from_str(&budget_text).expect("a budget reads back");
        prop_assert_eq!(budget_back, budget);
    }
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// Break: an explanation template makes a claim the engine cannot back up.
    #[test]
    fn wording_p15_explanation_templates_avoid_forbidden_phrases(
        input in arb_integer_input(),
        expected in -50_i64..50,
        got in -50_i64..50,
        class in prop::sample::select(vec!["ValueError", "IndexError", "KeyError", "RecursionError"]),
        message in ".{0,300}",
    ) {
        let limit = Budget::default().eval_limit;
        let cases = [
            (
                FailurePredicate { kind: FailureKind::WrongAnswer, exception_class: None },
                Some(Value::from(expected)),
                ProgramResult::Returned(Value::from(got)),
            ),
            (
                FailurePredicate { kind: FailureKind::RuntimeError, exception_class: Some(class.to_owned()) },
                None,
                ProgramResult::Crashed { exception_class: class.to_owned(), message: message.clone() },
            ),
            (
                FailurePredicate { kind: FailureKind::Timeout, exception_class: None },
                None,
                ProgramResult::TimedOut,
            ),
        ];
        for (predicate, expected_value, got_result) in cases {
            let autopsy = build_autopsy(
                &input,
                &predicate,
                expected_value.as_ref(),
                &got_result,
                limit,
                &[],
                &[],
            );
            assert_no_forbidden_phrase(&autopsy.explanation);
        }
    }
}
