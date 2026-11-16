//! The decision: which classes are plausible, whether the views agree, and how sure the fit is.
//!
//! This is the second half of the fit. [`decide`] takes the normalised points and
//! returns the confidence, the reported class (only at `Medium` or `High`) and the
//! reasons. Every route to `High` needs several independent conditions, so a wrong
//! class with high confidence needs the data to mimic another class on every view.

use super::models::{fit_model, log_basis, ModelFit};
use super::noise::combined_noise;
use super::views::{local_exponents, median_top_exponent, slope_view, top_slope, Slope};
use super::{size_as_f64, Class, Confidence, FitConfig, Mode, Reason};

/// The normalised inputs of the decision.
pub(crate) struct Inputs<'a> {
    /// Input sizes, strictly increasing.
    pub(crate) sizes: &'a [u64],
    /// Median times divided by the largest median, so the largest is 1.
    pub(crate) times: &'a [f64],
    /// Relative noise of each point.
    pub(crate) sigmas: &'a [f64],
}

/// The outcome of the decision, ready to become a [`super::FitReport`].
pub(crate) struct Outcome {
    /// How sure the fit is.
    pub(crate) confidence: Confidence,
    /// The best class, only at `Medium` or `High`.
    pub(crate) class: Option<Class>,
    /// The lowest-residual class.
    pub(crate) leaning: Class,
    /// Every class the noise cannot rule out, in rank order.
    pub(crate) plausible: Vec<Class>,
    /// The log-log slope and its standard error.
    pub(crate) slope: Slope,
    /// The local exponent of every consecutive pair.
    pub(crate) exponents: Vec<f64>,
    /// The residual of every model, in rank order.
    pub(crate) residuals: Vec<(Class, f64)>,
    /// The largest size over the smallest.
    pub(crate) range_ratio: f64,
    /// The combined noise.
    pub(crate) sigma_bar: f64,
    /// Why the confidence is not higher.
    pub(crate) reasons: Vec<Reason>,
}

/// Fits all seven models to the points, in rank order.
pub(crate) fn fit_all(sizes: &[u64], times: &[f64], sigmas: &[f64]) -> Vec<(Class, ModelFit)> {
    Class::ALL
        .into_iter()
        .map(|class| (class, fit_model(class, sizes, times, sigmas)))
        .collect()
}

/// The class with the smallest residual. A tie goes to the lower rank, because the scan keeps the first.
pub(crate) fn best_of(fits: &[(Class, ModelFit)]) -> (Class, f64) {
    let mut best = (fits[0].0, fits[0].1.residual);
    for (class, fit) in &fits[1..] {
        if fit.residual < best.1 {
            best = (*class, fit.residual);
        }
    }
    best
}

/// Makes the decision for the given points.
pub(crate) fn decide(inputs: &Inputs<'_>, mode: Mode, cfg: &FitConfig) -> Outcome {
    let sizes = inputs.sizes;
    let fits = fit_all(sizes, inputs.times, inputs.sigmas);
    let (best, e_best) = best_of(&fits);
    let sigma_bar = combined_noise(inputs.sigmas);
    let delta = cfg.delta_floor.max(cfg.delta_k * sigma_bar);

    let plausible: Vec<Class> = fits
        .iter()
        .filter(|(_, fit)| fit.residual <= e_best + delta)
        .map(|(class, _)| *class)
        .collect();
    let runner_up = fits
        .iter()
        .filter(|(class, _)| *class != best)
        .map(|(_, fit)| fit.residual)
        .fold(f64::INFINITY, f64::min);

    let slope = slope_view(sizes, inputs.times, sigma_bar);
    let exponents = local_exponents(sizes, inputs.times);
    let agrees = views_agree(best, sizes, inputs.times, &exponents, &slope, cfg);
    let stable = sizes.len() < cfg.min_points_high || is_stable(inputs, best);

    let range_ratio = size_as_f64(sizes[sizes.len() - 1]) / size_as_f64(sizes[0]);
    let (confidence, reasons) = grade(
        &Grading {
            points: sizes.len(),
            range_ratio,
            sigma_bar,
            e_best,
            best,
            runner_up,
            delta,
            plausible_count: plausible.len(),
            agrees,
            stable,
            mode,
        },
        cfg,
    );
    let class = (confidence >= Confidence::Medium).then_some(best);
    Outcome {
        confidence,
        class,
        leaning: best,
        plausible,
        slope,
        exponents,
        residuals: fits.iter().map(|(c, f)| (*c, f.residual)).collect(),
        range_ratio,
        sigma_bar,
        reasons,
    }
}

