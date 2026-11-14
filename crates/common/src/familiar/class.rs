//! Growth classes, confidence levels, moods and visual effects.
//!
//! These types are shared by the judge (which measures), the server (which
//! stores and reads) and the browser (which draws). The ids here are the
//! same strings as the stored text and the wire format.

use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

/// One growth class, from constant time up to exponential time.
///
/// The rank is the position in the list, so a larger rank means slower
/// growth. The gap between two classes is the difference of their ranks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum ClassId {
    /// Constant time, written `1`.
    Constant,
    /// Logarithmic time, written `log n`.
    Logarithmic,
    /// Linear time, written `n`.
    Linear,
    /// Linearithmic time, written `n log n`.
    Linearithmic,
    /// Quadratic time, written `n^2`.
    Quadratic,
    /// Cubic time, written `n^3`.
    Cubic,
    /// Exponential time, written `2^n`.
    Exponential,
}

impl ClassId {
    /// Every class, from the lowest rank to the highest.
    pub const ALL: [Self; 7] = [
        Self::Constant,
        Self::Logarithmic,
        Self::Linear,
        Self::Linearithmic,
        Self::Quadratic,
        Self::Cubic,
        Self::Exponential,
    ];

    /// The position of this class, from 0 (constant) to 6 (exponential).
    #[must_use]
    pub const fn rank(self) -> u8 {
        match self {
            Self::Constant => 0,
            Self::Logarithmic => 1,
            Self::Linear => 2,
            Self::Linearithmic => 3,
            Self::Quadratic => 4,
            Self::Cubic => 5,
            Self::Exponential => 6,
        }
    }

    /// The stored and wire id, for example `n log n`.
    #[must_use]
    pub const fn id(self) -> &'static str {
        match self {
            Self::Constant => "1",
            Self::Logarithmic => "log n",
            Self::Linear => "n",
            Self::Linearithmic => "n log n",
            Self::Quadratic => "n^2",
            Self::Cubic => "n^3",
            Self::Exponential => "2^n",
        }
    }

    /// The plain-English name shown to learners, for example "quadratic".
    #[must_use]
    pub const fn display_name(self) -> &'static str {
        match self {
            Self::Constant => "constant",
            Self::Logarithmic => "logarithmic",
            Self::Linear => "linear",
            Self::Linearithmic => "n log n",
            Self::Quadratic => "quadratic",
            Self::Cubic => "cubic",
            Self::Exponential => "exponential",
        }
    }
}

/// The error for text that is not a known class id.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownClass(String);

impl fmt::Display for UnknownClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown growth class `{}`", self.0)
    }
}

impl std::error::Error for UnknownClass {}

impl FromStr for ClassId {
    type Err = UnknownClass;

    /// Reads an id exactly as written. Other spellings are rejected.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|class| class.id() == text)
            .ok_or_else(|| UnknownClass(text.to_owned()))
    }
}

impl fmt::Display for ClassId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.id())
    }
}

impl Serialize for ClassId {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.id())
    }
}

impl<'de> Deserialize<'de> for ClassId {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let text = String::deserialize(deserializer)?;
        text.parse().map_err(serde::de::Error::custom)
    }
}

/// How sure the fit is about the class.
///
/// A class is reported only at `Medium` or `High`. The order is the order of
/// certainty, so `Low < Medium < High`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Confidence {
    /// The data does not single out one class.
    Low,
    /// One class fits and the checks agree.
    Medium,
    /// Strong evidence: a wide size range, low noise and a stable fit.
    High,
}

impl Confidence {
    /// The stored and wire text.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Low => "low",
            Self::Medium => "medium",
            Self::High => "high",
        }
    }
}

/// The error for a mood number outside 1 to 5.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MoodOutOfRange(pub u8);

impl fmt::Display for MoodOutOfRange {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "mood {} is outside 1 to 5", self.0)
    }
}

impl std::error::Error for MoodOutOfRange {}

/// The Familiar's mood, from 1 (furious) to 5 (laughing).
///
/// The number can only be built through [`Mood::try_from`], so a stored or
/// received mood is always in range.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Mood(u8);

impl Mood {
    /// Furious: a failing run, or the worst complexity tier.
    pub const FURIOUS: Self = Self(1);
    /// Angry: passes, but scales two classes worse than the reference.
    pub const ANGRY: Self = Self(2);
    /// Uneasy: passes, but scales one class worse than the reference.
    pub const UNEASY: Self = Self(3);
    /// Pleased: passes without a measured complaint, or with unsure results.
    pub const PLEASED: Self = Self(4);
    /// Laughing: passes and matches the reference growth.
    pub const LAUGHING: Self = Self(5);

    /// The number from 1 to 5.
    #[must_use]
    pub const fn value(self) -> u8 {
        self.0
    }

