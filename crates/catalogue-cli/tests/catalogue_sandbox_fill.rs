// SPDX-License-Identifier: MIT
//! Sandbox-fill tests (problem-catalogue C16). These use a scripted `PackRunner` —
//! the grimoire model-runner convention — so they are **not** sandbox evidence.
//! They prove the fill's check order, refusal mapping, and that the hidden expected
//! values are the reference's own answers kept in memory.

use std::collections::{HashMap, VecDeque};
use std::time::Duration;

use counterspell_catalogue_cli::sandbox_fill::{
    sandbox_fill, CaseSource, FillOutcome, FillRefusal, SandboxCheck,
};
use counterspell_engine::{Comparator, ProgramResult, Schema};
use counterspell_grimoire::ghosts::sandbox_ghosts;
use counterspell_grimoire::runner::{PackRunner, RunnerError, Target};
use counterspell_judge::catalogue::retired::{canonical_hash, RetiredFile};
use serde_json::{json, Value};

/// A scripted runner: `run` pops the next result scripted for its target —
/// `"reference"` for the trusted reference, or the program's `incorrect/` path.
/// `bench` is never exercised by the fill.
#[derive(Default)]
struct Scripted {
    reference: VecDeque<Result<ProgramResult, RunnerError>>,
    programs: HashMap<String, VecDeque<Result<ProgramResult, RunnerError>>>,
}

impl Scripted {
    fn reference(&mut self, results: impl IntoIterator<Item = ProgramResult>) {
        self.reference = results.into_iter().map(Ok).collect();
    }

    fn program(&mut self, path: &str, results: impl IntoIterator<Item = ProgramResult>) {
        self.programs
            .insert(path.to_owned(), results.into_iter().map(Ok).collect());
    }
}

impl PackRunner for Scripted {
    fn run(
        &mut self,
        target: Target<'_>,
        _input: &Value,
        _limit: Duration,
    ) -> Result<ProgramResult, RunnerError> {
        let queue = match target {
            Target::Reference => &mut self.reference,
            Target::Program(path) => self
                .programs
                .get_mut(path)
                .unwrap_or_else(|| panic!("unscripted program {path}")),
        };
        queue
            .pop_front()
            .unwrap_or_else(|| panic!("unscripted reference run"))
    }

    fn bench(
        &mut self,
        _sized_inputs: &[(u64, Value)],
        _limit: Duration,
    ) -> Result<Vec<(u64, u64)>, RunnerError> {
        Err(RunnerError::Infrastructure(
            "bench is not scripted".to_owned(),
        ))
    }
}

/// The smallest well-formed check: an int domain with plenty of room, one sample,
/// one curated case, three generated cases and no ghosts or retired values.
fn check<'a>(
    examples: &'a [(Value, Value)],
    curated: &'a [(Value, Value)],
    ghosts: &'a [counterspell_grimoire::ghosts::SandboxGhost],
    retired: &'a RetiredFile,
) -> SandboxCheck<'a> {
    SandboxCheck {
        slug: "count-pairs",
        pack_version: 1,
        comparator: Comparator::Exact,
        returns: "int",
        limit: Duration::from_millis(500),
        examples,
        curated,
        seed: 1729,
        hidden_count: 3,
        schema: Schema::Int { min: 0, max: 9 },
        ghosts,
        retired,
        image_digest: "sha256:test-image",
    }
}

#[test]
fn reference_matching_produces_the_fill() {
    let examples = vec![(json!(1), json!(1))];
    let curated = vec![(json!(7), json!(49))];
    let retired = RetiredFile::default();
    // One sample, one curated, three generated answers.
    let mut runner = Scripted::default();
    runner.reference([
        ProgramResult::Returned(json!(1)),
        ProgramResult::Returned(json!(49)),
        ProgramResult::Returned(json!(0)),
        ProgramResult::Returned(json!(1)),
        ProgramResult::Returned(json!(4)),
    ]);
    let ghosts = Vec::new();

    let outcome = sandbox_fill(&check(&examples, &curated, &ghosts, &retired), &mut runner)
        .expect("no infrastructure fault");

    let FillOutcome::Filled(fill) = outcome else {
        panic!("a clean script fills: {outcome:?}")
    };
    assert_eq!(fill.digest, "sha256:test-image");
    assert_eq!(fill.hidden.len(), 4, "one curated + three generated");
    assert_eq!(fill.hidden[0].source, CaseSource::Curated);
    assert_eq!(fill.hidden[0].expected, json!(49));
    assert!(
        fill.hidden[1..]
            .iter()
            .all(|case| case.source == CaseSource::Generated),
        "the rest are generated cases"
    );
    assert!(
        fill.hidden[1..].iter().all(|case| case.expected.is_i64()),
        "the generated expected values are the reference's own answers"
    );
}

