//! Pure calendar and index maths for the daily puzzle.
//!
//! Africa/Johannesburg is UTC+2 all year, so the local date is a fixed offset
//! and no timezone database is needed. `daily_index` returns `Option` so an
//! empty `puzzles` table maps to `not_imported` instead of panicking.

use time::{Date, Duration, Month, OffsetDateTime};

const SAST_OFFSET_HOURS: i64 = 2;

/// The South African date for a UTC instant (UTC+2, no daylight saving).
#[must_use]
pub fn sast_date(utc: OffsetDateTime) -> Date {
    (utc + Duration::hours(SAST_OFFSET_HOURS)).date()
}

/// Whole days between 1970-01-01 and `date`; may be negative before 1970.
#[must_use]
pub fn days_since_epoch(date: Date) -> i64 {
    let epoch = Date::from_calendar_date(1970, Month::January, 1)
        .expect("1970-01-01 is a valid date")
        .to_julian_day();
    i64::from(date.to_julian_day()) - i64::from(epoch)
}

/// Which puzzle today's SAST date selects, or `None` when none are imported.
#[must_use]
pub fn daily_index(days: i64, count: i64) -> Option<i64> {
    if count <= 0 {
        None
    } else {
        Some(days.rem_euclid(count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use proptest::prelude::*;
    use time::Time;

    fn at(hour: u8, minute: u8) -> OffsetDateTime {
        Date::from_calendar_date(2026, Month::November, 13)
            .expect("valid")
            .with_time(Time::from_hms(hour, minute, 0).expect("valid"))
            .assume_utc()
    }

    #[test]
    fn sast_midnight_boundary() {
        // 23:59 UTC on the 13th is already 01:59 SAST on the 14th;
        // 00:01 UTC on the 13th is 02:01 SAST on the 13th.
        assert_eq!(sast_date(at(23, 59)).day(), 14);
        assert_eq!(sast_date(at(0, 1)).day(), 13);
        // 21:59 UTC is 23:59 SAST on the 13th; 22:00 UTC is midnight SAST 14th.
        let d = sast_date(at(21, 59));
        assert_eq!((d.month(), d.day()), (Month::November, 13));
        assert_eq!(sast_date(at(22, 0)).day(), 14);
    }

    #[test]
    fn daily_index_empty() {
        assert_eq!(daily_index(0, 0), None);
        assert_eq!(daily_index(-5, 0), None);
    }

    #[test]
    fn daily_index_wraps_and_handles_negative_days() {
        assert_eq!(daily_index(0, 3), Some(0));
        assert_eq!(daily_index(4, 3), Some(1));
        assert_eq!(daily_index(-1, 3), Some(2));
        assert_eq!(daily_index(-3, 3), Some(0));
    }

    proptest! {
        #[test]
        fn daily_index_stays_in_range(days in -1_000_000i64..1_000_000, count in 1i64..10_000) {
            let i = daily_index(days, count).expect("count >= 1");
            prop_assert!((0..count).contains(&i));
        }
    }
}
