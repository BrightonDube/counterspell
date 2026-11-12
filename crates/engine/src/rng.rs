//! Deterministic seeded random generation.

#![allow(dead_code)]

const SPLITMIX_INCREMENT: u64 = 0x9e37_79b9_7f4a_7c15;
const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// A small deterministic generator with a platform-independent output sequence.
#[derive(Clone, Debug)]
pub struct SplitMix64 {
    state: u64,
}

impl SplitMix64 {
    /// Creates a generator from any 64-bit seed, including zero.
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Returns the next pseudorandom 64-bit value.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(SPLITMIX_INCREMENT);
        let mut value = self.state;
        value = (value ^ (value >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        value = (value ^ (value >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        value ^ (value >> 31)
    }

    /// Returns an integer uniformly from the inclusive range `lo..=hi`.
    pub fn gen_range(&mut self, lo: i64, hi: i64) -> i64 {
        if lo >= hi {
            return lo;
        }

        let Ok(width) = u128::try_from(i128::from(hi) - i128::from(lo) + 1) else {
            return lo;
        };
        let limit = (1_u128 << 64) - ((1_u128 << 64) % width);

        loop {
            let random = u128::from(self.next_u64());
            if random < limit {
                let offset = random % width;
                let Ok(offset) = i128::try_from(offset) else {
                    return lo;
                };
                let value = i128::from(lo) + offset;
                return match i64::try_from(value) {
                    Ok(value) => value,
                    Err(_) => lo,
                };
            }
        }
    }

    /// Returns true with the requested probability from zero through 100 percent.
    pub fn gen_bool(&mut self, percent: u8) -> bool {
        let percent = u64::from(percent.min(100));
        percent == 100 || (percent != 0 && self.next_u64() % 100 < percent)
    }
}

/// Derives a stable seed from the problem identity and engine version.
#[must_use]
pub fn derive_seed(problem_slug: &str, code_sha256: &str, language: &str) -> u64 {
    let mut hash = FNV_OFFSET;
    for field in [problem_slug, code_sha256, language, crate::ENGINE_VERSION] {
        let length = (field.len() as u64).to_le_bytes();
        for byte in length {
            hash ^= u64::from(byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        for byte in field.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(FNV_PRIME);
        }
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::{derive_seed, SplitMix64};

    #[test]
    fn splitmix64_has_pinned_first_outputs() {
        let expected = [
            (0, 0xe220_a839_7b1d_cdafu64),
            (1, 0x910a_2dec_8902_5cc1u64),
            (42, 0xbdd7_3226_2feb_6e95u64),
        ];

        for (seed, first) in expected {
            assert_eq!(SplitMix64::new(seed).next_u64(), first);
        }
    }

    #[test]
    fn splitmix64_sequence_is_pinned_for_seed_zero() {
        let mut rng = SplitMix64::new(0);
        assert_eq!(rng.next_u64(), 0xe220_a839_7b1d_cdafu64);
        assert_eq!(rng.next_u64(), 0x6e78_9e6a_a1b9_65f4u64);
        assert_eq!(rng.next_u64(), 0x06c4_5d18_8009_454fu64);
    }

    #[test]
    fn range_includes_full_i64_domain_without_leaving_bounds() {
        let mut rng = SplitMix64::new(0x1234_5678_9abc_def0);
        for _ in 0..512 {
            let value = rng.gen_range(i64::MIN, i64::MAX);
            assert!((i64::MIN..=i64::MAX).contains(&value));
        }
    }

    #[test]
    fn range_rejects_a_high_draw_before_sampling_the_next_value() {
        let mut rng = SplitMix64::new(0);
        assert_eq!(rng.gen_range(i64::MIN, 0), -1_263_085_514_660_420_108);
    }

    #[test]
    fn intermediate_boolean_percentages_use_the_next_draw() {
        let mut rng = SplitMix64::new(7);
        assert!(!rng.gen_bool(50));
        assert!(!rng.gen_bool(1));
        assert!(rng.gen_bool(99));
    }

    #[test]
    fn boolean_bounds_are_deterministic() {
        let mut rng = SplitMix64::new(7);
        assert!(!rng.gen_bool(0));
        assert!(rng.gen_bool(100));
    }

    #[test]
    fn derived_seed_is_pinned_and_length_prefixed() {
        assert_eq!(
            derive_seed("pair-count", "abc123", "python"),
            0xe6d2_3fe5_9419_93bdu64
        );
        assert_ne!(
            derive_seed("ab", "c", "python"),
            derive_seed("a", "bc", "python")
        );
    }

    #[test]
    fn seed_round_trip_through_postgres_signed_integer_is_lossless() {
        let seed = 0xfedc_ba98_7654_3210u64;
        let signed = i64::from_ne_bytes(seed.to_ne_bytes());
        assert_eq!(u64::from_ne_bytes(signed.to_ne_bytes()), seed);
    }
}
