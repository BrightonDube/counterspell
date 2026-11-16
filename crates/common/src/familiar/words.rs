//! Plain-text wording for the mood: headline, badge and the template bubble.
//!
//! All text is built here from stored facts. Results are phrased as
//! observations ("looks quadratic, medium confidence"), never as a verdict on
//! optimality. The top mood gets its own fixed labels.

use super::class::{ClassId, Confidence};
use super::mood::MoodReason;
use crate::status::{RunMode, Verdict};

/// Phrases that must never appear in any headline or badge.
pub const FORBIDDEN_WORDS: [&str; 5] = [
    "optimal",
    "optimum",
    "best possible",
    "fastest possible",
    "cannot be improved",
];

/// The facts the wording needs. They are the stored facts, not the code or any AI text.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WordFacts<'a> {
    /// Why the mood was chosen.
    pub reason: MoodReason,
    /// Whether this was a Run or a Submit.
    pub mode: RunMode,
    /// The fitted class, present for every measured reason.
    pub observed: Option<ClassId>,
    /// The fit's confidence.
    pub confidence: Option<Confidence>,
    /// The classes the fit could not rule out.
    pub plausible: &'a [ClassId],
    /// The declared reference class. `None` when the run was not measured, so no reference is known.
    pub reference: Option<ClassId>,
    /// Observed time over the baseline, when it exists.
    pub ratio: Option<f64>,
}

/// The two short texts shown with the Familiar.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wording {
    /// The sentence shown beside the face, at most 160 characters.
    pub headline: String,
    /// The label shown with the icon, at most 40 characters.
    pub badge: String,
}

/// Builds the headline and badge from stored facts.
#[must_use]
pub fn wording(facts: &WordFacts<'_>) -> Wording {
    let (headline, badge) = match facts.reason {
        MoodReason::Verdict(verdict) => failure_wording(verdict),
        MoodReason::NoBenchSpec => (
            "Passed. Efficiency was not measured for this problem.".to_owned(),
            "Passed".to_owned(),
        ),
        MoodReason::HostBusy => (
            "Passed. The host was too busy to measure efficiency reliably.".to_owned(),
            "Passed".to_owned(),
        ),
        MoodReason::BenchFailed => (
            "Passed. The efficiency measurement failed, so no speed claim is made.".to_owned(),
            "Passed".to_owned(),
        ),
        MoodReason::BenchIncomplete => (
            "Passed. Not enough sizes finished to measure efficiency.".to_owned(),
            "Passed".to_owned(),
        ),
        MoodReason::LowConfidence => low_wording(facts),
        MoodReason::WorstTier
        | MoodReason::TwoClassesWorse
        | MoodReason::OneClassWorse
        | MoodReason::NoBaseline
        | MoodReason::SlowerConstantFactor
        | MoodReason::MatchesReference
        | MoodReason::BetterThanReference => measured_wording(facts),
    };
    let headline = if facts.mode == RunMode::Run && is_measured(facts.reason) {
        format!("Quick check: {headline}")
    } else {
        headline
    };
    Wording { headline, badge }
}

/// The headline for the bubble when no AI text is shown. It is the same text as the headline,
/// so the two can never disagree.
#[must_use]
pub fn template_bubble(facts: &WordFacts<'_>) -> String {
    wording(facts).headline
}

/// True for the reasons that come from a fitted class (Run gets the quick-check prefix).
const fn is_measured(reason: MoodReason) -> bool {
    matches!(
        reason,
        MoodReason::LowConfidence
            | MoodReason::WorstTier
            | MoodReason::TwoClassesWorse
            | MoodReason::OneClassWorse
            | MoodReason::NoBaseline
            | MoodReason::SlowerConstantFactor
            | MoodReason::MatchesReference
            | MoodReason::BetterThanReference
    )
}

/// The fixed text for a run that failed. It never mentions complexity.
fn failure_wording(verdict: Verdict) -> (String, String) {
    let (headline, badge) = match verdict {
        Verdict::WrongAnswer => ("Wrong answer.", "Wrong answer"),
        Verdict::RuntimeError => ("The program crashed.", "Crashed"),
        Verdict::TimeLimit => ("A test timed out.", "Timeout"),
        Verdict::MemoryLimit => ("The program ran out of memory.", "Out of memory"),
        Verdict::OutputLimit => ("The program printed too much output.", "Output too large"),
        Verdict::CompileError => ("The program did not compile.", "Did not compile"),
        Verdict::Accepted => ("Passed.", "Passed"),
    };
    (headline.to_owned(), badge.to_owned())
}

