//! The one-attempt marker for the speech bubble.
//!
//! A run gets at most one AI generation. The first caller sets a Valkey key with `SET NX EX`; every
//! later caller finds the key and shows the template. `NX` makes the claim atomic, so two clicks at
//! once cannot both spend a provider call. The key expires after a day, and Valkey holds nothing else
//! about the run.

use thiserror::Error;
use uuid::Uuid;

/// How long the marker lives, in seconds. One day, the same as the daily AI quota window.
pub const MARKER_TTL_SECS: u64 = 86_400;

/// Valkey could not be reached or refused the command. The caller treats it as "no AI for this run".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("the speech attempt marker is unavailable")]
pub struct AttemptError;

/// Claims the single AI attempt for a run.
#[derive(Clone)]
pub struct AttemptMarker {
    client: redis::Client,
}

impl AttemptMarker {
    /// Prepares a marker store for `url`. No connection is opened until the first claim.
    ///
    /// # Errors
    ///
    /// Returns [`AttemptError`] when the URL is malformed. The URL is never printed.
    pub fn open(url: &str) -> Result<Self, AttemptError> {
        let client = redis::Client::open(url).map_err(|_| AttemptError)?;
        Ok(Self { client })
    }

    /// The key that marks `run`. It holds only the run id, never code or text.
    #[must_use]
    pub fn key(run: Uuid) -> String {
        format!("familiar:attempt:{run}")
    }

    /// Claims the attempt for `run`. `Ok(true)` means this caller is first and may generate;
    /// `Ok(false)` means the attempt was already taken.
    ///
    /// # Errors
    ///
    /// Returns [`AttemptError`] when Valkey cannot be reached or the command fails.
    pub async fn claim(&self, run: Uuid) -> Result<bool, AttemptError> {
        let mut connection = self
            .client
            .get_multiplexed_async_connection()
            .await
            .map_err(|_| AttemptError)?;
        let reply: Option<String> = redis::cmd("SET")
            .arg(Self::key(run))
            .arg(1)
            .arg("NX")
            .arg("EX")
            .arg(MARKER_TTL_SECS)
            .query_async(&mut connection)
            .await
            .map_err(|_| AttemptError)?;
        Ok(reply.is_some())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valkey_url() -> String {
        std::env::var("VALKEY_URL").expect("VALKEY_URL is set for the ignored tests")
    }

    // Breaks if Valkey being down is read as "first attempt", which would let AI run unguarded.
    #[tokio::test]
    async fn an_unreachable_valkey_is_an_error_not_a_claim() {
        let marker = AttemptMarker::open("redis://127.0.0.1:1").expect("a well-formed url");

        assert_eq!(marker.claim(Uuid::new_v4()).await, Err(AttemptError));
    }

    // Breaks if a key could carry anything but the run id.
    #[test]
    fn the_key_names_only_the_run() {
        let run = Uuid::from_u128(7);

        assert_eq!(
            AttemptMarker::key(run),
            "familiar:attempt:00000000-0000-0000-0000-000000000007"
        );
    }

    // Breaks if a second caller is also told it is first, which would spend two provider calls.
    #[tokio::test]
    #[ignore = "needs VALKEY_URL"]
    async fn valkey_only_the_first_claim_for_a_run_wins() {
        let marker = AttemptMarker::open(&valkey_url()).expect("url");
        let run = Uuid::new_v4();

        assert_eq!(marker.claim(run).await, Ok(true));
        assert_eq!(marker.claim(run).await, Ok(false));
        assert_eq!(marker.claim(Uuid::new_v4()).await, Ok(true));
    }

    // Breaks if the marker never expires and leaks one key per run for ever.
    #[tokio::test]
    #[ignore = "needs VALKEY_URL"]
    async fn valkey_the_marker_expires_within_a_day() {
        let marker = AttemptMarker::open(&valkey_url()).expect("url");
        let run = Uuid::new_v4();
        marker.claim(run).await.expect("claim");
        let client = redis::Client::open(valkey_url()).expect("url");
        let mut connection = client
            .get_multiplexed_async_connection()
            .await
            .expect("connection");

        let ttl: i64 = redis::cmd("TTL")
            .arg(AttemptMarker::key(run))
            .query_async(&mut connection)
            .await
            .expect("ttl");

        assert!(ttl > 86_000 && ttl <= 86_400, "ttl was {ttl}");
    }
}
