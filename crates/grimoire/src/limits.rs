//! Launch resource caps (MET-08).

use serde::Deserialize;

/// The per-test limits declared in `[limits]`. Range checks come with the launch caps.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    /// The per-test time limit in milliseconds.
    pub time_ms: i64,
    /// The memory limit in megabytes.
    pub mem_mb: i64,
}
