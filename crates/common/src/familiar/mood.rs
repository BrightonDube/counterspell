//! The mood rule. Requirement R5.1, in one pure function.
//!
//! `decide` reads only [`MeasuredFacts`]. Each branch below is one row of the
//! table in the requirements, in the same order.

use super::class::{ClassId, Confidence, Effect, Mood};
use super::facts::{BenchFacts, MeasuredFacts, NoMeasurement};
use crate::status::{RunMode, Verdict};

/// The machine reason for a mood. Each reason has a fixed code of lowercase
/// words and underscores, which is what is stored and sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoodReason {
    /// The run failed with this verdict (never `accepted`).
    Verdict(Verdict),
    /// The problem declares no bench.
    NoBenchSpec,
    /// The host was busy, so the bench was discarded.
    HostBusy,
    /// The bench call failed.
    BenchFailed,
    /// Too few sizes finished inside the budget.
    BenchIncomplete,
    /// The fit is unsure, so the result is not judged.
    LowConfidence,
    /// The observed class is three or more classes slower than the reference.
    WorstTier,
    /// The observed class is two classes slower than the reference.
    TwoClassesWorse,
    /// The observed class is one class slower than the reference.
    OneClassWorse,
    /// Same growth or better, but no usable baseline to compare the speed with.
    NoBaseline,
    /// Same growth or better, but more than twice the baseline time.
    SlowerConstantFactor,
    /// Same growth as the reference, within twice the baseline time.
    MatchesReference,
    /// Better growth than the reference, within twice the baseline time.
    BetterThanReference,
}

impl MoodReason {
    /// Every reason except the verdict-based ones, which are read from the verdict text.
    pub const ALL_REASONS: [Self; 12] = [
        Self::NoBenchSpec,
        Self::HostBusy,
        Self::BenchFailed,
        Self::BenchIncomplete,
        Self::LowConfidence,
        Self::WorstTier,
        Self::TwoClassesWorse,
        Self::OneClassWorse,
        Self::NoBaseline,
        Self::SlowerConstantFactor,
        Self::MatchesReference,
        Self::BetterThanReference,
    ];

    /// Reads a stored reason code back into a reason. Returns `None` for an unknown code, and for
    /// `accepted`, which is never a failure reason.
    #[must_use]
    pub fn from_code(text: &str) -> Option<Self> {
        if let Some(verdict) = Verdict::parse(text) {
            return (verdict != Verdict::Accepted).then_some(Self::Verdict(verdict));
        }
        Self::ALL_REASONS
            .into_iter()
            .find(|reason| reason.code() == text)
    }

    /// The stored and wire code. Verdict reasons use the verdict's own text.
    #[must_use]
    pub const fn code(self) -> &'static str {
        match self {
            Self::Verdict(verdict) => verdict.as_str(),
            Self::NoBenchSpec => "no_bench_spec",
            Self::HostBusy => "host_busy",
            Self::BenchFailed => "bench_failed",
            Self::BenchIncomplete => "bench_incomplete",
            Self::LowConfidence => "low_confidence",
            Self::WorstTier => "worst_tier",
            Self::TwoClassesWorse => "two_classes_worse",
            Self::OneClassWorse => "one_class_worse",
            Self::NoBaseline => "no_baseline",
            Self::SlowerConstantFactor => "slower_constant_factor",
            Self::MatchesReference => "matches_reference",
            Self::BetterThanReference => "better_than_reference",
        }
    }
}

/// The complete answer for one run: the mood, why, and the one effect to play.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoodDecision {
    /// The mood from 1 to 5.
    pub mood: Mood,
    /// The reason the mood was chosen.
    pub reason: MoodReason,
    /// The effect to play, if the event is live.
    pub effect: Effect,
}

