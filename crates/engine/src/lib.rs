//! Pure ghost-engine foundations.
//!
//! The engine will generate and shrink inputs, but it will never execute a
//! process or talk to Docker. The trusted judge will supply sandbox-backed
//! oracles in a later feature spec.

/// The engine implementation version recorded alongside future counterexamples.
pub const ENGINE_VERSION: &str = "0.1.0";
