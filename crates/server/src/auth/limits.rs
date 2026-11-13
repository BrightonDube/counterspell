//! Rate limits for the auth routes. Counters live in Valkey (through the shared window limiter);
//! keys hold a keyed hash of the identity, never the address itself.
//!
//! Mail actions fail closed: when the counter store is down, nothing is sent. Sign-in actions
//! fail open: a Valkey outage must not lock everyone out of their password.

use std::sync::Arc;

use hmac::{Hmac, Mac};
use sha2::Sha256;
use time::Date;

use super::{config::Redacted, errors::AuthError};
use crate::ratelimit::{Decision, Limiter, RateLimiter};

/// One kind of auth request that is limited.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    /// A magic sign-in link is requested.
    Magic,
    /// A registration is started.
    Register,
    /// A password reset is requested.
    Reset,
    /// An email address is added to a signed-in account.
    LinkEmail,
    /// A verification link is opened.
    Verify,
    /// A password sign-in fails.
    LoginFailure,
    /// A provider sign-in or link flow starts.
    OAuthStart,
    /// A provider link flow starts for a signed-in user.
    LinkStart,
}

/// Whom a limit counts: an address, an IP, an address and IP pair, or a user.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Scope {
    /// The normalised email address.
    Email,
    /// The client IP address.
    Ip,
    /// The email address and the client IP together.
    EmailAndIp,
    /// The signed-in user's id.
    User,
}

impl Action {
    /// The short name used in the Valkey key.
    fn name(self) -> &'static str {
        match self {
            Self::Magic => "magic",
            Self::Register => "register",
            Self::Reset => "reset",
            Self::LinkEmail => "link_email",
            Self::Verify => "verify",
            Self::LoginFailure => "login",
            Self::OAuthStart => "oauth_start",
            Self::LinkStart => "link_start",
        }
    }

    /// Mail actions stop when the counter store is down.
    fn fails_closed(self) -> bool {
        matches!(
            self,
            Self::Magic | Self::Register | Self::Reset | Self::LinkEmail
        )
    }

    /// The limit (hits allowed) and window (seconds) for this action and scope. `None` means the
    /// pair is not used.
    fn policy(self, scope: Scope) -> Option<(u64, u64)> {
        match (self, scope) {
            (Self::Magic | Self::Register | Self::Reset | Self::LinkEmail, Scope::Email) => {
                Some((3, 3600))
            }
            (Self::Magic | Self::Register | Self::Reset | Self::LinkEmail, Scope::Ip) => {
                Some((10, 3600))
            }
            (Self::Verify, Scope::Ip) => Some((30, 600)),
            (Self::LoginFailure, Scope::EmailAndIp) => Some((5, 900)),
            (Self::LoginFailure, Scope::Ip) => Some((20, 900)),
            (Self::OAuthStart, Scope::Ip) => Some((20, 600)),
            (Self::LinkStart, Scope::User) => Some((10, 3600)),
            _ => None,
        }
    }
}

impl Scope {
    fn name(self) -> &'static str {
        match self {
            Self::Email => "email",
            Self::Ip => "ip",
            Self::EmailAndIp => "email_ip",
            Self::User => "user",
        }
    }
}

/// Counts auth requests against their limits, and checks the global daily mail cap.
#[derive(Clone)]
pub struct AuthLimiter {
    limiter: Limiter,
    secret: Arc<Redacted<Vec<u8>>>,
}

impl AuthLimiter {
    /// Builds the limiter. The session secret keys the hashes, so keys cannot be reversed.
    #[must_use]
    pub fn new(limiter: Limiter, session_secret: &Redacted<Vec<u8>>) -> Self {
        Self {
            limiter,
            secret: Arc::new(Redacted::new(session_secret.expose().clone())),
        }
    }

    /// The Valkey key for one identity: `rl:auth:<action>:<scope>:<hmac>`.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::Internal`] when the action and scope are not a limited pair.
    pub fn key(&self, action: Action, scope: Scope, identity: &str) -> Result<String, AuthError> {
        if action.policy(scope).is_none() {
            return Err(AuthError::Internal);
        }
        let mut mac = Hmac::<Sha256>::new_from_slice(self.secret.expose())
            .map_err(|_| AuthError::Internal)?;
        mac.update(identity.as_bytes());
        let digest = hex::encode(mac.finalize().into_bytes());
        Ok(format!(
            "rl:auth:{}:{}:{digest}",
            action.name(),
            scope.name()
        ))
    }

    /// Counts one request. Returns [`AuthError::RateLimited`] when it is over the limit.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::RateLimited`] over the limit, [`AuthError::EmailUnavailable`] for a
    /// mail action when the counter store is down, and [`AuthError::Internal`] for an unknown pair.
    pub async fn hit(&self, action: Action, scope: Scope, identity: &str) -> Result<(), AuthError> {
        let Some((limit, window)) = action.policy(scope) else {
            return Err(AuthError::Internal);
        };
        let key = self.key(action, scope, identity)?;
        match self.limiter.hit(&key, limit, window).await {
            Decision::Limited {
                retry_after_seconds,
            } => Err(AuthError::RateLimited {
                retry_after_seconds,
            }),
            Decision::SkippedFailOpen if action.fails_closed() => Err(AuthError::EmailUnavailable),
            Decision::Allowed { .. } | Decision::SkippedFailOpen => Ok(()),
        }
    }

    /// Counts one mail for the global daily cap. The key holds the UTC date, so the count starts
    /// again on the next UTC day.
    ///
    /// # Errors
    ///
    /// Returns [`AuthError::EmailUnavailable`] when the cap is reached or the counter store is down.
    pub async fn hit_daily_mail_cap(&self, today: Date, cap: u32) -> Result<(), AuthError> {
        let key = format!("rl:auth:mail:day:{}", today.to_calendar_date_string());
        match self.limiter.hit(&key, u64::from(cap), 2 * 86_400).await {
            Decision::Allowed { .. } => Ok(()),
            Decision::Limited { .. } | Decision::SkippedFailOpen => {
                Err(AuthError::EmailUnavailable)
            }
        }
    }
}

/// Text form of a date as `yyyymmdd`, for example `20261113`.
trait CalendarText {
    fn to_calendar_date_string(self) -> String;
}

impl CalendarText for Date {
    fn to_calendar_date_string(self) -> String {
        format!(
            "{:04}{:02}{:02}",
            self.year(),
            u8::from(self.month()),
            self.day()
        )
    }
}
