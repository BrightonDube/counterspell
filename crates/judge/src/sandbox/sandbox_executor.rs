// SPDX-License-Identifier: MIT
//! The executor that runs one submission in the sandbox (judge-sandbox T24, design
//! section 14).
//!
//! The flow: language lookup, problem load, preflight gate, container permit,
//! `compiling` event, source re-check, run, per-test events, verdict, result. The
//! executor never writes a status or a verdict to the queue — [`RunContext`] has no
//! method for that. Cleanup is the run loop's job: `execute_run` closes its container
//! on every exit path, and the sweeper reaps whatever still escapes.

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use counterspell_common::events::{PublicEvent, PublicTest, SampleDetail};
use counterspell_common::familiar::class::ClassId;
use counterspell_common::familiar::payload::MeasuredColumns;
use counterspell_common::status::{FailureReason, RunMode, TestVerdict, Verdict};
use counterspell_engine::{classify, Classification, FailureKind, Oracle};
use tokio::runtime::Handle;
use tokio::sync::{watch, Semaphore};
use tokio::time::Instant;

use super::backend::{Backend, InfraError};
use super::bench::{SandboxBenchCollector, RUN_BENCH_BUDGET, SUBMIT_BENCH_BUDGET};
use super::compare::Expected;
use super::config::SandboxConfig;
use super::driver_protocol::parse_answer;
use super::oracle::SandboxOracle;
use super::precheck::precheck;
use super::preflight::PreflightState;
use super::problem::{ProblemSpec, ProblemTests, TestCase};
use super::run::{execute_run_and_keep, PlannedTest, RunResult, RunSettings};
use super::sanitize::{sanitize_display, DETAIL_CAP};
use super::session::Opened;
use super::spec::{build_tar, driver_file_name, SandboxFiles, SessionSpec};
use super::verdict::FinalVerdict;
use crate::executor::{ExecError, RunContext, RunExecutor, RunInput, RunOutcome};
use crate::familiar::gate::{BenchGate, BenchSettings};
use crate::familiar::load::ProcLoadAvg;
use crate::familiar::store::find_baseline;
use crate::familiar::{measure_run, BenchPlan, MeasureInput};
use crate::languages::{LanguageCatalog, LanguageDefinition};
use crate::queue::finish::{FinalResult, TestRecord};

/// The message stored as `compile_output` when the source is refused or fails to
/// compile. Fixed, so no compiler text — which could quote the source — ever leaks.
const COMPILE_MESSAGE: &str = "The source did not compile.";

/// The real executor: submissions run inside sandbox containers, never on the host.
///
/// `P` is where problems come from (`EmptyProblemTests` in production until the
/// catalogue exists), `B` the sandbox backend. The executor is `Send + Sync` so the
/// worker can share one instance across claimed runs.
pub struct SandboxExecutor<P: ProblemTests, B: Backend> {
    languages: LanguageCatalog,
    problems: P,
    backend: B,
    config: SandboxConfig,
    /// Caps live sandbox containers. A permit is held for a whole run and released when
    /// `execute_run` has closed its container.
    containers: Arc<Semaphore>,
    /// One preflight state per probed language. All must pass before a run may start.
    preflights: Vec<Arc<PreflightState>>,
    /// This worker's id, stamped on container labels.
    worker: String,
    /// The directory the catalogue's driver paths resolve against.
    driver_root: PathBuf,
    /// The bench permit gate (serialised benches by default).
    gate: BenchGate,
    /// The bench settings (serialise, max load).
    bench: BenchSettings,
}

