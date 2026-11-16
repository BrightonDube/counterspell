// SPDX-License-Identifier: MIT
//! The sandbox half of the Efficiency Familiar bench (language-drivers T08).
//!
//! After correctness passes, the run's container is handed over still open.
//! This collector drops the normal driver (its stdin closes, so the process
//! exits on its own) and starts a clean driver with the catalogue's
//! `bench_args` in the **same** container. All timing and budgets belong to
//! the worker clock — the driver only executes calls, exactly as in the
//! correctness path.

use std::time::Duration;

use counterspell_engine::complexity::{bench_seed, sized_input};
use counterspell_engine::Schema;
use serde_json::Value;
use thiserror::Error;
use tokio::sync::watch;
use tokio::time::{timeout_at, Instant};

use super::backend::{BackendSession, DriverChannel, DriverEvent, InfraError};
use super::call::STDOUT_BUDGET;
use super::driver_protocol::{encode_call, parse_answer};
use super::session::Opened;
use super::spec::ExecSpec;
use super::stream::LineReader;
use crate::executor::ExecError;
use crate::familiar::assess::BenchPoint;
use crate::familiar::{BenchCollector, BenchOutcome, BenchPlan};
use crate::languages::LanguageDefinition;

/// Untimed calls that warm the program before each size's timed repeats.
pub const WARM_UPS: usize = 2;

/// Timed calls per size; the point is their median.
pub const TIMED_REPS: usize = 5;

/// Why a bench call had to stop.
#[derive(Debug, Error)]
enum BenchStop {
    /// The driver exited, misbehaved or could not be driven: `BenchOutcome::Failed`.
    #[error("the bench driver failed: {0}")]
    Failed(String),
    /// The run's stop signal fired.
    #[error("stopped")]
    Stopped,
    /// The judge itself could not act on the sandbox.
    #[error("{0}")]
    Infrastructure(#[from] InfraError),
}

/// Times a program at growing input sizes inside the run's own container.
///
/// The collector borrows the still-open [`Opened`] session. On drop-in it ends
/// the normal driver and opens a bench-mode driver; on the way out the bench
/// driver is put back so the caller's normal close path stays valid.
pub struct SandboxBenchCollector<'a, S: BackendSession + 'static> {
    opened: &'a mut Opened<S>,
    lang: &'a LanguageDefinition,
    /// The parameters tuple schema, used to size inputs.
    schema: Schema,
    /// The per-call deadline, enforced with the worker clock.
    call_deadline: Duration,
    /// Splits driver stdout into protocol lines, with the same per-call byte
    /// budget the correctness call enforces.
    stdout: LineReader,
    /// The run's stop signal: cancel, lost lease, shutdown.
    stop: &'a mut watch::Receiver<bool>,
}

impl<'a, S: BackendSession + 'static> SandboxBenchCollector<'a, S> {
    /// Wraps the run's still-open session. `schema` is the signature's parameter
    /// tuple (`Schema::Tuple`), `call_deadline` the per-call limit.
    pub fn new(
        opened: &'a mut Opened<S>,
        lang: &'a LanguageDefinition,
        schema: Schema,
        call_deadline: Duration,
        stop: &'a mut watch::Receiver<bool>,
    ) -> Self {
        Self {
            opened,
            lang,
            schema,
            call_deadline,
            stdout: LineReader::with_budget(STDOUT_BUDGET, STDOUT_BUDGET),
            stop,
        }
    }

    /// Builds the argument tuple for size `n`. Every repeat of a size uses the
    /// same seeded input, so warm-ups and timed calls are comparable; the call
    /// id is still fresh for every call, as the protocol requires.
    fn call_args(&self, plan: &BenchPlan, n: u64) -> Result<Vec<Value>, BenchStop> {
        let input = sized_input(
            &self.schema,
            plan.size_param,
            n,
            bench_seed(&plan.problem_slug, plan.problem_version, n),
        )
        .map_err(|e| BenchStop::Failed(e.to_string()))?;
        let Value::Array(args) = input else {
            return Err(BenchStop::Failed(
                "a sized bench input is not an argument tuple".to_owned(),
            ));
        };
        Ok(args)
    }

