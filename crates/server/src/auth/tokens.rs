//! Single-use tokens: the values inside OAuth `state` and sessions.
//!
//! A raw token is 256 bits from the operating system's random source, encoded as unpadded
//! base64url (43 characters). Only its SHA-256 hash is ever stored. This file holds the pure
//! parts: generating and hashing.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::{rngs::OsRng, TryRngCore};
use sha2::{Digest, Sha256};
use time::Duration;

use super::errors::AuthError;

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

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
}
