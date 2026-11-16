//! Daily AI quota and spend reservations, counted in Valkey (hints spec H09).
//!
//! Valkey holds counters only. Each command is atomic, so `INCR` is the exact guard when many
//! requests arrive together. A reservation is taken before a provider call, and the unused part is
//! given back afterwards. If Valkey fails part-way, the counts already taken are undone on a best
//! effort basis, and the call fails closed with [`QuotaError::Unavailable`].

use std::fmt;

use counterspell_common::ids::UserId;
use time::OffsetDateTime;

/// Counters live this many seconds past the UTC midnight that ends their day, so a late reset
/// never drops a count early.
const EXPIRY_GRACE_SECS: i64 = 3_600;

/// The UTC date of `now` as `YYYY-MM-DD`. It is the day key in every counter name.
#[must_use]
pub fn day_key(now: OffsetDateTime) -> String {
    now.date().to_string()
}

/// The first moment of the next UTC day after `now`.
#[must_use]
pub fn next_utc_midnight(now: OffsetDateTime) -> OffsetDateTime {
    now.date()
        .next_day()
        .unwrap_or(now.date())
        .midnight()
        .assume_utc()
}

/// Whole seconds from `now` until the next UTC midnight.
#[must_use]
pub fn seconds_to_next_utc_midnight(now: OffsetDateTime) -> i64 {
    (next_utc_midnight(now) - now).whole_seconds()
}

/// Why a reservation was refused.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapReason {
    /// The learner has used all of today's AI generations.
    PerUser,
    /// The global spend cap for today is used up.
    GlobalSpend,
}

/// The outcome of a reservation attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reserve {
    /// The generation and the worst-case spend are reserved.
    Granted(Reservation),
    /// Nothing was reserved. `used` is the learner's count for today, unchanged by the attempt.
    CapReached {
        /// Which limit stopped the request.
        reason: CapReason,
        /// The learner's generations used today.
        used: u32,
    },
}

/// What one granted reservation holds, so it can be settled or released later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reservation {
    user: UserId,
    day: String,
    reserved_micros: i64,
    used_after: u32,
}

impl Reservation {
    /// The learner's generations used today, including this one.
    #[must_use]
    pub fn used_after(&self) -> u32 {
        self.used_after
    }

    /// The worst-case spend reserved, in micro-dollars.
    #[must_use]
    pub fn reserved_micros(&self) -> i64 {
        self.reserved_micros
    }
}

/// The counters could not be read or changed. It means Valkey is unavailable, so AI is off.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QuotaError {
    /// Valkey could not be reached, or a counter command failed.
    Unavailable,
}

impl fmt::Display for QuotaError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Unavailable => formatter.write_str("the hint quota store is unavailable"),
        }
    }
}

impl std::error::Error for QuotaError {}

/// The Valkey counters for quota and spend.
#[derive(Clone)]
pub struct QuotaStore {
    client: redis::Client,
    daily_limit: u32,
    spend_cap_micros: i64,
}

impl QuotaStore {
    /// Prepares a store for `url`. No connection is opened until the first call.
    ///
    /// # Errors
    ///
    /// Returns [`QuotaError`] when the URL is malformed. The URL is never printed.
    pub fn open(url: &str, daily_limit: u32, spend_cap_micros: i64) -> Result<Self, QuotaError> {
        let client = redis::Client::open(url).map_err(|_| QuotaError::Unavailable)?;
        Ok(Self {
            client,
            daily_limit,
            spend_cap_micros,
        })
    }

