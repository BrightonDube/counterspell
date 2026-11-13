//! Model-based interleaving test (run-lifecycle T18, design "Model-based test").
//!
//! Random sequences of queue calls run across three workers and three runs: claim,
//! renew, emit, finish, lease expiry, reap and cancel. The calls include stale holders
//! on purpose. A small model predicts which calls the fence accepts. After every call
//! the database must match the model, and the global invariants must hold.
//!
//! Lease expiry is aged with SQL, never with a sleep.
//!
//! Run with `cargo test -p counterspell-judge --test model -- --ignored pg_`.

use counterspell_common::events::PublicEvent;
use counterspell_common::ids::RunId;
use counterspell_common::status::{FailureReason, TestVerdict, Verdict};
use counterspell_judge::config::JudgeConfig;
use counterspell_judge::queue::claim::claim_next;
use counterspell_judge::queue::emit::emit;
use counterspell_judge::queue::finish::{finish, finish_failed, FinalResult, TestRecord};
use counterspell_judge::queue::lease::renew;
use counterspell_judge::queue::reap::reap_once;
use counterspell_judge::queue::{ClaimToken, QueueError};
use counterspell_judge::testkit::{age_lease, assert_invariants, seed_run};
use proptest::collection::vec;
use proptest::prelude::*;
use proptest::test_runner::{Config, TestCaseError, TestRunner};
use sqlx::PgPool;
use std::time::Duration;

/// Number of workers that take part in each case.
const WORKERS: usize = 3;
/// Number of runs seeded in each case.
const RUNS: usize = 3;
/// Attempts before a job is dead. Set on every job, so the model does not depend on the default.
const MAX_ATTEMPTS: i16 = 3;
/// Lease length passed to every claim and renewal. It never elapses in this test, because
/// expiry is simulated with SQL.
const LEASE: Duration = Duration::from_secs(30);

/// One queue call in a generated sequence.
#[derive(Debug, Clone, Copy)]
enum Op {
    /// A worker claims whatever run is ready.
    Claim(usize),
    /// A worker renews the lease it holds on a run.
    Renew(usize, usize),
    /// A worker appends a `compiling` event to a run.
    Emit(usize, usize),
    /// A worker finishes a run with an accepted verdict.
    Finish(usize, usize),
    /// The lease on a run runs out (simulated with SQL).
    Expire(usize),
    /// The reaper makes one pass.
    Reap,
    /// The user cancels a run.
    Cancel(usize),
}

/// Where the model thinks a run is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Phase {
    /// Queued and ready to claim.
    Ready,
    /// Claimed by one worker.
    Claimed,
    /// Finished. The job is in its end state and nothing changes it.
    Terminal,
}

/// The model's view of one run.
#[derive(Debug, Clone, Copy)]
struct RunModel {
    phase: Phase,
    /// The worker holding the claim, when claimed.
    owner: Option<usize>,
    /// Attempts used so far, as stored on the job.
    attempts: i16,
    /// True after the lease ran out and before a renewal or a reap.
    expired: bool,
    /// True once a `compiling` event was accepted in the current claim.
    compiled: bool,
    /// True once the user has cancelled the run.
    cancelled: bool,
    /// The job state the run must show once it is terminal.
    end_state: &'static str,
}

impl RunModel {
    const fn fresh() -> Self {
        Self {
            phase: Phase::Ready,
            owner: None,
            attempts: 0,
            expired: false,
            compiled: false,
            cancelled: false,
            end_state: "",
        }
    }
}

/// The state of one model case: the real runs, the tokens each worker holds and the model.
struct Harness {
    pool: PgPool,
    cfg: JudgeConfig,
    runs: Vec<RunId>,
    /// The last token each worker received for each run. It may be stale on purpose.
    tokens: Vec<Vec<Option<ClaimToken>>>,
    model: Vec<RunModel>,
}

/// Any `Debug` error becomes a plain message, so a failing case shows what went wrong.
fn text(error: impl std::fmt::Debug) -> String {
    format!("{error:?}")
}

/// The worker id for worker number `w`.
fn worker_id(w: usize) -> String {
    format!("judge-{w}")
}

/// A result that accepts the run with one passing hidden test.
fn accepted() -> FinalResult {
    FinalResult {
        verdict: Verdict::Accepted,
        passed: 1,
        total: 1,
        score: 100,
        runtime_ms: 1,
        compile_output: None,
        tests: vec![TestRecord::Hidden {
            ordinal: 1,
            verdict: TestVerdict::Pass,
            time_ms: 1,
        }],
    }
}

/// Requires the call to have been refused by the fence as a lost lease.
fn expect_lease_lost<T: std::fmt::Debug>(result: Result<T, QueueError>) -> Result<(), String> {
    match result {
        Err(QueueError::LeaseLost) => Ok(()),
        other => Err(format!("expected LeaseLost, got {other:?}")),
    }
}

