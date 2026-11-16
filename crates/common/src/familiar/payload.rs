//! Wire payloads and stored columns for the measured layer.
//!
//! Every field is a number, a class id, a mood, a reason code, or fixed text
//! written by `words`. No field can hold source code, a hidden input, an
//! expected output or stderr. Inputs use `deny_unknown_fields`, so an extra
//! key from a newer or hostile writer is rejected, not silently kept.

use super::class::{ClassId, Confidence, Effect, Mood};
use crate::events::{BoundedJson, BoundedJsonError};
use crate::status::RunMode;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The fit version written into every stored and streamed result.
pub const FIT_VERSION: &str = "fit-1";

/// The most points kept in the stored `fit` summary.
pub const MAX_STORED_POINTS: usize = 8;

/// Where a mood came from. Only measurements exist today, so only one value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layer {
    /// Set by the judge from measured facts.
    Measured,
}

/// Why a bench did or did not give a growth measurement, as stored and sent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BenchStatus {
    /// The bench ran and was fitted.
    Measured,
    /// The problem declares no bench.
    NotSpecified,
    /// The host was busy, so the timings were discarded.
    HostBusy,
    /// The bench call failed.
    Failed,
    /// Too few sizes finished inside the budget.
    Incomplete,
}

impl BenchStatus {
    /// The stored and wire text.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Measured => "measured",
            Self::NotSpecified => "not_specified",
            Self::HostBusy => "host_busy",
            Self::Failed => "failed",
            Self::Incomplete => "incomplete",
        }
    }
}

/// One measured size: the input size `n` and the median time in nanoseconds.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PointSummary {
    /// The input size.
    pub n: u64,
    /// The median of the repeats at this size.
    pub median_ns: u64,
}

/// Keeps at most [`MAX_STORED_POINTS`] points: the first four and the last four.
///
/// The fit uses every point, but the stored summary keeps the two ends,
/// which show the overall curve and fit within the 4 KiB column limit.
#[must_use]
pub fn capped_points(points: &[PointSummary]) -> Vec<PointSummary> {
    if points.len() <= MAX_STORED_POINTS {
        return points.to_vec();
    }
    let half = MAX_STORED_POINTS / 2;
    let mut kept = points[..half].to_vec();
    kept.extend_from_slice(&points[points.len() - half..]);
    kept
}

/// The `complexity` event. Sent only when a bench was fitted.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComplexityPayload {
    /// The reported class, or `null` when the fit did not report one.
    pub class: Option<ClassId>,
    /// How sure the fit is.
    pub confidence: Confidence,
    /// The class the author declared as the reference.
    pub reference: ClassId,
    /// Every class the fit could not rule out.
    pub plausible: Vec<ClassId>,
    /// The lowest-residual class, which is informational when unsure.
    pub leaning: ClassId,
    /// Observed time over the same-language baseline, or `null` when no baseline is usable.
    pub ratio_vs_baseline: Option<f64>,
    /// How many sizes were fitted.
    pub points: usize,
    /// The fit version that produced this result.
    pub fit_version: String,
    /// Whether this was a Run or a Submit.
    pub mode: RunMode,
}

impl ComplexityPayload {
    /// Turns the payload into an event object, refusing anything over the event cap.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload is over the event size cap.
    pub fn to_bounded_json(&self) -> Result<BoundedJson, BoundedJsonError> {
        BoundedJson::from_serialize(self)
    }
}

/// The `familiar` event. Sent once per completed attempt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FamiliarPayload {
    /// The mood from 1 to 5.
    pub mood: Mood,
    /// The reason code, for example `two_classes_worse`.
    pub reason: String,
    /// Where the mood came from.
    pub layer: Layer,
    /// The one effect to play on a live event.
    pub effect: Effect,
    /// The plain-text headline, from the wording rules.
    pub headline: String,
    /// The short plain-text badge.
    pub badge: String,
    /// Whether this was a Run or a Submit.
    pub mode: RunMode,
}

impl FamiliarPayload {
    /// Turns the payload into an event object, refusing anything over the event cap.
    ///
    /// # Errors
    ///
    /// Returns an error when the payload is over the event size cap.
    pub fn to_bounded_json(&self) -> Result<BoundedJson, BoundedJsonError> {
        BoundedJson::from_serialize(self)
    }
}

/// The compact fit summary stored in `submissions.fit` and shown in the explainer.
///
/// When nothing was fitted, only `fit_version`, `mode`, `bench_status` and
/// `reference` are written; every other field is left out.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FitSummary {
    /// The fit version.
    pub fit_version: String,
    /// Whether this was a Run or a Submit.
    pub mode: RunMode,
    /// Why there is or is not a measurement.
    pub bench_status: BenchStatus,
    /// The declared reference class.
    pub reference: ClassId,
    /// The reported class, if any.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub class: Option<ClassId>,
    /// The confidence, if a fit ran.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub confidence: Option<Confidence>,
    /// The classes the fit could not rule out.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub plausible: Vec<ClassId>,
    /// The lowest-residual class.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub leaning: Option<ClassId>,
    /// At most [`MAX_STORED_POINTS`] measured points.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub points: Vec<PointSummary>,
    /// The log-log slope over the top points.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub slope: Option<f64>,
    /// The standard error of the slope.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub slope_se: Option<f64>,
    /// The local exponents between consecutive points.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub exponents: Vec<f64>,
    /// The residual of each model, keyed by class id.
    #[serde(skip_serializing_if = "BTreeMap::is_empty", default)]
    pub residuals: BTreeMap<String, f64>,
    /// The relative noise across the repeats.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub noise: Option<f64>,
    /// The largest size over the smallest size.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub range_ratio: Option<f64>,
    /// Observed time over the same-language baseline.
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub ratio_vs_baseline: Option<f64>,
    /// The reason codes that explain the confidence.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub reasons: Vec<String>,
}

