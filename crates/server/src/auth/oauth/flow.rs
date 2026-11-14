//! One-time OAuth flows, stored in Postgres.
//!
//! A flow holds the PKCE verifier, the hashed `state` and, for sign-in, a hashed browser binding.
//! `consume` deletes the row before any check, so every callback uses up its flow, even a failed
//! one. A flow therefore works once and cannot be retried with a corrected value.

use std::sync::Arc;

use sqlx::PgPool;
use subtle::ConstantTimeEq;
use time::OffsetDateTime;

use crate::auth::{
    clock::Clock,
    errors::{database, AuthError},
    oauth::ProviderId,
    return_to::sanitize_return_to,
    session::SessionKey,
    tokens::{RawToken, TokenHash, FLOW_TTL},
};
use counterspell_common::ids::UserId;

/// What a new flow gives back: the values to send to the provider and to the browser.
pub struct NewFlow {
    /// The opaque `state` value sent to the provider. Only its hash is stored.
    pub state: RawToken,
    /// The S256 challenge sent with the authorize request. The verifier stays on the server.
    pub challenge: String,
    /// The browser binding for a sign-in flow, sent as a cookie. Link flows have none, because
    /// they are bound to the signed-in session instead.
    pub binding: Option<RawToken>,
}

/// Why the flow was started.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FlowPurpose {
    /// The browser is signing in.
    Login,
    /// A signed-in user is linking a provider.
    Link,
}

/// A flow that the callback has taken. Only [`FlowStore::consume`] creates one.
pub struct Flow {
    provider: ProviderId,
    purpose: FlowPurpose,
    pkce_verifier: String,
    binding_hash: Option<[u8; 32]>,
    session_key: Option<String>,
    user: Option<UserId>,
    return_to: String,
}

impl Flow {
    /// The verifier that the code exchange must send.
    #[must_use]
    pub fn pkce_verifier(&self) -> &str {
        &self.pkce_verifier
    }

    /// The provider the flow was started for.
    #[must_use]
    pub fn provider(&self) -> ProviderId {
        self.provider
    }

    /// Whether this flow signs in or links.
    #[must_use]
    pub fn purpose(&self) -> FlowPurpose {
        self.purpose
    }

    /// The session a link flow belongs to. `None` for sign-in flows.
    #[must_use]
    pub fn session_key(&self) -> Option<&str> {
        self.session_key.as_deref()
    }

    /// The user a link flow belongs to. `None` for sign-in flows.
    #[must_use]
    pub fn user(&self) -> Option<UserId> {
        self.user
    }

    /// The sanitised place to go after success.
    #[must_use]
    pub fn return_to(&self) -> &str {
        &self.return_to
    }

    /// Checks the browser binding cookie in constant time. A sign-in flow needs the cookie that
    /// was set when it started. A flow without a stored binding never matches.
    #[must_use]
    pub fn binding_matches(&self, binding: Option<&str>) -> bool {
        match (self.binding_hash, binding) {
            (Some(stored), Some(raw)) => {
                let presented = TokenHash::of(raw);
                bool::from(stored.as_slice().ct_eq(presented.as_bytes().as_slice()))
            }
            _ => false,
        }
    }
}

/// One stored flow row, in the column order of the `RETURNING` clause in [`FlowStore::consume`].
type FlowRow = (
    String,
    String,
    String,
    Option<Vec<u8>>,
    Option<String>,
    Option<sqlx::types::Uuid>,
    String,
    OffsetDateTime,
);

/// Creates and takes flows. It holds the pool and the clock, nothing else.
#[derive(Clone)]
pub struct FlowStore {
    pool: PgPool,
    clock: Arc<dyn Clock>,
}

impl FlowStore {
    /// Builds the store over a pool. The clock decides when a flow expires.
    #[must_use]
    pub fn new(pool: PgPool, clock: Arc<dyn Clock>) -> Self {
        Self { pool, clock }
    }

