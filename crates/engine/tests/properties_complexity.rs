//! Property tests for the complexity fit (P1 to P7) and checks on the synthetic corpus.
//!
//! The corpus generator itself is tested first, because a property test is only as good
//! as the data it draws. Expected values here are hand-derived literals or come from the
//! generator's documented formula, never from the fit under test.

// Medians are below 1e18, so their `f64` conversion only rounds the last few bits; that is
// accounted for in the tolerances below.
#![allow(clippy::cast_precision_loss)]

mod support;

use counterspell_engine::complexity::{compare, fit, Class, FitConfig, Mode, Point};
use proptest::collection;
use proptest::prelude::*;
use support::complexity::{arb_clean_series, arb_series, compute, realise, sizes};

proptest! {
    #![proptest_config(ProptestConfig::with_cases(256))]

    /// The corpus only produces the shapes and ranges its documentation promises.
    #[test]
    fn corpus_respects_its_parameter_ranges(spec in arb_series()) {
        let points = realise(&spec);
        prop_assert_eq!(points.len(), spec.k);
        prop_assert!((3..=8).contains(&points.len()));
        for pair in points.windows(2) {
            prop_assert!(pair[1].n > pair[0].n);
        }
        for point in &points {
            prop_assert!(point.median_ns >= 1);
            prop_assert_eq!(point.spread.is_some(), spec.spread_known);
            if let Some(spread) = point.spread {
                prop_assert!(spread.is_finite() && spread >= 0.0);
            }
        }
    }

    /// Without noise, consecutive medians follow `(overhead + g(n)) ratio` exactly.
    #[test]
    fn noise_free_series_follow_the_documented_formula(spec in arb_clean_series()) {
        let points = realise(&spec);
        let base = compute(spec.truth, spec.n0) * spec.overhead;
        let expected: Vec<f64> = sizes(&spec)
            .iter()
            .map(|&n| base + compute(spec.truth, n))
            .collect();
        // Medians are whole nanoseconds, so each ratio may be off by half a unit on each side.
        let first = points[0].median_ns as f64;
        let reference = expected[0];
        for (point, want) in points.iter().zip(&expected) {
            let median = point.median_ns as f64;
            let got = median / first;
            let want = want / reference;
            let rounding = (1.0 / median + 1.0 / first) * want;
            prop_assert!(
                (got - want).abs() <= rounding + 1.0e-12 * want,
                "got {got}, want {want}"
            );
        }
    }
}

/// A linear series with no overhead doubles at every step, which is hand-checkable.
#[test]
fn a_linear_series_with_no_overhead_doubles_at_each_step() {
    let spec = support::complexity::SeriesSpec {
        truth: Class::Linear,
        k: 3,
        factor: 2,
        n0: 64,
        overhead: 0.0,
        noise_sd: 0.0,
        spike_p: 0.0,
        spike_mult: 1.0,
        spread_known: true,
        seed: 1,
    };
    let points: Vec<Point> = realise(&spec);
    assert_eq!(
        points.iter().map(|p| p.n).collect::<Vec<_>>(),
        vec![64, 128, 256]
    );
    let ratio = points[1].median_ns as f64 / points[0].median_ns as f64;
    assert!((ratio - 2.0).abs() < 1.0e-9, "ratio {ratio}");
}

/// The fit and the comparison agree with each other for a clean series.
#[test]
fn a_clean_linear_series_compares_as_matching_its_reference() {
    let spec = support::complexity::SeriesSpec {
        truth: Class::Linear,
        k: 6,
        factor: 2,
        n0: 64,
        overhead: 0.0,
        noise_sd: 0.0,
        spike_p: 0.0,
        spike_mult: 1.0,
        spread_known: true,
        seed: 2,
    };
    let points = realise(&spec);
    let report = fit(&points, Mode::Full, &FitConfig::default()).expect("valid input");
    assert_eq!(compare(&report, Class::Linear).gap, Some(0));
}

