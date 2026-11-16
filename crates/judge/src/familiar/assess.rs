//! Turns what the judge measured into the mood, the event payloads and the stored columns.
//!
//! This module is pure. It does no I/O, reads no clock and takes everything as arguments. The
//! event payloads and the stored columns are built from the same values here, so a stored result
//! always matches what the learner saw.

use counterspell_common::familiar::class::{ClassId, Confidence};
use counterspell_common::familiar::facts::{BenchFacts, MeasuredFacts, NoMeasurement};
use counterspell_common::familiar::mood::decide;
use counterspell_common::familiar::payload::{
    capped_points, BenchStatus, ComplexityPayload, FamiliarPayload, FitSummary, Layer,
    MeasuredColumns, PointSummary,
};
use counterspell_common::familiar::words::{wording, WordFacts};
use counterspell_common::status::{RunMode, Verdict};
use counterspell_engine::complexity::{
    self as engine, Class as EngineClass, Confidence as EngineConfidence, FitConfig, FitReport,
    Mode as EngineMode, Point as EnginePoint, Reason as EngineReason,
};

use super::ratio::ratio_vs_baseline;

/// One timed size from the bench. `spread` is the relative spread of its repeats, when known.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BenchPoint {
    /// The input size.
    pub n: u64,
    /// The median time in nanoseconds.
    pub median_ns: u64,
    /// The relative spread of the repeats, when known.
    pub spread: Option<f64>,
}

/// What happened to the bench for this run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum BenchState<'a> {
    /// The problem declares no bench.
    NotSpecified,
    /// The host was too busy, so the bench was discarded.
    Discarded,
    /// The bench call failed.
    Failed,
    /// Too few sizes finished inside the budget.
    Incomplete,
    /// The bench finished with these points.
    Points(&'a [BenchPoint]),
}

/// Everything `assess` needs for one run.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AssessInput<'a> {
    /// The verdict of a completed run. `None` when the run did not complete, so it gets no mood.
    pub verdict: Option<Verdict>,
    /// Whether this was a Run or a Submit.
    pub mode: RunMode,
    /// The class the problem author declared as the reference.
    pub reference: ClassId,
    /// What the bench measured.
    pub bench: BenchState<'a>,
    /// Baseline points already filtered to this problem version, language and image digest.
    pub baseline: Option<&'a [(u64, u64)]>,
}

/// The payloads and columns for one run.
#[derive(Debug, Clone, PartialEq)]
pub struct Assessment {
    /// The `complexity` event. Present only when the points were fitted.
    pub complexity: Option<ComplexityPayload>,
    /// The `familiar` event. Present only for a completed run.
    pub familiar: Option<FamiliarPayload>,
    /// The values the terminal write stores.
    pub columns: MeasuredColumns,
}

impl Assessment {
    /// The result for a run that did not complete: nothing is measured or stored.
    #[must_use]
    pub fn not_completed() -> Self {
        Self {
            complexity: None,
            familiar: None,
            columns: MeasuredColumns {
                measured_complexity: None,
                complexity_confidence: None,
                fit: None,
                familiar_mood: None,
                familiar_reason: None,
            },
        }
    }
}

