// SPDX-License-Identifier: MIT
//! The submit-path ghost hunt (ide-workspace T25).
//!
//! A failing submission asks the engine to search for the minimal failing input
//! through the sandbox oracle. This module is the pure seam of that feature:
//! [`request`] decides whether a finished run may hunt and builds the engine
//! request from the problem spec, and [`events`] maps a hunt report to the
//! bounded public `shrink_step`/`ghost` events the browser parses. The executor
//! owns the sandbox machinery and the verdict — a hunt can only add events,
//! never change what the stored suite already decided.

use counterspell_common::events::{BoundedJson, PublicEvent};
use counterspell_common::status::{RunMode, Verdict};
use counterspell_engine::shrink::{ShrinkReport, ShrinkStep};
use counterspell_engine::{
    FailureKind, FailurePredicate, Ghost, HuntIdentity, HuntOutcome, HuntReport, HuntRequest,
    ProgramResult, Provenance,
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

use super::problem::ProblemSpec;
use crate::executor::RunInput;

/// Builds the hunt request for a finished submission, or `None` when the run
/// may not hunt. Only verdicts a failure predicate can describe hunt; a problem
/// without an input schema yields no candidates. `samples` carries public
/// sample inputs only — the stored hidden tests are never a candidate source,
/// so nothing secret leaves the judge through the request.
#[must_use]
pub(crate) fn request(
    run: &RunInput,
    spec: &ProblemSpec,
    image_digest: &str,
    verdict: Verdict,
) -> Option<HuntRequest> {
    // Hunts are submit-only: a sample run and a custom ghost run never hunt,
    // even when the problem has a schema and the verdict would qualify.
    if run.mode != RunMode::Submit || run.ghost_input.is_some() {
        return None;
    }
    let schema = spec.input_schema.clone()?;
    let kind = match verdict {
        Verdict::WrongAnswer => FailureKind::WrongAnswer,
        Verdict::RuntimeError => FailureKind::RuntimeError,
        Verdict::TimeLimit => FailureKind::Timeout,
        // Accepted has no failure to reproduce, a compile error never ran, and
        // the engine's predicate language does not describe resource limits.
        Verdict::Accepted | Verdict::CompileError | Verdict::MemoryLimit | Verdict::OutputLimit => {
            return None
        }
    };
    let identity = HuntIdentity {
        problem_slug: spec.slug.clone(),
        code_sha256: format!("{:x}", Sha256::digest(run.code.as_bytes())),
        language: run.lang.clone(),
        problem_version: spec.version,
        image_digest: image_digest.to_owned(),
    };
    let mut request = HuntRequest::new(schema, spec.comparator, identity);
    request.target = Some(FailurePredicate {
        kind,
        exception_class: None,
    });
    request.samples = spec
        .tests
        .iter()
        .filter(|test| test.is_sample)
        .map(|test| Value::Array(test.args.clone()))
        .collect();
    request.known_ghosts.clone_from(&spec.known_ghosts);
    request.concept_tags.clone_from(&spec.concept_tags);
    Some(request)
}

/// Maps a finished hunt to the public events it may emit. `Found` produces one
/// `shrink_step` per recorded step that fits the event cap and one `ghost`
/// event; every other outcome emits nothing — the run's verdict event already
/// carries the result, so a fruitless or aborted hunt stays silent after
/// `ghost_hunting`.
#[must_use]
pub(crate) fn events(report: &HuntReport) -> Vec<PublicEvent> {
    let HuntOutcome::Found(ghost) = &report.outcome else {
        return Vec::new();
    };
    // A step that cannot fit the event cap is skipped in both places it
    // appears — its own event and the trace inside the ghost — and counted in
    // `omitted_steps`, so the card's "more steps than shown" stays honest.
    let mut events = Vec::with_capacity(ghost.shrink.steps.len() + 1);
    let mut kept = Vec::with_capacity(ghost.shrink.steps.len());
    let mut dropped = 0_u32;
    for step in &ghost.shrink.steps {
        let payload = step_payload(step);
        match BoundedJson::from_serialize(&payload) {
            Ok(bounded) => {
                events.push(PublicEvent::ShrinkStep(bounded));
                kept.push(payload);
            }
            Err(error) => {
                dropped = dropped.saturating_add(1);
                tracing::warn!(%error, index = step.index, "a shrink step exceeded the event cap");
            }
        }
    }
    let shrink = shrink_payload(&ghost.shrink, &kept, dropped);
    let payload = ghost_payload(ghost, &report.provenance, &shrink, false);
    if let Ok(bounded) = BoundedJson::from_serialize(&payload) {
        events.push(PublicEvent::Ghost(bounded));
    } else {
        // A large `got`/`expected`/diff must not lose the ghost: the slim
        // payload keeps input, shrink trace, provenance and the one-liner.
        let slim = ghost_payload(ghost, &report.provenance, &shrink, true);
        match BoundedJson::from_serialize(&slim) {
            Ok(bounded) => events.push(PublicEvent::Ghost(bounded)),
            Err(error) => {
                tracing::warn!(%error, "the ghost payload exceeded the event cap");
            }
        }
    }
    events
}

/// The wire shape of one `shrink_step` event: exactly the fields the browser
/// parser reads.
fn step_payload(step: &ShrinkStep) -> Value {
    json!({
        "index": step.index,
        "input": step.input,
        "evaluations": step.evaluations,
    })
}

/// The wire shape of the `ghost` event — exactly the fields the browser parser
/// accepts. `slim` drops `expected`, `got` and the autopsy differences when the
/// full payload would exceed the event cap.
fn ghost_payload(ghost: &Ghost, provenance: &Provenance, shrink: &Value, slim: bool) -> Value {
    let mut autopsy = serde_json::to_value(&ghost.autopsy).unwrap_or_else(|_| json!({}));
    let (expected, got) = if slim {
        autopsy["differences"] = json!([]);
        autopsy["differences_truncated"] = json!(true);
        (Value::Null, Value::Null)
    } else {
        (
            ghost.expected.clone().unwrap_or(Value::Null),
            got_value(&ghost.got),
        )
    };
    json!({
        "kind": ghost.predicate.kind,
        "input": ghost.input,
        "expected": expected,
        "got": got,
        "explanation": ghost.autopsy.explanation,
        "budget_exhausted": ghost.budget_exhausted,
        "shrink": shrink,
        // A u64 seed cannot survive a JavaScript number, so the wire carries it
        // as a decimal string — the same convention `u128_as_string` uses for
        // the engine's size magnitude.
        "provenance": {
            "seed": provenance.seed.to_string(),
            "generator_version": provenance.generator_version,
            "engine_version": provenance.engine_version,
            "problem_version": provenance.problem_version,
            "image_digest": provenance.image_digest,
        },
        "autopsy": autopsy,
    })
}

/// The wire shape of the shrink trace inside the ghost payload: the steps that
/// fit the event cap, with the ones that did not counted as omitted.
fn shrink_payload(report: &ShrinkReport, kept: &[Value], dropped: u32) -> Value {
    json!({
        "steps": kept,
        "total_steps": report.total_steps,
        "omitted_steps": report.omitted_steps.saturating_add(dropped),
        "evaluations": report.evaluations,
        "stop": report.stop,
    })
}

/// The public `got` of a ghost: the returned JSON for a return, a small object
/// naming the exception class and bounded message for a crash, `null` for a
/// timeout, and a bounded snippet for garbled output. The engine already bounds
/// untrusted text when it stores a result in a ghost; it is sanitised again
/// here because this is the boundary where it becomes public.
fn got_value(result: &ProgramResult) -> Value {
    match result.clone().sanitised() {
        ProgramResult::Returned(value) => value,
        ProgramResult::Crashed {
            exception_class,
            message,
        } => json!({
            "exception_class": exception_class,
            "message": message,
        }),
        ProgramResult::TimedOut => Value::Null,
        ProgramResult::Garbled { snippet } => json!({ "garbled": snippet }),
    }
}

#[cfg(test)]
mod tests {
    use counterspell_common::ids::RunId;
    use counterspell_common::status::RunMode;
    use counterspell_engine::generate::Origin;
    use counterspell_engine::shrink::ShrinkStop;
    use counterspell_engine::{
        size_of, AbortReason, Autopsy, Budget, Comparator, HuntStats, KnownGhost, NotFoundReason,
        Schema,
    };

    use super::*;
    use crate::sandbox::problem::TestCase;

    /// A problem a failed submit may hunt on: an input schema, samples, known
    /// ghosts and concept tags.
    fn spec() -> ProblemSpec {
        ProblemSpec {
            slug: "add-two".to_owned(),
            version: 3,
            signature: json!({"function": "add_two"}),
            comparator: Comparator::Exact,
            time_limit_ms: None,
            tests: vec![
                TestCase {
                    ordinal: 1,
                    is_sample: true,
                    args: vec![json!(1), json!(2)],
                    expected: json!(3),
                },
                TestCase {
                    ordinal: 2,
                    is_sample: false,
                    args: vec![json!(7), json!(8)],
                    expected: json!(15),
                },
            ],
            reference_python: Some("def add_two(a, b):\n    return a + b\n".to_owned()),
            bench: None,
            generator_schema: None,
            reference_complexity: None,
            input_schema: Some(Schema::Tuple {
                items: vec![
                    Schema::Int { min: 0, max: 10 },
                    Schema::Int { min: 0, max: 10 },
                ],
            }),
            known_ghosts: vec![KnownGhost {
                id: "kg-1".to_owned(),
                input: json!([0, 0]),
                kind: FailureKind::WrongAnswer,
                exception_class: None,
                got: Some(json!(0)),
                explanation: "the author multiplied instead".to_owned(),
                concept_tags: vec!["arithmetic".to_owned()],
            }],
            concept_tags: vec!["arithmetic".to_owned()],
        }
    }

    /// The failing submission a hunt request is built for.
    fn run() -> RunInput {
        RunInput {
            run_id: RunId::new(),
            problem_slug: "add-two".to_owned(),
            problem_version: 3,
            lang: "python".to_owned(),
            mode: RunMode::Submit,
            ghost_input: None,
            code: "def add_two(a, b):\n    return a * b\n".to_owned(),
            attempt: 1,
        }
    }

    fn step(index: u32, input: Value) -> ShrinkStep {
        ShrinkStep {
            index,
            size: size_of(&input),
            input,
            evaluations: index + 1,
        }
    }

    /// A minimal ghost for the given failure, with a two-step shrink trace.
    fn ghost(kind: FailureKind, got: ProgramResult) -> Box<Ghost> {
        let input = json!([0, 0]);
        Box::new(Ghost {
            predicate: FailurePredicate {
                kind,
                exception_class: None,
            },
            input: input.clone(),
            expected: Some(json!(0)),
            got,
            found_via: Origin::Generated,
            size: size_of(&input),
            shrink: ShrinkReport {
                steps: vec![step(0, json!([5, 5])), step(1, json!([0, 0]))],
                total_steps: 1,
                omitted_steps: 0,
                evaluations: 12,
                stop: ShrinkStop::LocalMinimum,
            },
            budget_exhausted: false,
            verification_runs: 2,
            autopsy: Autopsy {
                explanation: "the product is not the sum".to_owned(),
                differences: Vec::new(),
                differences_truncated: false,
                concept_tags: vec!["arithmetic".to_owned()],
                authored_mistake: None,
            },
        })
    }

    fn report(outcome: HuntOutcome) -> HuntReport {
        HuntReport {
            outcome,
            provenance: Provenance {
                seed: 42,
                generator_version: "2026.1".to_owned(),
                engine_version: "0.1.0".to_owned(),
                problem_version: 3,
                image_digest: "sha256:img".to_owned(),
            },
            budget: Budget::default(),
            stats: HuntStats::default(),
        }
    }

    #[test]
    fn the_request_identifies_the_run() {
        let run = run();
        let digest = format!("{:x}", Sha256::digest(run.code.as_bytes()));
        let request = request(&run, &spec(), "sha256:img", Verdict::WrongAnswer)
            .expect("a failing submit on a schemad problem hunts");
        assert_eq!(request.identity.problem_slug, "add-two");
        assert_eq!(request.identity.code_sha256, digest);
        assert_eq!(request.identity.language, "python");
        assert_eq!(request.identity.problem_version, 3);
        assert_eq!(request.identity.image_digest, "sha256:img");
        assert_eq!(request.schema, spec().input_schema.unwrap());
        assert_eq!(request.comparator, Comparator::Exact);
        assert_eq!(request.budget, Budget::default());
    }

    #[test]
    fn the_request_targets_the_verdicts_failure() {
        let run = run();
        for (verdict, kind) in [
            (Verdict::WrongAnswer, FailureKind::WrongAnswer),
            (Verdict::RuntimeError, FailureKind::RuntimeError),
            (Verdict::TimeLimit, FailureKind::Timeout),
        ] {
            let request = request(&run, &spec(), "img", verdict).expect("hunts");
            assert_eq!(
                request.target,
                Some(FailurePredicate {
                    kind,
                    exception_class: None,
                }),
                "{verdict:?} hunts its own failure kind"
            );
        }
        for verdict in [
            Verdict::Accepted,
            Verdict::CompileError,
            Verdict::MemoryLimit,
            Verdict::OutputLimit,
        ] {
            assert!(
                request(&run, &spec(), "img", verdict).is_none(),
                "{verdict:?} never hunts"
            );
        }
    }

    #[test]
    fn a_spec_without_an_input_schema_never_requests_a_hunt() {
        let spec = ProblemSpec {
            input_schema: None,
            ..spec()
        };
        assert!(request(&run(), &spec, "img", Verdict::WrongAnswer).is_none());
    }

    #[test]
    fn sample_and_custom_runs_never_request_a_hunt() {
        let sample = RunInput {
            mode: RunMode::Run,
            ..run()
        };
        assert!(request(&sample, &spec(), "img", Verdict::WrongAnswer).is_none());
        let custom = RunInput {
            ghost_input: Some(json!([1, 2])),
            ..run()
        };
        assert!(request(&custom, &spec(), "img", Verdict::WrongAnswer).is_none());
    }

    #[test]
    fn the_request_carries_only_public_sample_inputs() {
        let request = request(&run(), &spec(), "img", Verdict::WrongAnswer).expect("hunts");
        assert_eq!(
            request.samples,
            [json!([1, 2])],
            "the hidden test's args are never a hunt seed"
        );
        assert_eq!(request.known_ghosts, spec().known_ghosts);
        assert_eq!(request.concept_tags, ["arithmetic"]);
    }

    #[test]
    fn the_ghost_payload_matches_the_public_contract() {
        let emitted = events(&report(HuntOutcome::Found(ghost(
            FailureKind::WrongAnswer,
            ProgramResult::Returned(json!(0)),
        ))));
        let names: Vec<_> = emitted.iter().map(PublicEvent::event_type).collect();
        assert_eq!(names, ["shrink_step", "shrink_step", "ghost"]);

        let first = emitted[0].payload();
        assert_eq!(
            first["index"],
            json!(0),
            "the first failing input is step 0"
        );
        assert_eq!(first["input"], json!([5, 5]));
        assert_eq!(first["evaluations"], json!(1));

        let ghost = emitted[2].payload();
        assert_eq!(ghost["kind"], json!("wrong"));
        assert_eq!(ghost["input"], json!([0, 0]));
        assert_eq!(ghost["expected"], json!(0));
        assert_eq!(ghost["got"], json!(0));
        assert_eq!(ghost["explanation"], json!("the product is not the sum"));
        assert_eq!(ghost["budget_exhausted"], json!(false));
        assert_eq!(ghost["shrink"]["total_steps"], json!(1));
        assert_eq!(ghost["shrink"]["omitted_steps"], json!(0));
        assert_eq!(ghost["shrink"]["stop"], json!("local_minimum"));
        assert_eq!(ghost["shrink"]["steps"].as_array().expect("steps").len(), 2);
        assert_eq!(
            ghost["provenance"]["seed"],
            json!("42"),
            "a u64 seed is a decimal string on the wire — a JS number would lose it"
        );
        assert_eq!(ghost["provenance"]["problem_version"], json!(3));
        assert_eq!(ghost["provenance"]["image_digest"], json!("sha256:img"));
        assert_eq!(
            ghost["autopsy"]["explanation"],
            json!("the product is not the sum")
        );
        assert_eq!(ghost["autopsy"]["concept_tags"], json!(["arithmetic"]));
        assert_eq!(
            ghost["autopsy"]["authored_mistake"],
            Value::Null,
            "no authored mistake reads as absent"
        );
    }

    #[test]
    fn a_crash_ghost_names_the_exception_class() {
        let mut crash = ghost(
            FailureKind::RuntimeError,
            ProgramResult::Crashed {
                exception_class: "ValueError".to_owned(),
                message: "bad literal".to_owned(),
            },
        );
        crash.predicate.exception_class = Some("ValueError".to_owned());
        let emitted = events(&report(HuntOutcome::Found(crash)));
        let ghost = emitted.last().expect("the ghost lands").payload();
        assert_eq!(ghost["kind"], json!("crash"));
        assert_eq!(ghost["got"]["exception_class"], json!("ValueError"));
        assert_eq!(ghost["got"]["message"], json!("bad literal"));
    }

    #[test]
    fn a_timeout_ghost_has_no_got_value() {
        let emitted = events(&report(HuntOutcome::Found(ghost(
            FailureKind::Timeout,
            ProgramResult::TimedOut,
        ))));
        let ghost = emitted.last().expect("the ghost lands").payload();
        assert_eq!(ghost["kind"], json!("timeout"));
        assert_eq!(ghost["got"], Value::Null, "a timeout has no value to show");
    }

    #[test]
    fn a_ghost_with_an_empty_trace_still_emits() {
        let mut bare = ghost(FailureKind::WrongAnswer, ProgramResult::Returned(json!(0)));
        bare.shrink.steps = Vec::new();
        let emitted = events(&report(HuntOutcome::Found(bare)));
        let names: Vec<_> = emitted.iter().map(PublicEvent::event_type).collect();
        assert_eq!(names, ["ghost"]);
    }

    #[test]
    fn an_oversized_step_is_skipped_but_the_rest_still_emit() {
        let mut big = ghost(FailureKind::WrongAnswer, ProgramResult::Returned(json!(0)));
        big.shrink.steps = vec![
            step(0, json!(1)),
            step(1, json!("x".repeat(40_000))),
            step(2, json!(2)),
        ];
        big.shrink.total_steps = 2;
        let emitted = events(&report(HuntOutcome::Found(big)));
        let names: Vec<_> = emitted.iter().map(PublicEvent::event_type).collect();
        assert_eq!(
            names,
            ["shrink_step", "shrink_step", "ghost"],
            "the oversized step is skipped; the kept steps and ghost still emit"
        );
        let ghost = emitted
            .iter()
            .find(|event| event.event_type() == "ghost")
            .expect("the ghost still lands")
            .payload();
        assert_eq!(
            ghost["shrink"]["steps"].as_array().expect("steps").len(),
            2,
            "the ghost's trace carries only the steps that fit"
        );
        assert_eq!(
            ghost["shrink"]["omitted_steps"],
            json!(1),
            "the dropped step counts as omitted"
        );
    }

    #[test]
    fn an_oversized_ghost_drops_the_optional_fields() {
        let mut fat = ghost(
            FailureKind::WrongAnswer,
            ProgramResult::Returned(json!("y".repeat(40_000))),
        );
        fat.expected = Some(json!("z".repeat(40_000)));
        let emitted = events(&report(HuntOutcome::Found(fat)));
        let last = emitted.last().expect("the slim ghost lands");
        assert_eq!(last.event_type(), "ghost");
        let ghost = last.payload();
        assert_eq!(ghost["input"], json!([0, 0]));
        assert_eq!(ghost["got"], Value::Null);
        assert_eq!(ghost["expected"], Value::Null);
        assert_eq!(ghost["explanation"], json!("the product is not the sum"));
        assert_eq!(ghost["autopsy"]["differences"], json!([]));
        assert_eq!(ghost["autopsy"]["differences_truncated"], json!(true));
        assert!(
            ghost["provenance"].is_object() && ghost["shrink"].is_object(),
            "the slim payload keeps the provenance and trace"
        );
    }

    #[test]
    fn a_ghost_past_the_payload_cap_is_dropped() {
        let mut huge = ghost(FailureKind::WrongAnswer, ProgramResult::Returned(json!(0)));
        huge.input = json!("i".repeat(40_000));
        let emitted = events(&report(HuntOutcome::Found(huge)));
        assert!(
            emitted
                .iter()
                .all(|event| event.event_type() == "shrink_step"),
            "a ghost that cannot fit is dropped, never truncated"
        );
    }

    #[test]
    fn nothing_is_emitted_without_a_ghost() {
        for outcome in [
            HuntOutcome::NotFound(NotFoundReason::Unreproducible),
            HuntOutcome::Aborted(AbortReason::ReferenceUnreliable),
        ] {
            assert!(
                events(&report(outcome)).is_empty(),
                "a fruitless or aborted hunt emits nothing after ghost_hunting"
            );
        }
    }
}
