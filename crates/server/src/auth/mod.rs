//! Sign-in, sessions, email links, OAuth sign-in and linking, passwords, and the demo account.
//!
//! Everything in this module is server-side. Postgres is the only store for users, identities,
//! sessions, email tokens and OAuth flows. Valkey holds counters only.

pub mod accounts;
pub mod capabilities;
pub mod cleanup;
pub mod clock;
pub mod config;
pub mod cookies;
pub mod csrf;
pub mod decision;
pub mod demo;
pub mod email;
pub mod errors;
pub mod extract;
pub mod handlers;
pub mod limits;
pub mod oauth;
mod passkeys;
pub mod password;
pub mod resolver;
pub mod return_to;
mod security;
pub mod session;
pub mod setup;
pub mod tokens;
mod totp;

use std::{collections::HashMap, sync::Arc};

use axum::{
    http::{header, HeaderValue},
    middleware,
    routing::{delete, get, post},
    Router,
};
use sqlx::PgPool;
use tower_http::set_header::SetResponseHeaderLayer;

use crate::client_ip::TrustedProxies;

use self::{
    clock::Clock,
    cookies::CookieSettings,
    email::{EmailAddress, EmailSender},
    handlers::config::{config_router, PublicSettings},
    limits::AuthLimiter,
    oauth::{flow::FlowStore, OAuthProvider, ProviderId},
    password::HashGate,
    session::SessionService,
    tokens::TokenStore,
};

/// How sign-in mail is sent and where its links point. Absent until the server wires mail in,
/// and then every mail-sending route refuses to run.
#[derive(Clone)]
pub struct MailPort {
    /// The sender. Production uses the guarded Resend sender.
    pub sender: Arc<dyn EmailSender>,
    /// The origin the sign-in links point to, for example `https://counterspell.example`.
    pub public_base_url: String,
    /// The demo account's address. Mail to it is never sent and no token is issued for it.
    pub demo_email: Option<EmailAddress>,
}

/// The OAuth providers that are enabled, and the public origin that callback URLs start with.
#[derive(Clone)]
pub struct OAuthPort {
    /// The origin for callback URLs, for example `https://counterspell.example`.
    pub base_url: String,
    /// One adapter per enabled provider. A provider with no entry is disabled.
    pub providers: HashMap<ProviderId, Arc<dyn OAuthProvider>>,
}

/// Everything the auth routes need. It is cheap to clone; each part is shared.
#[derive(Clone)]
pub struct AuthState {
    /// The database pool. Only auth code and the auth-owned tables use it.
    pub pool: PgPool,
    /// The clock that decides every expiry and timestamp.
    pub clock: Arc<dyn Clock>,
    /// Session creation, checks and removal.
    pub sessions: SessionService,
    /// Single-use email tokens: issue, peek and consume.
    pub tokens: TokenStore,
    /// The cookie names and attributes for this deployment.
    pub cookies: CookieSettings,
    /// The rate limits for auth requests.
    pub limits: AuthLimiter,
    /// The gate that limits password hashing and verification at once.
    pub hashes: HashGate,
    /// The proxies whose forwarding headers are believed when finding the client IP.
    pub trusted: Arc<TrustedProxies>,
    /// Mail settings. `None` means mail is not wired, so mail routes answer with an error.
    pub mail: Option<MailPort>,
    /// One-time OAuth flows: create and take.
    pub flows: FlowStore,
    /// The enabled OAuth providers. `None` means no provider is enabled.
    pub oauth: Option<OAuthPort>,
    /// The public settings that `GET /api/auth/config` shows: which sign-in methods are on.
    pub settings: PublicSettings,
    pub(crate) security: Option<security::SecuritySettings>,
}

impl AuthState {
    /// Builds the state from its parts. The clock decides every expiry.
    #[must_use]
    pub fn new(
        pool: PgPool,
        clock: Arc<dyn Clock>,
        cookies: CookieSettings,
        limits: AuthLimiter,
        hashes: HashGate,
        trusted: TrustedProxies,
    ) -> Self {
        let sessions = SessionService::new(pool.clone(), clock.clone());
        let tokens = TokenStore::new(pool.clone(), clock.clone());
        let flows = FlowStore::new(pool.clone(), clock.clone());
        Self {
            pool,
            clock,
            sessions,
            tokens,
            cookies,
            limits,
            hashes,
            trusted: Arc::new(trusted),
            mail: None,
            flows,
            oauth: None,
            settings: PublicSettings::new(false, false, false),
            security: None,
        }
    }

    /// Sets the public settings that the config route shows.
    #[must_use]
    pub fn with_settings(mut self, settings: PublicSettings) -> Self {
        self.settings = settings;
        self
    }

    /// Adds the mail settings. Without them, the mail routes refuse to send.
    #[must_use]
    pub fn with_mail(mut self, mail: MailPort) -> Self {
        self.mail = Some(mail);
        self
    }