impl<P: ProblemTests, B: Backend> SandboxExecutor<P, B> {
    /// Builds the executor. Driver paths resolve against the catalogue file's directory.
    #[must_use]
    pub fn new(
        languages: LanguageCatalog,
        problems: P,
        backend: B,
        config: SandboxConfig,
        preflights: Vec<Arc<PreflightState>>,
        worker: impl Into<String>,
    ) -> Self {
        let containers = Arc::new(Semaphore::new(config.max_containers));
        let driver_root = config
            .languages_file
            .parent()
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
        let bench = BenchSettings::default();
        Self {
            languages,
            problems,
            backend,
            config,
            containers,
            preflights,
            worker: worker.into(),
            driver_root,
            gate: BenchGate::new(bench.serialise),
            bench,
        }
    }

    /// Overrides the bench settings, for tests and deployment tuning.
    #[must_use]
    pub fn with_bench_settings(mut self, settings: BenchSettings) -> Self {
        self.gate = BenchGate::new(settings.serialise);
        self.bench = settings;
        self
    }

    /// Builds the tar archive the sandbox unpacks: the source, the driver file the
    /// catalogue names, and the problem's public signature. Nothing else enters the
    /// container.
    fn archive(
        &self,
        code: &str,
        lang: &LanguageDefinition,
        spec: &ProblemSpec,
    ) -> Result<Vec<u8>, ExecError> {
        let driver_path = self.driver_root.join(lang.driver().as_str());
        let driver_code = std::fs::read(&driver_path).map_err(|error| {
            tracing::warn!(%error, path = %driver_path.display(), "the driver file could not be read");
            ExecError::Infrastructure("driver_unreadable".to_owned())
        })?;
        let signature = serde_json::to_vec(&spec.signature)
            .map_err(|_| ExecError::Infrastructure("signature_unwritable".to_owned()))?;
        let driver_name = driver_file_name(lang)
            .ok_or_else(|| ExecError::Infrastructure("driver_name_invalid".to_owned()))?;
        let files = SandboxFiles::new(
            lang.file().clone(),
            driver_name,
            code.as_bytes().to_vec(),
            driver_code,
            signature,
        )
        .map_err(|error| ExecError::Infrastructure(error.to_string()))?;
        build_tar(&files).map_err(|error| ExecError::Infrastructure(error.to_string()))
    }

    /// Loads the pinned problem spec. `Ok(None)` means no store holds that version —
    /// the run reports `problem_unavailable`; `Err` is a broken store, which is
    /// infrastructure and never the user's fault.
    async fn load_problem(&self, run: &RunInput) -> Result<Option<ProblemSpec>, ExecError> {
        match self
            .problems
            .load(&run.problem_slug, run.problem_version, run.mode)
            .await
        {
            Ok(spec) => Ok(spec),
            Err(error) => {
                tracing::warn!(%error, "the problem store failed");
                Err(ExecError::Infrastructure("problem_store".to_owned()))
            }
        }
    }

    /// Emits one public event per executed test and builds the stored result, honouring
    /// the visibility rule: once a hidden test has run, every later record is
    /// verdict-and-time only.
    async fn report(
        &self,
        result: &RunResult,
        spec: &ProblemSpec,
        ctx: &RunContext,
        verdict: Verdict,
    ) -> Result<FinalResult, ExecError> {
        let mut hidden_phase = false;
        let mut records = Vec::with_capacity(result.timings.len());
        let mut passed = 0_u32;
        let mut runtime_ms = 0_u32;
        for timing in &result.timings {
            let Some(executed) = result
                .executed
                .iter()
                .find(|test| test.ordinal() == timing.ordinal)
            else {
                return Err(ExecError::Infrastructure("run_records_mismatch".to_owned()));
            };
            let test_verdict = test_verdict(executed.verdict());
            if test_verdict == TestVerdict::Pass {
                passed = passed.saturating_add(1);
            }
            let time_ms = u32::try_from(timing.elapsed.as_millis()).unwrap_or(u32::MAX);
            runtime_ms = runtime_ms.max(time_ms);
            let visible = timing.sample && !hidden_phase;
            if !timing.sample {
                hidden_phase = true;
            }
            let record = if visible {
                let case = spec
                    .tests
                    .iter()
                    .find(|test| test.ordinal == timing.ordinal);
                let detail = sample_detail(case, executed);
                ctx.emit(&PublicEvent::Test(PublicTest::Sample {
                    ordinal: timing.ordinal,
                    verdict: test_verdict,
                    time_ms,
                    detail: detail.clone(),
                }))
                .await?;
                TestRecord::Sample {
                    ordinal: timing.ordinal,
                    verdict: test_verdict,
                    time_ms,
                    detail,
                }
            } else {
                ctx.emit(&PublicEvent::Test(PublicTest::Hidden {
                    ordinal: timing.ordinal,
                    verdict: test_verdict,
                    time_ms,
                }))
                .await?;
                TestRecord::Hidden {
                    ordinal: timing.ordinal,
                    verdict: test_verdict,
                    time_ms,
                }
            };
            records.push(record);
        }
        let total = result.facts.planned;
        let score = passed.saturating_mul(100).checked_div(total).unwrap_or(0);
        Ok(FinalResult {
            verdict,
            passed,
            total,
            score,
            runtime_ms,
            compile_output: (verdict == Verdict::CompileError).then(|| COMPILE_MESSAGE.to_owned()),
            measured: None,
            tests: records,
        })
    }