/// Decides the mood from measured facts. The function is total and pure.
///
/// Confidence `Medium` or `High` with no observed class cannot come from a
/// correct fit. It is treated as `low_confidence`, so the function still
/// returns a mood.
#[must_use]
pub fn decide(facts: &MeasuredFacts) -> MoodDecision {
    if facts.verdict != Verdict::Accepted {
        return MoodDecision {
            mood: Mood::FURIOUS,
            reason: MoodReason::Verdict(facts.verdict),
            effect: Effect::CrackBurst,
        };
    }
    match facts.bench {
        BenchFacts::None(missing) => pleased(missing_reason(missing)),
        BenchFacts::Measured {
            confidence: Confidence::Low,
            ..
        }
        | BenchFacts::Measured { observed: None, .. } => pleased(MoodReason::LowConfidence),
        BenchFacts::Measured {
            confidence: _,
            observed: Some(observed),
            reference,
            ratio_vs_baseline,
        } => judged(facts.mode, observed, reference, ratio_vs_baseline),
    }
}

/// The reason for a passing run that has no growth measurement.
const fn missing_reason(missing: NoMeasurement) -> MoodReason {
    match missing {
        NoMeasurement::NotSpecified => MoodReason::NoBenchSpec,
        NoMeasurement::HostBusy => MoodReason::HostBusy,
        NoMeasurement::Failed => MoodReason::BenchFailed,
        NoMeasurement::Incomplete => MoodReason::BenchIncomplete,
    }
}

/// A mood of 4, which needs a valid mood constant.
const fn pleased(reason: MoodReason) -> MoodDecision {
    MoodDecision {
        mood: Mood::PLEASED,
        reason,
        effect: Effect::None,
    }
}