impl Harness {
    /// The worker's token is current when it is still the holder of the run's current claim.
    fn is_current(&self, w: usize, r: usize) -> bool {
        let Some(token) = &self.tokens[w][r] else {
            return false;
        };
        let m = &self.model[r];
        m.phase == Phase::Claimed && m.owner == Some(w) && m.attempts == token.attempts()
    }

    /// The index of a run in this case, found by its id.
    fn index_of(&self, run: RunId) -> Result<usize, String> {
        self.runs
            .iter()
            .position(|known| known.as_uuid() == run.as_uuid())
            .ok_or_else(|| "claimed a run this case did not seed".to_owned())
    }

    /// Applies one call to the database and the model, and checks the fence's answer.
    async fn apply(&mut self, op: Op) -> Result<(), String> {
        match op {
            Op::Claim(w) => self.claim(w).await,
            Op::Renew(w, r) => self.renew(w, r).await,
            Op::Emit(w, r) => self.emit(w, r).await,
            Op::Finish(w, r) => self.finish(w, r).await,
            Op::Expire(r) => self.expire(r).await,
            Op::Reap => self.reap().await,
            Op::Cancel(r) => self.cancel(r).await,
        }
    }

    async fn claim(&mut self, w: usize) -> Result<(), String> {
        let claimed = claim_next(&self.pool, &worker_id(w), LEASE)
            .await
            .map_err(text)?;
        let Some(run) = claimed else {
            let ready = self.model.iter().any(|m| m.phase == Phase::Ready);
            return if ready {
                Err("claim found nothing, but a run is ready".to_owned())
            } else {
                Ok(())
            };
        };
        let r = self.index_of(run.token.run_id())?;
        if self.model[r].phase != Phase::Ready {
            return Err("claimed a run the model says is not ready".to_owned());
        }
        if run.token.attempts() != self.model[r].attempts {
            return Err("claim reports a different attempt count".to_owned());
        }
        let m = &mut self.model[r];
        m.phase = Phase::Claimed;
        m.owner = Some(w);
        m.expired = false;
        m.compiled = false;
        m.cancelled = run.cancel_requested;
        self.tokens[w][r] = Some(run.token.clone());
        if run.cancel_requested {
            // The worker ends a cancelled claim without running the program.
            finish_failed(&self.pool, &run.token, FailureReason::Cancelled)
                .await
                .map_err(text)?;
            let m = &mut self.model[r];
            m.phase = Phase::Terminal;
            m.owner = None;
            m.end_state = "failed";
        }
        Ok(())
    }

    async fn renew(&mut self, w: usize, r: usize) -> Result<(), String> {
        let Some(token) = self.tokens[w][r].clone() else {
            return Ok(());
        };
        let valid = self.is_current(w, r);
        let result = renew(&self.pool, &token, LEASE).await;
        if !valid {
            return expect_lease_lost(result);
        }
        let renewal = result.map_err(text)?;
        if renewal.cancel_requested != self.model[r].cancelled {
            return Err("renewal reports a different cancel state".to_owned());
        }
        self.model[r].expired = false;
        Ok(())
    }

    async fn emit(&mut self, w: usize, r: usize) -> Result<(), String> {
        let Some(token) = self.tokens[w][r].clone() else {
            return Ok(());
        };
        let valid = self.is_current(w, r);
        let result = emit(&self.pool, &token, &PublicEvent::Compiling).await;
        if !valid {
            return expect_lease_lost(result);
        }
        result.map_err(text)?;
        self.model[r].compiled = true;
        Ok(())
    }

    async fn finish(&mut self, w: usize, r: usize) -> Result<(), String> {
        let Some(token) = self.tokens[w][r].clone() else {
            return Ok(());
        };
        let valid = self.is_current(w, r);
        let result = finish(&self.pool, &token, &accepted()).await;
        if !valid {
            return expect_lease_lost(result);
        }
        if !self.model[r].compiled {
            // A run must compile before it can end with a verdict. The write is refused,
            // and the claim stays as it was.
            return match result {
                Err(QueueError::LeaseLost) => Err("valid claim reported LeaseLost".to_owned()),
                Err(_) => Ok(()),
                Ok(()) => Err("finish accepted a run that never compiled".to_owned()),
            };
        }
        result.map_err(text)?;
        let m = &mut self.model[r];
        m.phase = Phase::Terminal;
        m.owner = None;
        m.end_state = "succeeded";
        Ok(())
    }

    async fn expire(&mut self, r: usize) -> Result<(), String> {
        if self.model[r].phase == Phase::Claimed {
            age_lease(&self.pool, self.runs[r]).await.map_err(text)?;
            self.model[r].expired = true;
        }
        Ok(())
    }

