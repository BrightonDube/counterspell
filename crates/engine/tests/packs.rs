//! Pack tests: the Rust models must reproduce the hand-traced vectors that
//! tie them to the Python files in `packs/_test/`.

mod support;

use counterspell_engine::ProgramResult;
use support::{incorrect, load_pack, reference, Expected, Model, Vector, PACKS};

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