/// Builds the payloads and columns for one run.
#[must_use]
pub fn assess(input: &AssessInput<'_>) -> Assessment {
    let Some(verdict) = input.verdict else {
        return Assessment::not_completed();
    };
    let trial = if verdict == Verdict::Accepted {
        run_trial(input)
    } else {
        Trial::Skipped
    };
    let bench_facts = match &trial {
        Trial::Skipped => BenchFacts::None(NoMeasurement::NotSpecified),
        Trial::Missing(missing) => BenchFacts::None(*missing),
        Trial::Fitted(fitted) => BenchFacts::Measured {
            confidence: to_common_confidence(fitted.report.confidence),
            observed: fitted.report.class.map(to_common_class),
            reference: input.reference,
            ratio_vs_baseline: fitted.ratio,
        },
    };
    let decision = decide(&MeasuredFacts {
        verdict,
        mode: input.mode,
        bench: bench_facts,
    });
    let plausible: Vec<ClassId> = match &trial {
        Trial::Fitted(fitted) => fitted
            .report
            .plausible
            .iter()
            .map(|class| to_common_class(*class))
            .collect(),
        Trial::Skipped | Trial::Missing(_) => Vec::new(),
    };
    let text = wording(&WordFacts {
        reason: decision.reason,
        mode: input.mode,
        observed: bench_facts_observed(&bench_facts),
        confidence: bench_facts_confidence(&bench_facts),
        plausible: &plausible,
        reference: Some(input.reference),
        ratio: bench_facts_ratio(&bench_facts),
    });
    let familiar = FamiliarPayload {
        mood: decision.mood,
        reason: decision.reason.code().to_owned(),
        layer: Layer::Measured,
        effect: decision.effect,
        headline: text.headline,
        badge: text.badge,
        mode: input.mode,
    };
    let (complexity, measured_complexity, complexity_confidence, fit) = match &trial {
        Trial::Skipped => (None, None, None, None),
        Trial::Missing(missing) => (
            None,
            None,
            None,
            Some(FitSummary::not_fitted(
                input.mode,
                status_of(*missing),
                input.reference,
            )),
        ),
        Trial::Fitted(fitted) => {
            let report = &fitted.report;
            let observed = report.class.map(to_common_class);
            let confidence = to_common_confidence(report.confidence);
            let payload = ComplexityPayload {
                class: observed,
                confidence,
                reference: input.reference,
                plausible: plausible.clone(),
                leaning: to_common_class(report.leaning),
                ratio_vs_baseline: fitted.ratio,
                points: report.points,
                fit_version: report.fit_version.to_owned(),
                mode: input.mode,
            };
            (
                Some(payload),
                observed,
                Some(confidence),
                Some(fit_summary(input, fitted)),
            )
        }
    };
    Assessment {
        complexity,
        familiar: Some(familiar),
        columns: MeasuredColumns {
            measured_complexity,
            complexity_confidence,
            fit,
            familiar_mood: Some(decision.mood),
            familiar_reason: Some(decision.reason.code().to_owned()),
        },
    }
}

/// What the bench produced for an accepted run.
enum Trial {
    /// The run was not accepted, so the bench did not count.
    Skipped,
    /// The bench gave no growth measurement.
    Missing(NoMeasurement),
    /// The bench was fitted.
    Fitted(Fitted),
}

/// A fitted bench with the baseline ratio, when a usable baseline exists.
struct Fitted {
    report: FitReport,
    points: Vec<PointSummary>,
    ratio: Option<f64>,
}

/// Runs the fit for an accepted run's bench state.
fn run_trial(input: &AssessInput<'_>) -> Trial {
    match input.bench {
        BenchState::NotSpecified => Trial::Missing(NoMeasurement::NotSpecified),
        BenchState::Discarded => Trial::Missing(NoMeasurement::HostBusy),
        BenchState::Failed => Trial::Missing(NoMeasurement::Failed),
        BenchState::Incomplete => Trial::Missing(NoMeasurement::Incomplete),
        BenchState::Points(points) => {
            let engine_points: Vec<EnginePoint> = points
                .iter()
                .map(|p| EnginePoint {
                    n: p.n,
                    median_ns: p.median_ns,
                    spread: p.spread,
                })
                .collect();
            match engine::fit(
                &engine_points,
                engine_mode(input.mode),
                &FitConfig::default(),
            ) {
                // Points the fit rejects mean the bench did not produce a usable curve.
                Err(_) => Trial::Missing(NoMeasurement::Failed),
                Ok(report) => {
                    let learner: Vec<(u64, u64)> =
                        points.iter().map(|p| (p.n, p.median_ns)).collect();
                    let ratio = input
                        .baseline
                        .and_then(|base| ratio_vs_baseline(&learner, base));
                    Trial::Fitted(Fitted {
                        report,
                        points: points
                            .iter()
                            .map(|p| PointSummary {
                                n: p.n,
                                median_ns: p.median_ns,
                            })
                            .collect(),
                        ratio,
                    })
                }
            }
        }
    }
}

