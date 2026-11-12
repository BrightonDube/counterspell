//! Pure ghost-engine foundations.
//!
//! The engine generates candidates, classifies oracle observations, and
//! shrinks failures. It never executes a process or talks to Docker. The
//! trusted judge supplies sandbox-backed oracles through the `Oracle` seam.
//!
//! The dependency direction is deliberately narrow:
//! `schema`, `rng`, `size`, and `compare` are leaf modules; `generate` uses
//! `schema` and `rng`; `classify` uses `compare`; `shrink` uses `schema` and
//! `size`; `oracle` and `budget` use only standard-library seams; `hunt` joins
//! these parts; and `autopsy` uses classification, size, and JSON values.
//!
//! # The three parts
//!
//! - **Generate:** [`generate`] builds seeded, edge-biased candidate inputs in
//!   a fixed order.
//! - **Classify:** [`classify()`] compares the user's result with the reference
//!   result. User output can only be compared; it never declares a verdict.
//! - **Shrink:** [`shrink`] reduces a failing input step by step. A step is
//!   kept only when it still fails the same way and the size metric drops.
//!
//! [`hunt()`] joins the three parts and returns a [`HuntReport`] for every
//! request, including requests that cannot run.
//!
//! # The Oracle contract
//!
//! The judge implements [`Oracle`] by running programs in the sandbox. The
//! engine never runs code itself. The trait's documentation lists the rules
//! an Oracle must follow.
//!
//! # What "minimized counterexample" means
//!
//! A ghost is a *minimized counterexample*: the smallest failing input the
//! search found within its budget. The shrinker is bounded, so it cannot prove
//! that no smaller failing input exists. [`GHOST_LABEL`] is the product label.
//!
//! # Public surface
//!
//! | Item | Use |
//! |---|---|
//! | [`hunt()`], [`HuntRequest`], [`HuntReport`] | run a hunt and read the result |
//! | [`verify_ghost`] | re-check a stored ghost on a fresh Oracle |
//! | [`Oracle`], [`Clock`], [`SystemClock`], [`ManualClock`] | the seams the judge and tests plug into |
//! | [`Budget`] | the limits, validated before a hunt starts |
//! | [`Comparator`], [`Classification`], [`classify()`] | output comparison |
//! | [`build_autopsy`], [`KnownGhost`], [`Autopsy`] | the Bug Autopsy data |
//! | [`generate`], [`shrink`] | candidate generation and shrinking |
//!
//! # Example
//!
//! ```
//! use counterspell_engine::{
//!     hunt, Comparator, HuntIdentity, HuntOutcome, HuntRequest, ManualClock, Observation,
//!     Oracle, OracleError, ProgramResult, Schema,
//! };
//! use serde_json::{json, Value};
//! use std::time::Duration;
//!
//! // A stand-in for the sandboxed judge: the user's program always answers -1
//! // and the reference always answers 0.
//! struct AlwaysWrong;
//!
//! impl Oracle for AlwaysWrong {
//!     fn evaluate(&mut self, _input: &Value, _limit: Duration) -> Result<Observation, OracleError> {
//!         Ok(Observation {
//!             user: ProgramResult::Returned(json!(-1)),
//!             reference: ProgramResult::Returned(json!(0)),
//!         })
//!     }
//! }
//!
//! let identity = HuntIdentity {
//!     problem_slug: "example".to_owned(),
//!     code_sha256: "abc123".to_owned(),
//!     language: "python".to_owned(),
//!     problem_version: 1,
//!     image_digest: "sha256:example".to_owned(),
//! };
//! let request = HuntRequest::new(Schema::Int { min: 0, max: 5 }, Comparator::Exact, identity);
//! let clock = ManualClock::new();
//! let report = hunt(&request, &mut AlwaysWrong, &clock);
//! assert!(matches!(report.outcome, HuntOutcome::Found(_)));
//! ```

#![warn(missing_docs)]

/// The engine implementation version recorded alongside counterexamples.
pub const ENGINE_VERSION: &str = "0.1.0";

