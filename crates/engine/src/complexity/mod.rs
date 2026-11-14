//! The complexity fit: decides which growth class a timing curve supports.
//!
//! The judge measures one median time per input size. This module turns those
//! points into a [`FitReport`] that says either the class (only when the data
//! supports it) or "unsure", with the classes it could not rule out. It is
//! pure and deterministic. It reads no clock, file or network.
//!
//! The engine has its own copy of the seven class names, so it does not depend
//! on `counterspell-common`. The judge converts between the two in one place.

use std::fmt;

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
    /// Temporary: the fit algorithm is not implemented yet (removed in T09).
    NotImplemented,
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
            Self::NotImplemented => f.write_str("the fit is not implemented yet"),
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
            se_k: 2.5,
            min_range_medium: 8.0,
            min_range_high: 32.0,
            min_points_high: 5,
            high_noise_max: 0.12,
            high_margin_k: 2.0,
        }
    }
}

/// Fits a timing curve. Temporary: the full algorithm lands in task T09.
///
/// # Errors
///
/// Returns [`FitError`] for input that cannot be fitted.
pub fn fit(points: &[Point], _mode: Mode, _cfg: &FitConfig) -> Result<(), FitError> {
    validate(points)?;
    Err(FitError::NotImplemented)
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
}
