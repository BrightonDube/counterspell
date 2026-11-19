// SPDX-License-Identifier: MIT
//! The `baseline` admission: time a known-correct fixture, fit it, and store the timings only
//! when the fit confirms the problem's declared reference class (R8, R13.2).
//!
//! The module runs nothing itself. Timing goes through [`BenchCollector`], the same trait a
//! learner's Submit uses, and storage goes through [`BaselineStore`], so the tests use a
//! scripted collector and an in-memory store.

use std::future::Future;

use counterspell_common::familiar::ClassId;
use counterspell_engine::complexity::{fit, Class, Confidence, FitConfig, FitError, Mode, Point};
use counterspell_engine::ENGINE_VERSION;
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
}