/// The candidate-generation and shrinking algorithm version.
pub const GENERATOR_VERSION: &str = "gen-1";

/// The honest product label for a bounded shrinking result.
pub const GHOST_LABEL: &str = "minimized counterexample";

/// Failure classification is a public part of the engine.
pub mod classify;
/// Candidate generation is a public part of the engine.
pub mod generate;
/// Failure shrinking is a public part of the engine.
pub mod shrink;

mod autopsy;
mod budget;
mod compare;
mod hunt;
mod oracle;
mod rng;
mod schema;
mod size;

pub use autopsy::{build_autopsy, AuthoredMistake, Autopsy, Difference, KnownGhost};
pub use budget::Budget;
pub use classify::{
    classify, Classification, FailureKind, FailurePredicate, Observation, ProgramResult,
};
pub use compare::Comparator;
pub use hunt::{
    hunt, verify_ghost, AbortReason, Ghost, HuntIdentity, HuntOutcome, HuntReport, HuntRequest,
    HuntStats, NotFoundReason, Provenance,
};
pub use oracle::{Clock, ManualClock, Oracle, OracleError, SystemClock};
pub use rng::derive_seed;
pub use schema::{Schema, SchemaError, Violation};
pub use size::{compare_size, size_of, SizeMetric};
#[cfg(test)]
mod tests {
    use proptest::prelude::*;
    use serde_json::json;

    #[test]
    fn foundation_exposes_stable_versions() {
        assert_eq!(super::GENERATOR_VERSION, "gen-1");
        assert_eq!(super::GHOST_LABEL, "minimized counterexample");
    }

    #[test]
    fn foundation_reexports_schema_violation() {
        let violation = super::Violation::new("/0", "bad value");
        assert_eq!(violation.path, "/0");
        assert_eq!(violation.reason, "bad value");
    }

    #[test]
    fn compare_comparator_contract_is_strict_and_unordered() {
        use super::Comparator;

        assert!(Comparator::Exact.equal(&json!(1), &json!(1)));
        assert_eq!(
            serde_json::to_value(Comparator::Exact).unwrap(),
            json!("exact")
        );
        assert_eq!(
            serde_json::to_value(Comparator::Unordered).unwrap(),
            json!("unordered"),
        );
        assert!(!Comparator::Exact.equal(&json!(1), &json!(1.0)));
        assert!(!Comparator::Exact.equal(&json!(true), &json!(1)));

        assert!(Comparator::Unordered.equal(&json!([1, 2, 2]), &json!([2, 1, 2])));
        assert!(!Comparator::Unordered.equal(&json!([1, 2]), &json!([1, 3])));
        assert!(!Comparator::Unordered.equal(&json!([1, 2]), &json!([1])));
        assert!(Comparator::Unordered.equal(&json!({"a": 1}), &json!({"a": 1})));
        assert!(!Comparator::Unordered.equal(&json!({"a": 1}), &json!({"a": 1.0})));
        assert!(!Comparator::Unordered.equal(&json!(1), &json!(1.0)));
    }

    #[test]
    fn compare_comparator_is_reflexive_and_symmetric() {
        use super::Comparator;

        for comparator in [Comparator::Exact, Comparator::Unordered] {
            assert!(comparator.equal(&json!(1), &json!(1)));
            assert!(comparator.equal(&json!([1, 2]), &json!([1, 2])));
            assert_eq!(
                comparator.equal(&json!(1), &json!([1, 2])),
                comparator.equal(&json!([1, 2]), &json!(1)),
            );
        }
    }

    proptest! {
        #[test]
        fn compare_p12_comparator_laws_for_integer_arrays(
            a in prop::collection::vec(any::<i64>(), 0..6),
            b in prop::collection::vec(any::<i64>(), 0..6),
        ) {
            use super::Comparator;

            for comparator in [Comparator::Exact, Comparator::Unordered] {
                prop_assert!(comparator.equal(&json!(a), &json!(a)));
                prop_assert_eq!(
                    comparator.equal(&json!(a), &json!(b)),
                    comparator.equal(&json!(b), &json!(a)),
                );
            }
        }
    }
}
