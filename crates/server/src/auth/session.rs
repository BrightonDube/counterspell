//! Server-side sessions: one row per signed-in browser, keyed by the hash of its cookie value.
//!
//! The database decides whether a session is valid. A session lasts 30 days from its last renewal,
//! and never more than 90 days from its creation. Renewal happens at most once an hour.

use std::sync::Arc;

use counterspell_common::ids::UserId;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

use super::{clock::Clock, errors::database, errors::AuthError, tokens::RawToken};

/// How long a session lasts after its last renewal.
pub const SESSION_IDLE: Duration = Duration::days(30);
/// The longest a session may live, however often it is used.
pub const SESSION_ABSOLUTE: Duration = Duration::days(90);
/// The shortest gap between two renewals of the same session.
pub const RENEW_EVERY: Duration = Duration::hours(1);

/// The stored key of a session: the SHA-256 of the cookie value, as lower-case hex.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SessionKey(String);

impl SessionKey {
    /// Hashes a raw cookie value into the key stored in the database.
    #[must_use]
    pub fn of(raw: &str) -> Self {
        Self(hex::encode(Sha256::digest(raw.as_bytes())))
    }

    /// The key as the text stored in the `sessions.id` column.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// How the user proved who they are. Stored for audit only.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AuthMethod {
    /// A single-use email sign-in link.
    MagicLink,
    /// GitHub sign-in.
    OauthGithub,
    /// Google sign-in.
    OauthGoogle,
    /// Email and password.
    Password,
    /// The first sign-in after a verified registration.
    Register,
}

impl AuthMethod {
    /// The text stored in `sessions.auth_method`. It must match the table's CHECK list.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::MagicLink => "magic_link",
            Self::OauthGithub => "oauth_github",
            Self::OauthGoogle => "oauth_google",
            Self::Password => "password",
            Self::Register => "register",
        }
    }
}

/// A session that was just created. The raw value goes into the cookie and nowhere else.
pub struct NewSession {
    /// The value for the `Set-Cookie` header. It is shown once.
    pub cookie_value: RawToken,
    /// The stored key of the new session.
    pub key: SessionKey,
}

/// A session that passed the checks in [`SessionService::authenticate`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Authenticated {
    /// The signed-in user.
    pub user_id: UserId,
    /// The stored key of the session.
    pub key: SessionKey,
    /// Whether the user is the judge demo account.
    pub is_demo: bool,
    /// True when this request moved the session's expiry forward.
    pub renewed: bool,
}

/// Creates, checks, renews and removes sessions. It holds the pool and the clock, nothing else.
#[derive(Clone)]
pub struct SessionService {
    pool: PgPool,
    clock: Arc<dyn Clock>,
}

impl SessionService {
    /// Builds the service over a pool and a clock.
    #[must_use]
    pub fn new(pool: PgPool, clock: Arc<dyn Clock>) -> Self {
        Self { pool, clock }
    }

    /// Starts a new session for `user`. Every call gives a new cookie value.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::Internal`] when the random source or the database fails.
    pub async fn create(&self, user: UserId, method: AuthMethod) -> Result<NewSession, AuthError> {
        let raw = RawToken::generate()?;
        let key = SessionKey::of(raw.expose());
        let now = self.clock.now();
        sqlx::query(
            "INSERT INTO sessions (id, user_id, auth_method, created_at, last_renewed_at,
                                   expires_at, absolute_expires_at)
             VALUES ($1, $2, $3, $4, $4, $5, $6)",
        )
        .bind(key.as_str())
        .bind(user.as_uuid())
        .bind(method.as_str())
        .bind(now)
        .bind(now + SESSION_IDLE)
        .bind(now + SESSION_ABSOLUTE)
        .execute(&self.pool)
        .await
        .map_err(database)?;
        Ok(NewSession {
            cookie_value: raw,
            key,
        })
    }

    /// Checks a cookie value. Returns the user when the session is still valid, renewing it when
    /// the last renewal is more than an hour old.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::Internal`] when the database fails.
    pub async fn authenticate(&self, raw: &str) -> Result<Option<Authenticated>, AuthError> {
        let key = SessionKey::of(raw);
        let now = self.clock.now();
        let row: Option<(Uuid, OffsetDateTime, bool)> = sqlx::query_as(
            "SELECT s.user_id, s.last_renewed_at, u.is_demo
               FROM sessions s JOIN users u ON u.id = s.user_id
              WHERE s.id = $1 AND s.expires_at > $2 AND s.absolute_expires_at > $2",
        )
        .bind(key.as_str())
        .bind(now)
        .fetch_optional(&self.pool)
        .await
        .map_err(database)?;
        let Some((user_id, last_renewed_at, is_demo)) = row else {
            return Ok(None);
        };

        let renewed = if now - last_renewed_at >= RENEW_EVERY {
            // The guard in the WHERE clause makes two concurrent requests renew only once.
            let result = sqlx::query(
                "UPDATE sessions
                    SET last_renewed_at = $2,
                        expires_at = LEAST($3, absolute_expires_at)
                  WHERE id = $1 AND last_renewed_at <= $4",
            )
            .bind(key.as_str())
            .bind(now)
            .bind(now + SESSION_IDLE)
            .bind(now - RENEW_EVERY)
            .execute(&self.pool)
            .await
            .map_err(database)?;
            result.rows_affected() == 1
        } else {
            false
        };

        Ok(Some(Authenticated {
            user_id: UserId::from_uuid(user_id),
            key,
            is_demo,
            renewed,
        }))
    }

    /// Ends one session. Ending a session that is already gone does nothing.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::Internal`] when the database fails.
    pub async fn revoke(&self, key: &SessionKey) -> Result<(), AuthError> {
        sqlx::query("DELETE FROM sessions WHERE id = $1")
            .bind(key.as_str())
            .execute(&self.pool)
            .await
            .map_err(database)?;
        Ok(())
    }

    /// Ends every session of one user, on every device. Returns how many were removed.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::Internal`] when the database fails.
    pub async fn revoke_all(&self, user: UserId) -> Result<u64, AuthError> {
        let result = sqlx::query("DELETE FROM sessions WHERE user_id = $1")
            .bind(user.as_uuid())
            .execute(&self.pool)
            .await
            .map_err(database)?;
        Ok(result.rows_affected())
    }

    /// Says whether a stored session is still valid. Long-lived streams call this on each
    /// heartbeat, so a logout ends them.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::Internal`] when the database fails.
    pub async fn is_valid(&self, key: &SessionKey) -> Result<bool, AuthError> {
        let now = self.clock.now();
        sqlx::query_scalar(
            "SELECT EXISTS (SELECT 1 FROM sessions
                             WHERE id = $1 AND expires_at > $2 AND absolute_expires_at > $2)",
        )
        .bind(key.as_str())
        .bind(now)
        .fetch_one(&self.pool)
        .await
        .map_err(database)
    }
}
