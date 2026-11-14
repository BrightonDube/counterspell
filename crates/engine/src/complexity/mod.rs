//! The complexity fit: decides which growth class a timing curve supports.
//!
//! The judge measures one median time per input size. This module turns those
//! points into a [`FitReport`] that says either the class (only when the data
//! supports it) or "unsure", with the classes it could not rule out. It is
//! pure and deterministic. It reads no clock, file or network.
//!
//! The engine has its own copy of the seven class names, so it does not depend
//! on `counterspell-common`. The judge converts between the two in one place.

use decide::{decide, Inputs};
use noise::point_noise;
use std::fmt;

pub use bench_input::{bench_seed, sized_input, BenchInputError};
mod bench_input;
mod decide;
mod models;
mod noise;
mod views;

/// Converts an input size to `f64`. Sizes are at most `u64::MAX`, so the rounding only
/// affects the last few bits, which is fine on a log scale.
#[allow(clippy::cast_precision_loss)]
pub(crate) fn size_as_f64(n: u64) -> f64 {
    n as f64
}

/// The fit version. Changing any constant or the algorithm must bump it.
pub const FIT_VERSION: &str = "fit-1";

/// The fewest points the fit accepts.
pub const MIN_POINTS: usize = 3;

/// The most points the fit accepts.
pub const MAX_POINTS: usize = 32;

/// One measured size. `spread` is `(max − min) / median` over the repeats, when known.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Point {
    /// The input size. Must be at least 2.
    pub n: u64,
    /// The median time in nanoseconds. Must be above zero.
    pub median_ns: u64,
    /// The relative spread of the repeats. Must be finite and not negative when given.
    pub spread: Option<f64>,
}

/// Whether the timings come from a sample-only Run or a hidden-test Submit.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    /// Fewer sizes and a quick check. Confidence is capped at `Medium`.
    Quick,
    /// The full bench. `High` is allowed.
    Full,
}

/// One growth class, in the same seven names as the requirements.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Class {
    /// Constant time, `1`.
    Constant,
    /// Logarithmic time, `log n`.
    Logarithmic,
    /// Linear time, `n`.
    Linear,
    /// Linearithmic time, `n log n`.
    Linearithmic,
    /// Quadratic time, `n^2`.
    Quadratic,
    /// Cubic time, `n^3`.
    Cubic,
    /// Exponential time, `2^n`.
    Exponential,
}

impl Class {
    /// Every class, from the lowest rank to the highest.
    pub const ALL: [Self; 7] = [
        Self::Constant,
        Self::Logarithmic,
        Self::Linear,
        Self::Linearithmic,
        Self::Quadratic,
        Self::Cubic,
        Self::Exponential,
    ];

    /// The position from 0 (constant) to 6 (exponential). The gap is a rank difference.
    #[must_use]
    pub const fn rank(self) -> u8 {
        match self {
            Self::Constant => 0,
            Self::Logarithmic => 1,
            Self::Linear => 2,
            Self::Linearithmic => 3,
            Self::Quadratic => 4,
            Self::Cubic => 5,
            Self::Exponential => 6,
        }
    }

    /// The id used in storage and on the wire, for example `n^2`.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Constant => "1",
            Self::Logarithmic => "log n",
            Self::Linear => "n",
            Self::Linearithmic => "n log n",
            Self::Quadratic => "n^2",
            Self::Cubic => "n^3",
            Self::Exponential => "2^n",
        }
    }
}

impl fmt::Display for Class {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id())
    }
}

impl serde::Serialize for Class {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.id())
    }
}

/// How sure the fit is. `Low` never carries a class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    /// The data does not single out one class.
    Low,
    /// One class fits and the checks agree.
    Medium,
    /// Strong evidence on every check.
    High,
}