    /// Runs the bench and the familiar assessment for one completed run, then
    /// returns the measured columns the terminal write stores. Whatever this
    /// returns, the kept session — when there is one — is closed first.
    async fn measure(&self, call: MeasureCall<'_, B>) -> Result<MeasuredColumns, ExecError> {
        let MeasureCall {
            mut session,
            spec,
            lang,
            run,
            verdict,
            ctx,
            stop,
        } = call;
        // A bench needs all three declarations; a partial one is a broken store.
        let bench_plan = match (
            &spec.bench,
            &spec.generator_schema,
            spec.reference_complexity,
        ) {
            (None, None, None) => None,
            (Some(bench), Some(_), Some(_)) => Some(BenchPlan {
                mode: run.mode,
                sizes: bench.sizes.clone(),
                size_param: bench.size_param,
                problem_slug: spec.slug.clone(),
                problem_version: spec.version,
                budget: match run.mode {
                    RunMode::Run => RUN_BENCH_BUDGET,
                    RunMode::Submit => SUBMIT_BENCH_BUDGET,
                },
            }),
            _ => {
                return Err(ExecError::Infrastructure(
                    "bench_declaration_incomplete".to_owned(),
                ));
            }
        };
        // Baseline points exist only on the queue path; a bench without them still fits.
        let baseline = if bench_plan.is_some() && verdict == Verdict::Accepted {
            match ctx.pool() {
                Some(pool) => find_baseline(
                    pool,
                    &spec.slug,
                    spec.version,
                    &run.lang,
                    lang.image().as_str(),
                )
                .await
                .map_err(|error| ExecError::Infrastructure(error.to_string()))?,
                None => Vec::new(),
            }
        } else {
            Vec::new()
        };
        let input = MeasureInput {
            verdict: Some(verdict),
            mode: run.mode,
            reference: spec.reference_complexity.unwrap_or(ClassId::Linear),
            baseline: Some(&baseline),
            plan: bench_plan.as_ref(),
            max_load: self.bench.max_load,
        };
        let call_deadline = Duration::from_millis(
            spec.time_limit_ms
                .map_or_else(|| lang.time_ms_per_test(), u64::from),
        );
        let mut collector =
            match (verdict, session.as_mut(), spec.generator_schema.clone()) {
                (Verdict::Accepted, Some(opened), Some(schema)) => Some(
                    SandboxBenchCollector::new(opened, lang, schema, call_deadline, ctx, stop),
                ),
                _ => None,
            };
        let columns = measure_run::<SandboxBenchCollector<B::Session>>(
            ctx,
            &self.gate,
            &ProcLoadAvg,
            collector.as_mut(),
            &input,
        )
        .await;
        drop(collector);
        if let Some(opened) = session {
            if let Err(error) = opened.session.close().await {
                tracing::warn!(%error, "closing the run's container did not confirm removal");
            }
        }
        columns
    }