    /// Starts a sign-in flow with a browser binding.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::Internal`] when the random source or the database fails.
    pub async fn create_login(
        &self,
        provider: ProviderId,
        return_to: &str,
    ) -> Result<NewFlow, AuthError> {
        let state = RawToken::generate()?;
        let binding = RawToken::generate()?;
        let (challenge, verifier) = oauth2::PkceCodeChallenge::new_random_sha256();
        let now = self.clock.now();
        sqlx::query(
            "INSERT INTO oauth_flows
                (state_hash, provider, purpose, pkce_verifier, binding_hash, return_to, created_at, expires_at)
             VALUES ($1, $2, 'login', $3, $4, $5, $6, $7)",
        )
        .bind(TokenHash::of(state.expose()).as_bytes().as_slice())
        .bind(provider.as_str())
        .bind(verifier.secret().as_str())
        .bind(TokenHash::of(binding.expose()).as_bytes().as_slice())
        .bind(sanitize_return_to(Some(return_to)))
        .bind(now)
        .bind(now + FLOW_TTL)
        .execute(&self.pool)
        .await
        .map_err(database)?;
        Ok(NewFlow {
            state,
            challenge: challenge.as_str().to_owned(),
            binding: Some(binding),
        })
    }

    /// Starts a link flow bound to the signed-in session and user.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::Internal`] when the random source or the database fails.
    pub async fn create_link(
        &self,
        provider: ProviderId,
        session: &SessionKey,
        user: UserId,
        return_to: &str,
    ) -> Result<NewFlow, AuthError> {
        let state = RawToken::generate()?;
        let (challenge, verifier) = oauth2::PkceCodeChallenge::new_random_sha256();
        let now = self.clock.now();
        sqlx::query(
            "INSERT INTO oauth_flows
                (state_hash, provider, purpose, pkce_verifier, session_id, user_id, return_to, created_at, expires_at)
             VALUES ($1, $2, 'link', $3, $4, $5, $6, $7, $8)",
        )
        .bind(TokenHash::of(state.expose()).as_bytes().as_slice())
        .bind(provider.as_str())
        .bind(verifier.secret().as_str())
        .bind(session.as_str())
        .bind(user.as_uuid())
        .bind(sanitize_return_to(Some(return_to)))
        .bind(now)
        .bind(now + FLOW_TTL)
        .execute(&self.pool)
        .await
        .map_err(database)?;
        Ok(NewFlow {
            state,
            challenge: challenge.as_str().to_owned(),
            binding: None,
        })
    }

    /// Takes the flow for `state`. The row is deleted first, so the flow is used up whatever the
    /// outcome. The result is `None` for an unknown, reused, expired or foreign-provider flow.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::Internal`] when the database fails or a stored value is malformed.
    pub async fn consume(
        &self,
        state: &str,
        provider: ProviderId,
    ) -> Result<Option<Flow>, AuthError> {
        let row: Option<FlowRow> = sqlx::query_as(
            "DELETE FROM oauth_flows WHERE state_hash = $1
             RETURNING provider, purpose, pkce_verifier, binding_hash, session_id, user_id,
                       return_to, expires_at",
        )
        .bind(TokenHash::of(state).as_bytes().as_slice())
        .fetch_optional(&self.pool)
        .await
        .map_err(database)?;

        let Some((stored_provider, purpose, verifier, binding, session, user, return_to, expires)) =
            row
        else {
            return Ok(None);
        };
        let live = expires > self.clock.now();
        let same_provider = ProviderId::parse(&stored_provider) == Some(provider);
        if !live || !same_provider {
            return Ok(None);
        }
        let purpose = match purpose.as_str() {
            "login" => FlowPurpose::Login,
            "link" => FlowPurpose::Link,
            _ => return Err(AuthError::Internal),
        };
        let binding_hash = match binding {
            Some(bytes) => {
                Some(<[u8; 32]>::try_from(bytes.as_slice()).map_err(|_| AuthError::Internal)?)
            }
            None => None,
        };
        Ok(Some(Flow {
            provider,
            purpose,
            pkce_verifier: verifier,
            binding_hash,
            session_key: session,
            user: user.map(UserId::from_uuid),
            return_to,
        }))
    }
}
