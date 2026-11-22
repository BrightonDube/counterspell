// SPDX-License-Identifier: MIT
//! The shared sandbox checks (`sandbox::run_checks`) driven by a model runner.
//!
//! `validate` and the catalogue import both call `run_checks`, so these tests pin the
//! behaviour both callers rely on: the same hidden set under either scope, curated
//! inputs never drawn again as generated cases, and where each scope stops. The model
//! reference doubles its integer input; expected values are hand-derived literals.

use counterspell_engine::generate::{CandidateStream, GeneratorConfig, Origin};
use counterspell_engine::{Comparator, ProgramResult, Schema};
use counterspell_grimoire::ghosts::sandbox_ghosts;
use counterspell_grimoire::runner::{PackRunner, RunnerError, Target};
use counterspell_grimoire::sandbox::{
    run_checks, Finding, Generated, HiddenCase, HiddenSource, SandboxInputs, Scope,
};
use serde_json::{json, Value};
use std::time::Duration;

/// The reference returns `2 * x`. The `incorrect/off-by-one.py` program returns
/// `2 * x + 1` for every `x >= 1`, and `0` for `x = 0`.
#[derive(Default)]
struct Doubler {
    calls: usize,
}

impl PackRunner for Doubler {
    fn run(
        &mut self,
        target: Target<'_>,
        input: &Value,
        _limit: Duration,
    ) -> Result<ProgramResult, RunnerError> {
        self.calls += 1;
        let x = input.as_i64().unwrap_or_default();
        let off_by_one = target == Target::Program("incorrect/off-by-one.py") && x >= 1;
        let value = if off_by_one { 2 * x + 1 } else { 2 * x };
        Ok(ProgramResult::Returned(json!(value)))
    }

    fn bench(
        &mut self,
        _sized_inputs: &[(u64, Value)],
        _limit: Duration,
    ) -> Result<Vec<(u64, u64)>, RunnerError> {
        Err(RunnerError::Infrastructure("no bench".to_owned()))
    }
}

const SCHEMA: Schema = Schema::Int { min: 0, max: 9 };
const SEED: u64 = 1729;

fn inputs<'a>(
    schema: &'a Schema,
    samples: &'a [(Value, Value)],
    curated: &'a [(Value, Value)],
    count: usize,
) -> SandboxInputs<'a> {
    SandboxInputs {
        comparator: Comparator::Exact,
        returns: "int",
        limit: Duration::from_millis(500),
        schema,
        samples,
        curated,
        generated: Some(Generated {
            seed: SEED,
            count,
            retired: None,
        }),
        ghosts: &[],
    }
}

/// The first value the seeded stream yields for `SCHEMA`.
fn first_candidate() -> Value {
    CandidateStream::new(
        SCHEMA.clone(),
        SEED,
        Vec::new(),
        Vec::new(),
        GeneratorConfig::default(),
    )
    .next()
    .expect("the stream yields a value")
    .value
}

#[test]
fn both_scopes_draw_the_same_hidden_set() {
    let curated = [(json!(4), json!(8))];
    let inputs = inputs(&SCHEMA, &[], &curated, 5);

    let full = run_checks(&inputs, Scope::Everything, &mut Doubler::default()).expect("runs");
    let import = run_checks(&inputs, Scope::FirstError, &mut Doubler::default()).expect("runs");

    assert_eq!(full.findings, Vec::<Finding>::new());
    assert_eq!(full.hidden, import.hidden);
    assert_eq!(full.hidden.len(), 6, "one curated + five generated");
    assert_eq!(
        full.hidden[0],
        HiddenCase {
            source: HiddenSource::Curated,
            input: json!(4),
            expected: json!(8),
        }
    );
}

#[test]
fn a_curated_input_is_never_drawn_again_as_a_generated_case() {
    let first = first_candidate();
    let curated = [(first.clone(), json!(2 * first.as_i64().unwrap_or_default()))];
    let inputs = inputs(&SCHEMA, &[], &curated, 5);

    let outcome = run_checks(&inputs, Scope::Everything, &mut Doubler::default()).expect("runs");

    let generated: Vec<&HiddenCase> = outcome
        .hidden
        .iter()
        .filter(|case| case.source == HiddenSource::Generated)
        .collect();
    assert_eq!(generated.len(), 5);
    assert!(generated.iter().all(|case| case.input != first));
}

