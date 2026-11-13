//! Sign-in, sessions, email links, OAuth sign-in and linking, passwords, and the judge demo account.
//!
//! Everything in this module is server-side. Postgres is the only store for users, identities,
//! sessions, email tokens and OAuth flows. Valkey holds counters only.

use std::sync::Arc;

use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use sqlx::PgPool;

use self::{clock::Clock, cookies::CookieSettings, session::SessionService};

pub mod capabilities;
pub mod clock;
pub mod config;
pub mod cookies;
pub mod csrf;
pub mod decision;
pub mod email;
pub mod errors;
pub mod extract;
pub mod handlers;
pub mod limits;
pub mod resolver;
pub mod return_to;
pub mod session;
pub mod tokens;

/// Everything the auth routes need. It is cheap to clone; each part is shared.
#[derive(Clone)]
pub struct AuthState {
    /// The database pool. Only auth code and the auth-owned tables use it.
    pub pool: PgPool,
    /// Session creation, checks and removal.
    pub sessions: SessionService,
    /// The cookie names and attributes for this deployment.
    pub cookies: CookieSettings,
}

impl AuthState {
    /// Builds the state from a pool, the clock that decides every expiry, and the cookie settings.
    #[must_use]
    pub fn new(pool: PgPool, clock: Arc<dyn Clock>, cookies: CookieSettings) -> Self {
        let sessions = SessionService::new(pool.clone(), clock);
        Self {
            pool,
            sessions,
            cookies,
        }
    }
}

/// The auth routes that exist so far: logout, logout everywhere and `GET /api/me`.
///
/// The cookie middleware sits on this router, so each handler can fill the cookie slot.
pub fn router(state: AuthState) -> Router {
    let routes = Router::new()
        .route("/api/auth/logout", post(handlers::session::logout))
        .route("/api/auth/logout-all", post(handlers::session::logout_all))
        .route("/api/me", get(handlers::me::me))
        .layer(middleware::from_fn(cookies::session_cookie))
        .with_state(state);
    csrf::json_body_guard_layer(routes)
}
