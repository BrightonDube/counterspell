//! The only module that talks to `fsrs`.
//!
//! `PersistedCard` mirrors `concept_cards`; `ReviewCalculation` is the
//! application-owned result later persisted to `concept_review_history`.

use fsrs::{MemoryState, FSRS};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use time::{Duration, OffsetDateTime};

const MAX_INTERVAL_SECONDS: f64 = 9_223_372_036_854_775_807.0;

/// A persisted FSRS card.
#[derive(Debug, Clone, PartialEq)]
pub struct PersistedCard {
    /// Long-term memory stability.
    pub stability: f64,
    /// FSRS-7's fast stability trace.
    pub stability_fast: f64,
    /// Memory difficulty.
    pub difficulty: f64,
    /// Successful scheduler repetitions.
    pub reps: u32,
    /// Failed review count.
    pub lapses: u32,
    /// The card lifecycle state persisted as text in PostgreSQL.
    pub state: CardState,
    /// When the card becomes due.
    pub due: OffsetDateTime,
    /// The most recent review. Required for every non-new card.
    pub last_review: Option<OffsetDateTime>,
    /// Optimistic concurrency version.
    pub version: i64,
}

/// The lifecycle states supported by `concept_cards.state`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CardState {
    /// No successful FSRS review has been persisted.
    New,
    /// The card is in short-term learning.
    Learning,
    /// The card is in normal review scheduling.
    Review,
    /// The card is being relearned after a failed review.
    Relearning,
}

impl CardState {
    /// The PostgreSQL text representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::New => "new",
            Self::Learning => "learning",
            Self::Review => "review",
            Self::Relearning => "relearning",
        }
    }

    /// Parses the PostgreSQL text representation.
    ///
    /// # Errors
    /// Returns `ReviewError::UnsupportedState` for any other value.
    pub fn parse(value: &str) -> Result<Self, ReviewError> {
        match value {
            "new" => Ok(Self::New),
            "learning" => Ok(Self::Learning),
            "review" => Ok(Self::Review),
            "relearning" => Ok(Self::Relearning),
            _ => Err(ReviewError::UnsupportedState),
        }
    }
}

/// The only grades accepted by the review API.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ReviewGrade {
    /// The learner forgot the concept.
    Again,
    /// The learner recalled it with difficulty.
    Hard,
    /// The learner recalled it normally.
    Good,
    /// The learner recalled it easily.
    Easy,
}

impl ReviewGrade {
    /// The stable external text representation.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Again => "again",
            Self::Hard => "hard",
            Self::Good => "good",
            Self::Easy => "easy",
        }
    }

    /// Parses an external grade.
    ///
    /// # Errors
    /// Returns `ReviewError::UnsupportedGrade` for any other value.
    pub fn parse(value: &str) -> Result<Self, ReviewError> {
        match value {
            "again" => Ok(Self::Again),
            "hard" => Ok(Self::Hard),
            "good" => Ok(Self::Good),
            "easy" => Ok(Self::Easy),
            _ => Err(ReviewError::UnsupportedGrade),
        }
    }
}

/// Scheduler configuration that callers can inject.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SchedulerConfig {
    /// Target probability of recall, strictly between zero and one.
    pub desired_retention: f32,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            desired_retention: 0.9,
        }
    }
}

/// The safe error surface for adapter failures.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ReviewError {
    /// The request used a grade outside the public contract.
    #[error("unsupported review grade")]
    UnsupportedGrade,
    /// The stored card used a lifecycle state this adapter does not support.
    #[error("unsupported card state")]
    UnsupportedState,
    /// Persisted card data could not produce a valid scheduler state.
    #[error("invalid persisted card")]
    InvalidCard,
    /// Scheduler configuration was outside its supported range.
    #[error("invalid scheduler configuration")]
    InvalidConfig,
    /// The third-party scheduler rejected the calculation.
    #[error("scheduler calculation failed")]
    Scheduler,
    /// The scheduler returned data that violated the adapter's invariants.
    #[error("invalid scheduler output")]
    InvalidOutput,
}

