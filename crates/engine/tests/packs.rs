//! Pack tests: the Rust models must reproduce the hand-traced vectors that
//! tie them to the Python files in `packs/_test/`.

mod support;

use counterspell_engine::{
    compare_size, derive_seed, hunt, verify_ghost, FailureKind, Ghost, HuntIdentity, HuntOutcome,
    HuntReport, HuntRequest, ManualClock, NotFoundReason, ProgramResult, ENGINE_VERSION,
    GENERATOR_VERSION,
};
use std::cmp::Ordering;
use std::time::Duration;
use support::{
    incorrect, load_pack, packs_root, reference, Expected, Model, ModelOracle, Pack, Vector, PACKS,
};

/// Checks one model against one vector, comparing values or exception classes.
fn assert_matches(model: Model, vector: &Vector, label: &str) {
    let got = model(&vector.input);
    match (&vector.expected, &got) {
        (Expected::Returned(want), ProgramResult::Returned(have)) => {
            assert_eq!(have, want, "{label}: input {}", vector.input);
        }
        (
            Expected::Crashed(class),
            ProgramResult::Crashed {
                exception_class, ..
            },
        ) => {
            assert_eq!(exception_class, class, "{label}: input {}", vector.input);
        }
        _ => panic!(
            "{label}: input {} expected {:?} but the model returned {:?}",
            vector.input, vector.expected, got
        ),
    }
}

#[test]
fn vectors_reference_models_reproduce_the_golden_and_sample_vectors() {
    for slug in PACKS {
        let pack = load_pack(slug);
        let model = reference(slug);
        for vector in pack.golden.iter().chain(pack.samples.iter()) {
            assert_matches(model, vector, &format!("{slug} reference"));
        }
    }
}

#[test]
fn vectors_incorrect_models_reproduce_their_hand_traced_vectors() {
    for slug in PACKS {
        let pack = load_pack(slug);
        for bug in &pack.incorrect {
            let model = incorrect(slug, &bug.id);
            for vector in &bug.vectors {
                assert_matches(model, vector, &format!("{slug}/{}", bug.id));
            }
        }
    }
}

/// The model oracle charges this much clock time per call.
const COST_PER_CALL: Duration = Duration::from_millis(5);

/// The hunt request for a pack, with its samples, known ghosts and tags.
fn hunt_request(pack: &Pack) -> HuntRequest {
    let identity = HuntIdentity {
        problem_slug: pack.slug.clone(),
        code_sha256: "00".to_owned(),
        language: "python".to_owned(),
        problem_version: pack.problem_version,
        image_digest: "sha256:model-oracle".to_owned(),
    };
    HuntRequest {
        samples: pack
            .samples
            .iter()
            .map(|vector| vector.input.clone())
            .collect(),
        known_ghosts: pack.known_ghosts.clone(),
        concept_tags: pack.concept_tags.clone(),
        ..HuntRequest::new(pack.schema.clone(), pack.comparator, identity)
    }
}

/// Runs one hunt with a reference model and a user model on a fresh manual clock.
fn run_hunt(req: &HuntRequest, reference_model: Model, user_model: Model) -> HuntReport {
    let clock = ManualClock::new();
    let mut oracle = ModelOracle::new(reference_model, user_model, &clock, COST_PER_CALL);
    hunt(req, &mut oracle, &clock)
}

/// Re-checks a ghost by calling the model functions directly and comparing
/// with plain `==`. It never calls `classify` or `Comparator`, so a bug they
/// share with the engine cannot hide here.
fn independent_recheck(reference_model: Model, user_model: Model, ghost: &Ghost, label: &str) {
    let reference_result = reference_model(&ghost.input);
    let user_result = user_model(&ghost.input);
    match ghost.predicate.kind {
        FailureKind::WrongAnswer => {
            let (ProgramResult::Returned(expected), ProgramResult::Returned(got)) =
                (&reference_result, &user_result)
            else {
                panic!("{label}: a wrong answer needs two returned values");
            };
            assert_ne!(got, expected, "{label}: the two results must differ");
            assert_eq!(ghost.expected.as_ref(), Some(expected), "{label}: expected");
            assert_eq!(
                ghost.got,
                ProgramResult::Returned(got.clone()),
                "{label}: got"
            );
        }
        FailureKind::RuntimeError => {
            let ProgramResult::Crashed {
                exception_class, ..
            } = &user_result
            else {
                panic!("{label}: a crash must raise an exception");
            };
            assert_eq!(
                ghost.predicate.exception_class.as_deref(),
                Some(exception_class.as_str()),
                "{label}: exception class"
            );
        }
        FailureKind::Timeout => panic!("{label}: the model oracles never time out"),
    }
}

/// Returns the name of the first `def` in a Python source text.
fn function_name(source: &str) -> Option<&str> {
    source
        .lines()
        .find_map(|line| line.trim_start().strip_prefix("def "))
        .and_then(|rest| rest.split('(').next())
}