    /// One call against the bench driver: write the line, then read until the
    /// answer line, the driver's exit, the call deadline or the budget deadline.
    /// `budget_deadline` caps the whole bench, not this call: `Ok(None)` means
    /// the budget ran out mid-call — a cutoff, not a driver failure. Stdout
    /// feeds the line splitter exactly as in the correctness call: one answer
    /// line and nothing after it.
    async fn timed_call(
        &mut self,
        driver: &mut S::Driver,
        line: &[u8],
        id: u64,
        call_deadline: Instant,
        budget_deadline: Instant,
    ) -> Result<Option<Duration>, BenchStop> {
        let limit = call_deadline.min(budget_deadline);
        let budget_cut = || limit == budget_deadline;
        let started = Instant::now();
        self.stdout.reset_budget(STDOUT_BUDGET);
        let wrote = timeout_at(limit, driver.write_line(line)).await;
        match wrote {
            Err(_) if budget_cut() => return Ok(None),
            Err(_) => {
                return Err(BenchStop::Failed(
                    "the call line write timed out".to_owned(),
                ));
            }
            Ok(inner) => inner?,
        }
        loop {
            let event = match timeout_at(limit, driver.next_event()).await {
                Err(_) if budget_cut() => return Ok(None),
                Err(_) => {
                    return Err(BenchStop::Failed(
                        "a bench call exceeded its deadline".to_owned(),
                    ));
                }
                Ok(inner) => inner?,
            };
            match event {
                DriverEvent::Line(bytes) | DriverEvent::Chunk(bytes) => {
                    let lines = self.stdout.push(&bytes).map_err(|_| {
                        BenchStop::Failed(
                            "the bench driver's output exceeded the call budget".to_owned(),
                        )
                    })?;
                    match lines.as_slice() {
                        [] => {}
                        [answer] if self.stdout.held() == 0 => {
                            parse_answer(answer, id)
                                .map_err(|e| BenchStop::Failed(e.to_string()))?;
                            return Ok(Some(started.elapsed()));
                        }
                        _ => {
                            return Err(BenchStop::Failed(
                                "the bench driver wrote more than its answer line".to_owned(),
                            ));
                        }
                    }
                }
                DriverEvent::Exited { code, oom_kill } => {
                    return Err(BenchStop::Failed(format!(
                        "the bench driver exited with code {code} (oom_kill {oom_kill})"
                    )));
                }
                DriverEvent::Stderr(_) => {}
            }
        }
    }
}

impl<S: BackendSession + 'static> BenchCollector for SandboxBenchCollector<'_, S> {
    async fn collect(&mut self, plan: &BenchPlan) -> Result<BenchOutcome, ExecError> {
        {
            // End the normal driver: its stdin closes, the process exits itself.
            drop(self.opened.driver.take());

            let collected = self.bench_loop(plan).await;
            match collected {
                Ok(outcome) => Ok(outcome),
                Err(BenchStop::Stopped) => Err(ExecError::Stopped),
                Err(BenchStop::Infrastructure(error)) => {
                    Err(ExecError::Infrastructure(error.to_string()))
                }
                Err(BenchStop::Failed(reason)) => {
                    tracing::debug!(%reason, "the bench failed inside the program");
                    Ok(BenchOutcome::Failed)
                }
            }
        }
    }
}

impl<S: BackendSession + 'static> SandboxBenchCollector<'_, S> {
    async fn bench_loop(&mut self, plan: &BenchPlan) -> Result<BenchOutcome, BenchStop> {
        if *self.stop.borrow() {
            return Err(BenchStop::Stopped);
        }

        let spec = ExecSpec::bench(self.lang);
        let container = self.opened.session.container_mut()?;
        let mut driver = container.open_driver(&spec).await?;

        let budget_deadline = Instant::now() + plan.budget;
        let mut points = Vec::new();
        let mut id = 0u64;
        let mut result = Ok(());

        'sizes: for &n in &plan.sizes {
            let args = self.call_args(plan, n)?;
            let mut timings = Vec::with_capacity(TIMED_REPS);
            for rep in 0..(WARM_UPS + TIMED_REPS) {
                if *self.stop.borrow() {
                    result = Err(BenchStop::Stopped);
                    break 'sizes;
                }
                if Instant::now() >= budget_deadline {
                    break 'sizes;
                }
                let line = encode_call(id, &args).map_err(|e| BenchStop::Failed(e.to_string()))?;
                let call_deadline = Instant::now() + self.call_deadline;
                match self
                    .timed_call(&mut driver, &line, id, call_deadline, budget_deadline)
                    .await
                {
                    Ok(Some(elapsed)) => {
                        if rep >= WARM_UPS {
                            timings.push(elapsed);
                        }
                        id += 1;
                    }
                    Ok(None) => break 'sizes,
                    Err(error) => {
                        result = Err(error);
                        break 'sizes;
                    }
                }
            }
            points.push(BenchPoint {
                n,
                median_ns: median_ns(&timings),
                spread: spread(&timings),
            });
        }

        // Put the bench driver back so the caller's close path sees a coherent Opened.
        self.opened.driver = Some(driver);

        match result {
            Ok(()) if points.len() == plan.sizes.len() => Ok(BenchOutcome::Complete(points)),
            Ok(()) => Ok(BenchOutcome::Incomplete(points)),
            Err(error) => Err(error),
        }
    }
}

/// The middle repeat time, in nanoseconds. A point is only built for a size that
/// finished all `TIMED_REPS` calls — a partial size is discarded, not measured.
fn median_ns(timings: &[Duration]) -> u64 {
    if timings.is_empty() {
        return 0;
    }
    let mut sorted: Vec<u64> = timings
        .iter()
        .map(|t| u64::try_from(t.as_nanos()).unwrap_or(u64::MAX))
        .collect();
    sorted.sort_unstable();
    sorted[sorted.len() / 2]
}

/// The relative spread `(max - min) / median`, or `None` when the median is zero.
#[allow(clippy::cast_precision_loss)]
fn spread(timings: &[Duration]) -> Option<f64> {
    let (mut lo, mut hi) = (u64::MAX, 0u64);
    for t in timings {
        let ns = u64::try_from(t.as_nanos()).unwrap_or(u64::MAX);
        lo = lo.min(ns);
        hi = hi.max(ns);
    }
    let median = median_ns(timings);
    (median > 0).then(|| (hi - lo) as f64 / median as f64)
}