/// An application-owned scheduling calculation.
#[derive(Debug, Clone, PartialEq)]
pub struct ReviewCalculation {
    /// The card state read before the review.
    pub previous_card: PersistedCard,
    /// The card state to persist after the review.
    pub next_card: PersistedCard,
    /// The accepted review grade.
    pub grade: ReviewGrade,
    /// The injected review timestamp.
    pub reviewed_at: OffsetDateTime,
    /// The non-negative interval until the next review.
    pub next_interval: Duration,
    /// The calculated due timestamp.
    pub due: OffsetDateTime,
}

/// The application adapter around the `fsrs` crate.
#[derive(Debug, Clone, Default)]
pub struct FsrsScheduler {
    scheduler: FSRS,
    config: SchedulerConfig,
}

impl FsrsScheduler {
    /// Builds an adapter with caller-provided configuration.
    ///
    /// # Errors
    /// Returns `ReviewError::InvalidConfig` when retention is not finite or not in `(0, 1)`.
    pub fn new(config: SchedulerConfig) -> Result<Self, ReviewError> {
        if !config.desired_retention.is_finite()
            || config.desired_retention <= 0.0
            || config.desired_retention >= 1.0
        {
            return Err(ReviewError::InvalidConfig);
        }
        Ok(Self {
            scheduler: FSRS::default(),
            config,
        })
    }

    /// Schedules one review at an explicitly injected timestamp.
    ///
    /// # Errors
    /// Returns an application-owned error when persisted state, elapsed time, the
    /// scheduler call, or the scheduler result violates an adapter invariant.
    pub fn review(
        &self,
        card: &PersistedCard,
        grade: ReviewGrade,
        reviewed_at: OffsetDateTime,
    ) -> Result<ReviewCalculation, ReviewError> {
        Self::validate_card(card)?;
        let memory = Self::memory_state(card)?;
        let elapsed_days = Self::elapsed_days(card, reviewed_at)?;
        let next_states = self
            .scheduler
            .next_states_with_elapsed_days(memory, self.config.desired_retention, elapsed_days)
            .map_err(|_| ReviewError::Scheduler)?;
        let item = match grade {
            ReviewGrade::Again => next_states.again,
            ReviewGrade::Hard => next_states.hard,
            ReviewGrade::Good => next_states.good,
            ReviewGrade::Easy => next_states.easy,
        };
        Self::validate_output(item.memory, item.interval)?;

        let interval_seconds = f64::from(item.interval) * 86_400.0;
        if interval_seconds > MAX_INTERVAL_SECONDS {
            return Err(ReviewError::InvalidOutput);
        }
        let next_interval = Duration::seconds_f64(interval_seconds);
        let due = reviewed_at
            .checked_add(next_interval)
            .ok_or(ReviewError::InvalidOutput)?;

        let mut next_card = card.clone();
        next_card.stability = f64::from(item.memory.stability);
        next_card.stability_fast = f64::from(item.memory.stability_fast);
        next_card.difficulty = f64::from(item.memory.difficulty);
        next_card.reps = card.reps.checked_add(1).ok_or(ReviewError::InvalidCard)?;
        next_card.lapses = if grade == ReviewGrade::Again {
            card.lapses.checked_add(1).ok_or(ReviewError::InvalidCard)?
        } else {
            card.lapses
        };
        next_card.state = next_state(card.state, grade);
        next_card.due = due;
        next_card.last_review = Some(reviewed_at);
        next_card.version = card
            .version
            .checked_add(1)
            .ok_or(ReviewError::InvalidCard)?;

        Ok(ReviewCalculation {
            previous_card: card.clone(),
            next_card,
            grade,
            reviewed_at,
            next_interval,
            due,
        })
    }

    fn validate_card(card: &PersistedCard) -> Result<(), ReviewError> {
        let finite_fields = [card.stability, card.stability_fast, card.difficulty];
        if finite_fields
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
            || card.version < 0
            || (card.state == CardState::New) != card.last_review.is_none()
        {
            return Err(ReviewError::InvalidCard);
        }
        Ok(())
    }

    fn memory_state(card: &PersistedCard) -> Result<Option<MemoryState>, ReviewError> {
        if card.state == CardState::New {
            return Ok(None);
        }
        Ok(Some(MemoryState {
            stability: Self::scheduler_f32(card.stability)?,
            stability_fast: Self::scheduler_f32(card.stability_fast)?,
            difficulty: Self::scheduler_f32(card.difficulty)?,
        }))
    }