    /// The precheck outcome for the judge's own source re-check (VAL-1, VAL-3): `None`
    /// when the source may run, or the completed compile-error result — with its
    /// familiar verdict, though never a bench — when the judge refuses it and no
    /// container is ever created.
    async fn rejected_before_sandbox(
        &self,
        spec: &ProblemSpec,
        lang: &LanguageDefinition,
        run: &RunInput,
        ctx: &RunContext,
        stop: &mut watch::Receiver<bool>,
    ) -> Result<Option<FinalResult>, ExecError> {
        let Err(rejection) = precheck(run.code.as_bytes(), lang.max_source_bytes()) else {
            return Ok(None);
        };
        tracing::info!(%rejection, "the judge refused the source before the sandbox");
        let total = u32::try_from(spec.tests.len()).unwrap_or(u32::MAX);
        let mut rejected = rejected_result(total);
        // A refused source is still a completed run: it gets the same familiar
        // verdict a compile error in the sandbox would, but never a bench.
        rejected.measured = Some(Box::new(
            self.measure(MeasureCall {
                session: None,
                spec,
                lang,
                run,
                verdict: Verdict::CompileError,
                ctx,
                stop,
            })
            .await?,
        ));
        Ok(Some(rejected))
    }
}

/// The pieces [`SandboxExecutor::measure`] needs, packed so the call sites stay small.
struct MeasureCall<'a, B: Backend> {
    /// The run's kept session, which `measure` closes at the end whatever happens.
    session: Option<Opened<B::Session>>,
    /// The problem the run measured against.
    spec: &'a ProblemSpec,
    /// The language the run used.
    lang: &'a LanguageDefinition,
    /// The run's input; its mode sizes the bench budget.
    run: &'a RunInput,
    /// The verdict the correctness tests produced.
    verdict: Verdict,
    /// The run's event and store sink.
    ctx: &'a RunContext,
    /// The run's stop signal.
    stop: &'a mut watch::Receiver<bool>,
}

