// SPDX-License-Identifier: MIT
//! The `baseline` admission: time a known-correct fixture, fit it, and store the timings only
//! when the fit confirms the problem's declared reference class (R8, R13.2).
//!
//! The module runs nothing itself. Timing goes through [`BenchCollector`], the same trait a
//! learner's Submit uses, and storage goes through [`BaselineStore`], so the tests use a
//! scripted collector and an in-memory store.

use std::future::Future;

use counterspell_common::familiar::ClassId;
use counterspell_engine::complexity::{
    bench_seed, fit, sized_input, BenchInputError, Class, Confidence, FitConfig, FitError, Mode,
    Point,
};
use counterspell_engine::{Schema, ENGINE_VERSION};
use serde_json::Value;
use sqlx::PgPool;

use super::assess::{to_common_class, BenchPoint};
use super::store::{insert_baselines, BaselineRow, StoreError};
use super::{BenchCollector, BenchOutcome, BenchPlan};
use crate::executor::ExecError;

/// Everything one baseline run needs to know.
#[derive(Debug, Clone)]
pub struct BaselineRequest {
    /// The problem the fixture solves.
    pub problem_slug: String,
    /// The problem version the timings belong to.
    pub problem_version: u32,
    /// The fixture's language.
    pub lang: String,
    /// The digest of the sandbox image the fixture runs in.
    pub image_digest: String,
    /// The class the pack author declared for the reference.
    pub reference: ClassId,
    /// The bench to run: the pack's sizes and the Submit budget.
    pub plan: BenchPlan,
}

/// Why a fixture's timings were not stored. A refusal is a finding about the fixture, not a fault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The bench ran out of budget before every size was timed.
    BenchIncomplete,
    /// The fixture crashed or timed out during the bench.
    FixtureFailed,
    /// The fit found a different class from the declared one.
    ClassMismatch {
        /// The class the data supports.
        observed: ClassId,
        /// The class the author declared.
        reference: ClassId,
    },
    /// The fit could not single out one class at `Medium` confidence or better.
    NotConfirmed {
        /// The class the data leans toward.
        leaning: ClassId,
        /// Every class the noise cannot rule out.
        plausible: Vec<ClassId>,
    },
    /// The points break the fit's rules (for example too few sizes).
    InvalidPoints(String),
}

impl std::fmt::Display for Refusal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BenchIncomplete => {
                write!(f, "the bench ran out of budget before every size was timed")
            }
            Self::FixtureFailed => write!(f, "the fixture failed during the bench"),
            Self::ClassMismatch {
                observed,
                reference,
            } => {
                write!(
                    f,
                    "the fixture looks {observed:?}, but the pack declares {reference:?}"
                )
            }
            Self::NotConfirmed { leaning, plausible } => write!(
                f,
                "the fit is not confirmed (leaning {leaning:?}, plausible {plausible:?})"
            ),
            Self::InvalidPoints(why) => write!(f, "the timings cannot be fitted: {why}"),
        }
    }
}

/// What a baseline run did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BaselineOutcome {
    /// The timings were accepted. `inserted` counts new rows; it is 0 when they already existed.
    Stored {
        /// How many rows were new.
        inserted: u64,
    },
    /// The timings were refused and nothing was stored.
    Refused(Refusal),
}

/// A baseline run that could not finish because the judge itself failed.
#[derive(Debug)]
pub enum BaselineError {
    /// The collector failed (sandbox or stop signal).
    Collector(ExecError),
    /// The store failed.
    Store(StoreError),
}

/// Where accepted baseline rows go. The real store is Postgres; tests use memory.
pub trait BaselineStore: Sync {
    /// Inserts rows and returns how many were new.
    fn insert(&self, rows: &[BaselineRow]) -> impl Future<Output = Result<u64, StoreError>> + Send;
}

impl BaselineStore for PgPool {
    fn insert(&self, rows: &[BaselineRow]) -> impl Future<Output = Result<u64, StoreError>> + Send {
        insert_baselines(self, rows)
    }
}

/// Maps the shared class type to the engine's. Exhaustive, so a new class is a compile error.
const fn to_engine_class(class: ClassId) -> Class {
    match class {
        ClassId::Constant => Class::Constant,
        ClassId::Logarithmic => Class::Logarithmic,
        ClassId::Linear => Class::Linear,
        ClassId::Linearithmic => Class::Linearithmic,
        ClassId::Quadratic => Class::Quadratic,
        ClassId::Cubic => Class::Cubic,
        ClassId::Exponential => Class::Exponential,
    }
}