    fn elapsed_days(card: &PersistedCard, reviewed_at: OffsetDateTime) -> Result<f32, ReviewError> {
        if card.state == CardState::New {
            return Ok(0.0);
        }
        let last_review = card.last_review.ok_or(ReviewError::InvalidCard)?;
        let elapsed_seconds = (reviewed_at - last_review).as_seconds_f64();
        if !elapsed_seconds.is_finite() || elapsed_seconds < 0.0 {
            return Err(ReviewError::InvalidCard);
        }
        Self::scheduler_f32(elapsed_seconds / 86_400.0)
    }

    #[allow(clippy::cast_possible_truncation)]
    fn scheduler_f32(value: f64) -> Result<f32, ReviewError> {
        if !value.is_finite() || value < 0.0 || value > f64::from(f32::MAX) {
            return Err(ReviewError::InvalidCard);
        }
        Ok(value as f32)
    }

    fn validate_output(memory: MemoryState, interval: f32) -> Result<(), ReviewError> {
        let fields = [
            memory.stability,
            memory.stability_fast,
            memory.difficulty,
            interval,
        ];
        if fields
            .iter()
            .any(|value| !value.is_finite() || *value < 0.0)
        {
            return Err(ReviewError::InvalidOutput);
        }
        Ok(())
    }
}

