//! Latency statistics for the definition-of-done report. The test computes every number; nothing is typed in
//! by hand. Percentiles use the nearest-rank method.

use std::fmt::Write as _;

use serde_json::{json, Value};

/// Summary of one metric, in milliseconds.
#[derive(Debug, Clone, PartialEq)]
pub struct Stats {
    /// How many samples were measured.
    pub count: usize,
    /// The arithmetic mean.
    pub mean: f64,
    /// The median (nearest-rank 50th percentile).
    pub p50: f64,
    /// The nearest-rank 95th percentile.
    pub p95: f64,
    /// The largest sample.
    pub max: f64,
}

/// The nearest-rank percentile of values that are already sorted in ascending order.
///
/// # Panics
///
/// Panics when `sorted` is empty. Callers check for that first, through [`stats`].
#[must_use]
#[allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]
pub fn percentile(sorted: &[f64], percent: f64) -> f64 {
    let rank = ((percent / 100.0) * sorted.len() as f64).ceil() as usize;
    sorted[rank.clamp(1, sorted.len()) - 1]
}

/// Summarises the samples. Returns `None` when there are none.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn stats(values: &[f64]) -> Option<Stats> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let mean = sorted.iter().sum::<f64>() / sorted.len() as f64;
    Some(Stats {
        count: sorted.len(),
        mean,
        p50: percentile(&sorted, 50.0),
        p95: percentile(&sorted, 95.0),
        max: sorted[sorted.len() - 1],
    })
}

/// The metrics as one JSON object per metric, for `dod-report.json`.
#[must_use]
pub fn to_json(rows: &[(&str, Option<Stats>)]) -> Value {
    let mut metrics = serde_json::Map::new();
    for (name, summary) in rows {
        let value = match summary {
            Some(s) => json!({
                "count": s.count, "mean_ms": s.mean, "p50_ms": s.p50, "p95_ms": s.p95, "max_ms": s.max
            }),
            None => json!({ "count": 0 }),
        };
        metrics.insert((*name).to_owned(), value);
    }
    Value::Object(metrics)
}

/// A Markdown table with one row per metric.
#[must_use]
pub fn markdown_table(rows: &[(&str, Option<Stats>)]) -> String {
    let mut out = String::from(
        "| Metric | Count | Mean ms | p50 ms | p95 ms | Max ms |\n|---|---:|---:|---:|---:|---:|\n",
    );
    for (name, summary) in rows {
        // Writing to a String cannot fail, so the result is ignored.
        let _ = match summary {
            Some(s) => writeln!(
                out,
                "| {name} | {} | {:.1} | {:.1} | {:.1} | {:.1} |",
                s.count, s.mean, s.p50, s.p95, s.max
            ),
            None => writeln!(out, "| {name} | 0 | - | - | - | - |"),
        };
    }
    out
}

#[cfg(test)]
mod tests {
    use super::{percentile, stats};

    #[test]
    fn percentiles_use_the_nearest_rank() {
        let values: Vec<f64> = (1..=10).map(f64::from).collect();

        let summary = stats(&values).expect("ten samples");

        assert_eq!(summary.count, 10);
        assert_eq!(summary.p50, 5.0, "rank ceil(0.5 * 10) = 5");
        assert_eq!(summary.p95, 10.0, "rank ceil(0.95 * 10) = 10");
        assert_eq!(summary.max, 10.0);
        assert!((summary.mean - 5.5).abs() < 1e-9);
    }

    #[test]
    fn percentile_of_a_single_sample_is_that_sample() {
        assert_eq!(percentile(&[7.0], 95.0), 7.0);
    }

    #[test]
    fn no_samples_means_no_summary() {
        assert!(stats(&[]).is_none());
    }
}
