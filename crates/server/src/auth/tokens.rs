//! Single-use tokens: the values inside email links, OAuth `state` and sessions.
//!
//! A raw token is 256 bits from the operating system's random source, encoded as unpadded
//! base64url (43 characters). Only its SHA-256 hash is ever stored. This file holds the pure
//! parts: generating, hashing, purposes and the expiry model. The database store comes later.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::{rngs::OsRng, TryRngCore};
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime};

use super::errors::AuthError;

/// How long an email token stays valid after it is issued.
pub const EMAIL_TOKEN_TTL: Duration = Duration::minutes(15);
/// How long an OAuth flow stays valid after it is created.
pub const FLOW_TTL: Duration = Duration::minutes(10);

/// The random value a person receives. It has no `Debug` or `Display` that shows its text.
pub struct RawToken(String);

impl RawToken {
    /// Makes a new token from 32 bytes of operating-system randomness.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::Internal`] when the operating system cannot supply randomness.
    pub fn generate() -> Result<Self, AuthError> {
        let mut bytes = [0_u8; 32];
        OsRng
            .try_fill_bytes(&mut bytes)
            .map_err(|_| AuthError::Internal)?;
        Ok(Self(URL_SAFE_NO_PAD.encode(bytes)))
    }

    /// The token text, for the email link, the URL or the cookie. Never log it.
    #[must_use]
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for RawToken {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("RawToken([redacted])")
    }
}

/// The SHA-256 hash of a raw token. This is what the database stores.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TokenHash([u8; 32]);

impl TokenHash {
    /// Hashes a raw token text.
    #[must_use]
    pub fn of(raw: &str) -> Self {
        Self(Sha256::digest(raw.as_bytes()).into())
    }

    /// The 32 hash bytes.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8; 32] {
        &self.0
    }
}

/// What a token may be used for. A token for one purpose is refused for every other.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Purpose {
    /// Signs a person in by email.
    MagicLink,
    /// Confirms a new account (registration).
    Verify,
    /// Sets a new password.
    Reset,
    /// Adds an email address to a signed-in account.
    LinkEmail,
}

impl Purpose {
    /// The text stored in the database for this purpose.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MagicLink => "magic_link",
            Self::Verify => "verify",
            Self::Reset => "reset",
            Self::LinkEmail => "link_email",
        }
    }

    /// Reads a stored purpose. Returns `None` for any unknown text.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        match text {
            "magic_link" => Some(Self::MagicLink),
            "verify" => Some(Self::Verify),
            "reset" => Some(Self::Reset),
            "link_email" => Some(Self::LinkEmail),
            _ => None,
        }
    }
}

/// Whether a stored token can be used at a given moment.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenStatus {
    /// Issued, not yet used, and inside its lifetime.
    Valid,
    /// Its lifetime has ended (or it is not yet valid).
    Expired,
    /// Already used once.
    Used,
}

/// The expiry time of an email token issued at `issued`.
#[must_use]
pub fn email_token_expiry(issued: OffsetDateTime) -> OffsetDateTime {
    issued + EMAIL_TOKEN_TTL
}

/// The reference rule for a token's state. Used is checked first, then the time window.
#[must_use]
pub fn token_status(
    issued: OffsetDateTime,
    expires: OffsetDateTime,
    used: Option<OffsetDateTime>,
    now: OffsetDateTime,
) -> TokenStatus {
    if used.is_some() {
        TokenStatus::Used
    } else if issued <= now && now < expires {
        TokenStatus::Valid
    } else {
        TokenStatus::Expired
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;
    use time::macros::datetime;

    #[test]
    fn generated_tokens_are_43_base64url_characters() {
        let token = RawToken::generate().expect("operating-system randomness");
        let text = token.expose();
        assert_eq!(text.len(), 43);
        assert!(text
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_'));
    }

    #[test]
    fn ten_thousand_generated_tokens_are_all_different() {
        let seen: HashSet<String> = (0..10_000)
            .map(|_| {
                RawToken::generate()
                    .expect("operating-system randomness")
                    .expose()
                    .to_owned()
            })
            .collect();
        assert_eq!(seen.len(), 10_000);
    }

    #[test]
    fn a_raw_token_never_prints_its_value() {
        let token = RawToken::generate().expect("operating-system randomness");
        let text = format!("{token:?}");
        assert!(!text.contains(token.expose()));
        assert!(text.contains("[redacted]"));
    }

    #[test]
    fn hashing_is_deterministic_and_matches_the_sha256_of_abc() {
        // SHA-256("abc") is a published test vector.
        let hash = TokenHash::of("abc");
        assert_eq!(
            hex::encode(hash.as_bytes()),
            "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
        );
        assert_eq!(TokenHash::of("abc"), hash);
        assert_ne!(TokenHash::of("abd"), hash);
    }

    #[test]
    fn purposes_round_trip_and_unknown_text_is_refused() {
        for purpose in [
            Purpose::MagicLink,
            Purpose::Verify,
            Purpose::Reset,
            Purpose::LinkEmail,
        ] {
            assert_eq!(Purpose::parse(purpose.as_str()), Some(purpose));
        }
        assert_eq!(Purpose::parse("Magic_Link"), None);
        assert_eq!(Purpose::parse(""), None);
    }

    #[test]
    fn an_email_token_lives_exactly_fifteen_minutes() {
        let issued = datetime!(2026-11-13 10:00 UTC);
        assert_eq!(email_token_expiry(issued) - issued, Duration::minutes(15));
    }

    proptest::proptest! {
        #[test]
        fn valid_exactly_when_unused_and_inside_the_window(
            issued_offset in 0_i64..10_000,
            lifetime in 0_i64..2_000,
            used in proptest::bool::ANY,
            now_offset in -100_i64..12_000,
        ) {
            let issued = datetime!(2026-11-13 10:00 UTC) + Duration::seconds(issued_offset);
            let expires = issued + Duration::seconds(lifetime);
            let now = datetime!(2026-11-13 10:00 UTC) + Duration::seconds(now_offset);
            let used_at = used.then_some(issued);
            let model_valid = used_at.is_none() && issued <= now && now < expires;
            let status = token_status(issued, expires, used_at, now);
            proptest::prop_assert_eq!(status == TokenStatus::Valid, model_valid);
        }
    }

    #[test]
    fn a_token_is_valid_exactly_inside_its_window_and_unused() {
        let issued = datetime!(2026-11-13 10:00 UTC);
        let expires = email_token_expiry(issued);
        let just_before = expires - Duration::seconds(1);
        assert_eq!(
            token_status(issued, expires, None, issued),
            TokenStatus::Valid
        );
        assert_eq!(
            token_status(issued, expires, None, just_before),
            TokenStatus::Valid
        );
        assert_eq!(
            token_status(issued, expires, None, expires),
            TokenStatus::Expired
        );
        assert_eq!(
            token_status(issued, expires, Some(issued), issued),
            TokenStatus::Used
        );
        assert_eq!(
            token_status(issued, expires, None, issued - Duration::seconds(1)),
            TokenStatus::Expired
        );
    }
}