impl<P: ProblemTests, B: Backend + Clone + Send + Sync + 'static> RunExecutor
    for SandboxExecutor<P, B>
{
    async fn execute(&self, run: RunInput, ctx: RunContext) -> Result<RunOutcome, ExecError> {
        // 1. The language must be known and enabled.
        let Some(lang) = self
            .languages
            .get(&run.lang)
            .filter(|language| language.is_enabled())
        else {
            return Ok(RunOutcome::Failed(FailureReason::LanguageUnavailable));
        };
        // 2. The problem must load. An absent problem fails the run; a broken store
        //    is infrastructure.
        let Some(spec) = self.load_problem(&run).await? else {
            return Ok(RunOutcome::Failed(FailureReason::ProblemUnavailable));
        };
        // 3. Every probed language's preflight must pass. No report counts as failing.
        if !self.preflights.iter().all(|state| state.passed()) {
            return Err(ExecError::Infrastructure(
                "sandbox_preflight_failed".to_owned(),
            ));
        }
        // 4. Wait for a container permit; a stop while waiting ends the run.
        let mut stop = ctx.stop_watch();
        let _permit = tokio::select! {
            biased;
            () = wait_stopped(&mut stop) => return Err(ExecError::Stopped),
            permit = self.containers.clone().acquire_owned() => match permit {
                Ok(permit) => permit,
                Err(_closed) => {
                    return Err(ExecError::Infrastructure("container_permits_closed".to_owned()));
                }
            },
        };
        // 5. `compiling` is always the first event of a run that can complete.
        ctx.emit(&PublicEvent::Compiling).await?;
        // 6. The judge re-checks the source (VAL-1, VAL-3). A refusal is a compile
        //    error, and no container is ever created for it.
        if let Some(rejected) = self
            .rejected_before_sandbox(&spec, lang, &run, &ctx, &mut stop)
            .await?
        {
            return Ok(RunOutcome::Completed(rejected));
        }
        // 6b. A custom ghost run skips the stored suite entirely: its one input is
        //     judged against the trusted reference by the sandbox oracle, in the
        //     same containers under the same caps (ide-workspace T02).
        if let Some(input) = run.ghost_input.clone() {
            return self
                .execute_ghost(&run, &spec, lang, &input, &ctx, &mut stop)
                .await;
        }
        // 7. Build the archive and the plan, then run it.
        let archive = self.archive(&run.code, lang, &spec)?;
        let plan = plan_of(&spec, lang);
        let settings = RunSettings {
            run_deadline: Instant::now() + self.config.run_wall_cap,
            kill_slack: self.config.kill_slack,
            max_consecutive_abnormal: self.config.max_consecutive_abnormal,
        };
        let worker = self.worker.clone();
        let expires = expiry(self.config.orphan_age);
        let run_cap = self.config.run_wall_cap;
        let runtime = self.config.runtime;
        let session_spec = |ordinal: u32| {
            SessionSpec::new(
                lang,
                run_cap,
                runtime,
                run.run_id.as_uuid(),
                &worker,
                ordinal,
                expires,
            )
            .map_err(|error| InfraError::Unexpected(error.to_string()))
        };
        let (result, session) = execute_run_and_keep(
            &self.backend,
            lang,
            &archive,
            session_spec,
            plan,
            settings,
            &mut stop,
        )
        .await
        .map_err(|error| {
            tracing::warn!(%error, "the sandbox failed mid-run");
            ExecError::Infrastructure("sandbox_infrastructure".to_owned())
        })?;
        // 8. The verdict decides the outcome. A stopped run is `Stopped`, an
        //    unexplained shortfall an internal fault — never a verdict.
        let verdict = match result.verdict() {
            FinalVerdict::Stopped => return Err(ExecError::Stopped),
            FinalVerdict::Incomplete => {
                return Err(ExecError::Infrastructure("run_incomplete".to_owned()));
            }
            verdict => final_verdict(verdict),
        };
        let mut finished = self.report(&result, &spec, &ctx, verdict).await?;
        // 9. Measure: every completed run gets its familiar verdict; an accepted
        //    run with a declared bench also gets timed points, still in the run's
        //    own container. `measure` closes the kept session, whatever it returns.
        finished.measured = Some(Box::new(
            self.measure(MeasureCall {
                session,
                spec: &spec,
                lang,
                run: &run,
                verdict,
                ctx: &ctx,
                stop: &mut stop,
            })
            .await?,
        ));
        Ok(RunOutcome::Completed(finished))
    }
}

/// The result for a source the judge refused before the sandbox: a compile error with
/// the fixed public message, no tests and no container.
fn rejected_result(total: u32) -> FinalResult {
    FinalResult {
        verdict: Verdict::CompileError,
        passed: 0,
        total,
        score: 0,
        runtime_ms: 0,
        compile_output: Some(COMPILE_MESSAGE.to_owned()),
        measured: None,
        tests: Vec::new(),
    }
}