#[test]
fn a_sample_the_reference_disagrees_with_is_refused() {
    let examples = vec![(json!(1), json!(2))];
    let retired = RetiredFile::default();
    let mut runner = Scripted::default();
    runner.reference([ProgramResult::Returned(json!(1))]);

    let outcome = sandbox_fill(&check(&examples, &[], &[], &retired), &mut runner)
        .expect("no infrastructure fault");
    assert_eq!(
        outcome,
        FillOutcome::Refused(FillRefusal::AuthoredDisagrees {
            source: CaseSource::Sample,
            index: 0
        })
    );
    assert_eq!(
        FillRefusal::AuthoredDisagrees {
            source: CaseSource::Sample,
            index: 0
        }
        .code(),
        "SBX-01"
    );
}

#[test]
fn a_curated_case_the_reference_disagrees_with_is_refused() {
    let curated = vec![(json!(7), json!(50))];
    let retired = RetiredFile::default();
    let mut runner = Scripted::default();
    runner.reference([ProgramResult::Returned(json!(49))]);

    let outcome = sandbox_fill(&check(&[], &curated, &[], &retired), &mut runner)
        .expect("no infrastructure fault");
    assert_eq!(
        outcome,
        FillOutcome::Refused(FillRefusal::AuthoredDisagrees {
            source: CaseSource::Curated,
            index: 0
        })
    );
}

#[test]
fn a_reference_timeout_on_a_hidden_input_is_refused() {
    let retired = RetiredFile::default();
    let mut runner = Scripted::default();
    runner.reference([ProgramResult::TimedOut]);

    let outcome = sandbox_fill(&check(&[], &[], &[], &retired), &mut runner)
        .expect("no infrastructure fault");
    assert_eq!(
        outcome,
        FillOutcome::Refused(FillRefusal::ReferenceFailed {
            source: CaseSource::Generated,
            index: 0
        })
    );
    assert_eq!(
        FillRefusal::ReferenceFailed {
            source: CaseSource::Generated,
            index: 0
        }
        .code(),
        "SBX-02"
    );
}

#[test]
fn a_wrong_shaped_reference_value_is_refused() {
    let retired = RetiredFile::default();
    let mut runner = Scripted::default();
    // The returns name is "int"; a string is the wrong shape.
    runner.reference([ProgramResult::Returned(json!("oops"))]);

    let outcome = sandbox_fill(&check(&[], &[], &[], &retired), &mut runner)
        .expect("no infrastructure fault");
    assert_eq!(
        outcome,
        FillOutcome::Refused(FillRefusal::ReferenceFailed {
            source: CaseSource::Generated,
            index: 0
        })
    );
}

#[test]
fn a_ghost_that_does_not_reproduce_is_refused() {
    let text = "[[ghost]]\n\
                id = \"g1\"\n\
                program = \"incorrect/off_by_one.py\"\n\
                kind = \"wrong\"\n\
                input = [3]\n\
                got = 4\n\
                concepts = []\n\
                explanation = \"The loop reads one past the end.\"\n";
    let ghosts = sandbox_ghosts(text);
    let retired = RetiredFile::default();
    let mut runner = Scripted::default();
    // Three generated cases pass, then the ghost program returns the same value as
    // the reference — the recorded wrong answer does not reproduce.
    runner.reference([
        ProgramResult::Returned(json!(0)),
        ProgramResult::Returned(json!(1)),
        ProgramResult::Returned(json!(4)),
        ProgramResult::Returned(json!(5)),
    ]);
    runner.program(
        "incorrect/off_by_one.py",
        [ProgramResult::Returned(json!(5))],
    );

    let outcome = sandbox_fill(&check(&[], &[], &ghosts, &retired), &mut runner)
        .expect("no infrastructure fault");
    assert_eq!(
        outcome,
        FillOutcome::Refused(FillRefusal::GhostNotReproduced {
            ghost_id: "g1".to_owned()
        })
    );
    assert_eq!(
        FillRefusal::GhostNotReproduced {
            ghost_id: "g1".to_owned()
        }
        .code(),
        "SBX-03"
    );
}