/// The compact fit summary stored in `submissions.fit`.
fn fit_summary(input: &AssessInput<'_>, fitted: &Fitted) -> FitSummary {
    let report = &fitted.report;
    FitSummary {
        fit_version: report.fit_version.to_owned(),
        mode: input.mode,
        bench_status: BenchStatus::Measured,
        reference: input.reference,
        class: report.class.map(to_common_class),
        confidence: Some(to_common_confidence(report.confidence)),
        plausible: report
            .plausible
            .iter()
            .map(|c| to_common_class(*c))
            .collect(),
        leaning: Some(to_common_class(report.leaning)),
        points: capped_points(&fitted.points),
        slope: Some(report.slope),
        slope_se: Some(report.slope_se),
        exponents: report.exponents.clone(),
        residuals: report
            .residuals
            .iter()
            .map(|(class, residual)| (class.id().to_owned(), *residual))
            .collect(),
        noise: Some(report.noise),
        range_ratio: Some(report.range_ratio),
        ratio_vs_baseline: fitted.ratio,
        reasons: report
            .reasons
            .iter()
            .map(|reason| reason_code(*reason).to_owned())
            .collect(),
    }
}

/// The bench status stored for a missing measurement.
const fn status_of(missing: NoMeasurement) -> BenchStatus {
    match missing {
        NoMeasurement::NotSpecified => BenchStatus::NotSpecified,
        NoMeasurement::HostBusy => BenchStatus::HostBusy,
        NoMeasurement::Failed => BenchStatus::Failed,
        NoMeasurement::Incomplete => BenchStatus::Incomplete,
    }
}

/// The fitted class, when the bench measured one.
const fn bench_facts_observed(facts: &BenchFacts) -> Option<ClassId> {
    match facts {
        BenchFacts::Measured { observed, .. } => *observed,
        BenchFacts::None(_) => None,
    }
}

/// The fit's confidence, when the bench was fitted.
const fn bench_facts_confidence(facts: &BenchFacts) -> Option<Confidence> {
    match facts {
        BenchFacts::Measured { confidence, .. } => Some(*confidence),
        BenchFacts::None(_) => None,
    }
}

/// The baseline ratio, when the bench was fitted and a baseline was usable.
const fn bench_facts_ratio(facts: &BenchFacts) -> Option<f64> {
    match facts {
        BenchFacts::Measured {
            ratio_vs_baseline, ..
        } => *ratio_vs_baseline,
        BenchFacts::None(_) => None,
    }
}

/// The engine's run mode for a stored run mode: a Run is a quick check, a Submit is the full fit.
const fn engine_mode(mode: RunMode) -> EngineMode {
    match mode {
        RunMode::Run => EngineMode::Quick,
        RunMode::Submit => EngineMode::Full,
    }
}

/// Converts an engine growth class to the shared class type. Exhaustive, so a new class is a compile error here.
#[must_use]
pub const fn to_common_class(class: EngineClass) -> ClassId {
    match class {
        EngineClass::Constant => ClassId::Constant,
        EngineClass::Logarithmic => ClassId::Logarithmic,
        EngineClass::Linear => ClassId::Linear,
        EngineClass::Linearithmic => ClassId::Linearithmic,
        EngineClass::Quadratic => ClassId::Quadratic,
        EngineClass::Cubic => ClassId::Cubic,
        EngineClass::Exponential => ClassId::Exponential,
    }
}

/// Converts an engine confidence to the shared confidence type.
const fn to_common_confidence(confidence: EngineConfidence) -> Confidence {
    match confidence {
        EngineConfidence::Low => Confidence::Low,
        EngineConfidence::Medium => Confidence::Medium,
        EngineConfidence::High => Confidence::High,
    }
}