impl<P: ProblemTests, B: Backend + Clone + Send + Sync + 'static> SandboxExecutor<P, B> {
    /// A custom ghost run (ide-workspace T02): the user's source and the trusted
    /// reference run in separate containers through the sandbox oracle, and the
    /// engine's classifier produces the verdict. The problem's stored tests are
    /// never loaded for this path — the oracle holds the signature only.
    async fn execute_ghost(
        &self,
        run: &RunInput,
        spec: &ProblemSpec,
        lang: &LanguageDefinition,
        input: &serde_json::Value,
        ctx: &RunContext,
        stop: &mut watch::Receiver<bool>,
    ) -> Result<RunOutcome, ExecError> {
        let Some(reference) = spec.reference_python.as_deref() else {
            return Ok(RunOutcome::Failed(FailureReason::ProblemUnavailable));
        };
        // Authored references are always Python. A disabled or unknown Python
        // makes the ghost run unjudgeable, never a wrong answer.
        let Some(reference_lang) = self
            .languages
            .get("python")
            .filter(|language| language.is_enabled())
        else {
            return Ok(RunOutcome::Failed(FailureReason::LanguageUnavailable));
        };
        let mut oracle = SandboxOracle::new(
            Handle::current(),
            self.backend.clone(),
            self.backend.clone(),
            lang.clone(),
            reference_lang.clone(),
            &spec.signature,
            &run.code,
            reference,
            &self.config,
        )
        .map_err(|error| ExecError::Infrastructure(error.to_string()))?;
        let limit = spec.time_limit_ms.map_or_else(
            || Duration::from_millis(lang.time_ms_per_test()),
            |ms| Duration::from_millis(u64::from(ms)),
        );
        let custom = input.clone();
        let started = Instant::now();
        let observation = tokio::task::spawn_blocking(move || oracle.evaluate(&custom, limit))
            .await
            .map_err(|_| ExecError::Infrastructure("oracle_join".to_owned()))?
            .map_err(|error| ExecError::Infrastructure(error.to_string()))?;
        let time_ms = u32::try_from(started.elapsed().as_millis()).unwrap_or(u32::MAX);
        if *stop.borrow() {
            return Err(ExecError::Stopped);
        }

        let (verdict, test_verdict) = match classify(&observation, &spec.comparator) {
            Classification::Pass => (Verdict::Accepted, TestVerdict::Pass),
            Classification::Fail(predicate) => match predicate.kind {
                FailureKind::WrongAnswer => (Verdict::WrongAnswer, TestVerdict::WrongAnswer),
                FailureKind::RuntimeError => (Verdict::RuntimeError, TestVerdict::RuntimeError),
                FailureKind::Timeout => (Verdict::TimeLimit, TestVerdict::TimeLimit),
            },
            // The trusted reference failing is a problem defect, not a verdict and
            // not a transient error worth a retry.
            Classification::ReferenceFailed => {
                return Ok(RunOutcome::Failed(FailureReason::ProblemUnavailable));
            }
        };

        // The ghost input is already the user's own data, so the one test shows
        // input, expected and got exactly like a sample. `sanitised` bounds any
        // untrusted exception text before it becomes display detail.
        let user = observation.user.sanitised();
        let reference_result = observation.reference.sanitised();
        let detail = SampleDetail::new(
            &input.to_string(),
            &result_text(&reference_result),
            &result_text(&user),
        );
        ctx.emit(&PublicEvent::Test(PublicTest::Sample {
            ordinal: 1,
            verdict: test_verdict,
            time_ms,
            detail: detail.clone(),
        }))
        .await?;
        let passed = u32::from(test_verdict == TestVerdict::Pass);
        Ok(RunOutcome::Completed(FinalResult {
            verdict,
            passed,
            total: 1,
            score: passed * 100,
            runtime_ms: time_ms,
            compile_output: None,
            measured: None,
            tests: vec![TestRecord::Sample {
                ordinal: 1,
                verdict: test_verdict,
                time_ms,
                detail,
            }],
        }))
    }
}

/// The short display text of one side's program result for the ghost test detail.
fn result_text(result: &counterspell_engine::ProgramResult) -> String {
    use counterspell_engine::ProgramResult;
    match result {
        ProgramResult::Returned(value) => value.to_string(),
        ProgramResult::Crashed {
            exception_class,
            message,
        } => format!("{exception_class}: {message}"),
        ProgramResult::TimedOut => String::new(),
        ProgramResult::Garbled { snippet } => snippet.clone(),
    }
}

