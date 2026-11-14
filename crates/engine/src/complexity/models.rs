//! Growth models fitted to the timings: `t ≈ a + c·g(n)` with `a, c ≥ 0`.
//!
//! The fit works on timings divided by their maximum, so a constant factor on
//! every median changes nothing. Each model's quality is the root-mean-square
//! error on a log scale, `E = sqrt(mean((ln t − ln(a + c·g(n)))²))`. That is a
//! relative error, so every size counts the same whatever its timing.

use super::{size_as_f64, Class};

/// The residual given to a prediction that is not positive. It is large but finite.
pub(crate) const LARGE_RESIDUAL: f64 = 1.0e3;

/// The fitted parameters of one model and its residual.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct ModelFit {
    /// The fixed-cost offset, never negative. For `2^n` it is the log-space intercept.
    pub(crate) a: f64,
    /// The growth coefficient, never negative. Zero for `2^n`, which has no coefficient.
    pub(crate) c: f64,
    /// The root-mean-square log error. Finite for every finite input.
    pub(crate) residual: f64,
}

/// The raw basis value `g(n)` for a class. `2^n` is handled in log space by the caller.
fn basis(class: Class, n: u64) -> f64 {
    let x = size_as_f64(n);
    match class {
        Class::Constant => 1.0,
        Class::Logarithmic => x.ln(),
        Class::Linear | Class::Exponential => x,
        Class::Linearithmic => x * x.ln(),
        Class::Quadratic => x * x,
        Class::Cubic => x * x * x,
        // The exponential model is not a basis; it is fitted in log space.
    }
}

/// Fits one model to the points. `sizes`, `times` and `sigma` have the same length.
///
/// `times` must be positive, normalised to a maximum of 1. `sigma` is the
/// relative noise of each point.
pub(crate) fn fit_model(class: Class, sizes: &[u64], times: &[f64], sigma: &[f64]) -> ModelFit {
    if class == Class::Exponential {
        return fit_exponential(sizes, times);
    }
    let raw: Vec<f64> = sizes.iter().map(|&n| basis(class, n)).collect();
    let top = raw.iter().copied().fold(0.0_f64, f64::max);
    let g: Vec<f64> = raw.iter().map(|v| v / top).collect();
    let (a, c) = weighted_nonnegative_fit(&g, times, sigma);
    let residual = log_residual(times, |i| {
        let predicted = a + c * g[i];
        (predicted > 0.0 && predicted.is_finite()).then(|| predicted.ln())
    });
    ModelFit { a, c, residual }
}

/// The root-mean-square of `ln t − ln(prediction)` over the points.
///
/// `predicted_ln` gives the log of the prediction for point `i`, or `None` when
/// the prediction is not positive. Such a point gets [`LARGE_RESIDUAL`].
fn log_residual(times: &[f64], predicted_ln: impl Fn(usize) -> Option<f64>) -> f64 {
    if times.is_empty() {
        return LARGE_RESIDUAL;
    }
    let total: f64 = times
        .iter()
        .enumerate()
        .map(|(i, t)| match predicted_ln(i) {
            Some(ln_pred) => {
                let err = t.ln() - ln_pred;
                err * err
            }
            None => LARGE_RESIDUAL * LARGE_RESIDUAL,
        })
        .sum();
    (total / size_as_f64(times.len() as u64)).sqrt()
}

/// Weighted least squares for `t ≈ a + c·g` with `a ≥ 0` and `c ≥ 0`.
///
/// Weights are `1 / (σ·t)²`, which makes the error relative. If the unconstrained
/// solution has a negative coefficient, that coefficient is set to zero and the
/// other one is refitted.
fn weighted_nonnegative_fit(g: &[f64], times: &[f64], sigma: &[f64]) -> (f64, f64) {
    let weights: Vec<f64> = times
        .iter()
        .zip(sigma)
        .map(|(t, s)| 1.0 / ((s * t) * (s * t)))
        .collect();
    let s0: f64 = weights.iter().sum();
    let s1: f64 = weights.iter().zip(g).map(|(w, x)| w * x).sum();
    let s2: f64 = weights.iter().zip(g).map(|(w, x)| w * x * x).sum();
    let y0: f64 = weights.iter().zip(times).map(|(w, t)| w * t).sum();
    let y1: f64 = weights
        .iter()
        .zip(g.iter().zip(times))
        .map(|(w, (x, t))| w * x * t)
        .sum();
    let det = s0 * s2 - s1 * s1;
    let mean_a = y0 / s0;
    let (mut a, mut c) = if det > 1.0e-12 * s0 * s2 {
        ((y0 * s2 - y1 * s1) / det, (s0 * y1 - s1 * y0) / det)
    } else {
        // The basis is constant across the sizes, so it cannot separate from the offset.
        (mean_a, 0.0)
    };
    if a < 0.0 {
        a = 0.0;
        c = if s2 > 0.0 { y1 / s2 } else { 0.0 };
    }
    if c < 0.0 {
        c = 0.0;
        a = mean_a;
    }
    (a.max(0.0), c.max(0.0))
}