/// A reason the fit is not more certain, or the mode cap that applied.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Reason {
    /// The largest size is less than 8 times the smallest.
    SizeRangeTooSmall,
    /// No growth model fits the timings within the noise.
    NoModelFits,
    /// More than one class could explain the timings.
    SeveralClassesPlausible,
    /// The three views (slope, ratios, best model) do not agree.
    ViewsDisagree,
    /// Dropping one point changes the best class.
    UnstableToOnePoint,
    /// `High` needs at least five sizes.
    TooFewPointsForHigh,
    /// `High` needs a size range of at least 32 times.
    RangeBelowHigh,
    /// `High` needs noise of at most 12 percent.
    NoisyForHigh,
    /// `High` needs a clear margin over the runner-up class.
    NoClearMargin,
    /// `High` is not given for the classes `1`, `log n` and `2^n`.
    ClassExcludedFromHigh,
    /// A Run is capped at `Medium`.
    RunCapped,
}

/// A reason the input cannot be fitted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FitError {
    /// Fewer than 3 or more than 32 points.
    PointCount {
        /// How many points were given.
        count: usize,
    },
    /// A size is below 2.
    SizeTooSmall,
    /// The sizes are not strictly increasing.
    SizesNotIncreasing,
    /// A median time is zero.
    ZeroMedian,
    /// A spread is negative or not finite.
    BadSpread,
}

impl fmt::Display for FitError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PointCount { count } => write!(
                f,
                "a fit needs {MIN_POINTS} to {MAX_POINTS} points, got {count}"
            ),
            Self::SizeTooSmall => f.write_str("every size must be at least 2"),
            Self::SizesNotIncreasing => f.write_str("sizes must be strictly increasing"),
            Self::ZeroMedian => f.write_str("every median time must be above zero"),
            Self::BadSpread => f.write_str("a spread must be finite and not negative"),
        }
    }
}

impl std::error::Error for FitError {}

/// The constants of the fit. Calibration may only move them toward less confidence,
/// and any change bumps [`FIT_VERSION`]. The values are estimates ([E]) until task T12 freezes them.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FitConfig {
    /// Lowest relative noise a point may have.
    pub noise_floor: f64,
    /// Highest relative noise a point may have.
    pub noise_cap: f64,
    /// Noise is this multiple of the repeat spread.
    pub spread_factor: f64,
    /// Noise used when the spread is unknown.
    pub default_noise: f64,
    /// Smallest plausibility slack between models.
    pub delta_floor: f64,
    /// Plausibility slack as a multiple of the combined noise.
    pub delta_k: f64,
    /// Smallest goodness-of-fit ceiling.
    pub fit_ok_floor: f64,
    /// Goodness-of-fit ceiling as a multiple of the combined noise.
    pub fit_ok_k: f64,
    /// Coarse tolerance on the log-log slope.
    pub slope_tol: f64,
    /// Coarse tolerance on the median local exponent.
    pub exp_tol: f64,
    /// Standard-error multiple used by the agreement checks.
    pub se_k: f64,
    /// Smallest size range (largest over smallest) for `Medium`.
    pub min_range_medium: f64,
    /// Smallest size range for `High`.
    pub min_range_high: f64,
    /// Fewest points for a stable `High` result.
    pub min_points_high: usize,
    /// Highest combined noise for `High`.
    pub high_noise_max: f64,
    /// Runner-up margin for `High`, as a multiple of the plausibility slack.
    pub high_margin_k: f64,
}

impl Default for FitConfig {
    fn default() -> Self {
        Self {
            noise_floor: 0.03,
            noise_cap: 1.0,
            spread_factor: 0.25,
            default_noise: 0.10,
            delta_floor: 0.025,
            delta_k: 0.75,
            fit_ok_floor: 0.05,
            fit_ok_k: 1.8,
            slope_tol: 0.30,
            exp_tol: 0.35,
            se_k: 1.0,
            min_range_medium: 8.0,
            min_range_high: 32.0,
            min_points_high: 5,
            high_noise_max: 0.12,
            high_margin_k: 2.0,
        }
    }
}

/// Checks the input before any arithmetic. Nothing here panics.
pub(crate) fn validate(points: &[Point]) -> Result<(), FitError> {
    if !(MIN_POINTS..=MAX_POINTS).contains(&points.len()) {
        return Err(FitError::PointCount {
            count: points.len(),
        });
    }
    for point in points {
        if point.n < 2 {
            return Err(FitError::SizeTooSmall);
        }
        if point.median_ns == 0 {
            return Err(FitError::ZeroMedian);
        }
        if let Some(spread) = point.spread {
            if !spread.is_finite() || spread < 0.0 {
                return Err(FitError::BadSpread);
            }
        }
    }
    if points.windows(2).any(|pair| pair[1].n <= pair[0].n) {
        return Err(FitError::SizesNotIncreasing);
    }
    Ok(())
}