/// The text for a pass where the fit could not say which class fits.
fn low_wording(facts: &WordFacts<'_>) -> (String, String) {
    let reference_clause = facts.reference.map_or_else(String::new, |class| {
        format!(" The reference is {}.", class.display_name())
    });
    let names: Vec<&str> = facts.plausible.iter().map(|c| c.display_name()).collect();
    let clause = if names.is_empty() {
        String::new()
    } else if names.len() <= 3 {
        format!("; it could be {}", names.join(" or "))
    } else {
        "; it could be several classes".to_owned()
    };
    (
        format!("Passed. Unsure how this scales (low confidence){clause}.{reference_clause}"),
        "Unsure how it scales".to_owned(),
    )
}

/// The observation text for a pass with a fitted class.
fn measured_wording(facts: &WordFacts<'_>) -> (String, String) {
    let Some(observed) = facts.observed else {
        // `decide` never produces a measured reason without a class. Fall back to unsure.
        return low_wording(facts);
    };
    let confidence = facts.confidence.unwrap_or(Confidence::Low).as_str();
    let Some(reference_class) = facts.reference else {
        // A measured class always comes with its reference, so this only guards corrupt input.
        return low_wording(facts);
    };
    let looks = observed.display_name();
    let reference = reference_class.display_name();
    let same_growth = observed == reference_class;
    match facts.reason {
        MoodReason::WorstTier => (
            format!("Looks {looks}, {confidence} confidence. The reference is {reference}."),
            "Much slower growth than the reference".to_owned(),
        ),
        MoodReason::TwoClassesWorse => (
            format!("Looks {looks}, {confidence} confidence. The reference is {reference}."),
            "Two classes slower than the reference".to_owned(),
        ),
        MoodReason::OneClassWorse => (
            format!("Looks {looks}, {confidence} confidence. The reference is {reference}."),
            "One class slower than the reference".to_owned(),
        ),
        MoodReason::MatchesReference => (
            format!("Matches reference complexity: looks {looks}, {confidence} confidence."),
            "Matches reference complexity".to_owned(),
        ),
        MoodReason::BetterThanReference => (
            format!(
                "Scales better than the reference: looks {looks} (reference {reference}), {confidence} confidence."
            ),
            "Scales better than the reference".to_owned(),
        ),
        MoodReason::NoBaseline if same_growth => (
            format!(
                "Looks {looks}, {confidence} confidence: same growth as the reference. No baseline yet to compare speed."
            ),
            "No baseline to compare speed".to_owned(),
        ),
        MoodReason::NoBaseline => (
            format!(
                "Better growth than the reference: looks {looks} (reference {reference}), {confidence} confidence. No baseline yet to compare speed."
            ),
            "Better growth, no baseline".to_owned(),
        ),
        MoodReason::SlowerConstantFactor => {
            let slower = match facts.ratio {
                Some(r) if r.is_finite() && r > 0.0 => format!("about {r:.1}× slower than the baseline"),
                _ => "slower than the baseline".to_owned(),
            };
            if same_growth {
                (
                    format!("Looks {looks}, {confidence} confidence: same growth as the reference, {slower}."),
                    "Same class, slower".to_owned(),
                )
            } else {
                (
                    format!(
                        "Better growth than the reference: looks {looks} (reference {reference}), {confidence} confidence, {slower}."
                    ),
                    "Better growth, slower constant".to_owned(),
                )
            }
        }
        _ => low_wording(facts),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::familiar::class::{ClassId, Confidence};
    use crate::familiar::mood::MoodReason;
    use crate::status::{RunMode, Verdict};

    fn facts(reason: MoodReason, mode: RunMode) -> WordFacts<'static> {
        WordFacts {
            reason,
            mode,
            observed: Some(ClassId::Quadratic),
            confidence: Some(Confidence::Medium),
            plausible: &[ClassId::Quadratic],
            reference: Some(ClassId::Linear),
            ratio: Some(3.1),
        }
    }

    #[test]
    fn a_wrong_answer_has_its_literal_headline_and_badge() {
        let w = wording(&facts(
            MoodReason::Verdict(Verdict::WrongAnswer),
            RunMode::Submit,
        ));
        assert_eq!(w.headline, "Wrong answer.");
        assert_eq!(w.badge, "Wrong answer");
    }

    #[test]
    fn a_timeout_has_its_literal_headline_and_badge() {
        let w = wording(&facts(
            MoodReason::Verdict(Verdict::TimeLimit),
            RunMode::Submit,
        ));
        assert_eq!(w.headline, "A test timed out.");
        assert_eq!(w.badge, "Timeout");
    }

    #[test]
    fn a_pass_without_a_measurement_says_efficiency_was_not_measured() {
        let w = wording(&facts(MoodReason::NoBenchSpec, RunMode::Submit));
        assert_eq!(
            w.headline,
            "Passed. Efficiency was not measured for this problem."
        );
        assert_eq!(w.badge, "Passed");
    }

    #[test]
    fn low_confidence_lists_the_plausible_classes_and_the_reference() {
        let f = WordFacts {
            reason: MoodReason::LowConfidence,
            mode: RunMode::Submit,
            observed: None,
            confidence: Some(Confidence::Low),
            plausible: &[ClassId::Linear, ClassId::Linearithmic],
            reference: Some(ClassId::Linear),
            ratio: None,
        };
        let w = wording(&f);
        assert_eq!(
            w.headline,
            "Passed. Unsure how this scales (low confidence); it could be linear or n log n. The reference is linear."
        );
        assert_eq!(w.badge, "Unsure how it scales");
    }

    #[test]
    fn a_long_plausible_list_is_summarised_to_keep_the_headline_short() {
        let f = WordFacts {
            reason: MoodReason::LowConfidence,
            mode: RunMode::Submit,
            observed: None,
            confidence: Some(Confidence::Low),
            plausible: &ClassId::ALL,
            reference: Some(ClassId::Linear),
            ratio: None,
        };
        assert!(wording(&f).headline.contains("it could be several classes"));
    }

    #[test]
    fn one_class_worse_uses_the_observation_sentence() {
        let f = WordFacts {
            reason: MoodReason::OneClassWorse,
            mode: RunMode::Submit,
            observed: Some(ClassId::Linearithmic),
            confidence: Some(Confidence::Medium),
            plausible: &[ClassId::Linearithmic],
            reference: Some(ClassId::Linear),
            ratio: Some(1.0),
        };
        let w = wording(&f);
        assert_eq!(
            w.headline,
            "Looks n log n, medium confidence. The reference is linear."
        );
        assert_eq!(w.badge, "One class slower than the reference");
    }

    #[test]
    fn two_classes_worse_uses_the_observation_sentence() {
        let w = wording(&facts(MoodReason::TwoClassesWorse, RunMode::Submit));
        assert_eq!(
            w.headline,
            "Looks quadratic, medium confidence. The reference is linear."
        );
        assert_eq!(w.badge, "Two classes slower than the reference");
    }

    #[test]
    fn matching_the_reference_is_the_exact_top_label() {
        let f = WordFacts {
            reason: MoodReason::MatchesReference,
            mode: RunMode::Submit,
            observed: Some(ClassId::Linear),
            confidence: Some(Confidence::High),
            plausible: &[ClassId::Linear],
            reference: Some(ClassId::Linear),
            ratio: Some(1.0),
        };
        let w = wording(&f);
        assert_eq!(
            w.headline,
            "Matches reference complexity: looks linear, high confidence."
        );
        assert_eq!(w.badge, "Matches reference complexity");
    }

    #[test]
    fn scaling_better_than_the_reference_is_the_exact_top_label() {
        let f = WordFacts {
            reason: MoodReason::BetterThanReference,
            mode: RunMode::Submit,
            observed: Some(ClassId::Linear),
            confidence: Some(Confidence::Medium),
            plausible: &[ClassId::Linear],
            reference: Some(ClassId::Linearithmic),
            ratio: Some(1.0),
        };
        let w = wording(&f);
        assert_eq!(
            w.headline,
            "Scales better than the reference: looks linear (reference n log n), medium confidence."
        );
        assert_eq!(w.badge, "Scales better than the reference");
    }

    #[test]
    fn a_slower_constant_states_the_ratio_to_one_decimal_place() {
        let f = WordFacts {
            reason: MoodReason::SlowerConstantFactor,
            mode: RunMode::Submit,
            observed: Some(ClassId::Linear),
            confidence: Some(Confidence::Medium),
            plausible: &[ClassId::Linear],
            reference: Some(ClassId::Linear),
            ratio: Some(3.1),
        };
        let w = wording(&f);
        assert_eq!(
            w.headline,
            "Looks linear, medium confidence: same growth as the reference, about 3.1× slower than the baseline."
        );
        assert_eq!(w.badge, "Same class, slower");
    }

    #[test]
    fn a_run_adds_the_quick_check_prefix_only_to_measured_results() {
        let measured = wording(&facts(MoodReason::TwoClassesWorse, RunMode::Run)).headline;
        assert!(
            measured.starts_with("Quick check: Looks quadratic"),
            "{measured}"
        );
        let failed = wording(&facts(
            MoodReason::Verdict(Verdict::WrongAnswer),
            RunMode::Run,
        ))
        .headline;
        assert_eq!(failed, "Wrong answer.");
        let unmeasured = wording(&facts(MoodReason::NoBenchSpec, RunMode::Run)).headline;
        assert!(!unmeasured.starts_with("Quick check"), "{unmeasured}");
    }

    #[test]
    fn the_template_bubble_is_exactly_the_headline() {
        let f = facts(MoodReason::TwoClassesWorse, RunMode::Submit);
        assert_eq!(template_bubble(&f), wording(&f).headline);
    }

    /// Measured reasons always have a fitted class and a confidence of medium or high.
    /// The low-confidence reason has no class. Other reasons take any values.
    fn consistent(
        reason: MoodReason,
        observed: Option<ClassId>,
        confidence: Option<Confidence>,
    ) -> bool {
        match reason {
            MoodReason::WorstTier
            | MoodReason::TwoClassesWorse
            | MoodReason::OneClassWorse
            | MoodReason::NoBaseline
            | MoodReason::SlowerConstantFactor
            | MoodReason::MatchesReference
            | MoodReason::BetterThanReference => {
                observed.is_some()
                    && matches!(confidence, Some(Confidence::Medium | Confidence::High))
            }
            MoodReason::LowConfidence => observed.is_none(),
            _ => true,
        }
    }

    /// Every reason, with every class, confidence, ratio, mode and plausible-list shape.
    fn all_facts() -> Vec<WordFacts<'static>> {
        let mut reasons: Vec<MoodReason> = Verdict::ALL
            .iter()
            .copied()
            .filter(|v| *v != Verdict::Accepted)
            .map(MoodReason::Verdict)
            .collect();
        reasons.extend([
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
        ]);
        let plausible_shapes: [&'static [ClassId]; 3] = [&[], &[ClassId::Quadratic], &ClassId::ALL];
        let mut out = Vec::new();
        for reason in reasons {
            for mode in [RunMode::Run, RunMode::Submit] {
                for observed in [None, Some(ClassId::Constant), Some(ClassId::Exponential)] {
                    for confidence in [None, Some(Confidence::Low), Some(Confidence::High)] {
                        if !consistent(reason, observed, confidence) {
                            continue;
                        }
                        for reference in ClassId::ALL {
                            for ratio in [None, Some(0.5), Some(3.25), Some(f64::NAN)] {
                                for plausible in plausible_shapes {
                                    out.push(WordFacts {
                                        reason,
                                        mode,
                                        observed,
                                        confidence,
                                        plausible,
                                        reference: Some(reference),
                                        ratio,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
        out
    }

    #[test]
    fn every_headline_and_badge_is_short_plain_and_free_of_forbidden_words() {
        let all = all_facts();
        assert!(
            all.len() > 10_000,
            "the enumeration covers {} cases",
            all.len()
        );
        for f in &all {
            let w = wording(f);
            assert!(
                w.headline.chars().count() <= 160,
                "headline too long: {}",
                w.headline
            );
            assert!(w.badge.chars().count() <= 40, "badge too long: {}", w.badge);
            for text in [&w.headline, &w.badge] {
                assert!(!text.contains(['<', '>', '`']), "markup in {text}");
                let lower = text.to_lowercase();
                for word in FORBIDDEN_WORDS {
                    assert!(!lower.contains(word), "`{word}` in {text}");
                }
            }
        }
    }

    #[test]
    fn failure_wording_never_mentions_complexity() {
        for f in all_facts() {
            if let MoodReason::Verdict(_) = f.reason {
                let w = wording(&f);
                let lower = w.headline.to_lowercase();
                for banned in [
                    "complex",
                    "looks",
                    "scale",
                    "reference",
                    "linear",
                    "quadratic",
                ] {
                    assert!(!lower.contains(banned), "`{banned}` in {}", w.headline);
                }
            }
        }
    }

    #[test]
    fn the_top_labels_appear_only_for_the_two_top_reasons() {
        for f in all_facts() {
            let w = wording(&f);
            let text = w
                .headline
                .strip_prefix("Quick check: ")
                .unwrap_or(&w.headline);
            let top_a = text.starts_with("Matches reference complexity");
            let top_b = text.starts_with("Scales better than the reference");
            let expected = match f.reason {
                MoodReason::MatchesReference => top_a,
                MoodReason::BetterThanReference => top_b,
                _ => !top_a && !top_b,
            };
            assert!(expected, "{} for {:?}", w.headline, f.reason);
        }
    }
}