/// Adjacent means one rank apart on the seven-class ladder.
fn adjacent(a: Class, b: Class) -> bool {
    (i16::from(a.rank()) - i16::from(b.rank())).abs() <= 1
}

/// Arbitrary, possibly invalid, points: sizes, medians and spreads of every shape, including NaN.
fn arb_points() -> impl Strategy<Value = Vec<Point>> {
    let spread = prop_oneof![
        any::<f64>(),
        Just(f64::NAN),
        Just(f64::INFINITY),
        Just(f64::NEG_INFINITY),
    ];
    collection::vec(
        (any::<u64>(), any::<u64>(), proptest::option::of(spread)),
        0..40,
    )
    .prop_map(|mut raw| {
        raw.sort_by_key(|item| item.0);
        raw.dedup_by_key(|item| item.0);
        raw.into_iter()
            .map(|(n, median_ns, spread)| Point {
                n,
                median_ns,
                spread,
            })
            .collect()
    })
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(4000))]

    /// P1 (soundness, R3.8). High means the true class. Medium means the true class or a neighbour.
    #[test]
    fn p1_soundness_high_is_true_and_medium_is_true_or_adjacent(spec in arb_series()) {
        let points = realise(&spec);
        let report = fit(&points, Mode::Full, &FitConfig::default()).expect("valid input");
        match report.confidence {
            counterspell_engine::complexity::Confidence::High => {
                prop_assert_eq!(report.class, Some(spec.truth), "{:?}", report);
            }
            counterspell_engine::complexity::Confidence::Medium => {
                let class = report.class.expect("medium carries a class");
                prop_assert!(adjacent(class, spec.truth), "class {class:?} vs truth {:?}", spec.truth);
            }
            counterspell_engine::complexity::Confidence::Low => {}
        }
    }

    /// P2 (usefulness, R3.9). Clean series spanning 32 times give the true class at Medium or better.
    #[test]
    fn p2_usefulness_clean_series_are_called_at_least_medium(spec in arb_clean_series()) {
        let points = realise(&spec);
        let report = fit(&points, Mode::Full, &FitConfig::default()).expect("valid input");
        prop_assert_eq!(report.class, Some(spec.truth), "{:?}", report);
        prop_assert!(report.confidence >= counterspell_engine::complexity::Confidence::Medium);
    }

    /// P3 (scale invariance, R3.10). Multiplying every median by 2^j changes nothing.
    #[test]
    fn p3_scale_invariance_under_powers_of_two(spec in arb_series(), j in 0_u32..=3) {
        let points = realise(&spec);
        let factor = 1_u64 << j;
        let Some(scaled) = points
            .iter()
            .map(|p| p.median_ns.checked_mul(factor).map(|m| Point { median_ns: m, ..*p }))
            .collect::<Option<Vec<Point>>>()
        else {
            return Ok(());
        };
        let a = fit(&points, Mode::Full, &FitConfig::default()).expect("valid input");
        let b = fit(&scaled, Mode::Full, &FitConfig::default()).expect("valid input");
        prop_assert_eq!(a.class, b.class);
        prop_assert_eq!(a.confidence, b.confidence);
    }

    /// P4 (totality, R3.1 and R3.10). Any input gives an error or a report of finite numbers, never a panic.
    #[test]
    fn p4_totality_on_arbitrary_points(points in arb_points()) {
        match fit(&points, Mode::Full, &FitConfig::default()) {
            Err(_) => {}
            Ok(report) => {
                prop_assert!(report.slope.is_finite());
                prop_assert!(report.slope_se.is_finite());
                prop_assert!(report.noise.is_finite());
                prop_assert!(report.range_ratio.is_finite());
                prop_assert!(report.exponents.iter().all(|e| e.is_finite()));
                prop_assert!(report.residuals.iter().all(|(_, r)| r.is_finite()));
                prop_assert!(!report.plausible.is_empty());
            }
        }
    }

    /// P5 (determinism, R3.10). The same input gives a bit-identical report.
    #[test]
    fn p5_determinism(spec in arb_series()) {
        let points = realise(&spec);
        let a = fit(&points, Mode::Full, &FitConfig::default());
        let b = fit(&points, Mode::Full, &FitConfig::default());
        prop_assert_eq!(a, b);
    }

    /// P6 (class only with confidence, R3.4 and R3.5). A class exists exactly at Medium or High,
    /// and the leaning class is always plausible.
    #[test]
    fn p6_class_only_with_confidence_and_leaning_is_plausible(spec in arb_series()) {
        let points = realise(&spec);
        let report = fit(&points, Mode::Full, &FitConfig::default()).expect("valid input");
        prop_assert_eq!(
            report.class.is_some(),
            report.confidence != counterspell_engine::complexity::Confidence::Low
        );
        prop_assert!(report.plausible.contains(&report.leaning));
    }

    /// P7 (run cap, R3.7). A Quick run never reports High.
    #[test]
    fn p7_quick_mode_never_reports_high(spec in arb_series()) {
        let points = realise(&spec);
        let report = fit(&points, Mode::Quick, &FitConfig::default()).expect("valid input");
        prop_assert!(report.confidence != counterspell_engine::complexity::Confidence::High);
    }
}