/// The snake-case reason code stored and sent for an engine fit reason. Exhaustive.
const fn reason_code(reason: EngineReason) -> &'static str {
    match reason {
        EngineReason::SizeRangeTooSmall => "size_range_too_small",
        EngineReason::NoModelFits => "no_model_fits",
        EngineReason::SeveralClassesPlausible => "several_classes_plausible",
        EngineReason::ViewsDisagree => "views_disagree",
        EngineReason::UnstableToOnePoint => "unstable_to_one_point",
        EngineReason::TooFewPointsForHigh => "too_few_points_for_high",
        EngineReason::RangeBelowHigh => "range_below_high",
        EngineReason::NoisyForHigh => "noisy_for_high",
        EngineReason::NoClearMargin => "no_clear_margin",
        EngineReason::ClassExcludedFromHigh => "class_excluded_from_high",
        EngineReason::RunCapped => "run_capped",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use counterspell_common::familiar::class::{Effect, Mood as MoodValue};

    /// Sizes that double from 64 to 2048.
    const POW2: [u64; 6] = [64, 128, 256, 512, 1024, 2048];

    /// Points for a linear curve, `1000 × n` nanoseconds, with 4 percent spread.
    fn linear_points() -> Vec<BenchPoint> {
        [64_000, 128_000, 256_000, 512_000, 1_024_000, 2_048_000]
            .iter()
            .zip(POW2)
            .map(|(&median_ns, n)| BenchPoint {
                n,
                median_ns,
                spread: Some(0.04),
            })
            .collect()
    }

    /// Points for a quadratic curve, `3n² + 500` nanoseconds, with 4 percent spread.
    fn quadratic_points() -> Vec<BenchPoint> {
        [12_788, 49_652, 197_108, 786_932, 3_146_228, 12_583_412]
            .iter()
            .zip(POW2)
            .map(|(&median_ns, n)| BenchPoint {
                n,
                median_ns,
                spread: Some(0.04),
            })
            .collect()
    }

    /// Points for a cubic curve, `0.5 n³` nanoseconds, with 4 percent spread.
    fn cubic_points() -> Vec<BenchPoint> {
        [
            131_072,
            1_048_576,
            8_388_608,
            67_108_864,
            536_870_912,
            4_294_967_296,
        ]
        .iter()
        .zip(POW2)
        .map(|(&median_ns, n)| BenchPoint {
            n,
            median_ns,
            spread: Some(0.04),
        })
        .collect()
    }

    /// Points for flat timings, 700 nanoseconds at every size.
    fn flat_points() -> Vec<BenchPoint> {
        POW2.iter()
            .map(|&n| BenchPoint {
                n,
                median_ns: 700,
                spread: Some(0.04),
            })
            .collect()
    }

    /// The baseline series for a linear curve, `1000 × n` nanoseconds.
    fn linear_baseline() -> Vec<(u64, u64)> {
        vec![
            (64, 64_000),
            (128, 128_000),
            (256, 256_000),
            (512, 512_000),
            (1024, 1_024_000),
            (2048, 2_048_000),
        ]
    }

    /// Learner points at three times the linear baseline.
    fn three_times_linear_points() -> Vec<BenchPoint> {
        [192_000, 384_000, 768_000, 1_536_000, 3_072_000, 6_144_000]
            .iter()
            .zip(POW2)
            .map(|(&median_ns, n)| BenchPoint {
                n,
                median_ns,
                spread: Some(0.04),
            })
            .collect()
    }

    fn input<'a>(
        verdict: Verdict,
        mode: RunMode,
        reference: ClassId,
        bench: BenchState<'a>,
        baseline: Option<&'a [(u64, u64)]>,
    ) -> AssessInput<'a> {
        AssessInput {
            verdict: Some(verdict),
            mode,
            reference,
            bench,
            baseline,
        }
    }

    /// Checks the one-value-two-uses rule: the event and the stored column read the same value.
    fn assert_event_and_columns_agree(assessment: &Assessment) {
        if let Some(familiar) = &assessment.familiar {
            assert_eq!(assessment.columns.familiar_mood, Some(familiar.mood));
            assert_eq!(
                assessment.columns.familiar_reason.as_deref(),
                Some(familiar.reason.as_str())
            );
        }
        if let Some(complexity) = &assessment.complexity {
            assert_eq!(assessment.columns.measured_complexity, complexity.class);
            assert_eq!(
                assessment.columns.complexity_confidence,
                Some(complexity.confidence)
            );
        }
    }

    #[test]
    fn a_run_that_did_not_complete_has_no_mood_and_stores_nothing() {
        let assessed = assess(&AssessInput {
            verdict: None,
            mode: RunMode::Submit,
            reference: ClassId::Linear,
            bench: BenchState::Points(&[]),
            baseline: None,
        });
        assert_eq!(assessed, Assessment::not_completed());
        assert_eq!(assessed.columns.familiar_mood, None);
    }

    #[test]
    fn a_wrong_answer_is_furious_with_a_crack_burst_and_no_measurement() {
        let assessed = assess(&input(
            Verdict::WrongAnswer,
            RunMode::Run,
            ClassId::Linear,
            BenchState::NotSpecified,
            None,
        ));
        let familiar = assessed
            .familiar
            .clone()
            .expect("a completed run has a familiar");
        assert_eq!(familiar.mood, MoodValue::FURIOUS);
        assert_eq!(familiar.reason, "wrong_answer");
        assert_eq!(familiar.effect, Effect::CrackBurst);
        assert!(assessed.complexity.is_none());
        assert!(assessed.columns.fit.is_none());
        assert_event_and_columns_agree(&assessed);
    }

    #[test]
    fn an_accepted_run_with_no_bench_spec_is_pleased_and_stores_a_not_fitted_summary() {
        let assessed = assess(&input(
            Verdict::Accepted,
            RunMode::Submit,
            ClassId::Linear,
            BenchState::NotSpecified,
            None,
        ));
        let familiar = assessed.familiar.clone().expect("familiar");
        assert_eq!(familiar.mood, MoodValue::PLEASED);
        assert_eq!(familiar.reason, "no_bench_spec");
        assert!(assessed.complexity.is_none());
        let fit = assessed
            .columns
            .fit
            .clone()
            .expect("a not-fitted summary is stored");
        assert_eq!(fit.bench_status, BenchStatus::NotSpecified);
        assert_eq!(fit.class, None);
        assert_event_and_columns_agree(&assessed);
    }

    #[test]
    fn a_discarded_bench_is_pleased_for_host_busy() {
        let assessed = assess(&input(
            Verdict::Accepted,
            RunMode::Submit,
            ClassId::Linear,
            BenchState::Discarded,
            None,
        ));
        let familiar = assessed.familiar.clone().expect("familiar");
        assert_eq!(familiar.mood, MoodValue::PLEASED);
        assert_eq!(familiar.reason, "host_busy");
        let fit = assessed.columns.fit.clone().expect("fit");
        assert_eq!(fit.bench_status, BenchStatus::HostBusy);
    }

    #[test]
    fn a_failed_or_incomplete_bench_gives_its_own_reason() {
        let failed = assess(&input(
            Verdict::Accepted,
            RunMode::Submit,
            ClassId::Linear,
            BenchState::Failed,
            None,
        ));
        assert_eq!(failed.familiar.expect("familiar").reason, "bench_failed");

        let incomplete = assess(&input(
            Verdict::Accepted,
            RunMode::Submit,
            ClassId::Linear,
            BenchState::Incomplete,
            None,
        ));
        let familiar = incomplete.familiar.expect("familiar");
        assert_eq!(familiar.reason, "bench_incomplete");
        assert_eq!(
            incomplete.columns.fit.clone().expect("fit").bench_status,
            BenchStatus::Incomplete
        );
    }

    #[test]
    fn points_the_fit_rejects_are_recorded_as_a_failed_bench() {
        let two_points = [
            BenchPoint {
                n: 64,
                median_ns: 100,
                spread: None,
            },
            BenchPoint {
                n: 128,
                median_ns: 200,
                spread: None,
            },
        ];
        let assessed = assess(&input(
            Verdict::Accepted,
            RunMode::Submit,
            ClassId::Linear,
            BenchState::Points(&two_points),
            None,
        ));
        assert!(assessed.complexity.is_none());
        assert_eq!(
            assessed.familiar.clone().expect("familiar").reason,
            "bench_failed"
        );
        assert_eq!(
            assessed.columns.fit.clone().expect("fit").bench_status,
            BenchStatus::Failed
        );
    }

    #[test]
    fn linear_timings_match_a_linear_reference_and_laugh_on_submit() {
        let points = linear_points();
        let baseline = linear_baseline();
        let assessed = assess(&input(
            Verdict::Accepted,
            RunMode::Submit,
            ClassId::Linear,
            BenchState::Points(&points),
            Some(&baseline),
        ));
        let familiar = assessed.familiar.clone().expect("familiar");
        assert_eq!(familiar.mood, MoodValue::LAUGHING);
        assert_eq!(familiar.reason, "matches_reference");
        assert_eq!(familiar.effect, Effect::LaughBurst);
        let complexity = assessed.complexity.clone().expect("complexity");
        assert_eq!(complexity.class, Some(ClassId::Linear));
        assert_eq!(assessed.columns.measured_complexity, Some(ClassId::Linear));
        assert_event_and_columns_agree(&assessed);
    }

    #[test]
    fn the_same_linear_run_on_run_gets_no_laugh_burst() {
        let points = linear_points();
        let baseline = linear_baseline();
        let assessed = assess(&input(
            Verdict::Accepted,
            RunMode::Run,
            ClassId::Linear,
            BenchState::Points(&points),
            Some(&baseline),
        ));
        let familiar = assessed.familiar.clone().expect("familiar");
        assert_eq!(familiar.mood, MoodValue::LAUGHING);
        assert_eq!(familiar.effect, Effect::None);
    }

    #[test]
    fn quadratic_timings_against_a_linear_reference_are_angry() {
        let points = quadratic_points();
        let assessed = assess(&input(
            Verdict::Accepted,
            RunMode::Submit,
            ClassId::Linear,
            BenchState::Points(&points),
            None,
        ));
        let familiar = assessed.familiar.clone().expect("familiar");
        assert_eq!(familiar.mood, MoodValue::ANGRY);
        assert_eq!(familiar.reason, "two_classes_worse");
        assert_eq!(
            assessed.columns.measured_complexity,
            Some(ClassId::Quadratic)
        );
        assert_event_and_columns_agree(&assessed);
    }

    #[test]
    fn cubic_timings_against_a_linear_reference_are_furious_with_no_burst() {
        let points = cubic_points();
        let assessed = assess(&input(
            Verdict::Accepted,
            RunMode::Submit,
            ClassId::Linear,
            BenchState::Points(&points),
            None,
        ));
        let familiar = assessed.familiar.clone().expect("familiar");
        assert_eq!(familiar.mood, MoodValue::FURIOUS);
        assert_eq!(familiar.reason, "worst_tier");
        assert_eq!(familiar.effect, Effect::None);
    }

    #[test]
    fn linear_timings_against_a_logarithmic_reference_are_uneasy() {
        let points = linear_points();
        let assessed = assess(&input(
            Verdict::Accepted,
            RunMode::Submit,
            ClassId::Logarithmic,
            BenchState::Points(&points),
            None,
        ));
        let familiar = assessed.familiar.clone().expect("familiar");
        assert_eq!(familiar.mood, MoodValue::UNEASY);
        assert_eq!(familiar.reason, "one_class_worse");
    }

    #[test]
    fn a_matching_class_with_no_baseline_is_pleased_not_laughing() {
        let points = linear_points();
        let assessed = assess(&input(
            Verdict::Accepted,
            RunMode::Submit,
            ClassId::Linear,
            BenchState::Points(&points),
            None,
        ));
        let familiar = assessed.familiar.clone().expect("familiar");
        assert_eq!(familiar.mood, MoodValue::PLEASED);
        assert_eq!(familiar.reason, "no_baseline");
    }

    #[test]
    fn a_constant_factor_over_two_times_the_baseline_is_pleased() {
        let points = three_times_linear_points();
        let baseline = linear_baseline();
        let assessed = assess(&input(
            Verdict::Accepted,
            RunMode::Submit,
            ClassId::Linear,
            BenchState::Points(&points),
            Some(&baseline),
        ));
        let familiar = assessed.familiar.clone().expect("familiar");
        assert_eq!(familiar.mood, MoodValue::PLEASED);
        assert_eq!(familiar.reason, "slower_constant_factor");
    }

    #[test]
    fn every_engine_class_maps_to_the_shared_class_with_the_same_id_and_rank() {
        for class in EngineClass::ALL {
            let shared = to_common_class(class);
            assert_eq!(shared.id(), class.id());
            assert_eq!(shared.rank(), class.rank());
        }
    }

    #[test]
    fn flat_timings_report_low_confidence_with_no_stored_class() {
        let points = flat_points();
        let assessed = assess(&input(
            Verdict::Accepted,
            RunMode::Submit,
            ClassId::Linear,
            BenchState::Points(&points),
            None,
        ));
        let familiar = assessed.familiar.clone().expect("familiar");
        assert_eq!(familiar.mood, MoodValue::PLEASED);
        assert_eq!(familiar.reason, "low_confidence");
        let complexity = assessed.complexity.clone().expect("complexity");
        assert_eq!(complexity.class, None);
        assert_eq!(complexity.confidence, Confidence::Low);
        assert_eq!(assessed.columns.measured_complexity, None);
        assert_eq!(
            assessed.columns.complexity_confidence,
            Some(Confidence::Low)
        );
        assert_event_and_columns_agree(&assessed);
    }
}