#[test]
fn a_ghost_that_reproduces_is_accepted() {
    let text = "[[ghost]]\n\
                id = \"g1\"\n\
                program = \"incorrect/off_by_one.py\"\n\
                kind = \"wrong\"\n\
                input = [3]\n\
                got = 4\n\
                concepts = []\n\
                explanation = \"The loop reads one past the end.\"\n";
    let ghosts = sandbox_ghosts(text);
    let retired = RetiredFile::default();
    let mut runner = Scripted::default();
    runner.reference([
        ProgramResult::Returned(json!(0)),
        ProgramResult::Returned(json!(1)),
        ProgramResult::Returned(json!(4)),
        ProgramResult::Returned(json!(5)),
    ]);
    // The program returns the recorded wrong answer on the ghost input.
    runner.program(
        "incorrect/off_by_one.py",
        [ProgramResult::Returned(json!(4))],
    );

    let outcome = sandbox_fill(&check(&[], &[], &ghosts, &retired), &mut runner)
        .expect("no infrastructure fault");
    assert!(
        matches!(outcome, FillOutcome::Filled(_)),
        "a reproducing ghost fills: {outcome:?}"
    );
}

#[test]
fn a_runner_fault_is_an_error_not_a_refusal() {
    let retired = RetiredFile::default();
    let mut runner = Scripted {
        reference: VecDeque::from([Err(RunnerError::Infrastructure(
            "daemon unreachable".to_owned(),
        ))]),
        programs: HashMap::new(),
    };

    let result = sandbox_fill(&check(&[], &[], &[], &retired), &mut runner);
    assert!(
        matches!(result, Err(RunnerError::Infrastructure(_))),
        "a runner fault is the exit-2 path, never a pack refusal"
    );
}

#[test]
fn a_retired_hidden_value_is_refused_by_prefix_only() {
    let curated = vec![(json!(7), json!(49))];
    let hash = canonical_hash(&json!(7), &json!(49));
    let hex_hash: String = {
        use std::fmt::Write as _;
        hash.iter().fold(String::new(), |mut out, byte| {
            let _ = write!(out, "{byte:02x}");
            out
        })
    };
    let text = format!("[[retired]]\nslug = \"count-pairs\"\nsha256 = \"{hex_hash}\"\n");
    let retired = parse_retired_text(&text);
    let mut runner = Scripted::default();
    // One curated plus three generated cases run before the retired check fires.
    runner.reference([
        ProgramResult::Returned(json!(49)),
        ProgramResult::Returned(json!(0)),
        ProgramResult::Returned(json!(1)),
        ProgramResult::Returned(json!(4)),
    ]);

    let outcome = sandbox_fill(&check(&[], &curated, &[], &retired), &mut runner)
        .expect("no infrastructure fault");
    let FillOutcome::Refused(FillRefusal::Retired(refusal)) = &outcome else {
        panic!("a retired value refuses: {outcome:?}")
    };
    assert_eq!(outcome_code(&outcome), "retired_hidden_value");
    let shown = refusal.to_string();
    assert!(
        shown.contains(&hex_hash[..12]),
        "the refusal shows the hash prefix: {shown}"
    );
    assert!(
        !shown.contains(&hex_hash[12..]),
        "the refusal never shows the full hash: {shown}"
    );
}