/// Draws a series spec from a seeded generator, the same space `arb_series` covers.
fn sampled_spec(
    rng: &mut support::complexity::Rng,
    truth: Class,
) -> support::complexity::SeriesSpec {
    let factors = [2_u64, 4];
    let k = 3 + usize::try_from(rng.next_u64() % 6).expect("small");
    support::complexity::SeriesSpec {
        truth,
        k,
        factor: factors[usize::try_from(rng.next_u64() % 2).expect("small")],
        n0: 1_u64 << (6 + rng.next_u64() % 5),
        overhead: rng.unit() * 3.0,
        noise_sd: rng.unit() * 0.25,
        spike_p: rng.unit() * 0.1,
        spike_mult: 1.5 + rng.unit() * 1.5,
        spread_known: rng.next_u64() % 2 == 0,
        seed: rng.next_u64(),
    }
}

/// Calibration report (ignored by default). Prints one row per true class for `fit-1`.
/// Run with `cargo test -p counterspell-engine --test properties_complexity -- --ignored --nocapture calibration`.
#[test]
#[ignore = "calibration report, run on demand"]
fn calibration_report() {
    use counterspell_engine::complexity::Confidence;
    let cases = 4000_u64;
    println!("| true class | cases | Low | Medium correct | Medium adjacent | High correct | wrong at High |");
    println!("|---|---|---|---|---|---|---|");
    for truth in [
        Class::Linear,
        Class::Linearithmic,
        Class::Quadratic,
        Class::Cubic,
    ] {
        let mut rng = support::complexity::Rng::new(0x5EED_0000 + u64::from(truth.rank()));
        let (mut low, mut med_ok, mut med_adj, mut high_ok, mut wrong_high) = (0, 0, 0, 0, 0);
        for _ in 0..cases {
            let spec = sampled_spec(&mut rng, truth);
            let report =
                fit(&realise(&spec), Mode::Full, &FitConfig::default()).expect("valid input");
            match (report.confidence, report.class) {
                (Confidence::Low, _) => low += 1,
                (Confidence::Medium, Some(c)) if c == truth => med_ok += 1,
                (Confidence::Medium, Some(c)) if adjacent(c, truth) => med_adj += 1,
                (Confidence::High, Some(c)) if c == truth => high_ok += 1,
                (Confidence::High, _) => wrong_high += 1,
                _ => wrong_high += 0,
            }
        }
        let pct = |n: u64| format!("{:.1}%", 100.0 * n as f64 / cases as f64);
        println!(
            "| {:?} | {cases} | {} | {} | {} | {} | {wrong_high} |",
            truth,
            pct(low),
            pct(med_ok),
            pct(med_adj),
            pct(high_ok)
        );
    }
}