/// Fits `ln t = α + n·ln 2` in log space. There is no coefficient, so `c` is zero.
///
/// The intercept is the weighted mean of `ln t − n·ln 2`. Working in logs
/// means `2^n` never overflows, even for `n` near `u64::MAX`.
fn fit_exponential(sizes: &[u64], times: &[f64]) -> ModelFit {
    let ln2 = std::f64::consts::LN_2;
    let count = size_as_f64(sizes.len() as u64);
    let alpha = sizes
        .iter()
        .zip(times)
        .map(|(&n, t)| t.ln() - size_as_f64(n) * ln2)
        .sum::<f64>()
        / count;
    let residual = log_residual(times, |i| Some(alpha + size_as_f64(sizes[i]) * ln2));
    ModelFit {
        a: alpha,
        c: 0.0,
        residual,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Builds normalised timings `t = offset + coeff·g(n)` for the given class, scaled to max 1.
    fn series(class: Class, sizes: &[u64], offset: f64, coeff: f64) -> Vec<f64> {
        let raw: Vec<f64> = sizes
            .iter()
            .map(|&n| offset + coeff * basis(class, n))
            .collect();
        let top = raw.iter().copied().fold(0.0_f64, f64::max);
        raw.iter().map(|v| v / top).collect()
    }

    const SIZES: [u64; 6] = [64, 128, 256, 512, 1024, 2048];
    const NOISE: [f64; 6] = [0.05; 6];

    #[test]
    fn data_from_a_linear_model_with_offset_is_recovered_exactly() {
        let times = series(Class::Linear, &SIZES, 0.3, 1.0);
        let fit = fit_model(Class::Linear, &SIZES, &times, &NOISE);
        assert!(fit.residual < 1.0e-9, "residual {}", fit.residual);
        assert!(fit.a >= 0.0 && fit.c >= 0.0);
    }

    #[test]
    fn the_true_model_fits_better_than_the_others() {
        // Hand-chosen shapes: each series is exact for its own model only.
        let cases = [
            (Class::Linearithmic, Class::Linear),
            (Class::Quadratic, Class::Linearithmic),
        ];
        for (truth, other) in cases {
            let times = series(truth, &SIZES, 0.0, 1.0);
            let own = fit_model(truth, &SIZES, &times, &NOISE).residual;
            let wrong = fit_model(other, &SIZES, &times, &NOISE).residual;
            assert!(own < 1.0e-6, "{truth:?} residual {own}");
            assert!(wrong > 0.01, "{other:?} residual {wrong} is too small");
        }
    }

    #[test]
    fn flat_timings_collapse_every_model_to_a_constant() {
        let times = vec![1.0; SIZES.len()];
        for class in Class::ALL.into_iter().filter(|c| *c != Class::Exponential) {
            let fit = fit_model(class, &SIZES, &times, &NOISE);
            assert!(fit.c.abs() < 1.0e-9, "{class:?} c = {}", fit.c);
            assert!(fit.residual < 1.0e-9, "{class:?} residual {}", fit.residual);
        }
    }

    #[test]
    fn flat_timings_are_a_poor_fit_for_exponential_growth() {
        // The 2^n model has no free coefficient, so it cannot describe a curve that never rises.
        let times = vec![1.0; SIZES.len()];
        let fit = fit_model(Class::Exponential, &SIZES, &times, &NOISE);
        assert!(fit.residual > 1.0, "residual {}", fit.residual);
    }

    #[test]
    fn decreasing_timings_never_give_a_negative_coefficient() {
        let times = [1.0, 0.5, 0.25, 0.125, 0.0625, 0.03125];
        for class in Class::ALL {
            let fit = fit_model(class, &SIZES, &times, &NOISE);
            assert!(
                fit.a >= 0.0 || class == Class::Exponential,
                "{class:?} a = {}",
                fit.a
            );
            assert!(fit.c >= 0.0, "{class:?} c = {}", fit.c);
        }
    }

    #[test]
    fn sizes_up_to_u64_max_give_finite_residuals_and_no_overflow() {
        let sizes = [
            u64::MAX / 16,
            u64::MAX / 8,
            u64::MAX / 4,
            u64::MAX / 2,
            u64::MAX,
        ];
        let times = [0.1, 0.2, 0.4, 0.8, 1.0];
        for class in Class::ALL {
            let fit = fit_model(class, &sizes, &times, &NOISE);
            assert!(
                fit.residual.is_finite(),
                "{class:?} residual {}",
                fit.residual
            );
            assert!(fit.a.is_finite() && fit.c.is_finite(), "{class:?}");
        }
    }
}
