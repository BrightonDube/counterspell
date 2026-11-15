//! Counterspell web server: the JSON API, the server-sent event stream and the static web app.
//!
//! The server never talks to Docker and never runs user code. It saves runs and their queue
//! tickets in Postgres, and the judge worker does the rest. Routes are assembled here so that
//! tests can build the same router that `main` serves.

#![warn(missing_docs)]

pub mod auth;
pub mod client_ip;
pub mod config;
pub mod error;
pub mod familiar;
pub mod ratelimit;
pub mod runs;
pub mod seams;
#[cfg(feature = "testkit")]
pub mod testkit;
pub mod validate;

use std::{io, path::Path, sync::Arc};

use axum::{
    http::{header::HeaderName, HeaderValue},
    routing::{any, get, post},
    Json, Router,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower_http::{
    limit::RequestBodyLimitLayer,
    services::{ServeDir, ServeFile},
    set_header::SetResponseHeaderLayer,
};

use crate::{
    auth::{
        csrf::{apply_api_hardening, WebOrigin},
        resolver::SessionUserResolver,
        AuthState,
    },
    config::ServerConfig,
    error::ApiError,
    ratelimit::Limiter,
    seams::{EmptyCatalogue, LanguageCatalog, ProblemLookup, Unauthenticated, UserResolver},
};

/// Shared state for every request handler. Cloning is cheap: the pool and the seams are shared.
#[derive(Clone)]
pub struct AppState {
    /// Postgres pool. Connections are opened lazily, so start-up does not need the database.
    pub pool: PgPool,
    /// Settings read at start-up.
    pub config: Arc<ServerConfig>,
    /// Identifies the caller. Production never identifies anyone until the auth spec lands.
    pub user_resolver: Arc<dyn UserResolver>,
    /// Finds published problems. Production publishes none until the catalogue lands.
    pub problems: Arc<dyn ProblemLookup>,
    /// The accepted languages and source limits.
    pub languages: Arc<LanguageCatalog>,
    /// Counts run submissions per user and per IP address.
    pub limiter: Limiter,
    /// Tells open event streams when a run has new events. Idle until `main` starts the listener.
    pub run_waker: runs::wake::RunWaker,
    /// The auth state. `None` mounts no auth routes and leaves the user resolver as it was.
    pub auth: Option<AuthState>,
    /// The site's own origin for the CSRF guard. Set together with `auth`.
    pub origin: Option<Arc<WebOrigin>>,
}

impl AppState {
    /// Bundles a pool, the settings and the rate limiter, with the production seams.
    #[must_use]
    pub fn new(pool: PgPool, config: ServerConfig, limiter: Limiter) -> Self {
        let languages = LanguageCatalog::launch(config.limits.max_source_bytes);
        Self {
            pool,
            config: Arc::new(config),
            user_resolver: Arc::new(Unauthenticated),
            problems: Arc::new(EmptyCatalogue),
            languages: Arc::new(languages),
            limiter,
            run_waker: runs::wake::RunWaker::idle(),
            auth: None,
            origin: None,
        }
    }

    /// Mounts the auth routes, makes sessions identify the caller, and turns on the CSRF guard for
    /// every `/api` route. `public_base_url` is the site origin the guard compares `Origin` with.
    ///
    /// # Errors
    ///
    /// Returns the URL parser's error when `public_base_url` is not an absolute URL.
    pub fn with_auth(
        mut self,
        auth: AuthState,
        public_base_url: &str,
    ) -> Result<Self, url::ParseError> {
        let origin = WebOrigin::from_base_url(public_base_url)?;
        self.user_resolver = Arc::new(SessionUserResolver::new(
            auth.sessions.clone(),
            auth.cookies.session_name().to_owned(),
        ));
        self.auth = Some(auth);
        self.origin = Some(Arc::new(origin));
        Ok(self)
    }

    /// Replaces the user resolver. Used by tests and, later, by the auth spec.
    #[must_use]
    pub fn with_user_resolver(mut self, resolver: impl UserResolver + 'static) -> Self {
        self.user_resolver = Arc::new(resolver);
        self
    }

    /// Replaces the run waker. `main` passes a started listener; tests can pass an idle one.
    #[must_use]
    pub fn with_run_waker(mut self, waker: runs::wake::RunWaker) -> Self {
        self.run_waker = waker;
        self
    }

    /// Replaces the problem lookup. Used by tests and, later, by the catalogue spec.
    #[must_use]
    pub fn with_problems(mut self, problems: impl ProblemLookup + 'static) -> Self {
        self.problems = Arc::new(problems);
        self
    }
}

/// Builds the router: `/api/*` routes, then the static web app with its single-page fallback.
///
/// Unknown `/api/*` paths return a JSON `404`, never the HTML fallback.
pub fn build_router(state: AppState) -> Router {
    let static_dir = state.config.static_dir.clone();
    let index_file = static_dir.join("index.html");
    let static_files = ServeDir::new(static_dir).fallback(ServeFile::new(index_file));
    let body_limit =
        usize::try_from(state.config.limits.request_body_limit_bytes).unwrap_or(usize::MAX);

    let hardening = state.auth.clone().zip(state.origin.clone());
    let api = Router::new()
        .route("/api/health", get(health))
        .route("/api/runs", post(runs::create::post_run))
        .route("/api/runs/{id}", get(runs::read::get_run))
        .route("/api/runs/{id}/cancel", post(runs::cancel::cancel_run))
        .route(
            "/api/me/familiar",
            get(familiar::api::get_prefs).patch(familiar::api::patch_prefs),
        )
        .route("/api", any(api_not_found))
        .route("/api/{*path}", any(api_not_found))
        .with_state(state);
    let api = match hardening {
        Some((auth, origin)) => apply_api_hardening(api.merge(auth::router(auth)), origin),
        None => api,
    };

    api.fallback_service(static_files)
        .layer(RequestBodyLimitLayer::new(body_limit))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn api_not_found() -> ApiError {
    ApiError::not_found("API route not found")
}

/// Checks that the static directory and its `index.html` exist, so a bad deploy fails at start-up.
///
/// # Errors
///
/// Returns an error naming the missing path when the directory or the entry file is absent.
pub fn ensure_static_files(static_dir: &Path) -> Result<(), io::Error> {
    let metadata = std::fs::metadata(static_dir).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "static directory {} is unavailable: {error}",
                static_dir.display()
            ),
        )
    })?;
    if !metadata.is_dir() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("static path {} is not a directory", static_dir.display()),
        ));
    }

    let index_file = static_dir.join("index.html");
    let index_metadata = std::fs::metadata(&index_file).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "SPA entry point {} is unavailable: {error}",
                index_file.display()
            ),
        )
    })?;
    if !index_metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("SPA entry point {} is not a file", index_file.display()),
        ));
    }

    Ok(())
}
