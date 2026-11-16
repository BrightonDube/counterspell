//! Single-use tokens: the values inside email links, OAuth `state` and sessions.
//!
//! A raw token is 256 bits from the operating system's random source, encoded as unpadded
//! base64url (43 characters). Only its SHA-256 hash is ever stored. This file holds the pure
//! parts: generating, hashing, purposes and the expiry model. The database store comes later.

use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use rand::{rngs::OsRng, TryRngCore};
use sha2::{Digest, Sha256};
use time::{Duration, OffsetDateTime};

use std::sync::Arc;

use counterspell_common::ids::UserId;
use sqlx::PgPool;
use uuid::Uuid;

use super::{
    clock::Clock,
    email::EmailAddress,
    errors::{database, AuthError},
};

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

/// A live token, read without using it. The verify handler uses it to choose the purpose and the
/// user to demand before it consumes the token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Peeked {
    /// What the token may be used for.
    pub purpose: Purpose,
    /// The account the token is bound to, if any.
    pub user_id: Option<UserId>,
}

/// A token that was just used. It carries the address and the data stored with it.
#[derive(Debug, Clone)]
pub struct Consumed {
    /// The address the token was issued for.
    pub email: EmailAddress,
    /// The account the token was bound to, if any.
    pub user_id: Option<UserId>,
    /// The data stored when the token was issued, for example a password hash for registration.
    pub payload: serde_json::Value,
}

/// Issues, reads and uses email tokens. Only hashes reach the database, and every use is one
/// atomic statement, so a token can be used once even when many requests race.
#[derive(Clone)]
pub struct TokenStore {
    pool: PgPool,
    clock: Arc<dyn Clock>,
}

impl TokenStore {
    /// Builds the store over a pool and the clock that decides every expiry.
    #[must_use]
    pub fn new(pool: PgPool, clock: Arc<dyn Clock>) -> Self {
        Self { pool, clock }
    }

    /// Issues a token for `email`. Any earlier live token for the same address and purpose stops
    /// working, so only the newest one can be used.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::Internal`] when the random source or the database fails.
    pub async fn issue(
        &self,
        purpose: Purpose,
        email: &EmailAddress,
        user: Option<UserId>,
        payload: serde_json::Value,
    ) -> Result<RawToken, AuthError> {
        let raw = RawToken::generate()?;
        let hash = TokenHash::of(raw.expose());
        let now = self.clock.now();
        let mut transaction = self.pool.begin().await.map_err(database)?;
        sqlx::query(
            "UPDATE email_tokens SET used_at = $3
              WHERE email = $1 AND purpose = $2 AND used_at IS NULL",
        )
        .bind(email.as_str())
        .bind(purpose.as_str())
        .bind(now)
        .execute(&mut *transaction)
        .await
        .map_err(database)?;
        sqlx::query(
            "INSERT INTO email_tokens (token_hash, purpose, email, user_id, payload, created_at, expires_at)
             VALUES ($1, $2, $3, $4, $5, $6, $7)",
        )
        .bind(hash.as_bytes().as_slice())
        .bind(purpose.as_str())
        .bind(email.as_str())
        .bind(user.map(UserId::as_uuid))
        .bind(payload)
        .bind(now)
        .bind(email_token_expiry(now))
        .execute(&mut *transaction)
        .await
        .map_err(database)?;
        transaction.commit().await.map_err(database)?;
        Ok(raw)
    }

    /// Reads a live token without using it. Returns `None` for an unknown, used, expired or
    /// unrecognised token.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::Internal`] when the database fails.
    pub async fn peek(&self, raw: &str) -> Result<Option<Peeked>, AuthError> {
        let now = self.clock.now();
        let row: Option<(String, Option<Uuid>)> = sqlx::query_as(
            "SELECT purpose, user_id FROM email_tokens
              WHERE token_hash = $1 AND used_at IS NULL AND expires_at > $2",
        )
        .bind(TokenHash::of(raw).as_bytes().as_slice())
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map_err(database)?;
        Ok(row.and_then(|(purpose, user)| {
            Purpose::parse(&purpose).map(|purpose| Peeked {
                purpose,
                user_id: user.map(UserId::from_uuid),
            })
        }))
    }

    /// Uses a token once. The purpose must match, the token must be unused and inside its
    /// lifetime, and when `bound_user` is given the token must belong to that user. Any failure
    /// returns the same error and changes nothing.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::InvalidOrExpiredToken`] for every refusal, and
    /// [`AuthError::Internal`] when the database fails.
    pub async fn consume(
        &self,
        raw: &str,
        purpose: Purpose,
        bound_user: Option<UserId>,
    ) -> Result<Consumed, AuthError> {
        let now = self.clock.now();
        let row: Option<(String, Option<Uuid>, serde_json::Value)> = sqlx::query_as(
            "UPDATE email_tokens SET used_at = $3
              WHERE token_hash = $1 AND purpose = $2 AND used_at IS NULL AND expires_at > $3
                AND ($4::uuid IS NULL OR user_id = $4)
          RETURNING email::text, user_id, payload",
        )
        .bind(TokenHash::of(raw).as_bytes().as_slice())
        .bind(purpose.as_str())
        .bind(now)
        .bind(bound_user.map(UserId::as_uuid))
        .fetch_optional(&self.pool)
        .await
        .map_err(database)?;
        let (email, user, payload) = row.ok_or(AuthError::InvalidOrExpiredToken)?;
        let email = EmailAddress::parse(&email).map_err(|_| AuthError::Internal)?;
        Ok(Consumed {
            email,
            user_id: user.map(UserId::from_uuid),
            payload,
        })
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
