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

#[cfg(test)]
mod tests {
    #[test]
    fn foundation_exposes_stable_versions() {
        assert_eq!(super::GENERATOR_VERSION, "gen-1");
        assert_eq!(super::GHOST_LABEL, "minimized counterexample");
    }
}
