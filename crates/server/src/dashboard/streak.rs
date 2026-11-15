//! The practice streak: consecutive UTC days with a completed run.
//!
//! This is a pure function. It takes the learner's activity dates and today's UTC date, so the tests
//! do not depend on the clock or on PostgreSQL.

use std::collections::BTreeSet;

use time::Date;

/// The streak as the pure calculation returns it. The response mapper formats the date for JSON.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Streak {
    /// Consecutive UTC days with activity, ending today or yesterday.
    pub current_days: u32,
    /// Whether the learner has activity on `today`.
    pub active_today: bool,
    /// The latest activity date, or `None` when there is no activity.
    pub last_activity: Option<Date>,
}

/// Counts the practice streak from the learner's activity dates.
///
/// Dates may repeat or come in any order. The streak ends today when the learner practised today,
/// or yesterday when they have not practised yet. Any other latest date gives a streak of zero.
#[must_use]
pub fn streak_from_dates(activity_days: &[Date], today: Date) -> Streak {
    let days: BTreeSet<Date> = activity_days.iter().copied().collect();
    let Some(&last) = days.last() else {
        return Streak {
            current_days: 0,
            active_today: false,
            last_activity: None,
        };
    };
    let yesterday = today.previous_day();
    let anchor = if last == today {
        Some(today)
    } else if Some(last) == yesterday {
        yesterday
    } else {
        None
    };
    let current_days = anchor.map_or(0, |start| count_back_from(&days, start));
    Streak {
        current_days,
        active_today: days.contains(&today),
        last_activity: Some(last),
    }
}

/// Counts consecutive days in `days`, walking back from `start` until a day is missing.
fn count_back_from(days: &BTreeSet<Date>, start: Date) -> u32 {
    let mut count = 0;
    let mut cursor = Some(start);
    while let Some(day) = cursor.filter(|day| days.contains(day)) {
        count += 1;
        cursor = day.previous_day();
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::{Date, Month};

    fn date(year: i32, month: Month, day: u8) -> Date {
        Date::from_calendar_date(year, month, day).expect("valid test date")
    }

    const TODAY: (i32, Month, u8) = (2026, Month::November, 15);

    fn today() -> Date {
        date(TODAY.0, TODAY.1, TODAY.2)
    }

    #[test]
    fn no_activity_is_a_zero_streak_with_no_last_date() {
        let streak = streak_from_dates(&[], today());
        assert_eq!(
            streak,
            Streak {
                current_days: 0,
                active_today: false,
                last_activity: None,
            }
        );
    }

    #[test]
    fn activity_today_counts_one_day_and_marks_today_active() {
        let days = [date(2026, Month::November, 15)];
        let streak = streak_from_dates(&days, today());
        assert_eq!(streak.current_days, 1);
        assert!(streak.active_today);
        assert_eq!(streak.last_activity, Some(date(2026, Month::November, 15)));
    }

    #[test]
    fn activity_only_yesterday_keeps_the_streak_but_today_is_not_active() {
        let days = [date(2026, Month::November, 14)];
        let streak = streak_from_dates(&days, today());
        assert_eq!(streak.current_days, 1);
        assert!(!streak.active_today);
    }

    #[test]
    fn consecutive_days_ending_today_are_counted_back_to_the_first_gap() {
        // 13, 14 and 15 are consecutive. 11 is before a gap on 12 and must not count.
        let days = [
            date(2026, Month::November, 11),
            date(2026, Month::November, 13),
            date(2026, Month::November, 14),
            date(2026, Month::November, 15),
        ];
        let streak = streak_from_dates(&days, today());
        assert_eq!(streak.current_days, 3);
        assert!(streak.active_today);
    }

    #[test]
    fn a_missing_yesterday_breaks_a_streak_that_ended_earlier() {
        // Last activity was on the 13th. Today is the 15th, so the 14th is missing.
        let days = [date(2026, Month::November, 13)];
        let streak = streak_from_dates(&days, today());
        assert_eq!(streak.current_days, 0);
        assert!(!streak.active_today);
        assert_eq!(streak.last_activity, Some(date(2026, Month::November, 13)));
    }

    #[test]
    fn several_runs_on_one_day_count_as_one_day() {
        let days = [
            date(2026, Month::November, 15),
            date(2026, Month::November, 15),
            date(2026, Month::November, 15),
        ];
        let streak = streak_from_dates(&days, today());
        assert_eq!(streak.current_days, 1);
    }

    #[test]
    fn unsorted_input_gives_the_same_streak() {
        let days = [
            date(2026, Month::November, 15),
            date(2026, Month::November, 13),
            date(2026, Month::November, 14),
        ];
        let streak = streak_from_dates(&days, today());
        assert_eq!(streak.current_days, 3);
    }

    #[test]
    fn a_streak_crosses_the_month_boundary_in_utc() {
        let days = [
            date(2026, Month::October, 31),
            date(2026, Month::November, 1),
        ];
        let streak = streak_from_dates(&days, date(2026, Month::November, 1));
        assert_eq!(streak.current_days, 2);
        assert!(streak.active_today);
    }
}
