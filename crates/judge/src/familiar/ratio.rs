//! Constant-factor speed against the same-language baseline (pure, no database).
//!
//! The ratio is the geometric mean of `learner ÷ baseline` over the sizes both series have.
//! A geometric mean treats a 2× slower size and a 2× faster size as cancelling, and it does not
//! let one large size dominate.

use std::collections::BTreeMap;

/// The fewest common sizes that give a ratio.
pub const MIN_COMMON_SIZES: usize = 2;

/// Returns the geometric-mean ratio of learner to baseline, or `None` when it is not meaningful.
///
/// `None` when fewer than two sizes are common, or when any common median is zero. Input order does
/// not matter; the points are matched by size.
#[must_use]
pub fn ratio_vs_baseline(learner: &[(u64, u64)], baseline: &[(u64, u64)]) -> Option<f64> {
    let base: BTreeMap<u64, u64> = baseline.iter().copied().collect();
    let mut log_ratios = Vec::new();
    for &(n, learner_ns) in learner {
        let Some(&base_ns) = base.get(&n) else {
            continue;
        };
        if learner_ns == 0 || base_ns == 0 {
            return None;
        }
        log_ratios.push(ln(learner_ns) - ln(base_ns));
    }
    if log_ratios.len() < MIN_COMMON_SIZES {
        return None;
    }
    let mean = log_ratios.iter().sum::<f64>() / len_as_f64(log_ratios.len());
    Some(mean.exp())
}

/// Natural log of a positive nanosecond count.
#[allow(clippy::cast_precision_loss)]
fn ln(nanoseconds: u64) -> f64 {
    (nanoseconds as f64).ln()
}

/// A count as a float. Counts here are far below 2^53.
#[allow(clippy::cast_precision_loss)]
fn len_as_f64(count: usize) -> f64 {
    count as f64
}

/// One stored baseline sample, as read from `problem_baselines`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BaselineRow {
    /// The problem version the sample was measured on.
    pub problem_version: u32,
    /// The runtime image digest the sample was measured with.
    pub image_digest: String,
    /// The input size.
    pub n: u64,
    /// The median time in nanoseconds.
    pub median_ns: u64,
}

/// Keeps only the rows measured on this problem version and this runtime image digest, and returns
/// them as `(n, median_ns)` pairs sorted by size. A stale or differently built baseline is ignored,
/// so the ratio is never computed against the wrong runtime.
#[must_use]
pub fn usable_points(
    rows: &[BaselineRow],
    problem_version: u32,
    image_digest: &str,
) -> Vec<(u64, u64)> {
    let mut points: Vec<(u64, u64)> = rows
        .iter()
        .filter(|row| row.problem_version == problem_version && row.image_digest == image_digest)
        .map(|row| (row.n, row.median_ns))
        .collect();
    points.sort_unstable();
    points
}

#[cfg(test)]
mod tests {
    use super::*;

    fn close(a: f64, b: f64) -> bool {
        (a - b).abs() < 1.0e-9 * b.abs().max(1.0)
    }

    #[test]
    fn a_series_against_itself_has_ratio_one() {
        let series = [(64, 100), (128, 210), (256, 450)];
        assert!(close(
            ratio_vs_baseline(&series, &series).expect("ratio"),
            1.0
        ));
    }

    #[test]
    fn a_learner_three_times_slower_has_ratio_three() {
        // Baseline 50, 100 ns; learner 150, 300 ns. Every size is exactly 3 times slower.
        let baseline = [(64, 50), (128, 100)];
        let learner = [(64, 150), (128, 300)];
        assert!(close(
            ratio_vs_baseline(&learner, &baseline).expect("ratio"),
            3.0
        ));
    }

    #[test]
    fn the_ratio_scales_with_the_learner_time() {
        let baseline = [(64, 50), (128, 100)];
        let learner = [(64, 100), (128, 200)];
        let lambda = 2.5;
        let scaled: Vec<(u64, u64)> = learner
            .iter()
            .map(|&(n, t)| (n, scale(t, lambda)))
            .collect();
        let base = ratio_vs_baseline(&learner, &baseline).expect("ratio");
        let grown = ratio_vs_baseline(&scaled, &baseline).expect("ratio");
        assert!(close(grown, base * lambda), "{grown} vs {}", base * lambda);
    }

    /// Multiplies a nanosecond count and keeps it a whole number of at least 1.
    #[allow(
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss,
        clippy::cast_precision_loss
    )]
    fn scale(nanoseconds: u64, lambda: f64) -> u64 {
        ((nanoseconds as f64) * lambda).round() as u64
    }

    #[test]
    fn fewer_than_two_common_sizes_give_no_ratio() {
        let learner = [(64, 100), (256, 100)];
        let baseline = [(64, 50), (128, 100)];
        assert_eq!(ratio_vs_baseline(&learner, &baseline), None);
    }

    #[test]
    fn a_zero_median_on_either_side_gives_no_ratio() {
        let baseline = [(64, 50), (128, 0)];
        let learner = [(64, 100), (128, 200)];
        assert_eq!(ratio_vs_baseline(&learner, &baseline), None);
        assert_eq!(ratio_vs_baseline(&baseline, &learner), None);
    }

    #[test]
    fn the_order_of_the_points_does_not_change_the_ratio() {
        let baseline = [(64, 50), (128, 100), (256, 260)];
        let learner = [(64, 120), (128, 230), (256, 500)];
        let mut reversed = learner;
        reversed.reverse();
        let mut shuffled_base = baseline;
        shuffled_base.reverse();
        let a = ratio_vs_baseline(&learner, &baseline).expect("ratio");
        let b = ratio_vs_baseline(&reversed, &shuffled_base).expect("ratio");
        assert!(close(a, b));
    }

    fn row(version: u32, digest: &str, n: u64, median_ns: u64) -> BaselineRow {
        BaselineRow {
            problem_version: version,
            image_digest: digest.to_owned(),
            n,
            median_ns,
        }
    }

    #[test]
    fn a_stale_problem_version_is_not_used() {
        let rows = [row(1, "sha256:a", 64, 50), row(2, "sha256:a", 128, 100)];
        assert_eq!(usable_points(&rows, 2, "sha256:a"), vec![(128, 100)]);
    }

    #[test]
    fn a_different_image_digest_is_not_used() {
        let rows = [row(2, "sha256:old", 64, 50), row(2, "sha256:new", 128, 100)];
        assert_eq!(usable_points(&rows, 2, "sha256:new"), vec![(128, 100)]);
    }

    #[test]
    fn usable_points_are_sorted_by_size_whatever_the_stored_order() {
        let rows = [row(2, "d", 256, 9), row(2, "d", 64, 3), row(2, "d", 128, 5)];
        assert_eq!(
            usable_points(&rows, 2, "d"),
            vec![(64, 3), (128, 5), (256, 9)]
        );
    }
}