    /// Reserves one generation for `user` and `worst_micros` of spend for the UTC day of `now`.
    ///
    /// # Errors
    ///
    /// Returns [`QuotaError`] when Valkey cannot be reached or a command fails.
    pub async fn reserve(
        &self,
        user: UserId,
        now: OffsetDateTime,
        worst_micros: i64,
    ) -> Result<Reserve, QuotaError> {
        let day = day_key(now);
        let keys = Keys::new(user, &day);
        let ttl = seconds_to_next_utc_midnight(now) + EXPIRY_GRACE_SECS;
        let mut connection = self.connection().await?;
        let mut taken = Taken::default();
        match self
            .take(&mut connection, &keys, worst_micros, ttl, &mut taken)
            .await
        {
            Ok(Attempt::Granted { used_after }) => Ok(Reserve::Granted(Reservation {
                user,
                day,
                reserved_micros: worst_micros,
                used_after,
            })),
            Ok(Attempt::Refused { reason, used }) => Ok(Reserve::CapReached { reason, used }),
            Err(error) => {
                tracing::warn!(error = %error, "hint quota reserve failed; undoing partial counts");
                taken.undo(&mut connection, &keys).await;
                Err(QuotaError::Unavailable)
            }
        }
    }

    /// Records the real cost of a reservation, giving back any unused part of the spend.
    ///
    /// # Errors
    ///
    /// Returns [`QuotaError`] when Valkey cannot be reached.
    pub async fn settle(
        &self,
        reservation: &Reservation,
        actual_micros: i64,
    ) -> Result<(), QuotaError> {
        let keys = Keys::new(reservation.user, &reservation.day);
        let mut connection = self.connection().await?;
        let difference = actual_micros - reservation.reserved_micros;
        incr_by(&mut connection, &keys.spend, difference)
            .await
            .map(|_| ())
            .map_err(|_| QuotaError::Unavailable)
    }

    /// Gives back a reservation whose provider call failed, so the failure costs the learner nothing.
    ///
    /// # Errors
    ///
    /// Returns [`QuotaError`] when Valkey cannot be reached.
    pub async fn release(&self, reservation: &Reservation) -> Result<(), QuotaError> {
        let keys = Keys::new(reservation.user, &reservation.day);
        let mut connection = self.connection().await?;
        let mut taken = Taken {
            quota: true,
            spend_micros: reservation.reserved_micros,
        };
        taken.undo(&mut connection, &keys).await;
        Ok(())
    }

    /// The spend recorded for the UTC day of `now`, in micro-dollars.
    ///
    /// # Errors
    ///
    /// Returns [`QuotaError`] when Valkey cannot be reached.
    pub async fn spend_micros(&self, now: OffsetDateTime) -> Result<i64, QuotaError> {
        let keys = Keys::new(UserId::from_uuid(uuid::Uuid::nil()), &day_key(now));
        let mut connection = self.connection().await?;
        let value: Option<i64> = redis::cmd("GET")
            .arg(&keys.spend)
            .query_async(&mut connection)
            .await
            .map_err(|_| QuotaError::Unavailable)?;
        Ok(value.unwrap_or(0))
    }

    /// The seconds left before the learner's quota key expires. Used to check the expiry is set.
    ///
    /// # Errors
    ///
    /// Returns [`QuotaError`] when Valkey cannot be reached.
    pub async fn quota_ttl_seconds(
        &self,
        user: UserId,
        now: OffsetDateTime,
    ) -> Result<i64, QuotaError> {
        let keys = Keys::new(user, &day_key(now));
        let mut connection = self.connection().await?;
        redis::cmd("TTL")
            .arg(&keys.quota)
            .query_async(&mut connection)
            .await
            .map_err(|_| QuotaError::Unavailable)
    }

    async fn connection(&self) -> Result<redis::aio::MultiplexedConnection, QuotaError> {
        self.client
            .get_multiplexed_async_connection()
            .await
            .map_err(|_| QuotaError::Unavailable)
    }

