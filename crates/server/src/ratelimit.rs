//! Window rate limits for run submissions, counted in Valkey.
//!
//! Valkey holds counters only. It is never a queue or an event bus. If Valkey is down, a limit is
//! skipped and a warning is logged (fail open). Postgres still enforces the queue caps, so the
//! queue cannot be flooded.

use std::{
    collections::HashMap,
    future::Future,
    net::IpAddr,
    sync::{Mutex, PoisonError},
};

use counterspell_common::ids::UserId;
use sha2::{Digest, Sha256};

/// What a limiter decided for one hit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Decision {
    /// The hit is counted and allowed. `remaining` is how many more hits fit in this window.
    Allowed {
        /// Hits left in the current window.
        remaining: u64,
    },
    /// The window is full. The client should wait this many seconds.
    Limited {
        /// Seconds until the window resets, at least 1.
        retry_after_seconds: u64,
    },
    /// The counter store was unreachable, so the limit was not applied.
    SkippedFailOpen,
}

/// Counts hits against a named key within a fixed window.
pub trait RateLimiter: Send + Sync {
    /// Counts one hit on `key`. At most `limit` hits are allowed in each `window_secs` window.
    fn hit(&self, key: &str, limit: u64, window_secs: u64)
        -> impl Future<Output = Decision> + Send;
}

/// The one-step atomic counter. It increments, sets the window's expiry on the first hit, and returns
/// the count and the seconds left, so no other client can slip between the steps.
const HIT_SCRIPT: &str = r"
local count = redis.call('INCR', KEYS[1])
if count == 1 then
  redis.call('EXPIRE', KEYS[1], ARGV[1])
end
return {count, redis.call('TTL', KEYS[1])}
";

/// A limiter whose counters live in Valkey.
#[derive(Clone)]
pub struct ValkeyLimiter {
    client: redis::Client,
}

impl ValkeyLimiter {
    /// Prepares a client for `url`. No connection is made until the first hit.
    ///
    /// # Errors
    ///
    /// Returns an error when the URL is malformed. The error text never includes the URL.
    pub fn open(url: &str) -> Result<Self, redis::RedisError> {
        Ok(Self {
            client: redis::Client::open(url)?,
        })
    }

    /// Reads a counter without changing it. Missing keys read as zero. `None` means Valkey could
    /// not be reached.
    pub async fn current(&self, key: &str) -> Option<u64> {
        let mut connection = match self.client.get_multiplexed_async_connection().await {
            Ok(connection) => connection,
            Err(error) => {
                tracing::warn!(error = %error, "rate limit read skipped: Valkey is unavailable");
                return None;
            }
        };
        let value: Result<Option<i64>, redis::RedisError> = redis::cmd("GET")
            .arg(key)
            .query_async(&mut connection)
            .await;
        match value {
            Ok(count) => Some(count.and_then(|c| u64::try_from(c).ok()).unwrap_or(0)),
            Err(error) => {
                tracing::warn!(error = %error, "rate limit read skipped: Valkey is unavailable");
                None
            }
        }
    }

    async fn count(&self, key: &str, window_secs: u64) -> Result<(u64, u64), redis::RedisError> {
        let mut connection = self.client.get_multiplexed_async_connection().await?;
        let (count, ttl): (i64, i64) = redis::Script::new(HIT_SCRIPT)
            .key(key)
            .arg(window_secs)
            .invoke_async(&mut connection)
            .await?;
        Ok((
            u64::try_from(count).unwrap_or(u64::MAX),
            u64::try_from(ttl).unwrap_or(0),
        ))
    }
}

impl RateLimiter for ValkeyLimiter {
    async fn hit(&self, key: &str, limit: u64, window_secs: u64) -> Decision {
        match self.count(key, window_secs).await {
            Ok((count, ttl_secs)) => decide(count, ttl_secs, limit),
            Err(error) => {
                tracing::warn!(error = %error, "rate limit skipped: Valkey is unavailable");
                Decision::SkippedFailOpen
            }
        }
    }
}

/// An in-process limiter for tests. It never expires a window.
#[cfg(feature = "testkit")]
#[derive(Debug, Default)]
pub struct MemoryLimiter {
    counts: Mutex<HashMap<String, u64>>,
}

#[cfg(feature = "testkit")]
impl MemoryLimiter {
    /// Reads a counter without counting a hit.
    #[must_use]
    pub fn current(&self, key: &str) -> Option<u64> {
        let counts = self.counts.lock().unwrap_or_else(PoisonError::into_inner);
        Some(counts.get(key).copied().unwrap_or(0))
    }
}

#[cfg(feature = "testkit")]
impl RateLimiter for MemoryLimiter {
    fn hit(
        &self,
        key: &str,
        limit: u64,
        window_secs: u64,
    ) -> impl Future<Output = Decision> + Send {
        let mut counts = self.counts.lock().unwrap_or_else(PoisonError::into_inner);
        let count = counts.entry(key.to_owned()).or_insert(0);
        *count += 1;
        std::future::ready(decide(*count, window_secs, limit))
    }
}

/// The limiter the server uses, chosen once at start-up.
#[derive(Clone)]
pub enum Limiter {
    /// Counters in Valkey. The production choice.
    Valkey(ValkeyLimiter),
    /// Counters in memory. Tests only.
    #[cfg(feature = "testkit")]
    Memory(std::sync::Arc<MemoryLimiter>),
}

impl Limiter {
    /// Reads a counter without counting a hit. `None` means the counter store is down.
    pub async fn current(&self, key: &str) -> Option<u64> {
        match self {
            Limiter::Valkey(valkey) => valkey.current(key).await,
            #[cfg(feature = "testkit")]
            Limiter::Memory(memory) => memory.current(key),
        }
    }
}

impl RateLimiter for Limiter {
    async fn hit(&self, key: &str, limit: u64, window_secs: u64) -> Decision {
        match self {
            Limiter::Valkey(valkey) => valkey.hit(key, limit, window_secs).await,
            #[cfg(feature = "testkit")]
            Limiter::Memory(memory) => memory.hit(key, limit, window_secs).await,
        }
    }
}

/// The Valkey key for one user's run count: `rl:runs:user:<uuid>`.
#[must_use]
pub fn user_key(user: UserId) -> String {
    format!("rl:runs:user:{user}")
}

/// The Valkey key for one IP address's run count. The address is hashed, so Valkey never holds it.
#[must_use]
pub fn ip_key(address: IpAddr) -> String {
    let digest = Sha256::digest(address.to_string().as_bytes());
    format!("rl:runs:ip:{digest:x}")
}

fn decide(count: u64, ttl_secs: u64, limit: u64) -> Decision {
    if count > limit {
        Decision::Limited {
            retry_after_seconds: ttl_secs.max(1),
        }
    } else {
        Decision::Allowed {
            remaining: limit - count,
        }
    }
}
