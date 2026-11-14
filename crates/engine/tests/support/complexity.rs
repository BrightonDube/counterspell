//! Synthetic timing corpus for the complexity fit (test support only; nothing here ships).
//!
//! A series has a true class (`n`, `n log n`, `n^2` or `n^3`), a fixed per-call
//! overhead, multiplicative lognormal noise, occasional spikes and five repeats
//! per size. Each point's median is taken over those repeats, and its spread is
//! `(max − min) / median`. The generator is seeded, so every case can be replayed.

use counterspell_engine::complexity::{Class, Point};
use proptest::prelude::*;

/// Repeats per size, as the judge measures them.
pub const REPEATS: usize = 5;

/// The shape of one synthetic series. Every field has a range the strategies respect.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SeriesSpec {
    /// The true growth class. Only the four classes the corpus uses.
    pub truth: Class,
    /// Number of sizes, from 3 to 8.
    pub k: usize,
    /// Multiplier between consecutive sizes: 2 or 4.
    pub factor: u64,
    /// The smallest size: a power of two from 2^6 to 2^10.
    pub n0: u64,
    /// Fixed per-call cost, as a multiple of the compute at `n0` (0 to 3).
    pub overhead: f64,
    /// Standard deviation of the lognormal noise on each repeat (0 to 0.25).
    pub noise_sd: f64,
    /// Probability that a repeat is a spike (0 to 0.1).
    pub spike_p: f64,
    /// Multiplier of a spike (1.5 to 3).
    pub spike_mult: f64,
    /// Whether the points carry their measured spread, or none.
    pub spread_known: bool,
    /// Seed for the noise and spikes.
    pub seed: u64,
}

/// Scale that puts the largest median at 1e18, inside `u64` range. Smaller medians keep their
/// rounding error below one part in their own value, which the corpus tests state.
const MEDIAN_SCALE: f64 = 1.0e18;

/// The compute of a true class at size `n`, before overhead. `n` is at least 64 here.
///
/// # Panics
///
/// Panics for a class the corpus does not generate (`1`, `log n`, `2^n`).
#[allow(clippy::cast_precision_loss)]
pub fn compute(truth: Class, n: u64) -> f64 {
    let x = n as f64;
    match truth {
        Class::Linear => x,
        Class::Linearithmic => x * x.ln(),
        Class::Quadratic => x * x,
        Class::Cubic => x * x * x,
        other => panic!("the corpus does not generate {other:?}"),
    }
}

/// The sizes of a series: `n0 · factor^i` for `i` in `0..k`.
pub fn sizes(spec: &SeriesSpec) -> Vec<u64> {
    let mut size = spec.n0;
    (0..spec.k)
        .map(|_| {
            let current = size;
            size *= spec.factor;
            current
        })
        .collect()
}

/// A small seeded generator (splitmix64). Each call advances the state once.
pub struct Rng(u64);

impl Rng {
    /// Starts a generator from a seed.
    pub fn new(seed: u64) -> Self {
        Self(seed)
    }

    /// The next 64 random bits.
    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// A uniform number in [0, 1).
    #[allow(clippy::cast_precision_loss)]
    pub fn unit(&mut self) -> f64 {
        // 53 random bits give every representable value in [0, 1) equal weight.
        (self.next_u64() >> 11) as f64 / (1_u64 << 53) as f64
    }

    /// A standard normal number (Box-Muller).
    pub fn gaussian(&mut self) -> f64 {
        let u1 = self.unit().max(f64::MIN_POSITIVE);
        let u2 = self.unit();
        (-2.0 * u1.ln()).sqrt() * (std::f64::consts::TAU * u2).cos()
    }
}

/// Builds the measured points for a series, with noise and spikes applied per repeat.
pub fn realise(spec: &SeriesSpec) -> Vec<Point> {
    let sizes = sizes(spec);
    let base = compute(spec.truth, spec.n0) * spec.overhead;
    let mut rng = Rng::new(spec.seed);
    // Raw medians first, then one scale, so the largest median sits at `MEDIAN_SCALE`.
    let mut raw = Vec::with_capacity(sizes.len());
    let mut spreads = Vec::with_capacity(sizes.len());
    for &n in &sizes {
        let exact = base + compute(spec.truth, n);
        let mut repeats: Vec<f64> = (0..REPEATS)
            .map(|_| {
                let noisy = exact * (spec.noise_sd * rng.gaussian()).exp();
                if rng.unit() < spec.spike_p {
                    noisy * spec.spike_mult
                } else {
                    noisy
                }
            })
            .collect();
        repeats.sort_by(f64::total_cmp);
        let median = repeats[REPEATS / 2];
        let spread = (repeats[REPEATS - 1] - repeats[0]) / median;
        raw.push(median);
        spreads.push(spread);
    }
    let largest = raw.iter().copied().fold(0.0_f64, f64::max);
    sizes
        .iter()
        .zip(raw.iter().zip(&spreads))
        .map(|(&n, (&median, &spread))| Point {
            n,
            median_ns: scaled(median / largest),
            spread: spec.spread_known.then_some(spread),
        })
        .collect()
}

/// Turns a value in (0, 1] into a median in nanoseconds, never below 1.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn scaled(unit: f64) -> u64 {
    // Rounding to the nearest integer, then keeping at least 1, gives a valid median.
    ((unit * MEDIAN_SCALE).round() as u64).max(1)
}

/// Strategy for the whole series space, with noise and spikes.
pub fn arb_series() -> BoxedStrategy<SeriesSpec> {
    (
        prop_oneof![
            Just(Class::Linear),
            Just(Class::Linearithmic),
            Just(Class::Quadratic),
            Just(Class::Cubic)
        ],
        3_usize..=8,
        prop_oneof![Just(2_u64), Just(4_u64)],
        (6_u32..=10).prop_map(|e| 1_u64 << e),
        0.0_f64..=3.0,
        0.0_f64..=0.25,
        0.0_f64..=0.1,
        1.5_f64..=3.0,
        any::<bool>(),
        any::<u64>(),
    )
        .prop_map(
            |(
                truth,
                k,
                factor,
                n0,
                overhead,
                noise_sd,
                spike_p,
                spike_mult,
                spread_known,
                seed,
            )| {
                SeriesSpec {
                    truth,
                    k,
                    factor,
                    n0,
                    overhead,
                    noise_sd,
                    spike_p,
                    spike_mult,
                    spread_known,
                    seed,
                }
            },
        )
        .boxed()
}

/// Strategy for noise-free series (for the usefulness property P2): 5 to 8 points spanning at least
/// 32 times, with overhead up to 100 percent.
pub fn arb_clean_series() -> BoxedStrategy<SeriesSpec> {
    (
        prop_oneof![
            Just(Class::Linear),
            Just(Class::Linearithmic),
            Just(Class::Quadratic),
            Just(Class::Cubic)
        ],
        5_usize..=8,
        prop_oneof![Just(2_u64), Just(4_u64)],
        (6_u32..=10).prop_map(|e| 1_u64 << e),
        0.0_f64..=1.0,
        any::<u64>(),
    )
        .prop_filter("span at least 32 times", |(_, k, factor, _, _, _)| {
            (1..*k).fold(1_u64, |span, _| span * factor) >= 32
        })
        .prop_map(|(truth, k, factor, n0, overhead, seed)| SeriesSpec {
            truth,
            k,
            factor,
            n0,
            overhead,
            noise_sd: 0.0,
            spike_p: 0.0,
            spike_mult: 1.0,
            spread_known: true,
            seed,
        })
        .boxed()
}