#[test]
fn packs_static_checks_list_every_incorrect_file_and_share_the_reference_name() {
    for slug in PACKS {
        let pack = load_pack(slug);
        let dir = packs_root().join(slug);
        assert!(dir.join("problem.md").is_file(), "{slug}: problem.md");

        let reference_source = std::fs::read_to_string(dir.join("reference/solution.py"))
            .unwrap_or_else(|error| panic!("{slug}: reference missing: {error}"));
        let name = function_name(&reference_source)
            .unwrap_or_else(|| panic!("{slug}: reference has no def line"));

        let listed: Vec<&str> = pack.incorrect.iter().map(|bug| bug.file.as_str()).collect();
        let on_disk = std::fs::read_dir(dir.join("incorrect"))
            .unwrap_or_else(|error| panic!("{slug}: incorrect/ missing: {error}"));
        for entry in on_disk {
            let path = entry.expect("readable directory entry").path();
            let file = format!(
                "incorrect/{}",
                path.file_name()
                    .and_then(|name| name.to_str())
                    .unwrap_or("")
            );
            assert!(
                listed.contains(&file.as_str()),
                "{slug}: {file} is not listed in ghost.json"
            );
        }

        for bug in &pack.incorrect {
            let source = std::fs::read_to_string(dir.join(&bug.file))
                .unwrap_or_else(|error| panic!("{slug}/{}: {error}", bug.id));
            assert_eq!(
                function_name(&source),
                Some(name),
                "{slug}/{}: must define the same function as the reference",
                bug.id
            );
        }
    }
}

#[test]
fn packs_each_incorrect_program_yields_a_verified_minimized_ghost() {
    for slug in PACKS {
        let pack = load_pack(slug);
        let reference_model = reference(slug);
        for bug in &pack.incorrect {
            let label = format!("{slug}/{}", bug.id);
            let user_model = incorrect(slug, &bug.id);
            let req = hunt_request(&pack);

            let report = run_hunt(&req, reference_model, user_model);
            let HuntOutcome::Found(ghost) = &report.outcome else {
                panic!("{label}: expected a ghost, got {:?}", report.outcome);
            };

            assert_eq!(ghost.predicate.kind, bug.kind, "{label}: failure kind");
            assert_eq!(
                ghost.predicate.exception_class, bug.exception_class,
                "{label}: exception class"
            );
            assert!(
                req.schema.contains(&ghost.input).is_ok(),
                "{label}: input breaks the schema"
            );

            assert_eq!(
                report.provenance.generator_version, GENERATOR_VERSION,
                "{label}"
            );
            assert_eq!(report.provenance.engine_version, ENGINE_VERSION, "{label}");
            assert_eq!(
                report.provenance.problem_version, pack.problem_version,
                "{label}"
            );
            assert_eq!(
                report.provenance.image_digest, req.identity.image_digest,
                "{label}"
            );
            assert_eq!(
                report.provenance.seed,
                derive_seed(
                    &req.identity.problem_slug,
                    &req.identity.code_sha256,
                    &req.identity.language
                ),
                "{label}: seed"
            );

            // Regression expectation from the design's table, not a claim of minimality.
            assert_eq!(ghost.input, bug.smallest_input, "{label}: smallest input");
            assert_eq!(ghost.size.nodes, bug.smallest_nodes, "{label}: size nodes");
            assert_eq!(
                ghost.size.magnitude, bug.smallest_magnitude,
                "{label}: size magnitude"
            );

            let clock = ManualClock::new();
            let mut fresh = ModelOracle::new(reference_model, user_model, &clock, COST_PER_CALL);
            assert!(
                verify_ghost(&req, ghost, &mut fresh, &clock),
                "{label}: fresh re-run"
            );

            independent_recheck(reference_model, user_model, ghost, &label);

            let again = run_hunt(&req, reference_model, user_model);
            assert_eq!(report, again, "{label}: a second run must match");

            let steps = &ghost.shrink.steps;
            assert!(steps.len() <= 20, "{label}: trace cap");
            assert_eq!(
                ghost.shrink.total_steps + 1,
                u32::try_from(steps.len()).expect("small trace") + ghost.shrink.omitted_steps,
                "{label}: trace counts"
            );
            assert_eq!(
                steps.last().map(|step| &step.input),
                Some(&ghost.input),
                "{label}: last step"
            );
            for pair in steps.windows(2) {
                assert!(
                    pair[0].index < pair[1].index,
                    "{label}: indices must increase"
                );
                assert_eq!(
                    compare_size(&pair[1].input, &pair[0].input),
                    Ordering::Less,
                    "{label}: recorded steps must strictly shrink"
                );
            }
        }
    }
}

#[test]
fn packs_each_reference_hunted_against_itself_reports_not_found() {
    for slug in PACKS {
        let pack = load_pack(slug);
        let model = reference(slug);
        let req = hunt_request(&pack);
        let report = run_hunt(&req, model, model);
        match report.outcome {
            HuntOutcome::NotFound(reason) => {
                assert_eq!(reason, NotFoundReason::CandidateBudget, "{slug}: reason");
                assert!(
                    reason.to_string().starts_with("No failing input found"),
                    "{slug}: wording"
                );
            }
            other => panic!("{slug}: a correct reference must give NotFound, got {other:?}"),
        }
    }
}