/// Decides whether timed points may become a baseline: the full-mode fit must report the
/// declared class at `Medium` confidence or better.
fn admit(points: &[BenchPoint], reference: ClassId) -> Result<(), Refusal> {
    let fit_points: Vec<Point> = points
        .iter()
        .map(|p| Point {
            n: p.n,
            median_ns: p.median_ns,
            spread: p.spread,
        })
        .collect();
    let report = fit(&fit_points, Mode::Full, &FitConfig::default())
        .map_err(|error: FitError| Refusal::InvalidPoints(error.to_string()))?;
    match report.class {
        Some(class) if report.confidence >= Confidence::Medium => {
            if class == to_engine_class(reference) {
                Ok(())
            } else {
                Err(Refusal::ClassMismatch {
                    observed: to_common_class(class),
                    reference,
                })
            }
        }
        _ => Err(Refusal::NotConfirmed {
            leaning: to_common_class(report.leaning),
            plausible: report
                .plausible
                .iter()
                .map(|c| to_common_class(*c))
                .collect(),
        }),
    }
}

/// Times the fixture, admits or refuses it, and stores the accepted timings.
///
/// # Errors
///
/// Returns [`BaselineError`] when the collector or the store fails. A fixture that is merely
/// wrong is a [`BaselineOutcome::Refused`], never an error.
pub async fn run_baseline<C: BenchCollector, S: BaselineStore>(
    collector: &mut C,
    store: &S,
    request: &BaselineRequest,
) -> Result<BaselineOutcome, BaselineError> {
    let outcome = collector
        .collect(&request.plan)
        .await
        .map_err(BaselineError::Collector)?;
    let points = match outcome {
        BenchOutcome::Complete(points) => points,
        BenchOutcome::Incomplete(_) => {
            return Ok(BaselineOutcome::Refused(Refusal::BenchIncomplete))
        }
        BenchOutcome::Failed => return Ok(BaselineOutcome::Refused(Refusal::FixtureFailed)),
    };
    if let Err(refusal) = admit(&points, request.reference) {
        return Ok(BaselineOutcome::Refused(refusal));
    }
    let rows: Vec<BaselineRow> = points
        .iter()
        .map(|p| BaselineRow {
            problem_slug: request.problem_slug.clone(),
            problem_version: request.problem_version,
            lang: request.lang.clone(),
            n: p.n,
            median_ns: p.median_ns,
            image_digest: request.image_digest.clone(),
            engine_version: ENGINE_VERSION.to_owned(),
            fit_version: counterspell_engine::complexity::FIT_VERSION.to_owned(),
        })
        .collect();
    let inserted = store.insert(&rows).await.map_err(BaselineError::Store)?;
    Ok(BaselineOutcome::Stored { inserted })
}

/// What the command line asked for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselineArgs {
    /// The problem slug.
    pub problem: String,
    /// The fixture's language name, as in `languages.toml`.
    pub lang: String,
    /// A file holding the known-correct fixture. Without it the stored Python reference is used.
    pub fixture: Option<std::path::PathBuf>,
}

/// Parses `--problem <slug> --lang <lang> [--fixture <path>]`, in any order.
///
/// # Errors
///
/// Returns a message when a flag is unknown, repeated or missing its value, or when
/// `--problem` or `--lang` is absent.
pub fn parse_args(args: &[String]) -> Result<BaselineArgs, String> {
    let mut problem = None;
    let mut lang = None;
    let mut fixture = None;
    let mut iter = args.iter();
    while let Some(flag) = iter.next() {
        let slot = match flag.as_str() {
            "--problem" => &mut problem,
            "--lang" => &mut lang,
            "--fixture" => &mut fixture,
            other => return Err(format!("unknown argument {other}")),
        };
        let value = iter.next().ok_or_else(|| format!("{flag} needs a value"))?;
        if slot.replace(value.clone()).is_some() {
            return Err(format!("{flag} was given twice"));
        }
    }
    Ok(BaselineArgs {
        problem: problem.ok_or("--problem is required")?,
        lang: lang.ok_or("--lang is required")?,
        fixture: fixture.map(std::path::PathBuf::from),
    })
}