/// Resolves when the stop signal fires or its sender is gone.
async fn wait_stopped(stop: &mut watch::Receiver<bool>) {
    loop {
        if *stop.borrow() {
            return;
        }
        if stop.changed().await.is_err() {
            return;
        }
    }
}

/// The plan for a run: the problem's tests in stored order (the run sorts them), each
/// with the problem's time limit or the language default.
fn plan_of(spec: &ProblemSpec, lang: &LanguageDefinition) -> Vec<PlannedTest> {
    let fallback = Duration::from_millis(lang.time_ms_per_test());
    spec.tests
        .iter()
        .map(|case| PlannedTest {
            ordinal: case.ordinal,
            sample: case.is_sample,
            args: case.args.clone(),
            expected: Expected::new(case.expected.clone()),
            comparator: spec.comparator,
            time_limit: spec
                .time_limit_ms
                .map_or(fallback, |ms| Duration::from_millis(u64::from(ms))),
        })
        .collect()
}

/// The unix time after which a container counts as an orphan: now plus the orphan age,
/// matching the sweeper's `created_at + orphan_age` rule.
fn expiry(orphan_age: Duration) -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| {
            i64::try_from(since.as_secs()).unwrap_or(i64::MAX)
        });
    now.saturating_add(i64::try_from(orphan_age.as_secs()).unwrap_or(i64::MAX))
}

/// The public detail of a visible sample test. `got` is the parsed answer value when the
/// line is well-formed, the sanitised raw line when it is not, and empty when the program
/// never answered.
fn sample_detail(case: Option<&TestCase>, executed: &super::verdict::ExecutedTest) -> SampleDetail {
    let input = case
        .and_then(|case| serde_json::to_string(&case.args).ok())
        .unwrap_or_default();
    let expected = case
        .map(|case| case.expected.to_string())
        .unwrap_or_default();
    let got = executed
        .answer()
        .and_then(|line| {
            parse_answer(line, u64::from(executed.ordinal()))
                .map(|value| value.to_string())
                .ok()
        })
        .or_else(|| {
            executed
                .answer()
                .map(|line| sanitize_display(line, DETAIL_CAP).text)
        })
        .unwrap_or_default();
    SampleDetail::new(&input, &expected, &got)
}

/// Maps the sandbox's test verdict onto the queue's verdict enum. The two enums list the
/// same outcomes; `WrongReason` detail stays inside the judge.
fn test_verdict(verdict: &super::compare::TestVerdict) -> TestVerdict {
    match verdict {
        super::compare::TestVerdict::Pass(_) => TestVerdict::Pass,
        super::compare::TestVerdict::WrongAnswer(_) => TestVerdict::WrongAnswer,
        super::compare::TestVerdict::RuntimeError => TestVerdict::RuntimeError,
        super::compare::TestVerdict::TimeLimit => TestVerdict::TimeLimit,
        super::compare::TestVerdict::MemoryLimit => TestVerdict::MemoryLimit,
        super::compare::TestVerdict::OutputLimit => TestVerdict::OutputLimit,
    }
}

/// Maps the run verdict onto the queue's verdict enum. `Stopped` and `Incomplete` have no
/// verdict and never reach this function.
fn final_verdict(verdict: FinalVerdict) -> Verdict {
    match verdict {
        FinalVerdict::Accepted => Verdict::Accepted,
        FinalVerdict::CompileError => Verdict::CompileError,
        FinalVerdict::WrongAnswer => Verdict::WrongAnswer,
        FinalVerdict::TimeLimit => Verdict::TimeLimit,
        FinalVerdict::MemoryLimit => Verdict::MemoryLimit,
        FinalVerdict::OutputLimit => Verdict::OutputLimit,
        // These never reach this function — the caller returns first. The arm exists so
        // the match stays total if the verdict enum grows.
        FinalVerdict::RuntimeError | FinalVerdict::Stopped | FinalVerdict::Incomplete => {
            Verdict::RuntimeError
        }
    }
}
