//! Noise: how much each timing can be trusted, as a relative error on a log scale.
//!
//! A point's noise comes from the spread of its repeats. When the spread is
//! unknown, [`DEFAULT_NOISE`](super::FitConfig) is used. The combined noise is the
//! root-mean-square over the points, so one very noisy point raises it.

use super::FitConfig;

/// The noise of one point. `spread` is `(max − min) / median` over the repeats.
///
/// The result is `spread_factor × spread`, held between `noise_floor` and
/// `noise_cap`. An unknown spread gives `default_noise`.
#[must_use]
pub(crate) fn point_noise(spread: Option<f64>, cfg: &FitConfig) -> f64 {
    match spread {
        Some(s) => (cfg.spread_factor * s).clamp(cfg.noise_floor, cfg.noise_cap),
        None => cfg.default_noise.clamp(cfg.noise_floor, cfg.noise_cap),
    }
}

/// The root-mean-square of the per-point noise values. Empty input gives zero.
#[must_use]
pub(crate) fn combined_noise(sigmas: &[f64]) -> f64 {
    if sigmas.is_empty() {
        return 0.0;
    }
    let mean_square = sigmas.iter().map(|s| s * s).sum::<f64>() / len_as_f64(sigmas.len());
    mean_square.sqrt()
}

/// The count as a float. The fit never has more than 32 points, so this is exact.
#[allow(clippy::cast_precision_loss)]
pub(super) fn len_as_f64(count: usize) -> f64 {
    count as f64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cfg() -> FitConfig {
        FitConfig::default()
    }

    #[test]
    fn a_known_spread_scales_into_noise() {
        // Spread 0.4 with factor 0.25 gives 0.1 exactly, inside the floor and cap.
        assert!((point_noise(Some(0.4), &cfg()) - 0.1).abs() < 1.0e-12);
    }

    #[test]
    fn a_tiny_spread_is_lifted_to_the_floor() {
        assert!((point_noise(Some(0.0), &cfg()) - 0.03).abs() < 1.0e-12);
    }

    #[test]
    fn a_huge_spread_is_capped() {
        assert!((point_noise(Some(10.0), &cfg()) - 1.0).abs() < 1.0e-12);
    }

    #[test]
    fn an_unknown_spread_uses_the_default_noise() {
        assert!((point_noise(None, &cfg()) - 0.10).abs() < 1.0e-12);
    }

    #[test]
    fn combined_noise_is_the_root_mean_square() {
        // Two points at 0.03 and 0.4 give sqrt((0.0009 + 0.16) / 2) = 0.283637.
        let combined = combined_noise(&[0.03, 0.4]);
        assert!((combined - 0.283_637).abs() < 1.0e-5, "got {combined}");
    }

    #[test]
    fn combined_noise_of_nothing_is_zero() {
        assert_eq!(combined_noise(&[]), 0.0);
    }
}