/// Builds one argument tuple per size with the seed a learner's bench uses, so the baseline
/// and a learner's timings are taken on identical inputs.
///
/// # Errors
///
/// Returns [`BenchInputError`] when a size cannot be built from the schema.
pub fn fixture_inputs(
    schema: &Schema,
    size_param: usize,
    slug: &str,
    version: u32,
    sizes: &[u64],
) -> Result<Vec<(u64, Value)>, BenchInputError> {
    sizes
        .iter()
        .map(|&n| sized_input(schema, size_param, n, bench_seed(slug, version, n)).map(|v| (n, v)))
        .collect()
}

/// Times a known-correct fixture inside a sandbox container, on inputs prepared up front.
///
/// This is the collector the `baseline` command hands to [`run_baseline`]. It runs nothing on
/// the host: the fixture goes through the same driver, limits and bench loop a learner's
/// Submit uses.
#[cfg(feature = "sandbox")]
pub struct FixtureCollector<B: crate::sandbox::backend::Backend> {
    side: crate::sandbox::oracle::Side<B>,
    inputs: Vec<(u64, Value)>,
    call_limit: std::time::Duration,
    stop: tokio::sync::watch::Receiver<bool>,
}

#[cfg(feature = "sandbox")]
impl<B: crate::sandbox::backend::Backend> FixtureCollector<B> {
    /// Wraps a fixture's sandbox side with the inputs to time and the per-call limit.
    pub fn new(
        side: crate::sandbox::oracle::Side<B>,
        inputs: Vec<(u64, Value)>,
        call_limit: std::time::Duration,
        stop: tokio::sync::watch::Receiver<bool>,
    ) -> Self {
        Self {
            side,
            inputs,
            call_limit,
            stop,
        }
    }
}

#[cfg(feature = "sandbox")]
impl<B> BenchCollector for FixtureCollector<B>
where
    B: crate::sandbox::backend::Backend + Send + Sync,
    B::Session: Send,
{
    async fn collect(&mut self, plan: &BenchPlan) -> Result<BenchOutcome, ExecError> {
        crate::sandbox::oracle::bench_side(
            &mut self.side,
            &self.inputs,
            self.call_limit,
            plan.budget,
            self.stop.clone(),
        )
        .await
        .map_err(|error| ExecError::Infrastructure(error.to_string()))
    }
}

/// Why `baseline` could not run at all. Both kinds exit with code 2.
#[cfg(feature = "sandbox")]
#[derive(Debug)]
pub enum CommandError {
    /// Something the command needs is missing or unusable (config, daemon, problem, fixture).
    Setup(String),
    /// The bench or the store failed while running.
    Run(BaselineError),
}

/// Picks the fixture's source: the `--fixture` file when given, otherwise the stored Python
/// reference. Any other language needs a file, because only Python references are stored.
#[cfg(feature = "sandbox")]
fn fixture_source(
    args: &BaselineArgs,
    stored_python: Option<&str>,
) -> Result<String, CommandError> {
    let setup = |what: String| CommandError::Setup(what);
    match (&args.fixture, args.lang.as_str()) {
        (Some(path), _) => std::fs::read_to_string(path)
            .map_err(|e| setup(format!("fixture {}: {e}", path.display()))),
        (None, "python") => stored_python
            .map(str::to_owned)
            .ok_or_else(|| setup("the problem has no stored Python reference".to_owned())),
        (None, other) => Err(setup(format!(
            "no stored fixture for {other}; pass --fixture <path>"
        ))),
    }
}