/// The rules for a passing run with a fitted class, in the order of R5.1.
fn judged(
    mode: RunMode,
    observed: ClassId,
    reference: ClassId,
    ratio: Option<f64>,
) -> MoodDecision {
    // The rank difference is small, so `i16` holds it without any cast.
    let gap = i16::from(observed.rank()) - i16::from(reference.rank());
    let (mood, reason) = match gap {
        3.. => (Mood::FURIOUS, MoodReason::WorstTier),
        2 => (Mood::ANGRY, MoodReason::TwoClassesWorse),
        1 => (Mood::UNEASY, MoodReason::OneClassWorse),
        _ => match ratio {
            None => (Mood::PLEASED, MoodReason::NoBaseline),
            Some(r) if !(r.is_finite() && r > 0.0) => (Mood::PLEASED, MoodReason::NoBaseline),
            Some(r) if r > 2.0 => (Mood::PLEASED, MoodReason::SlowerConstantFactor),
            Some(_) if gap == 0 => (Mood::LAUGHING, MoodReason::MatchesReference),
            Some(_) => (Mood::LAUGHING, MoodReason::BetterThanReference),
        },
    };
    let effect = if mood == Mood::LAUGHING && mode == RunMode::Submit {
        Effect::LaughBurst
    } else {
        Effect::None
    };
    MoodDecision {
        mood,
        reason,
        effect,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::familiar::facts::{BenchFacts, MeasuredFacts, NoMeasurement};
    use crate::status::{RunMode, Verdict};

    // Breaks if a stored reason cannot be read back, so the read model cannot rebuild its wording.
    #[test]
    fn every_reason_code_reads_back_to_the_same_reason() {
        for reason in MoodReason::ALL_REASONS {
            assert_eq!(MoodReason::from_code(reason.code()), Some(reason));
        }
    }

    // Breaks if an unknown code or `accepted` is read as a failure reason, which would give a false wording.
    #[test]
    fn unknown_codes_and_accepted_are_not_reasons() {
        assert_eq!(MoodReason::from_code("optimal"), None);
        assert_eq!(MoodReason::from_code(""), None);
        assert_eq!(MoodReason::from_code("accepted"), None);
    }

    fn measured(
        observed: Option<ClassId>,
        reference: ClassId,
        confidence: Confidence,
        ratio: Option<f64>,
    ) -> BenchFacts {
        BenchFacts::Measured {
            confidence,
            observed,
            reference,
            ratio_vs_baseline: ratio,
        }
    }

    fn facts(verdict: Verdict, mode: RunMode, bench: BenchFacts) -> MeasuredFacts {
        MeasuredFacts {
            verdict,
            mode,
            bench,
        }
    }

    #[test]
    fn a_failing_verdict_is_furious_with_a_crack_burst() {
        let d = decide(&facts(
            Verdict::WrongAnswer,
            RunMode::Submit,
            measured(
                Some(ClassId::Constant),
                ClassId::Linear,
                Confidence::High,
                Some(0.5),
            ),
        ));
        assert_eq!(d.mood.value(), 1);
        assert_eq!(d.reason.code(), "wrong_answer");
        assert_eq!(d.effect, Effect::CrackBurst);
    }

    #[test]
    fn every_failing_verdict_gives_furious_whatever_the_bench_says() {
        for verdict in [
            Verdict::RuntimeError,
            Verdict::TimeLimit,
            Verdict::MemoryLimit,
            Verdict::OutputLimit,
            Verdict::CompileError,
        ] {
            let d = decide(&facts(
                verdict,
                RunMode::Submit,
                BenchFacts::None(NoMeasurement::NotSpecified),
            ));
            assert_eq!(d.mood, Mood::FURIOUS, "{verdict:?}");
            assert_eq!(d.reason.code(), verdict.as_str());
        }
    }

    #[test]
    fn accepted_without_a_bench_spec_is_pleased_with_no_bench_spec() {
        let d = decide(&facts(
            Verdict::Accepted,
            RunMode::Submit,
            BenchFacts::None(NoMeasurement::NotSpecified),
        ));
        assert_eq!(d.mood, Mood::PLEASED);
        assert_eq!(d.reason.code(), "no_bench_spec");
        assert_eq!(d.effect, Effect::None);
    }

    #[test]
    fn accepted_with_each_missing_measurement_is_pleased_with_its_reason() {
        let cases = [
            (NoMeasurement::HostBusy, "host_busy"),
            (NoMeasurement::Failed, "bench_failed"),
            (NoMeasurement::Incomplete, "bench_incomplete"),
        ];
        for (missing, code) in cases {
            let d = decide(&facts(
                Verdict::Accepted,
                RunMode::Submit,
                BenchFacts::None(missing),
            ));
            assert_eq!(d.mood, Mood::PLEASED);
            assert_eq!(d.reason.code(), code);
        }
    }

    #[test]
    fn low_confidence_is_pleased_and_never_lower() {
        let d = decide(&facts(
            Verdict::Accepted,
            RunMode::Submit,
            measured(None, ClassId::Constant, Confidence::Low, Some(0.5)),
        ));
        assert_eq!(d.mood, Mood::PLEASED);
        assert_eq!(d.reason.code(), "low_confidence");
    }

    #[test]
    fn a_gap_of_three_or_more_is_furious_worst_tier_without_a_burst() {
        // Reference constant, observed cubic: rank 5 minus rank 0 is a gap of 5.
        let d = decide(&facts(
            Verdict::Accepted,
            RunMode::Submit,
            measured(
                Some(ClassId::Cubic),
                ClassId::Constant,
                Confidence::High,
                Some(1.0),
            ),
        ));
        assert_eq!(d.mood, Mood::FURIOUS);
        assert_eq!(d.reason.code(), "worst_tier");
        assert_eq!(d.effect, Effect::None);
    }

    #[test]
    fn a_gap_of_two_is_angry_two_classes_worse() {
        // Reference linear (rank 2), observed quadratic (rank 4).
        let d = decide(&facts(
            Verdict::Accepted,
            RunMode::Submit,
            measured(
                Some(ClassId::Quadratic),
                ClassId::Linear,
                Confidence::Medium,
                Some(1.0),
            ),
        ));
        assert_eq!(d.mood, Mood::ANGRY);
        assert_eq!(d.reason.code(), "two_classes_worse");
    }

    #[test]
    fn a_gap_of_one_is_uneasy_one_class_worse() {
        // Reference linear (rank 2), observed n log n (rank 3).
        let d = decide(&facts(
            Verdict::Accepted,
            RunMode::Submit,
            measured(
                Some(ClassId::Linearithmic),
                ClassId::Linear,
                Confidence::Medium,
                Some(1.0),
            ),
        ));
        assert_eq!(d.mood, Mood::UNEASY);
        assert_eq!(d.reason.code(), "one_class_worse");
    }

    #[test]
    fn same_class_without_a_baseline_is_pleased_no_baseline() {
        let d = decide(&facts(
            Verdict::Accepted,
            RunMode::Submit,
            measured(
                Some(ClassId::Linear),
                ClassId::Linear,
                Confidence::High,
                None,
            ),
        ));
        assert_eq!(d.mood, Mood::PLEASED);
        assert_eq!(d.reason.code(), "no_baseline");
    }

    #[test]
    fn same_class_but_more_than_twice_the_baseline_is_pleased_slower_constant() {
        let d = decide(&facts(
            Verdict::Accepted,
            RunMode::Submit,
            measured(
                Some(ClassId::Linear),
                ClassId::Linear,
                Confidence::High,
                Some(2.5),
            ),
        ));
        assert_eq!(d.mood, Mood::PLEASED);
        assert_eq!(d.reason.code(), "slower_constant_factor");
    }

    #[test]
    fn exactly_two_times_the_baseline_still_counts_as_within_two() {
        let d = decide(&facts(
            Verdict::Accepted,
            RunMode::Submit,
            measured(
                Some(ClassId::Linear),
                ClassId::Linear,
                Confidence::High,
                Some(2.0),
            ),
        ));
        assert_eq!(d.mood, Mood::LAUGHING);
        assert_eq!(d.reason.code(), "matches_reference");
    }

    #[test]
    fn matching_the_reference_within_two_times_is_laughing_with_a_burst_on_submit() {
        let d = decide(&facts(
            Verdict::Accepted,
            RunMode::Submit,
            measured(
                Some(ClassId::Linear),
                ClassId::Linear,
                Confidence::Medium,
                Some(1.4),
            ),
        ));
        assert_eq!(d.mood, Mood::LAUGHING);
        assert_eq!(d.reason.code(), "matches_reference");
        assert_eq!(d.effect, Effect::LaughBurst);
    }

    #[test]
    fn a_run_can_laugh_but_never_bursts() {
        let d = decide(&facts(
            Verdict::Accepted,
            RunMode::Run,
            measured(
                Some(ClassId::Linear),
                ClassId::Linear,
                Confidence::Medium,
                Some(1.0),
            ),
        ));
        assert_eq!(d.mood, Mood::LAUGHING);
        assert_eq!(d.effect, Effect::None);
    }

    #[test]
    fn scaling_better_than_the_reference_within_two_times_is_laughing() {
        let d = decide(&facts(
            Verdict::Accepted,
            RunMode::Submit,
            measured(
                Some(ClassId::Linear),
                ClassId::Linearithmic,
                Confidence::High,
                Some(1.0),
            ),
        ));
        assert_eq!(d.mood, Mood::LAUGHING);
        assert_eq!(d.reason.code(), "better_than_reference");
    }

    #[test]
    fn scaling_better_but_more_than_twice_the_baseline_is_pleased() {
        let d = decide(&facts(
            Verdict::Accepted,
            RunMode::Submit,
            measured(
                Some(ClassId::Linear),
                ClassId::Linearithmic,
                Confidence::High,
                Some(3.0),
            ),
        ));
        assert_eq!(d.mood, Mood::PLEASED);
        assert_eq!(d.reason.code(), "slower_constant_factor");
    }

    #[test]
    fn a_non_finite_or_zero_ratio_is_treated_as_no_baseline() {
        for ratio in [f64::NAN, f64::INFINITY, 0.0, -1.0] {
            let d = decide(&facts(
                Verdict::Accepted,
                RunMode::Submit,
                measured(
                    Some(ClassId::Linear),
                    ClassId::Linear,
                    Confidence::High,
                    Some(ratio),
                ),
            ));
            assert_eq!(d.mood, Mood::PLEASED, "ratio {ratio}");
            assert_eq!(d.reason.code(), "no_baseline");
        }
    }

    /// Every combination of the finite inputs. The list is small, so the test is exhaustive.
    fn all_facts() -> Vec<MeasuredFacts> {
        let ratios = [None, Some(0.5), Some(2.0), Some(2.5), Some(f64::NAN)];
        let mut out = Vec::new();
        for verdict in Verdict::ALL.iter().copied() {
            for mode in [RunMode::Run, RunMode::Submit] {
                for missing in [
                    NoMeasurement::NotSpecified,
                    NoMeasurement::HostBusy,
                    NoMeasurement::Failed,
                    NoMeasurement::Incomplete,
                ] {
                    out.push(facts(verdict, mode, BenchFacts::None(missing)));
                }
                for confidence in [Confidence::Low, Confidence::Medium, Confidence::High] {
                    for observed in std::iter::once(None).chain(ClassId::ALL.map(Some)) {
                        for reference in ClassId::ALL {
                            for ratio in ratios {
                                out.push(facts(
                                    verdict,
                                    mode,
                                    measured(observed, reference, confidence, ratio),
                                ));
                            }
                        }
                    }
                }
            }
        }
        out
    }

    #[test]
    fn every_combination_obeys_the_mood_invariants() {
        for input in all_facts() {
            let d = decide(&input);
            let value = d.mood.value();
            assert!((1..=5).contains(&value));
            if input.verdict != Verdict::Accepted {
                assert_eq!(d.mood, Mood::FURIOUS, "{input:?}");
                assert_eq!(d.effect, Effect::CrackBurst, "{input:?}");
            }
            if input.verdict == Verdict::Accepted {
                assert_ne!(d.effect, Effect::CrackBurst, "{input:?}");
            }
            if let BenchFacts::Measured {
                confidence,
                observed,
                reference,
                ratio_vs_baseline,
            } = input.bench
            {
                if confidence == Confidence::Low && input.verdict == Verdict::Accepted {
                    assert_eq!(d.mood, Mood::PLEASED, "{input:?}");
                }
                if d.mood == Mood::LAUGHING && input.verdict == Verdict::Accepted {
                    let class = observed.expect("laughing needs an observed class");
                    assert!(confidence >= Confidence::Medium, "{input:?}");
                    assert!(class.rank() <= reference.rank(), "{input:?}");
                    let ratio = ratio_vs_baseline.expect("laughing needs a ratio");
                    assert!(ratio.is_finite() && ratio <= 2.0, "{input:?}");
                }
            }
            let bursts = d.effect == Effect::LaughBurst;
            assert_eq!(
                bursts,
                d.mood == Mood::LAUGHING && input.mode == RunMode::Submit,
                "{input:?}"
            );
        }
    }

    #[test]
    fn a_larger_gap_never_gives_a_larger_mood() {
        // Fix everything except the observed class, with a usable baseline.
        for reference in ClassId::ALL {
            let mut last = u8::MAX;
            for observed in ClassId::ALL {
                let d = decide(&facts(
                    Verdict::Accepted,
                    RunMode::Submit,
                    measured(Some(observed), reference, Confidence::High, Some(1.0)),
                ));
                assert!(d.mood.value() <= last, "{reference:?} vs {observed:?}");
                last = d.mood.value();
            }
        }
    }

    #[test]
    fn mood_and_reason_codes_are_stable_words_not_free_text() {
        let codes: Vec<&str> = [
            MoodReason::NoBenchSpec,
            MoodReason::HostBusy,
            MoodReason::BenchFailed,
            MoodReason::BenchIncomplete,
            MoodReason::LowConfidence,
            MoodReason::WorstTier,
            MoodReason::TwoClassesWorse,
            MoodReason::OneClassWorse,
            MoodReason::NoBaseline,
            MoodReason::SlowerConstantFactor,
            MoodReason::MatchesReference,
            MoodReason::BetterThanReference,
        ]
        .iter()
        .map(|r| r.code())
        .collect();
        assert_eq!(codes.len(), 12);
        for code in codes {
            assert!(code.len() <= 40);
            assert!(
                code.bytes().all(|b| b.is_ascii_lowercase() || b == b'_'),
                "{code}"
            );
        }
    }
}