/// The measured quantities that decide the grade. Gathered in one place so the rule reads clearly.
struct Grading {
    points: usize,
    range_ratio: f64,
    sigma_bar: f64,
    e_best: f64,
    best: Class,
    runner_up: f64,
    delta: f64,
    plausible_count: usize,
    agrees: bool,
    stable: bool,
    mode: Mode,
}

/// Turns the measured quantities into a confidence and the reasons for it.
fn grade(g: &Grading, cfg: &FitConfig) -> (Confidence, Vec<Reason>) {
    let mut low = Vec::new();
    if g.range_ratio < cfg.min_range_medium {
        low.push(Reason::SizeRangeTooSmall);
    }
    if g.e_best > cfg.fit_ok_floor.max(cfg.fit_ok_k * g.sigma_bar) {
        low.push(Reason::NoModelFits);
    }
    if g.plausible_count > 1 {
        low.push(Reason::SeveralClassesPlausible);
    }
    if !g.agrees {
        low.push(Reason::ViewsDisagree);
    }
    if !g.stable {
        low.push(Reason::UnstableToOnePoint);
    }
    if !low.is_empty() {
        return (Confidence::Low, low);
    }

    let mut blockers = Vec::new();
    if g.points < cfg.min_points_high {
        blockers.push(Reason::TooFewPointsForHigh);
    }
    if g.range_ratio < cfg.min_range_high {
        blockers.push(Reason::RangeBelowHigh);
    }
    if g.sigma_bar > cfg.high_noise_max {
        blockers.push(Reason::NoisyForHigh);
    }
    if g.runner_up < g.e_best + cfg.high_margin_k * g.delta {
        blockers.push(Reason::NoClearMargin);
    }
    if matches!(
        g.best,
        Class::Constant | Class::Logarithmic | Class::Exponential
    ) {
        blockers.push(Reason::ClassExcludedFromHigh);
    }
    if !blockers.is_empty() {
        return (Confidence::Medium, blockers);
    }
    match g.mode {
        Mode::Full => (Confidence::High, Vec::new()),
        Mode::Quick => (Confidence::Medium, vec![Reason::RunCapped]),
    }
}

/// True when the slope, the ratio view and the best model agree on the growth.
fn views_agree(
    best: Class,
    sizes: &[u64],
    times: &[f64],
    exponents: &[f64],
    slope: &Slope,
    cfg: &FitConfig,
) -> bool {
    if best == Class::Exponential {
        // `2^n` has a steep slope and its exponents rise along the sizes.
        let first = exponents.first().copied().unwrap_or(0.0);
        let last = exponents.last().copied().unwrap_or(0.0);
        return slope.b >= 3.0 - cfg.se_k * slope.se && last >= first;
    }
    let model_slope = top_slope(sizes, &log_basis(best, sizes));
    let median = median_top_exponent(sizes, times);
    let slope_tol = cfg.slope_tol.max(cfg.se_k * slope.se);
    let exponent_tol = cfg.exp_tol.max(cfg.se_k * 1.2 * slope.se);
    (slope.b - model_slope).abs() <= slope_tol && (median - model_slope).abs() <= exponent_tol
}

/// True when dropping any single point leaves the best class unchanged. Only checked from five points.
fn is_stable(inputs: &Inputs<'_>, best: Class) -> bool {
    (0..inputs.sizes.len()).all(|skip| {
        let mut sizes = inputs.sizes.to_vec();
        let mut times = inputs.times.to_vec();
        let mut sigmas = inputs.sigmas.to_vec();
        sizes.remove(skip);
        times.remove(skip);
        sigmas.remove(skip);
        let fits = fit_all(&sizes, &times, &sigmas);
        best_of(&fits).0 == best
    })
}

#[cfg(test)]
mod tests {
    use super::super::views::ols_slope;
    use super::*;

    #[test]
    fn a_tie_in_residuals_goes_to_the_lower_rank() {
        let fit = |residual| ModelFit {
            a: 0.0,
            c: 0.0,
            residual,
        };
        let fits = vec![
            (Class::Constant, fit(0.5)),
            (Class::Logarithmic, fit(0.2)),
            (Class::Linear, fit(0.2)),
        ];
        assert_eq!(best_of(&fits).0, Class::Logarithmic);
    }

    #[test]
    fn ols_slope_of_a_straight_line_is_exact() {
        // y = 2x + 1 has slope 2 whatever the offset.
        assert!((ols_slope(&[1.0, 2.0, 3.0], &[3.0, 5.0, 7.0]) - 2.0).abs() < 1.0e-12);
    }
}
