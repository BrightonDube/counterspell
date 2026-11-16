//! The `complexity` and `familiar` sections of the run read model (efficiency-familiar R7.4, R7.8).
//!
//! Both sections are rebuilt from the stored measured columns only. The headline and badge are not
//! stored; they come from the same pure wording function the judge uses, so a reload shows the text the
//! live event showed. Nothing here reads code, hidden data or any AI text, and nothing here writes.

use counterspell_common::{
    familiar::{
        mood::MoodReason,
        payload::{ComplexitySection, FamiliarSection, FitSummary},
        words::{wording, WordFacts},
        Mood,
    },
    status::RunMode,
};

/// The measured columns of one run, as the read model loads them.
#[derive(Debug)]
pub struct StoredMeasure {
    /// The run's mode, from the `mode` column.
    pub mode: RunMode,
    /// The stored mood from 1 to 5, or `None` when the run has no measured mood.
    pub mood: Option<i16>,
    /// The stored reason code, present whenever a mood is.
    pub reason: Option<String>,
    /// The stored `fit` JSON, or `None` when nothing was fitted.
    pub fit: Option<serde_json::Value>,
}

/// The two optional read-model sections. Each is `None` when its stored data is absent.
#[derive(Debug, PartialEq)]
pub struct Sections {
    /// The fitted complexity, when the fit reports a confidence.
    pub complexity: Option<ComplexitySection>,
    /// The mood, reason and wording, when a mood is stored.
    pub familiar: Option<FamiliarSection>,
}

/// A stored value that breaks the shape the judge writes. This never happens through the judge, so the
/// caller reports it as an internal error instead of showing a guessed result.
#[derive(Debug, PartialEq, Eq)]
pub enum SectionError {
    /// The `fit` JSON does not match the fit summary shape.
    BadFit,
    /// The mood is outside 1 to 5.
    BadMood,
    /// A mood is stored without a known reason code.
    BadReason,
}

impl std::fmt::Display for SectionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let text = match self {
            Self::BadFit => "stored fit does not match the fit summary shape",
            Self::BadMood => "stored mood is outside 1 to 5",
            Self::BadReason => "stored mood has no known reason code",
        };
        f.write_str(text)
    }
}

/// Builds both sections from the stored columns.
///
/// # Errors
///
/// Returns a [`SectionError`] when a stored value does not have the shape the judge writes.
pub fn sections(stored: &StoredMeasure) -> Result<Sections, SectionError> {
    let fit: Option<FitSummary> = stored
        .fit
        .clone()
        .map(serde_json::from_value)
        .transpose()
        .map_err(|_| SectionError::BadFit)?;
    Ok(Sections {
        complexity: complexity_section(fit.as_ref()),
        familiar: familiar_section(stored, fit.as_ref())?,
    })
}

/// The complexity section is present only when the fit reports a confidence.
fn complexity_section(fit: Option<&FitSummary>) -> Option<ComplexitySection> {
    let fit = fit?;
    let confidence = fit.confidence?;
    Some(ComplexitySection {
        class: fit.class,
        confidence,
        reference: fit.reference,
        plausible: fit.plausible.clone(),
        points: fit.points.clone(),
        slope: fit.slope,
        slope_se: fit.slope_se,
        residuals: fit.residuals.clone(),
        ratio_vs_baseline: fit.ratio_vs_baseline,
        fit_version: fit.fit_version.clone(),
    })
}

