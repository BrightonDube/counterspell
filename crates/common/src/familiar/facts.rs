//! The only measured facts that can set a mood.
//!
//! [`MeasuredFacts`] is the whole input of `decide`. It holds the verdict,
//! the run mode and what the judge measured. It has no field for code, model
//! text, browser state or user preferences, so those cannot affect a mood.

use super::class::{ClassId, Confidence};
use crate::status::{RunMode, Verdict};

/// Why a bench produced no growth measurement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoMeasurement {
    /// The problem declares no bench, so nothing was timed.
    NotSpecified,
    /// The host was too busy to trust the timings, so the bench was discarded.
    HostBusy,
    /// The bench call itself failed for a reason that is not an infrastructure error.
    Failed,
    /// Too few sizes finished inside the bench budget to fit a curve.
    Incomplete,
}

/// What the judge measured about growth, when a bench ran.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BenchFacts {
    /// No growth measurement. The reason says why.
    None(NoMeasurement),
    /// A bench ran and was fitted.
    Measured {
        /// How sure the fit is.
        confidence: Confidence,
        /// The class the fit reports. Only present at `Medium` or `High`.
        observed: Option<ClassId>,
        /// The class the problem author declared as the reference.
        reference: ClassId,
        /// Observed time divided by the same-language baseline, when a usable baseline exists.
        ratio_vs_baseline: Option<f64>,
    },
}

/// Everything `decide` may look at.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeasuredFacts {
    /// The judge's verdict for the whole run.
    pub verdict: Verdict,
    /// Whether this was a sample-only Run or a hidden-test Submit.
    pub mode: RunMode,
    /// What the bench measured, or why it did not.
    pub bench: BenchFacts,
}
