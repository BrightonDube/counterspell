//! Counterspell web server: the JSON API, the server-sent event stream and the static web app.
//!
//! The server never talks to Docker and never runs user code. It saves runs and their queue
//! tickets in Postgres, and the judge worker does the rest. Routes are assembled here so that
//! tests can build the same router that `main` serves.

#![warn(missing_docs)]

pub mod config;
pub mod error;
pub mod seams;
#[cfg(feature = "testkit")]
pub mod testkit;

use std::{io, path::Path, sync::Arc};

use axum::{
    http::{header::HeaderName, HeaderValue},
    routing::{any, get},
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
    config::ServerConfig,
    error::ApiError,
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
}

impl AppState {
    /// Bundles a pool and the settings, with the production seams.
    #[must_use]
    pub fn new(pool: PgPool, config: ServerConfig) -> Self {
        let languages = LanguageCatalog::launch(config.limits.max_source_bytes);
        Self {
            pool,
            config: Arc::new(config),
            user_resolver: Arc::new(Unauthenticated),
            problems: Arc::new(EmptyCatalogue),
            languages: Arc::new(languages),
        }
    }

    /// Replaces the user resolver. Used by tests and, later, by the auth spec.
    #[must_use]
    pub fn with_user_resolver(mut self, resolver: impl UserResolver + 'static) -> Self {
        self.user_resolver = Arc::new(resolver);
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

    Router::new()
        .route("/api/health", get(health))
        .route("/api", any(api_not_found))
        .route("/api/{*path}", any(api_not_found))
        .fallback_service(static_files)
        .layer(RequestBodyLimitLayer::new(body_limit))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
        .with_state(state)
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