    /// The reserve steps. Each is one atomic command, and `taken` records each one as it succeeds,
    /// so the caller can undo exactly what was counted.
    async fn take(
        &self,
        connection: &mut redis::aio::MultiplexedConnection,
        keys: &Keys,
        worst_micros: i64,
        ttl: i64,
        taken: &mut Taken,
    ) -> Result<Attempt, redis::RedisError> {
        let quota_before = incr_by(connection, &keys.quota, 1).await?;
        taken.quota = true;
        expire_if_unset(connection, &keys.quota, ttl).await?;
        if quota_before > i64::from(self.daily_limit) {
            taken.undo(connection, keys).await;
            return Ok(Attempt::Refused {
                reason: CapReason::PerUser,
                used: clamp_used(quota_before - 1),
            });
        }
        let spend_after = incr_by(connection, &keys.spend, worst_micros).await?;
        taken.spend_micros = worst_micros;
        expire_if_unset(connection, &keys.spend, ttl).await?;
        if spend_after > self.spend_cap_micros {
            taken.undo(connection, keys).await;
            return Ok(Attempt::Refused {
                reason: CapReason::GlobalSpend,
                used: clamp_used(quota_before - 1),
            });
        }
        Ok(Attempt::Granted {
            used_after: clamp_used(quota_before),
        })
    }
}

/// The result of the reserve steps before they become a [`Reserve`].
enum Attempt {
    Granted { used_after: u32 },
    Refused { reason: CapReason, used: u32 },
}

/// The counts a reserve has added so far, so they can be undone.
#[derive(Default)]
struct Taken {
    quota: bool,
    spend_micros: i64,
}

impl Taken {
    /// Takes back every count recorded here. Failures are logged and ignored: a count left behind
    /// only makes the limit stricter until the day ends.
    async fn undo(&mut self, connection: &mut redis::aio::MultiplexedConnection, keys: &Keys) {
        if self.quota {
            let _ = incr_by(connection, &keys.quota, -1).await;
            self.quota = false;
        }
        if self.spend_micros != 0 {
            let _ = incr_by(connection, &keys.spend, -self.spend_micros).await;
            self.spend_micros = 0;
        }
    }
}

/// The two counter names for one learner and day.
struct Keys {
    quota: String,
    spend: String,
}

impl Keys {
    fn new(user: UserId, day: &str) -> Self {
        Self {
            quota: format!("hint:quota:{}:{day}", user.as_uuid()),
            spend: format!("hint:spend:{day}"),
        }
    }
}

/// Adds `amount` to a counter and returns the value after the add. Negative amounts subtract.
async fn incr_by(
    connection: &mut redis::aio::MultiplexedConnection,
    key: &str,
    amount: i64,
) -> Result<i64, redis::RedisError> {
    redis::cmd("INCRBY")
        .arg(key)
        .arg(amount)
        .query_async(connection)
        .await
}

/// Sets the expiry only when the key has none, so later writes never push it back.
async fn expire_if_unset(
    connection: &mut redis::aio::MultiplexedConnection,
    key: &str,
    ttl: i64,
) -> Result<(), redis::RedisError> {
    redis::cmd("EXPIRE")
        .arg(key)
        .arg(ttl)
        .arg("NX")
        .query_async::<i64>(connection)
        .await
        .map(|_| ())
}

/// Turns a counter value into a `u32` for display, never below zero.
fn clamp_used(value: i64) -> u32 {
    u32::try_from(value.max(0)).unwrap_or(u32::MAX)
}

#[cfg(test)]
mod tests {
    use super::*;
    use counterspell_common::ids::UserId;
    use time::OffsetDateTime;

    /// 2000-01-01 23:30 UTC, which is 1800 seconds before the next UTC midnight.
    const LATE_EVENING_2000: i64 = 946_769_400;

    /// A day that no other test run uses, so the global spend counter starts at zero.
    fn fresh_day() -> OffsetDateTime {
        let spread = uuid::Uuid::new_v4().as_u128() % 1_000_000;
        let seconds = 4_102_444_800 + i64::try_from(spread).unwrap_or(0) * 86_400;
        OffsetDateTime::from_unix_timestamp(seconds).unwrap_or(OffsetDateTime::UNIX_EPOCH)
    }