    async fn reap(&mut self) -> Result<(), String> {
        reap_once(&self.pool, &self.cfg).await.map_err(text)?;
        for m in &mut self.model {
            if m.phase != Phase::Claimed || !m.expired {
                continue;
            }
            m.owner = None;
            m.expired = false;
            if m.cancelled {
                // The reaper ends an orphaned cancelled run instead of retrying it.
                m.phase = Phase::Terminal;
                m.end_state = "failed";
            } else if m.attempts + 1 >= MAX_ATTEMPTS {
                m.phase = Phase::Terminal;
                m.end_state = "dead";
            } else {
                m.phase = Phase::Ready;
                m.attempts += 1;
                m.compiled = false;
            }
        }
        // A retry waits out a backoff. Clear it with SQL so the next claim is not delayed.
        sqlx::query("UPDATE jobs SET run_after = now() WHERE state = 'queued'")
            .execute(&self.pool)
            .await
            .map_err(text)?;
        Ok(())
    }

    async fn cancel(&mut self, r: usize) -> Result<(), String> {
        if self.model[r].phase == Phase::Terminal {
            return Ok(());
        }
        sqlx::query(
            "UPDATE jobs SET cancel_requested_at = coalesce(cancel_requested_at, now()) \
             WHERE submission_id = $1",
        )
        .bind(self.runs[r].as_uuid())
        .execute(&self.pool)
        .await
        .map_err(text)?;
        self.model[r].cancelled = true;
        Ok(())
    }

    /// Compares every run's job state and attempts with the model, then runs the global invariants.
    async fn check(&self) -> Result<(), String> {
        for (r, m) in self.model.iter().enumerate() {
            let (state, attempts): (String, i16) =
                sqlx::query_as("SELECT state, attempts FROM jobs WHERE submission_id = $1")
                    .bind(self.runs[r].as_uuid())
                    .fetch_one(&self.pool)
                    .await
                    .map_err(text)?;
            let expected = match m.phase {
                Phase::Ready => "queued",
                Phase::Claimed => "claimed",
                Phase::Terminal => m.end_state,
            };
            if state != expected || attempts != m.attempts {
                return Err(format!(
                    "run {r}: database has {state}/{attempts}, model expects {expected}/{}",
                    m.attempts
                ));
            }
        }
        assert_invariants(&self.pool).await
    }
}

/// One generated call. Indexes are small, so most calls hit the same few runs and workers.
fn op() -> impl Strategy<Value = Op> {
    prop_oneof![
        (0..WORKERS).prop_map(Op::Claim),
        (0..WORKERS, 0..RUNS).prop_map(|(w, r)| Op::Renew(w, r)),
        (0..WORKERS, 0..RUNS).prop_map(|(w, r)| Op::Emit(w, r)),
        (0..WORKERS, 0..RUNS).prop_map(|(w, r)| Op::Finish(w, r)),
        (0..RUNS).prop_map(Op::Expire),
        Just(Op::Reap),
        (0..RUNS).prop_map(Op::Cancel),
    ]
}

/// Empties the queue tables, seeds the runs and runs one sequence of calls.
async fn run_case(pool: &PgPool, ops: &[Op]) -> Result<(), String> {
    sqlx::query("TRUNCATE submissions CASCADE")
        .execute(pool)
        .await
        .map_err(text)?;
    let mut runs = Vec::new();
    for _ in 0..RUNS {
        runs.push(seed_run(pool).await.map_err(text)?);
    }
    sqlx::query("UPDATE jobs SET max_attempts = $1")
        .bind(MAX_ATTEMPTS)
        .execute(pool)
        .await
        .map_err(text)?;
    let cfg = JudgeConfig::from_lookup(|_| None).map_err(text)?;
    let mut harness = Harness {
        pool: pool.clone(),
        cfg,
        runs,
        tokens: vec![vec![None; RUNS]; WORKERS],
        model: vec![RunModel::fresh(); RUNS],
    };
    harness.check().await?;
    for op in ops {
        harness
            .apply(*op)
            .await
            .map_err(|error| format!("after {op:?}: {error}"))?;
        harness
            .check()
            .await
            .map_err(|error| format!("after {op:?}: {error}"))?;
    }
    Ok(())
}

#[ignore = "needs DATABASE_URL"]
#[test]
fn pg_model_interleavings() {
    let url = std::env::var("DATABASE_URL").expect("DATABASE_URL is set for database tests");
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("a runtime can be built");
    let pool = runtime.block_on(async {
        let pool = PgPool::connect(&url).await.map_err(text)?;
        sqlx::migrate!("../../migrations")
            .run(&pool)
            .await
            .map_err(text)?;
        Ok::<PgPool, String>(pool)
    });
    let pool = pool.expect("the database is reachable and migrates");

    let mut runner = TestRunner::new(Config {
        cases: 32,
        ..Config::default()
    });
    let outcome = runner.run(&vec(op(), 1..40), |ops| {
        runtime
            .block_on(run_case(&pool, &ops))
            .map_err(TestCaseError::fail)
    });
    if let Err(error) = outcome {
        panic!("model check failed: {error}");
    }
}