impl FitSummary {
    /// The summary for a run that produced no fit.
    #[must_use]
    pub fn not_fitted(mode: RunMode, bench_status: BenchStatus, reference: ClassId) -> Self {
        Self {
            fit_version: FIT_VERSION.to_owned(),
            mode,
            bench_status,
            reference,
            class: None,
            confidence: None,
            plausible: Vec::new(),
            leaning: None,
            points: Vec::new(),
            slope: None,
            slope_se: None,
            exponents: Vec::new(),
            residuals: BTreeMap::new(),
            noise: None,
            range_ratio: None,
            ratio_vs_baseline: None,
            reasons: Vec::new(),
        }
    }
}

/// The five values the terminal write stores. They come from the same value as the events.
#[derive(Debug, Clone, PartialEq)]
pub struct MeasuredColumns {
    /// Stored only with `medium` or `high` confidence.
    pub measured_complexity: Option<ClassId>,
    /// Stored only when a fit ran.
    pub complexity_confidence: Option<Confidence>,
    /// The compact fit summary.
    pub fit: Option<FitSummary>,
    /// The mood, only for a completed run.
    pub familiar_mood: Option<Mood>,
    /// The reason code that goes with the mood.
    pub familiar_reason: Option<String>,
}

/// True when text matches the stored reason pattern `^[a-z_]{1,40}$`.
#[must_use]
pub fn is_reason_code(text: &str) -> bool {
    (1..=40).contains(&text.len()) && text.bytes().all(|b| b.is_ascii_lowercase() || b == b'_')
}

/// The read-model section for a fitted run, built only from stored columns.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComplexitySection {
    /// The reported class, or `null`.
    pub class: Option<ClassId>,
    /// The confidence.
    pub confidence: Confidence,
    /// The declared reference class.
    pub reference: ClassId,
    /// The classes the fit could not rule out.
    pub plausible: Vec<ClassId>,
    /// The stored points.
    pub points: Vec<PointSummary>,
    /// The log-log slope.
    pub slope: Option<f64>,
    /// The standard error of the slope.
    pub slope_se: Option<f64>,
    /// The residual of each model.
    pub residuals: BTreeMap<String, f64>,
    /// Observed over baseline, or `null`.
    pub ratio_vs_baseline: Option<f64>,
    /// The fit version.
    pub fit_version: String,
}