#[test]
fn a_curated_case_the_reference_disagrees_with_is_reported_and_not_kept() {
    // The reference doubles 3 to 6, so an authored 7 disagrees.
    let curated = [(json!(3), json!(7))];
    let inputs = inputs(&SCHEMA, &[], &curated, 5);

    let outcome = run_checks(&inputs, Scope::Everything, &mut Doubler::default()).expect("runs");

    assert_eq!(outcome.findings, [Finding::CuratedDisagrees { index: 0 }]);
    assert!(outcome
        .hidden
        .iter()
        .all(|case| case.source == HiddenSource::Generated));
}

#[test]
fn first_error_stops_at_the_first_refusal_and_everything_reports_all() {
    // The reference doubles 1 to 2 and 2 to 4; both authored values are wrong.
    let samples = [(json!(1), json!(3)), (json!(2), json!(5))];
    let mut inputs = inputs(&SCHEMA, &samples, &[], 5);
    inputs.generated = None;

    let mut runner = Doubler::default();
    let import = run_checks(&inputs, Scope::FirstError, &mut runner).expect("runs");
    let full = run_checks(&inputs, Scope::Everything, &mut Doubler::default()).expect("runs");

    assert_eq!(import.findings, [Finding::SampleDisagrees { index: 0 }]);
    assert_eq!(runner.calls, 1);
    assert_eq!(
        full.findings,
        [
            Finding::SampleDisagrees { index: 0 },
            Finding::SampleDisagrees { index: 1 },
        ]
    );
}

#[test]
fn a_domain_too_small_for_hidden_count_is_a_shortfall() {
    // 0, 1 and 2 are the only values, so five unique inputs cannot exist.
    let schema = Schema::Int { min: 0, max: 2 };
    let inputs = inputs(&schema, &[], &[], 5);

    let outcome = run_checks(&inputs, Scope::FirstError, &mut Doubler::default()).expect("runs");

    assert_eq!(
        outcome.findings,
        [Finding::TooFewGenerated { wanted: 5, got: 3 }]
    );
}

#[test]
fn only_everything_checks_that_a_known_ghost_is_minimal() {
    // off-by-one fails on every x >= 1, so the authored input 5 shrinks to 1.
    let ghosts = sandbox_ghosts(
        r#"[[ghost]]
id = "off-by-one"
program = "incorrect/off-by-one.py"
kind = "wrong"
input = 5
got = 11
explanation = "The program adds one to every positive answer."
"#,
    );
    let mut inputs = inputs(&SCHEMA, &[], &[], 5);
    inputs.generated = None;
    inputs.ghosts = &ghosts;

    let full = run_checks(&inputs, Scope::Everything, &mut Doubler::default()).expect("runs");
    let import = run_checks(&inputs, Scope::FirstError, &mut Doubler::default()).expect("runs");

    assert_eq!(
        full.findings,
        [Finding::GhostNotMinimal {
            index: 0,
            id: "off-by-one".to_owned(),
        }]
    );
    assert_eq!(import.findings, Vec::<Finding>::new());
}

#[test]
fn a_ghost_that_does_not_reproduce_carries_its_id() {
    // off-by-one returns 11 for 5, not the authored 12.
    let ghosts = sandbox_ghosts(
        r#"[[ghost]]
id = "off-by-one"
program = "incorrect/off-by-one.py"
kind = "wrong"
input = 5
got = 12
explanation = "The program adds one to every positive answer."
"#,
    );
    let mut inputs = inputs(&SCHEMA, &[], &[], 5);
    inputs.generated = None;
    inputs.ghosts = &ghosts;

    let outcome = run_checks(&inputs, Scope::FirstError, &mut Doubler::default()).expect("runs");

    assert_eq!(
        outcome.findings,
        [Finding::GhostNotReproduced {
            index: 0,
            id: "off-by-one".to_owned(),
        }]
    );
}

#[test]
fn a_retired_generated_case_is_drawn_again() {
    let schema = Schema::Int { min: 0, max: 1000 };
    let leaked = CandidateStream::new(
        schema.clone(),
        SEED,
        Vec::new(),
        Vec::new(),
        GeneratorConfig::default(),
    )
    .find(|candidate| candidate.origin == Origin::Generated)
    .expect("a seeded value")
    .value;
    let retired = |input: &Value, _expected: &Value| *input == leaked;
    let mut inputs = inputs(&schema, &[], &[], 5);
    inputs.generated = Some(Generated {
        seed: SEED,
        count: 5,
        retired: Some(&retired),
    });

    let outcome = run_checks(&inputs, Scope::FirstError, &mut Doubler::default()).expect("runs");

    assert_eq!(outcome.findings, Vec::<Finding>::new());
    assert_eq!(outcome.hidden.len(), 5);
    assert!(outcome.hidden.iter().all(|case| case.input != leaked));
}