/// The familiar section is present only when a mood is stored.
fn familiar_section(
    stored: &StoredMeasure,
    fit: Option<&FitSummary>,
) -> Result<Option<FamiliarSection>, SectionError> {
    let Some(raw_mood) = stored.mood else {
        return Ok(None);
    };
    let mood = u8::try_from(raw_mood)
        .ok()
        .and_then(|value| Mood::try_from(value).ok())
        .ok_or(SectionError::BadMood)?;
    let reason_code = stored.reason.as_deref().ok_or(SectionError::BadReason)?;
    let reason = MoodReason::from_code(reason_code).ok_or(SectionError::BadReason)?;
    let plausible: &[_] = fit.map_or(&[], |f| f.plausible.as_slice());
    let text = wording(&WordFacts {
        reason,
        mode: stored.mode,
        observed: fit.and_then(|f| f.class),
        confidence: fit.and_then(|f| f.confidence),
        plausible,
        reference: fit.map(|f| f.reference),
        ratio: fit.and_then(|f| f.ratio_vs_baseline),
    });
    Ok(Some(FamiliarSection {
        mood,
        reason: reason.code().to_owned(),
        headline: text.headline,
        badge: text.badge,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use counterspell_common::familiar::class::{ClassId, Confidence};

    /// A fit that was measured, with the given class, confidence and reference.
    fn fit_json(class: ClassId, confidence: Confidence, reference: ClassId) -> serde_json::Value {
        let summary = FitSummary {
            fit_version: "fit-1".to_owned(),
            mode: RunMode::Submit,
            bench_status: counterspell_common::familiar::payload::BenchStatus::Measured,
            reference,
            class: Some(class),
            confidence: Some(confidence),
            plausible: vec![class],
            leaning: Some(class),
            points: Vec::new(),
            slope: Some(1.0),
            slope_se: Some(0.05),
            exponents: Vec::new(),
            residuals: std::collections::BTreeMap::new(),
            noise: None,
            range_ratio: None,
            ratio_vs_baseline: None,
            reasons: Vec::new(),
        };
        serde_json::to_value(summary).expect("a fit summary always serialises")
    }

    fn stored(
        mood: Option<i16>,
        reason: Option<&str>,
        fit: Option<serde_json::Value>,
    ) -> StoredMeasure {
        StoredMeasure {
            mode: RunMode::Submit,
            mood,
            reason: reason.map(str::to_owned),
            fit,
        }
    }

    // Breaks if a run with nothing stored shows an invented section.
    #[test]
    fn a_run_with_no_measured_columns_has_no_sections() {
        let result = sections(&stored(None, None, None)).expect("empty columns are valid");
        assert_eq!(
            result,
            Sections {
                complexity: None,
                familiar: None
            }
        );
    }

    // Breaks if a matched result shows the wrong sentence, so the reload differs from the live event.
    #[test]
    fn a_matched_result_reads_back_with_its_fixed_headline_and_badge() {
        let fit = fit_json(ClassId::Linear, Confidence::Medium, ClassId::Linear);
        let result = sections(&stored(Some(5), Some("matches_reference"), Some(fit)))
            .expect("a valid matched result");
        let familiar = result.familiar.expect("a mood is stored");
        assert_eq!(familiar.mood.value(), 5);
        assert_eq!(familiar.reason, "matches_reference");
        assert_eq!(
            familiar.headline,
            "Matches reference complexity: looks linear, medium confidence."
        );
        assert_eq!(familiar.badge, "Matches reference complexity");
    }

    // Breaks if the complexity section carries a class that the fit did not report.
    #[test]
    fn the_complexity_section_carries_the_fitted_class_and_confidence() {
        let fit = fit_json(ClassId::Quadratic, Confidence::Medium, ClassId::Linear);
        let result = sections(&stored(Some(2), Some("two_classes_worse"), Some(fit)))
            .expect("a valid fitted result");
        let complexity = result.complexity.expect("a confidence is stored");
        assert_eq!(complexity.class, Some(ClassId::Quadratic));
        assert_eq!(complexity.confidence, Confidence::Medium);
        assert_eq!(complexity.reference, ClassId::Linear);
        assert_eq!(complexity.fit_version, "fit-1");
    }

    // Breaks if a fit with no confidence shows a complexity section, which would claim a class nobody reported.
    #[test]
    fn a_fit_without_confidence_has_no_complexity_section() {
        let summary = FitSummary::not_fitted(
            RunMode::Submit,
            counterspell_common::familiar::payload::BenchStatus::HostBusy,
            ClassId::Linear,
        );
        let fit = serde_json::to_value(summary).expect("serialise");
        let result = sections(&stored(Some(4), Some("host_busy"), Some(fit))).expect("valid");
        assert!(result.complexity.is_none());
        assert!(result.familiar.is_some());
    }

    // Breaks if a failed run gets complexity wording, which would talk about speed on a wrong answer.
    #[test]
    fn a_failed_run_reads_back_its_failure_wording_without_a_complexity_section() {
        let result = sections(&stored(Some(1), Some("wrong_answer"), None)).expect("valid failure");
        assert!(result.complexity.is_none());
        let familiar = result.familiar.expect("a mood is stored");
        assert_eq!(familiar.headline, "Wrong answer.");
        assert_eq!(familiar.badge, "Wrong answer");
    }

    // Breaks if a mood with an unknown reason is shown anyway, so a corrupt row looks like a valid result.
    #[test]
    fn an_unknown_reason_code_is_an_error_not_a_guess() {
        let result = sections(&stored(Some(3), Some("made_up_reason"), None));
        assert_eq!(result, Err(SectionError::BadReason));
    }

    // Breaks if a mood with no reason is shown, since the wording needs the reason.
    #[test]
    fn a_mood_without_a_reason_is_an_error() {
        assert_eq!(
            sections(&stored(Some(3), None, None)),
            Err(SectionError::BadReason)
        );
    }

    // Breaks if an out-of-range mood is accepted, because the database check would have refused it.
    #[test]
    fn a_mood_outside_one_to_five_is_an_error() {
        assert_eq!(
            sections(&stored(Some(6), Some("wrong_answer"), None)),
            Err(SectionError::BadMood)
        );
    }

    // Breaks if a malformed fit is read as an empty one, which would hide the damage.
    #[test]
    fn a_malformed_fit_is_an_error() {
        let fit = serde_json::json!({ "not": "a fit" });
        assert_eq!(
            sections(&stored(Some(4), Some("host_busy"), Some(fit))),
            Err(SectionError::BadFit)
        );
    }
}