/// The result of one fit: the class (when supported), how sure it is, and the evidence.
#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct FitReport {
    /// The fit version that produced this report.
    pub fit_version: &'static str,
    /// The reported class. Present only at `Medium` or `High`.
    pub class: Option<Class>,
    /// How sure the fit is.
    pub confidence: Confidence,
    /// The lowest-residual class. Informational when the confidence is `Low`.
    pub leaning: Class,
    /// Every class the noise cannot rule out, in rank order. Never empty.
    pub plausible: Vec<Class>,
    /// The log-log slope over the largest points.
    pub slope: f64,
    /// The standard error of the slope.
    pub slope_se: f64,
    /// The local exponent between each consecutive pair of points.
    pub exponents: Vec<f64>,
    /// The residual of every model, in rank order.
    pub residuals: Vec<(Class, f64)>,
    /// The combined relative noise.
    pub noise: f64,
    /// The largest size over the smallest.
    pub range_ratio: f64,
    /// How many points were fitted.
    pub points: usize,
    /// Why the confidence is not higher, or the mode cap that applied.
    pub reasons: Vec<Reason>,
}

/// How a fitted class compares with the reference class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Comparison {
    /// The rank difference, observed minus reference. Present only when a class was reported.
    /// A negative gap means the solution scales better than the reference.
    pub gap: Option<i8>,
    /// Whether the reference class is in the plausible set, even when the fit is unsure.
    pub reference_plausible: bool,
}

/// Fits a timing curve and says which growth class the data supports.
///
/// The points must follow the validation rules: 3 to 32 points, sizes of at least 2 in strictly
/// increasing order, and positive medians. Scaling every median by a positive constant does
/// not change the result.
///
/// # Errors
///
/// Returns [`FitError`] for input that breaks the validation rules. It never panics.
pub fn fit(points: &[Point], mode: Mode, cfg: &FitConfig) -> Result<FitReport, FitError> {
    validate(points)?;
    // Divide by the largest median, so the timings lie in (0, 1] and a constant factor cancels.
    let largest = points
        .iter()
        .map(|p| size_as_f64(p.median_ns))
        .fold(0.0_f64, f64::max);
    let sizes: Vec<u64> = points.iter().map(|p| p.n).collect();
    let times: Vec<f64> = points
        .iter()
        .map(|p| size_as_f64(p.median_ns) / largest)
        .collect();
    let sigmas: Vec<f64> = points.iter().map(|p| point_noise(p.spread, cfg)).collect();
    let outcome = decide(
        &Inputs {
            sizes: &sizes,
            times: &times,
            sigmas: &sigmas,
        },
        mode,
        cfg,
    );
    Ok(FitReport {
        fit_version: FIT_VERSION,
        class: outcome.class,
        confidence: outcome.confidence,
        leaning: outcome.leaning,
        plausible: outcome.plausible,
        slope: outcome.slope.b,
        slope_se: outcome.slope.se,
        exponents: outcome.exponents,
        residuals: outcome.residuals,
        noise: outcome.sigma_bar,
        range_ratio: outcome.range_ratio,
        points: points.len(),
        reasons: outcome.reasons,
    })
}

