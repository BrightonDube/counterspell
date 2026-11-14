//! The slope view (a) and the ratio view (b) of the fit.
//!
//! Both use the largest sizes, where a fixed per-call cost matters least. The
//! top segment has `max(3, ceil(k/2))` points.

use super::noise::len_as_f64;
use super::size_as_f64;

/// The slope of `ln t` against `ln n` over the top segment, with its standard error.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct Slope {
    /// The fitted exponent.
    pub(crate) b: f64,
    /// Its standard error, never below the noise-implied error.
    pub(crate) se: f64,
}

/// How many of the largest points form the top segment: `max(3, ceil(k/2))`.
#[must_use]
pub(crate) fn top_count(k: usize) -> usize {
    k.div_ceil(2).max(3)
}

/// The log-log coordinates `(ln n, ln t)` of every point.
fn log_coordinates(sizes: &[u64], times: &[f64]) -> (Vec<f64>, Vec<f64>) {
    let xs = sizes.iter().map(|&n| size_as_f64(n).ln()).collect();
    let ys = times.iter().map(|t| t.ln()).collect();
    (xs, ys)
}

/// The least-squares slope over the top segment. The standard error is the larger of the
/// regression error and `sigma_bar / sqrt(Sxx)`, so exact data still carries honest uncertainty.
pub(crate) fn slope_view(sizes: &[u64], times: &[f64], sigma_bar: f64) -> Slope {
    let (xs, ys) = log_coordinates(sizes, times);
    let count = top_count(sizes.len());
    let start = xs.len() - count;
    let (xs, ys) = (&xs[start..], &ys[start..]);
    let mean_x = xs.iter().sum::<f64>() / len_as_f64(count);
    let mean_y = ys.iter().sum::<f64>() / len_as_f64(count);
    let sxx: f64 = xs.iter().map(|x| (x - mean_x).powi(2)).sum();
    let sxy: f64 = xs
        .iter()
        .zip(ys)
        .map(|(x, y)| (x - mean_x) * (y - mean_y))
        .sum();
    let b = sxy / sxx;
    let residual_ss: f64 = xs
        .iter()
        .zip(ys)
        .map(|(x, y)| (y - mean_y - b * (x - mean_x)).powi(2))
        .sum();
    // `count` is at least 3, so the degrees of freedom `count − 2` are at least 1.
    let se_regression = (residual_ss / len_as_f64(count - 2)).sqrt() / sxx.sqrt();
    let se_noise = sigma_bar / sxx.sqrt();
    Slope {
        b,
        se: se_regression.max(se_noise),
    }
}

/// The least-squares slope of `ys` against `xs`. The slope of `ln v` against `ln n` for a model `v`.
pub(crate) fn ols_slope(xs: &[f64], ys: &[f64]) -> f64 {
    let count = len_as_f64(xs.len());
    let mean_x = xs.iter().sum::<f64>() / count;
    let mean_y = ys.iter().sum::<f64>() / count;
    let sxx: f64 = xs.iter().map(|x| (x - mean_x).powi(2)).sum();
    let sxy: f64 = xs
        .iter()
        .zip(ys)
        .map(|(x, y)| (x - mean_x) * (y - mean_y))
        .sum();
    sxy / sxx
}

/// The slope of a model's log values against `ln n`, over the top segment.
pub(crate) fn top_slope(sizes: &[u64], model_ln: &[f64]) -> f64 {
    let count = top_count(sizes.len());
    let start = sizes.len() - count;
    let xs: Vec<f64> = sizes[start..]
        .iter()
        .map(|&n| size_as_f64(n).ln())
        .collect();
    ols_slope(&xs, &model_ln[start..])
}

/// The local exponent between each pair of consecutive points, over every point.
pub(crate) fn local_exponents(sizes: &[u64], times: &[f64]) -> Vec<f64> {
    let (xs, ys) = log_coordinates(sizes, times);
    xs.windows(2)
        .zip(ys.windows(2))
        .map(|(x, y)| (y[1] - y[0]) / (x[1] - x[0]))
        .collect()
}

/// The median local exponent over the top segment. The top segment has one fewer pair than points.
pub(crate) fn median_top_exponent(sizes: &[u64], times: &[f64]) -> f64 {
    let count = top_count(sizes.len());
    let start = sizes.len() - count;
    let mut exponents = local_exponents(&sizes[start..], &times[start..]);
    exponents.sort_by(f64::total_cmp);
    let mid = exponents.len() / 2;
    if exponents.len() % 2 == 1 {
        exponents[mid]
    } else {
        (exponents[mid - 1] + exponents[mid]) / 2.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Timings `n^1.5` for the sizes, which is an exact power law.
    fn power_law(sizes: &[u64], exponent: f64) -> Vec<f64> {
        sizes
            .iter()
            .map(|&n| size_as_f64(n).powf(exponent))
            .collect()
    }

    const SIX_SIZES: [u64; 6] = [64, 128, 256, 512, 1024, 2048];

    #[test]
    fn the_top_segment_has_three_points_or_half_of_them_rounded_up() {
        assert_eq!(top_count(3), 3);
        assert_eq!(top_count(5), 3);
        assert_eq!(top_count(6), 3);
        assert_eq!(top_count(7), 4);
        assert_eq!(top_count(8), 4);
    }

    #[test]
    fn an_exact_power_law_gives_its_exponent_as_the_slope() {
        let times = power_law(&SIX_SIZES, 1.5);
        let slope = slope_view(&SIX_SIZES, &times, 0.1);
        assert!((slope.b - 1.5).abs() < 1.0e-9, "slope {}", slope.b);
    }

    #[test]
    fn every_local_exponent_of_a_power_law_is_its_exponent() {
        let times = power_law(&SIX_SIZES, 1.5);
        let exponents = local_exponents(&SIX_SIZES, &times);
        assert_eq!(exponents.len(), 5);
        for e in exponents {
            assert!((e - 1.5).abs() < 1.0e-9, "exponent {e}");
        }
        assert!((median_top_exponent(&SIX_SIZES, &times) - 1.5).abs() < 1.0e-9);
    }

    #[test]
    fn with_exact_data_the_standard_error_comes_from_the_noise() {
        // Three top points at n = 512, 1024, 2048 have Sxx = 2·(ln 2)² ≈ 0.960906.
        // The noise term is 0.1 / sqrt(0.960906) ≈ 0.102021, and it beats the zero regression error.
        let times = power_law(&SIX_SIZES, 1.5);
        let slope = slope_view(&SIX_SIZES, &times, 0.1);
        assert!((slope.se - 0.102_021).abs() < 1.0e-4, "se {}", slope.se);
    }

    #[test]
    fn with_three_points_the_noise_still_sets_the_standard_error() {
        // n = 2, 4, 8: x = ln 2, 2 ln 2, 3 ln 2, Sxx = 2·(ln 2)², so the same 0.102021 holds.
        let sizes = [2_u64, 4, 8];
        let times = power_law(&sizes, 1.0);
        let slope = slope_view(&sizes, &times, 0.1);
        assert!((slope.b - 1.0).abs() < 1.0e-9);
        assert!((slope.se - 0.102_021).abs() < 1.0e-4, "se {}", slope.se);
    }

    #[test]
    fn a_noisy_point_widens_the_regression_error() {
        let sizes = [64_u64, 128, 256, 512, 1024, 2048];
        let mut times = power_law(&sizes, 1.5);
        times[4] *= 1.3;
        let slope = slope_view(&sizes, &times, 0.0);
        assert!(slope.se > 0.0, "a bent point must give a non-zero error");
    }
}
