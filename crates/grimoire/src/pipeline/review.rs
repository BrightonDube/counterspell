//! Review scores, findings and the pass rule for one review round (PIP-21 to PIP-24).
//!
//! The pass rule is applied to integer scores, so no rounding is involved. Failures
//! name the rule that failed. They never repeat the text of a finding, because that
//! text may point at hidden data (PIP-32).

use std::fmt;

use serde::{Deserialize, Serialize};

/// The number of review categories (PIP-21).
const CATEGORY_COUNT: u32 = 7;

/// The lowest mean allowed for a pass (PIP-23).
const MIN_MEAN: u32 = 4;

/// The one score that correctness must reach (PIP-23).
const CORRECTNESS_REQUIRED: u8 = 5;

/// The lowest score any category may have (PIP-23).
const CATEGORY_FLOOR: u8 = 3;

/// The lowest originality score allowed (PIP-23).
const ORIGINALITY_FLOOR: u8 = 4;

/// The seven review categories, in the order used by [`Scores`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Category {
    Correctness,
    Clarity,
    Originality,
    DifficultyCalibration,
    HintQuality,
    GhostCoverage,
    AccessibilityOfWording,
}

impl Category {
    /// Every category, in score order.
    pub const ALL: [Category; 7] = [
        Category::Correctness,
        Category::Clarity,
        Category::Originality,
        Category::DifficultyCalibration,
        Category::HintQuality,
        Category::GhostCoverage,
        Category::AccessibilityOfWording,
    ];

    /// The position of this category in [`Scores`].
    fn index(self) -> usize {
        match self {
            Category::Correctness => 0,
            Category::Clarity => 1,
            Category::Originality => 2,
            Category::DifficultyCalibration => 3,
            Category::HintQuality => 4,
            Category::GhostCoverage => 5,
            Category::AccessibilityOfWording => 6,
        }
    }
}

/// One score from 1 to 5 for each category.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Scores {
    values: [u8; 7],
}

impl Scores {
    /// Builds the scores from values in [`Category::ALL`] order.
    ///
    /// # Errors
    ///
    /// Returns [`ReviewError::ScoreOutOfRange`] for the first score outside 1 to 5.
    pub fn new(values: [u8; 7]) -> Result<Self, ReviewError> {
        for (category, value) in Category::ALL.into_iter().zip(values) {
            if !(1..=5).contains(&value) {
                return Err(ReviewError::ScoreOutOfRange(category));
            }
        }
        Ok(Self { values })
    }

    fn get(self, category: Category) -> u8 {
        self.values[category.index()]
    }

    fn sum(self) -> u32 {
        self.values.iter().map(|value| u32::from(*value)).sum()
    }
}

/// How serious a finding is (PIP-24).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Severity {
    Blocker,
    Major,
    Minor,
}

/// One finding from a review round.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Finding {
    /// The finding ID, for example `F1-3`.
    pub id: String,
    pub category: Category,
    pub severity: Severity,
    /// A file and a line, or a `/case/n` pointer. Never a hidden value.
    pub pointer: String,
    pub message: String,
    /// A grimoire code such as `ORG-01`, when a validator raised the finding.
    #[serde(default)]
    pub code: Option<String>,
}

/// The author's answer to one finding (PIP-24).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Answer {
    /// Fixed, with the commit or content hash that holds the fix.
    Fixed(String),
    /// Rebutted, with the reason.
    Rebutted(String),
}

/// One rule of the pass rule that a round failed. Display gives the rule name only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rule {
    CategoryBelowThree(Category),
    MeanBelowFour,
    CorrectnessNotFive,
    OriginalityBelowFour,
    OriginalityFinding,
    BlockerFinding,
}

impl fmt::Display for Rule {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Rule::CategoryBelowThree(category) => {
                write!(formatter, "{category:?} is below {CATEGORY_FLOOR}")
            }
            Rule::MeanBelowFour => write!(formatter, "mean is below {MIN_MEAN}"),
            Rule::CorrectnessNotFive => {
                write!(formatter, "correctness is not {CORRECTNESS_REQUIRED}")
            }
            Rule::OriginalityBelowFour => {
                write!(formatter, "originality is below {ORIGINALITY_FLOOR}")
            }
            Rule::OriginalityFinding => write!(formatter, "an ORG finding is present"),
            Rule::BlockerFinding => write!(formatter, "a blocker finding is present"),
        }
    }
}

/// Whether a round passed, and if not, which rules it broke.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Pass,
    Fail(Vec<Rule>),
}

/// The result of one review round, tied to the content hash it reviewed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Verdict {
    pub content_sha256: String,
    pub outcome: Outcome,
}

/// Why a review input or answer is refused. Messages name the field, never a value.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReviewError {
    ScoreOutOfRange(Category),
    InvalidHash,
    EmptyAnswer,
}

impl fmt::Display for ReviewError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ReviewError::ScoreOutOfRange(category) => {
                write!(formatter, "the score for {category:?} must be from 1 to 5")
            }
            ReviewError::InvalidHash => {
                write!(
                    formatter,
                    "the content hash must be 64 lowercase hex characters"
                )
            }
            ReviewError::EmptyAnswer => write!(formatter, "an answer to a finding cannot be empty"),
        }
    }
}

impl std::error::Error for ReviewError {}

/// Applies the pass rule of PIP-23 to one round.
///
/// # Errors
///
/// Returns [`ReviewError::InvalidHash`] when `content_sha256` is not 64 lowercase hex characters.
pub fn judge(
    content_sha256: &str,
    scores: &Scores,
    findings: &[Finding],
) -> Result<Verdict, ReviewError> {
    if !is_sha256_hex(content_sha256) {
        return Err(ReviewError::InvalidHash);
    }
    let mut rules = Vec::new();
    for category in Category::ALL {
        if scores.get(category) < CATEGORY_FLOOR {
            rules.push(Rule::CategoryBelowThree(category));
        }
    }
    if scores.sum() < MIN_MEAN * CATEGORY_COUNT {
        rules.push(Rule::MeanBelowFour);
    }
    if scores.get(Category::Correctness) != CORRECTNESS_REQUIRED {
        rules.push(Rule::CorrectnessNotFive);
    }
    if scores.get(Category::Originality) < ORIGINALITY_FLOOR {
        rules.push(Rule::OriginalityBelowFour);
    }
    if findings.iter().any(|finding| {
        finding
            .code
            .as_deref()
            .is_some_and(|code| code.starts_with("ORG-"))
    }) {
        rules.push(Rule::OriginalityFinding);
    }
    if findings
        .iter()
        .any(|finding| finding.severity == Severity::Blocker)
    {
        rules.push(Rule::BlockerFinding);
    }
    let outcome = if rules.is_empty() {
        Outcome::Pass
    } else {
        Outcome::Fail(rules)
    };
    Ok(Verdict {
        content_sha256: content_sha256.to_owned(),
        outcome,
    })
}

/// Refuses an answer that is empty or only spaces (PIP-24).
///
/// # Errors
///
/// Returns [`ReviewError::EmptyAnswer`] when the answer has no text.
pub fn check_answer(answer: &Answer) -> Result<(), ReviewError> {
    let text = match answer {
        Answer::Fixed(text) | Answer::Rebutted(text) => text,
    };
    if text.trim().is_empty() {
        return Err(ReviewError::EmptyAnswer);
    }
    Ok(())
}

/// Returns `true` for exactly 64 lowercase hexadecimal characters.
fn is_sha256_hex(text: &str) -> bool {
    text.len() == 64
        && text
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}
