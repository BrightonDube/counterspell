//! Types shared by the server and trusted judge worker.
//!
//! This crate deliberately has no web, database, Docker, or process-execution
//! dependencies. Feature specs add only types that cross that boundary.
//! It must never depend on `sqlx`, `axum`, `tokio` or `bollard`.

#![warn(missing_docs)]

pub mod events;
pub mod ids;
pub mod limits;
pub mod run_state;
pub mod status;

/// A placeholder version for shared wire types before the API is introduced.
pub const COMMON_VERSION: &str = "0.1.0";
