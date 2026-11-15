//! Efficiency Familiar, judge side: pure assessment helpers and the bench orchestration.
//!
//! The module itself runs no code and touches no database. Timing runs through the
//! `BenchCollector` trait, which the sandbox side implements. The sandbox and the storage
//! live in their own modules and call into these functions.

pub mod assess;
pub mod gate;
pub mod load;
pub mod ratio;
pub mod store;

use std::future::Future;
use std::time::Duration;

use counterspell_common::events::PublicEvent;
use counterspell_common::familiar::ClassId;
use counterspell_common::status::{RunMode, Verdict};

use crate::executor::{ExecError, RunContext};
use assess::{assess, AssessInput, Assessment, BenchPoint, BenchState};
use gate::BenchGate;
use load::{quiet, sample, LoadSource};

/// What the collector should time for one run.
#[derive(Debug, Clone, PartialEq)]
pub struct BenchPlan {
    /// Run gets a short bench, Submit gets the full one.
    pub mode: RunMode,
    /// The input sizes, in increasing order.
    pub sizes: Vec<u64>,
    /// Index of the tuple item that grows with `n`.
    pub size_param: usize,
    /// The problem slug, used with the version to seed the bench inputs.
    pub problem_slug: String,
    /// The problem version, used with the slug to seed the bench inputs.
    pub problem_version: u32,
    /// The wall-clock budget for the whole bench.
    pub budget: Duration,
}

/// What the collector returned.
#[derive(Debug, Clone, PartialEq)]
pub enum BenchOutcome {
    /// Every planned size was timed.
    Complete(Vec<BenchPoint>),
    /// The budget ended early. The points that did finish are kept.
    Incomplete(Vec<BenchPoint>),
    /// The bench call failed inside the program (a crash or a timeout), not in the judge.
    Failed,
}

/// Times the program at each planned size. The sandbox side implements this trait.
///
/// An `Err` means the judge itself failed (an infrastructure error or a stop signal). A
/// failed or short bench is an `Ok` with `BenchOutcome::Failed` or `Incomplete`.
pub trait BenchCollector: Send {
    /// Times the program for `plan`.
    fn collect(
        &mut self,
        plan: &BenchPlan,
    ) -> impl Future<Output = Result<BenchOutcome, ExecError>> + Send;
}

/// The facts `measure_run` needs about one run.
#[derive(Debug, Clone, Copy)]
pub struct MeasureInput<'a> {
    /// The verdict of the correctness tests. `None` when the run did not complete.
    pub verdict: Option<Verdict>,
    /// Run or Submit.
    pub mode: RunMode,
    /// The reference class the problem author declared.
    pub reference: ClassId,
    /// Baseline points for this problem version, language and image digest, if usable.
    pub baseline: Option<&'a [(u64, u64)]>,
    /// The bench to run, or `None` when the problem has no bench.
    pub plan: Option<&'a BenchPlan>,
    /// The highest 1-minute load average at which a bench is kept.
    pub max_load: f64,
}

/// Runs the bench for one run and emits its events.
///
/// The bench runs only for an accepted run with a plan. The order is: take the bench gate,
/// read the load, run the collector, read the load again, assess, then emit `complexity`
/// (only when it was fitted) and `familiar`. A collector error is returned unchanged, so
/// the caller retries the run. A failed or short bench is not an error: it gives a missing
/// measurement and the run still completes.
///
/// # Errors
///
/// Returns the collector's `ExecError`, the gate's error as an infrastructure error, or an
/// infrastructure error when an event payload cannot be encoded.
pub async fn measure_run<C: BenchCollector>(
    ctx: &RunContext,
    gate: &BenchGate,
    load: &impl LoadSource,
    collector: &mut C,
    input: &MeasureInput<'_>,
) -> Result<(), ExecError> {
    let assessment = match (input.verdict == Some(Verdict::Accepted), input.plan) {
        (true, Some(plan)) => {
            let _permit = gate
                .acquire()
                .await
                .map_err(|error| ExecError::Infrastructure(error.to_string()))?;
            let before = sample(load);
            let outcome = collector.collect(plan).await?;
            let after = sample(load);
            let state = bench_state(&outcome, quiet(before, after, input.max_load));
            assess(&assess_input(input, state))
        }
        (true, None) => assess(&assess_input(input, BenchState::NotSpecified)),
        (false, _) => assess(&AssessInput {
            baseline: None,
            ..assess_input(input, BenchState::NotSpecified)
        }),
    };
    emit_assessment(ctx, assessment).await
}

/// Builds the assessment input for one run from its measured state.
fn assess_input<'a>(input: &MeasureInput<'a>, bench: BenchState<'a>) -> AssessInput<'a> {
    AssessInput {
        verdict: input.verdict,
        mode: input.mode,
        reference: input.reference,
        bench,
        baseline: input.baseline,
    }
}

/// Maps a collector outcome to the bench state `assess` takes. A bench read under a busy host
/// is discarded, whatever the collector returned.
fn bench_state(outcome: &BenchOutcome, quiet_host: bool) -> BenchState<'_> {
    if !quiet_host {
        return BenchState::Discarded;
    }
    match outcome {
        BenchOutcome::Complete(points) => BenchState::Points(points),
        BenchOutcome::Incomplete(_) => BenchState::Incomplete,
        BenchOutcome::Failed => BenchState::Failed,
    }
}

/// Emits the `complexity` event (when fitted) and the `familiar` event, in that order.
async fn emit_assessment(ctx: &RunContext, assessment: Assessment) -> Result<(), ExecError> {
    if let Some(complexity) = &assessment.complexity {
        let data = complexity
            .to_bounded_json()
            .map_err(|error| ExecError::Infrastructure(error.to_string()))?;
        ctx.emit(&PublicEvent::Complexity(data)).await?;
    }
    if let Some(familiar) = &assessment.familiar {
        let data = familiar
            .to_bounded_json()
            .map_err(|error| ExecError::Infrastructure(error.to_string()))?;
        ctx.emit(&PublicEvent::Familiar(data)).await?;
    }
    Ok(())
}