/// The read-model section for the mood. It has no effect, so a reload never replays an animation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FamiliarSection {
    /// The mood from 1 to 5.
    pub mood: Mood,
    /// The reason code.
    pub reason: String,
    /// The headline, rebuilt from stored facts.
    pub headline: String,
    /// The badge, rebuilt from stored facts.
    pub badge: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::familiar::class::{ClassId, Confidence, Effect, Mood};
    use crate::status::RunMode;
    use serde_json::json;

    fn sample_complexity() -> ComplexityPayload {
        ComplexityPayload {
            class: Some(ClassId::Quadratic),
            confidence: Confidence::Medium,
            reference: ClassId::Linear,
            plausible: vec![ClassId::Quadratic],
            leaning: ClassId::Quadratic,
            ratio_vs_baseline: Some(3.1),
            points: 5,
            fit_version: "fit-1".to_owned(),
            mode: RunMode::Submit,
        }
    }

    fn sample_familiar() -> FamiliarPayload {
        FamiliarPayload {
            mood: Mood::ANGRY,
            reason: "two_classes_worse".to_owned(),
            layer: Layer::Measured,
            effect: Effect::None,
            headline: "Looks quadratic, medium confidence. The reference is linear.".to_owned(),
            badge: "Two classes slower than the reference".to_owned(),
            mode: RunMode::Submit,
        }
    }

    #[test]
    fn complexity_event_has_the_documented_wire_shape() {
        let value = serde_json::to_value(sample_complexity()).expect("serialise");
        assert_eq!(
            value,
            json!({
                "class": "n^2",
                "confidence": "medium",
                "reference": "n",
                "plausible": ["n^2"],
                "leaning": "n^2",
                "ratio_vs_baseline": 3.1,
                "points": 5,
                "fit_version": "fit-1",
                "mode": "submit"
            })
        );
    }

    #[test]
    fn complexity_event_sends_a_null_class_when_nothing_was_reported() {
        let mut payload = sample_complexity();
        payload.class = None;
        payload.confidence = Confidence::Low;
        payload.ratio_vs_baseline = None;
        let value = serde_json::to_value(&payload).expect("serialise");
        assert!(value["class"].is_null());
        assert!(value["ratio_vs_baseline"].is_null());
    }

    #[test]
    fn familiar_event_has_the_documented_wire_shape() {
        let value = serde_json::to_value(sample_familiar()).expect("serialise");
        assert_eq!(
            value,
            json!({
                "mood": 2,
                "reason": "two_classes_worse",
                "layer": "measured",
                "effect": "none",
                "headline": "Looks quadratic, medium confidence. The reference is linear.",
                "badge": "Two classes slower than the reference",
                "mode": "submit"
            })
        );
    }

    #[test]
    fn unknown_fields_are_rejected_on_every_input_type() {
        let complexity = json!({
            "class": "n", "confidence": "medium", "reference": "n", "plausible": ["n"],
            "leaning": "n", "ratio_vs_baseline": null, "points": 3, "fit_version": "fit-1",
            "mode": "submit", "source": "int main() {}"
        });
        assert!(serde_json::from_value::<ComplexityPayload>(complexity).is_err());

        let familiar = json!({
            "mood": 4, "reason": "no_bench_spec", "layer": "measured", "effect": "none",
            "headline": "h", "badge": "b", "mode": "submit", "stderr": "boom"
        });
        assert!(serde_json::from_value::<FamiliarPayload>(familiar).is_err());
    }

    #[test]
    fn a_mood_outside_one_to_five_cannot_be_read_from_a_payload() {
        let zero = json!({
            "mood": 0, "reason": "no_bench_spec", "layer": "measured", "effect": "none",
            "headline": "h", "badge": "b", "mode": "submit"
        });
        assert!(serde_json::from_value::<FamiliarPayload>(zero).is_err());
    }

    #[test]
    fn a_familiar_payload_fits_the_event_cap() {
        assert!(sample_familiar().to_bounded_json().is_ok());
    }

    #[test]
    fn an_oversized_headline_is_refused_by_the_event_cap() {
        let mut payload = sample_familiar();
        payload.headline = "x".repeat(20_000);
        assert!(payload.to_bounded_json().is_err());
    }

    #[test]
    fn a_maximal_fit_summary_keeps_at_most_eight_points_and_stays_under_four_kib() {
        let points: Vec<PointSummary> = (0..32u64)
            .map(|i| PointSummary {
                n: u64::MAX - i,
                median_ns: u64::MAX - i,
            })
            .collect();
        let mut residuals = std::collections::BTreeMap::new();
        for class in ClassId::ALL {
            residuals.insert(class.id().to_owned(), 1_000_000.5);
        }
        let summary = FitSummary {
            fit_version: "fit-1".to_owned(),
            mode: RunMode::Submit,
            bench_status: BenchStatus::Measured,
            reference: ClassId::Linear,
            class: Some(ClassId::Quadratic),
            confidence: Some(Confidence::Medium),
            plausible: ClassId::ALL.to_vec(),
            leaning: Some(ClassId::Quadratic),
            points: capped_points(&points),
            slope: Some(1.96),
            slope_se: Some(0.08),
            exponents: vec![2.0; 31],
            residuals,
            noise: Some(0.05),
            range_ratio: Some(16.0),
            ratio_vs_baseline: Some(3.1),
            reasons: vec!["two_classes_worse".to_owned()],
        };
        assert_eq!(summary.points.len(), 8);
        let text = serde_json::to_string(&summary).expect("serialise");
        assert!(text.len() < 4096, "fit is {} bytes", text.len());
    }

    #[test]
    fn a_fit_summary_for_a_bench_that_was_not_fitted_holds_only_the_basic_fields() {
        let summary = FitSummary::not_fitted(RunMode::Run, BenchStatus::HostBusy, ClassId::Linear);
        let value = serde_json::to_value(&summary).expect("serialise");
        assert_eq!(
            value,
            json!({
                "fit_version": "fit-1",
                "mode": "run",
                "bench_status": "host_busy",
                "reference": "n"
            })
        );
    }

    #[test]
    fn bench_status_has_the_five_stored_words() {
        let texts: Vec<&str> = [
            BenchStatus::Measured,
            BenchStatus::NotSpecified,
            BenchStatus::HostBusy,
            BenchStatus::Failed,
            BenchStatus::Incomplete,
        ]
        .iter()
        .map(|s| s.as_str())
        .collect();
        assert_eq!(
            texts,
            vec![
                "measured",
                "not_specified",
                "host_busy",
                "failed",
                "incomplete"
            ]
        );
    }

    #[test]
    fn reason_codes_match_the_database_pattern() {
        for good in ["no_bench_spec", "n", "two_classes_worse"] {
            assert!(is_reason_code(good), "{good}");
        }
        let too_long = "a".repeat(41);
        for bad in [
            "",
            "Wrong",
            "n log n",
            "two-classes",
            "x1",
            too_long.as_str(),
        ] {
            assert!(!is_reason_code(bad), "{bad}");
        }
    }
}