    /// Enables OAuth providers. Each adapter is keyed by the provider it serves.
    #[must_use]
    pub fn with_oauth(mut self, base_url: String, providers: Vec<Arc<dyn OAuthProvider>>) -> Self {
        let providers = providers.into_iter().map(|p| (p.id(), p)).collect();
        self.oauth = Some(OAuthPort {
            base_url,
            providers,
        });
        self
    }

    /// The OAuth settings, or an internal error when none were set.
    ///
    /// # Errors
    ///
    /// Returns [`errors::AuthError::Internal`] when [`AuthState::with_oauth`] was never called.
    pub fn oauth_port(&self) -> Result<&OAuthPort, errors::AuthError> {
        self.oauth.as_ref().ok_or(errors::AuthError::Internal)
    }

    /// The mail settings, or `email_unavailable` when email sign-in is not configured.
    ///
    /// # Errors
    ///
    /// Returns [`errors::AuthError::EmailUnavailable`] when [`AuthState::with_mail`] was never called.
    pub fn mail_port(&self) -> Result<&MailPort, errors::AuthError> {
        self.mail
            .as_ref()
            .ok_or(errors::AuthError::EmailUnavailable)
    }
}

/// Every auth route that changes data, as one concrete path each. The demo matrix test sends a
/// request to each path, so a route missing from the router fails the test. Add a new mutating
/// route here as well.
pub const MUTATING_ROUTES: &[&str] = &[
    "/api/auth/logout",
    "/api/auth/logout-all",
    "/api/auth/login",
    "/api/auth/register",
    "/api/auth/email/magic",
    "/api/auth/email/verify",
    "/api/auth/password/reset",
    "/api/auth/password/reset/confirm",
    "/api/auth/link/github/start",
    "/api/auth/link/email",
    "/api/auth/totp/start",
    "/api/auth/totp/enable",
    "/api/auth/totp/disable",
    "/api/auth/totp/verify",
    "/api/auth/passkeys/register/start",
    "/api/auth/passkeys/register/finish",
    "/api/auth/passkeys/login/start",
    "/api/auth/passkeys/login/finish",
];

/// The auth routes that exist so far: password and email-link sign-in, session status, logout,
/// logout everywhere and `GET /api/me`.
///
/// The cookie middleware sits on this router, so each handler can fill the cookie slot.
pub fn router(state: AuthState) -> Router {
    let settings = state.settings.clone();
    let routes = Router::new()
        .route("/api/auth/session", get(handlers::session::status))
        .route("/api/auth/logout", post(handlers::session::logout))
        .route("/api/auth/logout-all", post(handlers::session::logout_all))
        .route("/api/auth/login", post(handlers::login::login))
        .route("/api/auth/register", post(handlers::register::register))
        .route(
            "/api/auth/password/reset",
            post(handlers::reset::request_reset),
        )
        .route(
            "/api/auth/password/reset/confirm",
            post(handlers::reset::confirm_reset),
        )
        .route(
            "/api/auth/email/magic",
            post(handlers::magic::request_magic),
        )
        .route("/api/auth/email/verify", post(handlers::verify::verify))
        .route(
            "/api/auth/oauth/{provider}/start",
            get(handlers::oauth::start),
        )
        .route(
            "/api/auth/oauth/{provider}/callback",
            get(handlers::oauth::callback),
        )
        .route(
            "/api/auth/link/{provider}/start",
            post(handlers::link::start),
        )
        .route("/api/auth/link/email", post(handlers::link::email))
        .route("/api/auth/security", get(security::summary))
        .route("/api/auth/totp/start", post(totp::start))
        .route("/api/auth/totp/enable", post(totp::enable))
        .route("/api/auth/totp/disable", post(totp::disable))
        .route("/api/auth/totp/verify", post(security::verify))
        .route(
            "/api/auth/passkeys/register/start",
            post(passkeys::register_start),
        )
        .route(
            "/api/auth/passkeys/register/finish",
            post(passkeys::register_finish),
        )
        .route(
            "/api/auth/passkeys/login/start",
            post(passkeys::login_start),
        )
        .route(
            "/api/auth/passkeys/login/finish",
            post(passkeys::login_finish),
        )
        .route("/api/auth/passkeys/{id}", delete(passkeys::remove))
        .route(
            "/api/me",
            get(handlers::me::me).delete(handlers::delete::delete_account),
        )
        .layer(middleware::from_fn(cookies::session_cookie))
        .with_state(state);
    let routes = routes.merge(config_router(settings));
    csrf::json_body_guard_layer(routes).layer(SetResponseHeaderLayer::if_not_present(
        header::CACHE_CONTROL,
        HeaderValue::from_static("no-store"),
    ))
}
