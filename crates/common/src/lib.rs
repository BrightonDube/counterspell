//! Types shared by the server and trusted judge worker.
//!
//! This crate deliberately has no web, database, Docker, or process-execution
//! dependencies. Feature specs add only types that cross that boundary.

/// A placeholder version for shared wire types before the API is introduced.
pub const COMMON_VERSION: &str = "0.1.0";