fn next_state(previous: CardState, grade: ReviewGrade) -> CardState {
    match (previous, grade) {
        (CardState::New | CardState::Learning, ReviewGrade::Again | ReviewGrade::Hard) => {
            CardState::Learning
        }
        (CardState::Review, ReviewGrade::Again)
        | (CardState::Relearning, ReviewGrade::Again | ReviewGrade::Hard) => CardState::Relearning,
        (
            CardState::New | CardState::Learning | CardState::Relearning,
            ReviewGrade::Good | ReviewGrade::Easy,
        )
        | (CardState::Review, ReviewGrade::Hard | ReviewGrade::Good | ReviewGrade::Easy) => {
            CardState::Review
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use time::macros::datetime;

    fn card(state: CardState) -> PersistedCard {
        PersistedCard {
            stability: 1.0,
            stability_fast: 1.0,
            difficulty: 5.0,
            reps: 0,
            lapses: 0,
            state,
            due: datetime!(2026-11-21 12:00 UTC),
            last_review: (state != CardState::New).then_some(datetime!(2026-11-20 12:00 UTC)),
            version: 0,
        }
    }

    #[test]
    fn supported_grades_and_states_round_trip() {
        for value in ["again", "hard", "good", "easy"] {
            let grade = ReviewGrade::parse(value).expect("supported grade");
            assert_eq!(grade.as_str(), value);
        }
        for value in ["new", "learning", "review", "relearning"] {
            let state = CardState::parse(value).expect("supported state");
            assert_eq!(state.as_str(), value);
        }
    }

    #[test]
    fn unsupported_grade_and_state_return_safe_errors() {
        assert_eq!(
            ReviewGrade::parse("skip"),
            Err(ReviewError::UnsupportedGrade)
        );
        assert_eq!(
            CardState::parse("suspended"),
            Err(ReviewError::UnsupportedState)
        );
        assert!(FsrsScheduler::new(SchedulerConfig {
            desired_retention: 1.0,
        })
        .is_err());
    }

    #[test]
    fn new_card_review_uses_the_injected_clock_and_config() {
        let scheduler = FsrsScheduler::new(SchedulerConfig {
            desired_retention: 0.9,
        })
        .expect("valid config");
        let reviewed_at = datetime!(2026-11-21 12:34:56 UTC);

        let calculation = scheduler
            .review(&card(CardState::New), ReviewGrade::Good, reviewed_at)
            .expect("new card should schedule");

        assert_eq!(calculation.grade, ReviewGrade::Good);
        assert_eq!(calculation.reviewed_at, reviewed_at);
        assert!(calculation.next_interval >= Duration::ZERO);
        assert_eq!(calculation.due, reviewed_at + calculation.next_interval);
        assert_eq!(calculation.next_card.state, CardState::Review);
        assert_eq!(calculation.next_card.reps, 1);
        assert_eq!(calculation.next_card.lapses, 0);
        assert_eq!(calculation.next_card.version, 1);
        assert_eq!(calculation.next_card.due, calculation.due);
        assert_eq!(calculation.next_card.last_review, Some(reviewed_at));
    }

    #[test]
    fn every_supported_card_state_can_be_scheduled() {
        let scheduler = FsrsScheduler::default();
        let reviewed_at = datetime!(2026-11-21 13:00 UTC);

        for state in [
            CardState::New,
            CardState::Learning,
            CardState::Review,
            CardState::Relearning,
        ] {
            let calculation = scheduler
                .review(&card(state), ReviewGrade::Good, reviewed_at)
                .expect("supported state should schedule");
            assert!(calculation.next_interval >= Duration::ZERO);
            assert!(calculation.next_card.stability_fast >= 0.0);
        }
    }

    #[test]
    fn malformed_cards_return_safe_errors() {
        let scheduler = FsrsScheduler::default();
        let reviewed_at = datetime!(2026-11-21 13:00 UTC);
        let mut invalid = card(CardState::Review);
        invalid.stability_fast = f64::NAN;
        assert_eq!(
            scheduler.review(&invalid, ReviewGrade::Good, reviewed_at),
            Err(ReviewError::InvalidCard)
        );

        let mut missing_history = card(CardState::Review);
        missing_history.last_review = None;
        assert_eq!(
            scheduler.review(&missing_history, ReviewGrade::Good, reviewed_at),
            Err(ReviewError::InvalidCard)
        );

        let mut clock_reversed = card(CardState::Review);
        clock_reversed.last_review = Some(reviewed_at + Duration::seconds(1));
        assert_eq!(
            scheduler.review(&clock_reversed, ReviewGrade::Good, reviewed_at),
            Err(ReviewError::InvalidCard)
        );
    }

    #[test]
    fn repeated_reviews_are_deterministic_for_the_same_inputs() {
        let scheduler = FsrsScheduler::default();
        let reviewed_at = datetime!(2026-11-21 13:00 UTC);
        let input = card(CardState::Review);

        let first = scheduler
            .review(&input, ReviewGrade::Hard, reviewed_at)
            .expect("first review");
        let second = scheduler
            .review(&input, ReviewGrade::Hard, reviewed_at)
            .expect("second review");

        assert_eq!(first, second);
    }

    fn generated_state() -> impl Strategy<Value = CardState> {
        prop::sample::select(vec![
            CardState::New,
            CardState::Learning,
            CardState::Review,
            CardState::Relearning,
        ])
    }

    fn generated_grade() -> impl Strategy<Value = ReviewGrade> {
        prop::sample::select(vec![
            ReviewGrade::Again,
            ReviewGrade::Hard,
            ReviewGrade::Good,
            ReviewGrade::Easy,
        ])
    }

    fn generated_card_and_reviewed_at() -> impl Strategy<Value = (PersistedCard, OffsetDateTime)> {
        let base = datetime!(2026-11-20 12:00 UTC);
        (
            0.01f64..=30.0,
            0.01f64..=30.0,
            0.5f64..=10.0,
            0u32..=10_000,
            0u32..=10_000,
            generated_state(),
            0i64..=1_000_000,
            0i64..=31_536_000,
            0i64..=31_536_000,
        )
            .prop_map(
                move |(
                    stability,
                    stability_fast,
                    difficulty,
                    reps,
                    lapses,
                    state,
                    version,
                    due_offset_seconds,
                    reviewed_offset_seconds,
                )| {
                    (
                        PersistedCard {
                            stability,
                            stability_fast,
                            difficulty,
                            reps,
                            lapses,
                            state,
                            due: base + Duration::seconds(due_offset_seconds),
                            last_review: (state != CardState::New).then_some(base),
                            version,
                        },
                        base + Duration::seconds(reviewed_offset_seconds),
                    )
                },
            )
    }

    proptest! {
        #![proptest_config(ProptestConfig::with_cases(128))]

        #[test]
        fn successful_reviews_never_schedule_into_the_past(
            (input, reviewed_at) in generated_card_and_reviewed_at(),
            grade in generated_grade(),
        ) {
            let scheduler = FsrsScheduler::default();
            let result = scheduler.review(&input, grade, reviewed_at);

            prop_assume!(result.is_ok());
            let calculation = result.expect("assumed successful calculation");
            prop_assert!(calculation.next_interval >= Duration::ZERO);
            prop_assert!(calculation.due >= reviewed_at);
            prop_assert_eq!(calculation.due, reviewed_at + calculation.next_interval);
            prop_assert_eq!(calculation.next_card.due, calculation.due);
        }

        #[test]
        fn repeated_calculations_are_deterministic(
            (input, reviewed_at) in generated_card_and_reviewed_at(),
            grade in generated_grade(),
        ) {
            let scheduler = FsrsScheduler::default();
            let first = scheduler.review(&input, grade, reviewed_at);
            let second = scheduler.review(&input, grade, reviewed_at);

            prop_assert_eq!(first, second);
        }

        #[test]
        fn repeated_good_reviews_do_not_shorten_the_interval(
            (input, _reviewed_at) in generated_card_and_reviewed_at(),
        ) {
            let scheduler = FsrsScheduler::default();
            let first = scheduler.review(&input, ReviewGrade::Good, input.due);
            prop_assume!(first.is_ok());
            let first = first.expect("assumed first calculation");

            let second = scheduler.review(&first.next_card, ReviewGrade::Good, first.due);
            prop_assume!(second.is_ok());
            let second = second.expect("assumed second calculation");

            prop_assert!(
                second.next_interval >= first.next_interval,
                "first {:?}, second {:?}",
                first.next_interval,
                second.next_interval
            );
        }

        #[test]
        fn malformed_persisted_cards_are_rejected(
            (input, reviewed_at) in generated_card_and_reviewed_at(),
            field in 0usize..6,
            bad_float in prop_oneof![
                Just(f64::NAN),
                Just(f64::INFINITY),
                Just(f64::NEG_INFINITY),
                Just(-0.01),
            ],
        ) {
            let scheduler = FsrsScheduler::default();
            let mut invalid = input;
            match field {
                0 => invalid.stability = bad_float,
                1 => invalid.stability_fast = bad_float,
                2 => invalid.difficulty = bad_float,
                3 => invalid.version = -1,
                4 => {
                    invalid.last_review = if invalid.state == CardState::New {
                        Some(reviewed_at)
                    } else {
                        None
                    };
                }
                _ => {
                    if invalid.state == CardState::New {
                        invalid.last_review = Some(datetime!(2026-11-19 12:00 UTC));
                    } else {
                        invalid.last_review = Some(reviewed_at + Duration::seconds(1));
                    }
                }
            }

            prop_assert_eq!(
                scheduler.review(&invalid, ReviewGrade::Good, reviewed_at),
                Err(ReviewError::InvalidCard)
            );
        }

        #[test]
        fn invalid_scheduler_results_are_rejected(
            field in 0usize..4,
            bad_value in prop_oneof![
                Just(f32::NAN),
                Just(f32::INFINITY),
                Just(f32::NEG_INFINITY),
                Just(-0.01),
            ],
        ) {
            let mut values = [1.0f32; 4];
            values[field] = bad_value;
            let memory = MemoryState {
                stability: values[0],
                stability_fast: values[1],
                difficulty: values[2],
            };

            prop_assert_eq!(
                FsrsScheduler::validate_output(memory, values[3]),
                Err(ReviewError::InvalidOutput)
            );
        }
    }

    #[test]
    fn documented_good_schedule_is_longer_than_again() {
        let scheduler = FsrsScheduler::default();
        let reviewed_at = datetime!(2026-11-21 13:00 UTC);
        let input = PersistedCard {
            stability: 10.0,
            stability_fast: 10.0,
            difficulty: 5.0,
            reps: 4,
            lapses: 1,
            state: CardState::Review,
            due: reviewed_at,
            last_review: Some(datetime!(2026-11-14 13:00 UTC)),
            version: 4,
        };

        let again = scheduler
            .review(&input, ReviewGrade::Again, reviewed_at)
            .expect("again should schedule");
        let good = scheduler
            .review(&input, ReviewGrade::Good, reviewed_at)
            .expect("good should schedule");

        assert!(good.next_interval > again.next_interval);
    }
}