    /// The plain word for this mood, used in text and CSS classes.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self.0 {
            1 => "furious",
            2 => "angry",
            3 => "uneasy",
            4 => "pleased",
            _ => "laughing",
        }
    }
}

impl TryFrom<u8> for Mood {
    type Error = MoodOutOfRange;

    /// Accepts 1 to 5 and rejects everything else.
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if (1..=5).contains(&value) {
            Ok(Self(value))
        } else {
            Err(MoodOutOfRange(value))
        }
    }
}

impl From<Mood> for u8 {
    fn from(mood: Mood) -> Self {
        mood.0
    }
}

impl Serialize for Mood {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_u8(self.0)
    }
}

impl<'de> Deserialize<'de> for Mood {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = u8::deserialize(deserializer)?;
        Self::try_from(value).map_err(serde::de::Error::custom)
    }
}

/// The one visual effect a result may play.
///
/// The effect is decided by the judge and sent with a live event. It is never
/// stored or sent in a reload, so a reload cannot replay an animation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Effect {
    /// No animation.
    None,
    /// The Familiar cracks and bursts. Used for every failing verdict.
    CrackBurst,
    /// The Familiar laughs. Used only for mood 5 on a Submit.
    LaughBurst,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ranks_run_from_zero_to_six_in_order() {
        let ranks: Vec<u8> = ClassId::ALL.iter().map(|c| c.rank()).collect();
        assert_eq!(ranks, vec![0, 1, 2, 3, 4, 5, 6]);
    }

    #[test]
    fn every_class_id_round_trips_through_text() {
        let texts = ["1", "log n", "n", "n log n", "n^2", "n^3", "2^n"];
        for (class, text) in ClassId::ALL.iter().zip(texts) {
            assert_eq!(class.id(), text);
            assert_eq!(text.parse::<ClassId>(), Ok(*class));
        }
    }

    #[test]
    fn unknown_or_differently_spelled_class_ids_are_rejected() {
        for text in ["n^1.5", "O(n)", "N", "n log(n)", "n**2", " n", "", "2^n "] {
            assert!(text.parse::<ClassId>().is_err(), "accepted `{text}`");
        }
    }

    #[test]
    fn class_serde_form_is_the_id_string() {
        let json = serde_json::to_string(&ClassId::Linearithmic).expect("serialise class");
        assert_eq!(json, "\"n log n\"");
        let back: ClassId = serde_json::from_str("\"n^2\"").expect("parse class");
        assert_eq!(back, ClassId::Quadratic);
        assert!(serde_json::from_str::<ClassId>("\"n^1.5\"").is_err());
    }

    #[test]
    fn confidence_orders_low_below_medium_below_high() {
        assert!(Confidence::Low < Confidence::Medium);
        assert!(Confidence::Medium < Confidence::High);
        assert_eq!(Confidence::Medium.as_str(), "medium");
        let json = serde_json::to_string(&Confidence::High).expect("serialise confidence");
        assert_eq!(json, "\"high\"");
    }

    #[test]
    fn mood_accepts_one_to_five_and_rejects_the_rest() {
        for value in 1..=5u8 {
            let mood = Mood::try_from(value).expect("in range");
            assert_eq!(mood.value(), value);
        }
        assert_eq!(Mood::try_from(0), Err(MoodOutOfRange(0)));
        assert_eq!(Mood::try_from(6), Err(MoodOutOfRange(6)));
        assert_eq!(Mood::try_from(255), Err(MoodOutOfRange(255)));
    }

    #[test]
    fn mood_names_follow_the_five_words() {
        let names: Vec<&str> = (1..=5u8)
            .map(|v| Mood::try_from(v).expect("in range").name())
            .collect();
        assert_eq!(
            names,
            vec!["furious", "angry", "uneasy", "pleased", "laughing"]
        );
    }

    #[test]
    fn mood_serde_is_a_number_and_out_of_range_input_fails() {
        let json = serde_json::to_string(&Mood::LAUGHING).expect("serialise mood");
        assert_eq!(json, "5");
        assert!(serde_json::from_str::<Mood>("0").is_err());
        assert!(serde_json::from_str::<Mood>("6").is_err());
        let back: Mood = serde_json::from_str("2").expect("parse mood");
        assert_eq!(back, Mood::ANGRY);
    }

    #[test]
    fn effect_serde_form_is_snake_case() {
        let cases = [
            (Effect::None, "\"none\""),
            (Effect::CrackBurst, "\"crack_burst\""),
            (Effect::LaughBurst, "\"laugh_burst\""),
        ];
        for (effect, text) in cases {
            assert_eq!(serde_json::to_string(&effect).expect("serialise"), text);
        }
    }
}
