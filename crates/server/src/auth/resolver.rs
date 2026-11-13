//! The run-lifecycle `UserResolver` backed by real sessions. Run endpoints use it through the
//! same `authenticate` path as the auth routes, so both see the same user.

use async_trait::async_trait;
use axum::http::HeaderMap;
use counterspell_common::ids::UserId;

use crate::seams::UserResolver;

use super::{cookies::read_cookie, session::SessionService};

/// Finds the signed-in user from the session cookie. A database failure counts as signed out,
/// so the run endpoints refuse the request instead of guessing.
#[derive(Clone)]
pub struct SessionUserResolver {
    sessions: SessionService,
    cookie_name: String,
}

impl SessionUserResolver {
    /// Builds the resolver for the named session cookie.
    #[must_use]
    pub fn new(sessions: SessionService, cookie_name: String) -> Self {
        Self {
            sessions,
            cookie_name,
        }
    }
}

#[async_trait]
impl UserResolver for SessionUserResolver {
    async fn resolve(&self, headers: &HeaderMap) -> Option<UserId> {
        let raw = read_cookie(headers, &self.cookie_name)?;
        self.sessions
            .authenticate(raw)
            .await
            .ok()
            .flatten()
            .map(|found| found.user_id)
    }
}