/// Compares a fitted report with the reference class the author declared.
#[must_use]
pub fn compare(report: &FitReport, reference: Class) -> Comparison {
    let gap = report.class.map(|class| {
        // Ranks are 0 to 6, so the difference is always between -6 and 6 and fits in i8.
        i8::try_from(i16::from(class.rank()) - i16::from(reference.rank())).unwrap_or(0)
    });
    Comparison {
        gap,
        reference_plausible: report.plausible.contains(&reference),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn point(n: u64, median_ns: u64, spread: Option<f64>) -> Point {
        Point {
            n,
            median_ns,
            spread,
        }
    }

    fn good_points(count: u64) -> Vec<Point> {
        (1..=count).map(|i| point(i * 10, 100 * i, None)).collect()
    }

    #[test]
    fn three_to_thirty_two_points_pass_the_count_check() {
        assert_eq!(validate(&good_points(3)), Ok(()));
        assert_eq!(validate(&good_points(32)), Ok(()));
    }

    #[test]
    fn two_points_and_thirty_three_points_are_rejected_by_count() {
        assert_eq!(
            validate(&good_points(2)),
            Err(FitError::PointCount { count: 2 })
        );
        assert_eq!(
            validate(&good_points(33)),
            Err(FitError::PointCount { count: 33 })
        );
    }

    #[test]
    fn a_size_below_two_is_rejected() {
        let points = vec![point(1, 10, None), point(4, 20, None), point(8, 30, None)];
        assert_eq!(validate(&points), Err(FitError::SizeTooSmall));
    }

    #[test]
    fn sizes_that_are_not_strictly_increasing_are_rejected() {
        let equal = vec![point(4, 10, None), point(4, 20, None), point(8, 30, None)];
        assert_eq!(validate(&equal), Err(FitError::SizesNotIncreasing));
        let falling = vec![point(8, 10, None), point(4, 20, None), point(16, 30, None)];
        assert_eq!(validate(&falling), Err(FitError::SizesNotIncreasing));
    }

    #[test]
    fn a_zero_median_is_rejected() {
        let points = vec![point(4, 10, None), point(8, 0, None), point(16, 30, None)];
        assert_eq!(validate(&points), Err(FitError::ZeroMedian));
    }

    #[test]
    fn negative_and_non_finite_spreads_are_rejected() {
        for spread in [-0.1, f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            let mut points = good_points(3);
            points[1].spread = Some(spread);
            assert_eq!(
                validate(&points),
                Err(FitError::BadSpread),
                "spread {spread}"
            );
        }
    }

    #[test]
    fn a_zero_spread_and_an_unknown_spread_are_both_accepted() {
        let mut points = good_points(3);
        points[0].spread = Some(0.0);
        assert_eq!(validate(&points), Ok(()));
    }

    #[test]
    fn class_ranks_and_ids_follow_the_requirements() {
        let ranks: Vec<u8> = Class::ALL.iter().map(|c| c.rank()).collect();
        assert_eq!(ranks, vec![0, 1, 2, 3, 4, 5, 6]);
        let ids: Vec<&str> = Class::ALL.iter().map(|c| c.id()).collect();
        assert_eq!(ids, vec!["1", "log n", "n", "n log n", "n^2", "n^3", "2^n"]);
    }

    /// Points with median `median(n)` nanoseconds at each size.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    fn curve(sizes: &[u64], median: impl Fn(f64) -> f64, spread: Option<f64>) -> Vec<Point> {
        sizes
            .iter()
            .map(|&n| Point {
                n,
                median_ns: median(n as f64).round() as u64,
                spread,
            })
            .collect()
    }

    const POW2: [u64; 6] = [64, 128, 256, 512, 1024, 2048];

    #[test]
    fn clean_linear_data_is_called_linear_with_at_least_medium_confidence() {
        let points = curve(&POW2, |n| 1000.0 * n, Some(0.04));
        let report = fit(&points, Mode::Full, &FitConfig::default()).expect("valid input");
        assert_eq!(report.class, Some(Class::Linear));
        assert!(
            report.confidence >= Confidence::Medium,
            "{:?}",
            report.reasons
        );
    }

    #[test]
    fn clean_quadratic_data_is_called_quadratic() {
        let points = curve(&POW2, |n| 3.0 * n * n + 500.0, Some(0.04));
        let report = fit(&points, Mode::Full, &FitConfig::default()).expect("valid input");
        assert_eq!(report.class, Some(Class::Quadratic));
        assert!(
            report.confidence >= Confidence::Medium,
            "{:?}",
            report.reasons
        );
    }

    #[test]
    fn clean_cubic_data_is_called_cubic() {
        let points = curve(&POW2, |n| 0.5 * n * n * n, Some(0.04));
        let report = fit(&points, Mode::Full, &FitConfig::default()).expect("valid input");
        assert_eq!(report.class, Some(Class::Cubic));
        assert!(
            report.confidence >= Confidence::Medium,
            "{:?}",
            report.reasons
        );
    }

    #[test]
    fn flat_data_is_unsure_and_lists_constant_among_the_plausible_classes() {
        let points = curve(&POW2, |_| 700.0, Some(0.04));
        let report = fit(&points, Mode::Full, &FitConfig::default()).expect("valid input");
        assert_eq!(report.class, None);
        assert_eq!(report.confidence, Confidence::Low);
        assert!(report.plausible.contains(&Class::Constant));
        assert!(report.reasons.contains(&Reason::SeveralClassesPlausible));
    }

    #[test]
    fn a_single_spiked_point_never_gives_high_confidence() {
        let mut points = curve(&POW2, |n| 1000.0 * n, None);
        points[3].median_ns *= 3;
        let report = fit(&points, Mode::Full, &FitConfig::default()).expect("valid input");
        assert_ne!(report.confidence, Confidence::High);
        if let Some(class) = report.class {
            assert_eq!(class, Class::Linear);
        }
    }

    #[test]
    fn a_range_of_two_times_is_low_with_the_size_range_reason() {
        let sizes = [64_u64, 80, 96, 112, 128];
        let points = curve(&sizes, |n| 1000.0 * n, Some(0.04));
        let report = fit(&points, Mode::Full, &FitConfig::default()).expect("valid input");
        assert_eq!(report.confidence, Confidence::Low);
        assert!(report.reasons.contains(&Reason::SizeRangeTooSmall));
    }

    #[test]
    fn a_class_outside_the_model_set_is_unsure() {
        let points = curve(&POW2, |n| 1000.0 * n.powf(1.5), Some(0.04));
        let report = fit(&points, Mode::Full, &FitConfig::default()).expect("valid input");
        assert_eq!(report.confidence, Confidence::Low);
        assert_eq!(report.class, None);
    }

    #[test]
    fn a_quick_run_with_three_points_is_at_most_medium_and_says_why() {
        let sizes = [64_u64, 640, 6400];
        let points = curve(&sizes, |n| 1000.0 * n, Some(0.04));
        let report = fit(&points, Mode::Quick, &FitConfig::default()).expect("valid input");
        assert_eq!(report.class, Some(Class::Linear));
        assert_eq!(report.confidence, Confidence::Medium);
        assert!(report.reasons.contains(&Reason::TooFewPointsForHigh));
    }

    #[test]
    fn a_quick_run_never_reports_high_even_when_every_other_check_passes() {
        let points = curve(&POW2, |n| 1000.0 * n, Some(0.04));
        let full = fit(&points, Mode::Full, &FitConfig::default()).expect("valid input");
        let quick = fit(&points, Mode::Quick, &FitConfig::default()).expect("valid input");
        assert_ne!(quick.confidence, Confidence::High);
        if full.confidence == Confidence::High {
            assert!(quick.reasons.contains(&Reason::RunCapped));
        }
    }

    #[test]
    fn a_class_is_reported_only_at_medium_or_high() {
        let cases = [
            curve(&POW2, |_| 700.0, Some(0.04)),
            curve(&POW2, |n| 1000.0 * n.powf(1.5), Some(0.04)),
        ];
        for points in cases {
            let report = fit(&points, Mode::Full, &FitConfig::default()).expect("valid input");
            assert_eq!(report.class.is_some(), report.confidence != Confidence::Low);
            assert!(report.plausible.contains(&report.leaning));
        }
    }

    #[test]
    fn compare_gives_the_rank_gap_only_when_a_class_was_reported() {
        let points = curve(&POW2, |n| 1000.0 * n * n, Some(0.04));
        let report = fit(&points, Mode::Full, &FitConfig::default()).expect("valid input");
        let vs_linear = compare(&report, Class::Linear);
        assert_eq!(vs_linear.gap, Some(2));
        let flat = curve(&POW2, |_| 700.0, Some(0.04));
        let unsure = fit(&flat, Mode::Full, &FitConfig::default()).expect("valid input");
        assert_eq!(compare(&unsure, Class::Linear).gap, None);
    }
}