#[test]
fn a_retired_slug_without_rotation_is_refused() {
    // The retired hash does not match any case, but the slug has retired values and
    // this pack version has no rotation record.
    let retired = parse_retired_text("[[retired]]\nslug = \"count-pairs\"\nsha256 = \"00\"\n");
    let mut runner = Scripted::default();
    runner.reference([
        ProgramResult::Returned(json!(0)),
        ProgramResult::Returned(json!(1)),
        ProgramResult::Returned(json!(4)),
    ]);

    let outcome = sandbox_fill(&check(&[], &[], &[], &retired), &mut runner)
        .expect("no infrastructure fault");
    assert!(
        matches!(outcome, FillOutcome::Refused(FillRefusal::Retired(_))),
        "a missing rotation refuses: {outcome:?}"
    );
    assert_eq!(outcome_code(&outcome), "rotation_missing");
}

#[test]
fn recorded_rotation_skips_the_seed_independent_prefix() {
    use std::fmt::Write as _;

    let mut hash = String::new();
    for byte in canonical_hash(&json!(0), &json!(0)) {
        write!(hash, "{byte:02x}").expect("write hex to string");
    }
    let retired = parse_retired_text(&format!(
        "[[retired]]\nslug = \"count-pairs\"\nsha256 = \"{hash}\"\n\
         [[rotated]]\nslug = \"count-pairs\"\npack_version = 1\nrotated_on = \"2026-11-21\"\n"
    ));
    let mut check = check(&[], &[], &[], &retired);
    check.schema = Schema::Int { min: 0, max: 1000 };
    let mut runner = Scripted::default();
    runner.reference((0..check.hidden_count).map(|_| ProgramResult::Returned(json!(0))));

    let outcome = sandbox_fill(&check, &mut runner).expect("runner answers");

    let FillOutcome::Filled(fill) = outcome else {
        panic!("recorded rotation should select distinct generated values: {outcome:?}");
    };
    assert_eq!(fill.hidden.len(), check.hidden_count);
    assert!(fill.hidden.iter().all(|case| case.input != json!(0)));
}

#[test]
fn recorded_rotation_resamples_retired_seeded_values() {
    use std::fmt::Write as _;

    use counterspell_engine::generate::{CandidateStream, GeneratorConfig, Origin};
    use counterspell_judge::catalogue::retired::{Retired, Rotated};

    let empty = RetiredFile::default();
    let mut check = check(&[], &[], &[], &empty);
    check.schema = Schema::Int { min: 0, max: 1000 };
    let leaked = CandidateStream::new(
        check.schema.clone(),
        check.seed,
        Vec::new(),
        Vec::new(),
        GeneratorConfig::default(),
    )
    .find(|candidate| candidate.origin == Origin::Generated)
    .expect("seeded value")
    .value;
    let mut hash = String::new();
    for byte in canonical_hash(&leaked, &json!(0)) {
        write!(hash, "{byte:02x}").expect("write hash");
    }
    let retired = RetiredFile {
        retired: vec![Retired {
            slug: "count-pairs".into(),
            sha256: hash,
        }],
        rotated: vec![Rotated {
            slug: "count-pairs".into(),
            pack_version: 1,
            rotated_on: "2026-11-21".into(),
        }],
    };
    check.retired = &retired;
    let mut runner = Scripted::default();
    runner.reference((0..=check.hidden_count).map(|_| ProgramResult::Returned(json!(0))));

    let outcome = sandbox_fill(&check, &mut runner).expect("runner answers");
    let FillOutcome::Filled(fill) = outcome else {
        panic!("an approved rotation should replace a retired generated value: {outcome:?}");
    };
    assert_eq!(fill.hidden.len(), check.hidden_count);
    assert!(fill.hidden.iter().all(|case| case.input != leaked));
}

/// Parses retired-values text the same way the importer does.
fn parse_retired_text(text: &str) -> RetiredFile {
    counterspell_judge::catalogue::retired::parse_retired(text).expect("the retired file parses")
}

/// The stable code of a refusal outcome.
fn outcome_code(outcome: &FillOutcome) -> &'static str {
    match outcome {
        FillOutcome::Refused(refusal) => refusal.code(),
        FillOutcome::Filled(_) => "filled",
    }
}