    fn store(limit: u32, cap_micros: i64) -> QuotaStore {
        let url =
            std::env::var("VALKEY_URL").expect("VALKEY_URL is set for the ignored valkey tests");
        QuotaStore::open(&url, limit, cap_micros).expect("VALKEY_URL is a valid URL")
    }

    fn unreachable_store() -> QuotaStore {
        QuotaStore::open("redis://127.0.0.1:1/", 1, 1).expect("a fixed local URL always parses")
    }

    #[test]
    fn the_day_key_is_the_utc_date() {
        let at = OffsetDateTime::from_unix_timestamp(LATE_EVENING_2000)
            .unwrap_or(OffsetDateTime::UNIX_EPOCH);

        assert_eq!(day_key(at), "2000-01-01");
    }

    #[test]
    fn the_reset_is_the_next_utc_midnight() {
        let at = OffsetDateTime::from_unix_timestamp(LATE_EVENING_2000)
            .unwrap_or(OffsetDateTime::UNIX_EPOCH);

        assert_eq!(seconds_to_next_utc_midnight(at), 1_800);
        assert_eq!(next_utc_midnight(at).unix_timestamp(), 946_771_200);
    }

    #[tokio::test]
    async fn an_unreachable_valkey_fails_closed() {
        let quota = unreachable_store();

        let result = quota.reserve(UserId::new(), fresh_day(), 1).await;

        assert_eq!(result.err(), Some(QuotaError::Unavailable));
    }

    #[tokio::test]
    #[ignore = "needs VALKEY_URL"]
    async fn valkey_exactly_the_limit_of_concurrent_reserves_succeed() {
        let quota = store(3, i64::MAX / 4);
        let user = UserId::new();
        let day = fresh_day();

        let attempts = (0..10).map(|_| quota.reserve(user, day, 1));
        let results = futures_util::future::join_all(attempts).await;

        let granted = results
            .iter()
            .filter(|result| matches!(result, Ok(Reserve::Granted(_))))
            .count();
        assert_eq!(granted, 3);
    }

    #[tokio::test]
    #[ignore = "needs VALKEY_URL"]
    async fn valkey_a_spend_refusal_gives_back_the_quota_it_took() {
        let quota = store(10, 100);
        let user = UserId::new();
        let day = fresh_day();

        let refused = quota.reserve(user, day, 150).await;
        let granted = quota.reserve(user, day, 10).await;

        assert!(matches!(
            refused,
            Ok(Reserve::CapReached {
                reason: CapReason::GlobalSpend,
                used: 0
            })
        ));
        assert!(matches!(granted, Ok(Reserve::Granted(ref r)) if r.used_after() == 1));
    }

    #[tokio::test]
    #[ignore = "needs VALKEY_URL"]
    async fn valkey_release_restores_the_quota_and_the_spend() {
        let quota = store(10, 1_000);
        let user = UserId::new();
        let day = fresh_day();
        let Ok(Reserve::Granted(reservation)) = quota.reserve(user, day, 400).await else {
            panic!("the first reserve is granted");
        };

        quota.release(&reservation).await.expect("valkey reachable");
        let again = quota.reserve(user, day, 0).await;

        assert!(matches!(again, Ok(Reserve::Granted(ref r)) if r.used_after() == 1));
        let spend = quota.spend_micros(day).await.expect("valkey reachable");
        assert_eq!(spend, 0);
    }

    #[tokio::test]
    #[ignore = "needs VALKEY_URL"]
    async fn valkey_the_first_reserve_sets_an_expiry_on_the_quota_key() {
        let quota = store(10, 1_000);
        let user = UserId::new();
        let day = fresh_day();

        quota.reserve(user, day, 1).await.expect("valkey reachable");

        let ttl = quota
            .quota_ttl_seconds(user, day)
            .await
            .expect("valkey reachable");
        assert!(ttl > 0, "the key expires after the day ends, got {ttl}");
    }
}