/// Runs the whole `baseline` command: loads the published problem, reads the fixture, times it
/// in the sandbox and admits or refuses it.
///
/// # Errors
///
/// Returns [`CommandError`] for anything that is not a finding about the fixture itself.
#[cfg(feature = "sandbox")]
pub async fn execute(args: &BaselineArgs, pool: &PgPool) -> Result<BaselineOutcome, CommandError> {
    use crate::languages::LanguageCatalog;
    use crate::problem_store::PgProblemTests;
    use crate::sandbox::bench::SUBMIT_BENCH_BUDGET;
    use crate::sandbox::bollard_backend::BollardBackend;
    use crate::sandbox::config::SandboxConfig;
    use crate::sandbox::daemon::{connect, verify};
    use crate::sandbox::oracle::{side, Role};
    use crate::sandbox::problem::ProblemTests;
    use counterspell_common::status::RunMode;

    let setup = |what: String| CommandError::Setup(what);
    let cfg = SandboxConfig::from_env().map_err(|e| setup(format!("sandbox config: {e}")))?;
    let docker = connect(&cfg.docker_socket, cfg.api_timeout)
        .map_err(|e| setup(format!("sandbox daemon: {e}")))?;
    // The identity guard: a rootful or main daemon is refused before anything starts.
    verify(&docker, cfg.runtime, cfg.api_timeout)
        .await
        .map_err(|e| setup(format!("sandbox daemon: {e}")))?;
    let catalog = LanguageCatalog::load(&cfg.languages_file)
        .map_err(|e| setup(format!("language catalogue: {e}")))?;
    let lang = catalog
        .get(&args.lang)
        .filter(|lang| lang.is_enabled())
        .ok_or_else(|| setup(format!("language {} is not enabled", args.lang)))?
        .clone();

    let version = super::store::published_version(pool, &args.problem)
        .await
        .map_err(|e| setup(format!("published version: {e}")))?
        .ok_or_else(|| setup(format!("{} has no published version", args.problem)))?;
    let spec = PgProblemTests::new(pool.clone())
        .load(&args.problem, version, RunMode::Run)
        .await
        .map_err(|e| setup(e.to_string()))?
        .ok_or_else(|| {
            setup(format!(
                "{} v{version} is not in the catalogue",
                args.problem
            ))
        })?;
    let (Some(bench), Some(schema), Some(reference)) = (
        spec.bench.as_ref(),
        spec.generator_schema.as_ref(),
        spec.reference_complexity,
    ) else {
        return Err(setup(format!(
            "{} v{version} declares no bench, so it has no baseline to take",
            args.problem
        )));
    };

    let source = fixture_source(args, spec.reference_python.as_deref())?;
    let inputs = fixture_inputs(
        schema,
        bench.size_param,
        &args.problem,
        version,
        &bench.sizes,
    )
    .map_err(|e| setup(format!("bench inputs: {e}")))?;
    let signature =
        serde_json::to_vec(&spec.signature).map_err(|e| setup(format!("signature: {e}")))?;
    let driver_root = cfg.languages_file.parent().map_or_else(
        || std::path::PathBuf::from("."),
        std::path::Path::to_path_buf,
    );
    let backend = BollardBackend::new(docker, cfg.api_timeout);
    let fixture_side = side(
        backend,
        lang.clone(),
        &driver_root,
        &signature,
        &source,
        &cfg,
        Role::Reference,
    )
    .map_err(|e| setup(e.to_string()))?;

    let (_stop, stop_rx) = tokio::sync::watch::channel(false);
    let call_limit = std::time::Duration::from_millis(
        spec.time_limit_ms
            .map_or_else(|| lang.time_ms_per_test(), u64::from),
    );
    let mut collector = FixtureCollector::new(fixture_side, inputs, call_limit, stop_rx);
    let request = BaselineRequest {
        problem_slug: args.problem.clone(),
        problem_version: version,
        lang: args.lang.clone(),
        image_digest: lang.image().as_str().to_owned(),
        reference,
        plan: BenchPlan {
            mode: RunMode::Submit,
            sizes: bench.sizes.clone(),
            size_param: bench.size_param,
            problem_slug: args.problem.clone(),
            problem_version: version,
            budget: SUBMIT_BENCH_BUDGET,
        },
    };
    run_baseline(&mut collector, pool, &request)
        .await
        .map_err(CommandError::Run)
}

