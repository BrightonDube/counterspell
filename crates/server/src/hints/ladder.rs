//! The ladder gate: decides whether a learner may reveal a rung yet.
//!
//! Rung 1 is always open. Rung `k` opens only after rung `k - 1` has been revealed for the same
//! submission. This module is pure: it takes the revealed set and returns a decision.

use std::collections::BTreeSet;

/// What the learner may do with one rung.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Gate {
    /// The rung can be revealed now.
    Open,
    /// The rung was revealed before. Return the stored text.
    Revealed,
    /// The rung cannot be revealed yet. `missing` is the earlier rung that must come first.
    Locked {
        /// The rung that must be revealed first.
        missing: u8,
    },
}

/// Decides whether `rung` (1 to 4) may be revealed, given the rungs already revealed.
#[must_use]
pub fn gate(revealed: &BTreeSet<u8>, rung: u8) -> Gate {
    if revealed.contains(&rung) {
        return Gate::Revealed;
    }
    let prerequisite = rung.saturating_sub(1);
    if prerequisite == 0 || revealed.contains(&prerequisite) {
        Gate::Open
    } else {
        Gate::Locked {
            missing: prerequisite,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use std::collections::BTreeSet;

    fn revealed(rungs: &[u8]) -> BTreeSet<u8> {
        rungs.iter().copied().collect()
    }

    #[test]
    fn rung_one_is_open_before_anything_is_revealed() {
        assert_eq!(gate(&revealed(&[]), 1), Gate::Open);
    }

    #[test]
    fn rung_two_is_locked_until_rung_one_is_revealed() {
        assert_eq!(gate(&revealed(&[]), 2), Gate::Locked { missing: 1 });
        assert_eq!(gate(&revealed(&[1]), 2), Gate::Open);
    }

    #[test]
    fn rung_three_names_rung_two_as_the_missing_step() {
        assert_eq!(gate(&revealed(&[1]), 3), Gate::Locked { missing: 2 });
    }

    #[test]
    fn a_revealed_rung_is_reported_as_revealed_not_open() {
        assert_eq!(gate(&revealed(&[1, 2]), 1), Gate::Revealed);
        assert_eq!(gate(&revealed(&[1, 2]), 2), Gate::Revealed);
    }

    proptest! {
        // P1: whatever order a learner asks in, the rungs they reveal form the prefix 1..=k.
        #[test]
        fn revealed_rungs_are_always_a_prefix(attempts in proptest::collection::vec(1u8..=4, 0..40)) {
            let mut set = BTreeSet::new();
            for rung in attempts {
                if gate(&set, rung) == Gate::Open {
                    set.insert(rung);
                }
            }
            let expected: BTreeSet<u8> = (1..=4u8)
                .filter(|rung| usize::from(*rung) <= set.len())
                .collect();
            prop_assert_eq!(set, expected);
        }

        // P1: an open rung always has its predecessor revealed, or is rung 1.
        #[test]
        fn an_open_rung_always_has_its_predecessor(
            rungs in proptest::collection::btree_set(1u8..=4, 0..=4),
            rung in 1u8..=4,
        ) {
            if gate(&rungs, rung) == Gate::Open {
                prop_assert!(rung == 1 || rungs.contains(&(rung - 1)));
            }
        }
    }
}