/// The process exit code: 0 stored, 1 refused (a finding), 2 infrastructure fault.
#[must_use]
pub fn exit_code(result: &Result<BaselineOutcome, BaselineError>) -> u8 {
    match result {
        Ok(BaselineOutcome::Stored { .. }) => 0,
        Ok(BaselineOutcome::Refused(_)) => 1,
        Err(_) => 2,
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Mutex;

    use counterspell_common::familiar::ClassId;
    use counterspell_common::status::RunMode;

    use super::*;
    use crate::executor::ExecError;
    use crate::familiar::assess::BenchPoint;
    use crate::familiar::store::StoreError;
    use crate::familiar::{BenchOutcome, BenchPlan};
    use crate::testkit::ScriptedCollector;

    /// An in-memory store that, like the real one, ignores rows it already holds.
    #[derive(Default)]
    struct MemoryStore {
        rows: Mutex<Vec<BaselineRow>>,
    }

    impl BaselineStore for MemoryStore {
        fn insert(
            &self,
            rows: &[BaselineRow],
        ) -> impl std::future::Future<Output = Result<u64, StoreError>> + Send {
            let mut held = self.rows.lock().expect("the lock is not poisoned");
            let mut inserted = 0;
            for row in rows {
                if !held.iter().any(|r| {
                    r.n == row.n
                        && r.lang == row.lang
                        && r.problem_slug == row.problem_slug
                        && r.problem_version == row.problem_version
                        && r.image_digest == row.image_digest
                }) {
                    held.push(row.clone());
                    inserted += 1;
                }
            }
            std::future::ready(Ok(inserted))
        }
    }

    fn request(reference: ClassId) -> BaselineRequest {
        BaselineRequest {
            problem_slug: "count-pairs".to_owned(),
            problem_version: 3,
            lang: "python".to_owned(),
            image_digest: "sha256:abc".to_owned(),
            reference,
            plan: BenchPlan {
                mode: RunMode::Submit,
                sizes: vec![256, 512, 1024, 2048, 4096, 8192],
                size_param: 0,
                problem_slug: "count-pairs".to_owned(),
                problem_version: 3,
                budget: std::time::Duration::from_secs(4),
            },
        }
    }

    fn points(f: impl Fn(u64) -> u64) -> Vec<BenchPoint> {
        [256_u64, 512, 1024, 2048, 4096, 8192]
            .iter()
            .map(|&n| BenchPoint {
                n,
                median_ns: f(n),
                spread: Some(0.02),
            })
            .collect()
    }

    fn collector(outcome: BenchOutcome) -> ScriptedCollector {
        ScriptedCollector::new(Ok(outcome))
    }

    #[tokio::test]
    async fn a_fixture_that_matches_the_reference_class_is_stored_with_its_versions() {
        let store = MemoryStore::default();
        let mut c = collector(BenchOutcome::Complete(points(|n| n * 1000)));
        let outcome = run_baseline(&mut c, &store, &request(ClassId::Linear))
            .await
            .expect("no infrastructure fault");
        assert_eq!(outcome, BaselineOutcome::Stored { inserted: 6 });
        let rows = store.rows.lock().expect("lock");
        assert_eq!(rows.len(), 6);
        assert_eq!(rows[0].n, 256);
        assert_eq!(rows[0].median_ns, 256_000);
        assert_eq!(rows[0].problem_version, 3);
        assert_eq!(rows[0].image_digest, "sha256:abc");
        assert_eq!(rows[0].fit_version, "fit-1");
    }

    #[tokio::test]
    async fn a_fixture_of_the_wrong_class_is_refused_and_nothing_is_stored() {
        let store = MemoryStore::default();
        let mut c = collector(BenchOutcome::Complete(points(|n| n * n)));
        let outcome = run_baseline(&mut c, &store, &request(ClassId::Linear))
            .await
            .expect("no infrastructure fault");
        assert_eq!(
            outcome,
            BaselineOutcome::Refused(Refusal::ClassMismatch {
                observed: ClassId::Quadratic,
                reference: ClassId::Linear
            })
        );
        assert!(store.rows.lock().expect("lock").is_empty());
    }

    #[tokio::test]
    async fn an_unconfirmed_low_confidence_fit_is_refused() {
        let store = MemoryStore::default();
        let noisy = [100_u64, 9000, 200, 7000, 150, 8000];
        let pts: Vec<BenchPoint> = [256_u64, 512, 1024, 2048, 4096, 8192]
            .iter()
            .zip(noisy)
            .map(|(&n, t)| BenchPoint {
                n,
                median_ns: t,
                spread: Some(0.02),
            })
            .collect();
        let mut c = collector(BenchOutcome::Complete(pts));
        let outcome = run_baseline(&mut c, &store, &request(ClassId::Linear))
            .await
            .expect("no infrastructure fault");
        assert!(
            matches!(
                outcome,
                BaselineOutcome::Refused(Refusal::NotConfirmed { .. })
            ),
            "{outcome:?}"
        );
        assert!(store.rows.lock().expect("lock").is_empty());
    }

    #[tokio::test]
    async fn an_incomplete_or_failed_bench_is_refused() {
        let store = MemoryStore::default();
        let mut short = collector(BenchOutcome::Incomplete(points(|n| n)));
        assert_eq!(
            run_baseline(&mut short, &store, &request(ClassId::Linear))
                .await
                .expect("ok"),
            BaselineOutcome::Refused(Refusal::BenchIncomplete)
        );
        let mut failed = collector(BenchOutcome::Failed);
        assert_eq!(
            run_baseline(&mut failed, &store, &request(ClassId::Linear))
                .await
                .expect("ok"),
            BaselineOutcome::Refused(Refusal::FixtureFailed)
        );
        assert!(store.rows.lock().expect("lock").is_empty());
    }

    #[tokio::test]
    async fn storing_the_same_baseline_twice_adds_nothing() {
        let store = MemoryStore::default();
        for expected in [6, 0] {
            let mut c = collector(BenchOutcome::Complete(points(|n| n * 1000)));
            let outcome = run_baseline(&mut c, &store, &request(ClassId::Linear))
                .await
                .expect("ok");
            assert_eq!(outcome, BaselineOutcome::Stored { inserted: expected });
        }
        assert_eq!(store.rows.lock().expect("lock").len(), 6);
    }

    #[tokio::test]
    async fn a_collector_fault_is_an_error_not_a_refusal_and_exit_code_two() {
        let store = MemoryStore::default();
        let mut c =
            ScriptedCollector::new(Err(ExecError::Infrastructure("daemon down".to_owned())));
        let result = run_baseline(&mut c, &store, &request(ClassId::Linear)).await;
        let error = result.expect_err("a fault is an error");
        assert_eq!(exit_code(&Err(error)), 2);
        assert_eq!(exit_code(&Ok(BaselineOutcome::Stored { inserted: 6 })), 0);
        assert_eq!(
            exit_code(&Ok(BaselineOutcome::Refused(Refusal::BenchIncomplete))),
            1
        );
        assert!(store.rows.lock().expect("lock").is_empty());
    }

    fn argv(parts: &[&str]) -> Vec<String> {
        parts.iter().map(|part| (*part).to_owned()).collect()
    }

    /// R8: the command names one problem and one language, nothing else is guessed.
    #[test]
    fn the_command_line_needs_a_problem_and_a_language() {
        let args = parse_args(&argv(&["--problem", "count-pairs", "--lang", "python"]))
            .expect("a complete command line parses");
        assert_eq!(args.problem, "count-pairs");
        assert_eq!(args.lang, "python");
        assert_eq!(args.fixture, None);

        let with_fixture = parse_args(&argv(&[
            "--lang",
            "javascript",
            "--problem",
            "count-pairs",
            "--fixture",
            "/tmp/solution.js",
        ]))
        .expect("flags may come in any order");
        assert_eq!(
            with_fixture.fixture.as_deref(),
            Some(std::path::Path::new("/tmp/solution.js"))
        );

        for bad in [
            argv(&["--problem", "count-pairs"]),
            argv(&["--lang", "python"]),
            argv(&["--problem", "count-pairs", "--lang"]),
            argv(&["--problem", "a", "--lang", "python", "--force"]),
            argv(&["--problem", "a", "--problem", "b", "--lang", "python"]),
        ] {
            assert!(parse_args(&bad).is_err(), "{bad:?} must be refused");
        }
    }

    /// R8.2: the baseline sees the same inputs a learner's bench sees, so the two timings
    /// compare like with like.
    #[test]
    fn fixture_inputs_are_the_ones_a_learners_bench_generates() {
        let schema = Schema::Tuple {
            items: vec![Schema::IntArray {
                min_len: 0,
                max_len: 10,
                min: -5,
                max: 5,
                sorted: false,
                unique: false,
            }],
        };
        let inputs = fixture_inputs(&schema, 0, "count-pairs", 3, &[8, 16]).expect("sized");
        assert_eq!(inputs.len(), 2);
        assert_eq!(inputs[0].0, 8);
        assert_eq!(inputs[1].0, 16);
        assert_eq!(inputs[0].1[0].as_array().map(Vec::len), Some(8));
        assert_eq!(inputs[1].1[0].as_array().map(Vec::len), Some(16));
        let learner = sized_input(&schema, 0, 16, bench_seed("count-pairs", 3, 16)).expect("ok");
        assert_eq!(inputs[1].1, learner);
    }

    /// R13.2: a bench that cannot be sized is a finding about the pack, not a silent skip.
    #[test]
    fn a_size_parameter_outside_the_tuple_is_an_error() {
        let schema = Schema::Tuple {
            items: vec![Schema::Int { min: 0, max: 9 }],
        };
        assert!(fixture_inputs(&schema, 4, "p", 1, &[8]).is_err());
    }
}
