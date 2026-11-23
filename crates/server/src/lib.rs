//! Counterspell web server: the JSON API, the server-sent event stream and the static web app.
//!
//! The server never talks to Docker and never runs user code. It saves runs and their queue
//! tickets in Postgres, and the judge worker does the rest. Routes are assembled here so that
//! tests can build the same router that `main` serves.

#![warn(missing_docs)]

pub mod auth;
pub mod breakroom;
pub mod catalogue;
pub mod client_ip;
pub mod config;
pub mod dashboard;
pub mod error;
pub mod familiar;
pub mod health;
pub mod hints;
pub mod ratelimit;
pub mod reviews;
pub mod runs;
pub mod seams;
pub mod spa;
#[cfg(feature = "testkit")]
pub mod testkit;
pub mod validate;

use std::{io, path::Path, sync::Arc};

use axum::{
    body::Body,
    extract::State,
    http::{
        header::{self, HeaderName},
        HeaderValue, Method, Request, StatusCode,
    },
    middleware::{self, Next},
    response::Response,
    routing::{any, get, post},
    Json, Router,
};
use serde_json::{json, Value};
use sqlx::PgPool;
use tower::ServiceExt;
use tower_http::{
    limit::RequestBodyLimitLayer,
    services::{ServeDir, ServeFile},
    set_header::{SetResponseHeader, SetResponseHeaderLayer},
};

use crate::{
    auth::{
        csrf::{apply_api_hardening, WebOrigin},
        resolver::SessionUserResolver,
        AuthState,
    },
    catalogue::lookup::PgProblemLookup,
    config::ServerConfig,
    error::ApiError,
    ratelimit::Limiter,
    seams::{LanguageCatalog, ProblemLookup, Unauthenticated, UserResolver},
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
    /// Limits how many event streams one user may hold open.
    pub stream_slots: runs::streams::StreamSlots,
    /// The auth state. `None` mounts no auth routes and leaves the user resolver as it was.
    pub auth: Option<AuthState>,
    /// The site's own origin for the CSRF guard. Set together with `auth`.
    pub origin: Option<Arc<WebOrigin>>,
    /// The dependencies that `GET /api/ready` pings (Postgres and Valkey in production).
    pub readiness: Arc<health::Readiness>,
    /// Hint settings, quota counters and the provider transport. AI stays off until `with_hints`.
    pub hints: Arc<hints::handlers::HintState>,
    /// The one-attempt marker for the familiar's speech bubble. `None` keeps the bubble on the template.
    pub familiar_attempt: Option<familiar::attempt::AttemptMarker>,
}

impl AppState {
    /// Bundles a pool, the settings and the rate limiter, with the production seams. Problems are
    /// read from the catalogue through the server role (CAT-L2).
    #[must_use]
    pub fn new(pool: PgPool, config: ServerConfig, limiter: Limiter) -> Self {
        let languages = LanguageCatalog::launch(config.limits.max_source_bytes);
        let problems = PgProblemLookup::new(pool.clone());
        let stream_slots = runs::streams::StreamSlots::new(config.sse.max_streams_per_user);
        let readiness = health::Readiness::new(vec![
            Arc::new(health::PostgresProbe::new(pool.clone())),
            Arc::new(health::LimiterProbe::new(limiter.clone())),
        ]);
        let hints = Arc::new(hints::handlers::HintState::disabled(pool.clone()));
        Self {
            pool,
            config: Arc::new(config),
            user_resolver: Arc::new(Unauthenticated),
            problems: Arc::new(problems),
            languages: Arc::new(languages),
            limiter,
            run_waker: runs::wake::RunWaker::idle(),
            stream_slots,
            auth: None,
            origin: None,
            readiness: Arc::new(readiness),
            hints,
            familiar_attempt: None,
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

    /// Replaces the dependencies that readiness pings. Used by tests with fake dependencies.
    #[must_use]
    pub fn with_readiness(mut self, probes: Vec<Arc<dyn health::Probe>>) -> Self {
        self.readiness = Arc::new(health::Readiness::new(probes));
        self
    }

    /// Sets the one-attempt marker the speech bubble uses. `main` builds it from the Valkey URL.
    #[must_use]
    pub fn with_familiar_attempt(mut self, marker: familiar::attempt::AttemptMarker) -> Self {
        self.familiar_attempt = Some(marker);
        self
    }

    /// Replaces the hint state (settings, quota store and transport). `main` builds it from the environment.
    #[must_use]
    pub fn with_hints(mut self, hints: hints::handlers::HintState) -> Self {
        self.hints = Arc::new(hints);
        self
    }
}

/// File name of the single-page fallback. The `SvelteKit` static adapter writes it, so `/` keeps its own prerendered `index.html`.
pub const SPA_FALLBACK_FILE: &str = "200.html";

/// Marks responses that came from the SPA entry file, so the policy layer can
/// tell a fallback hit from a real file without re-deriving the request path.
const SPA_FALLBACK_HEADER: HeaderName = HeaderName::from_static("x-counterspell-spa-fallback");

/// Builds the router: `/api/*` routes, then the static web app with its single-page fallback.
///
/// Unknown `/api/*` paths return a JSON `404`, never the HTML fallback.
pub fn build_router(state: AppState) -> Router {
    let static_dir = state.config.static_dir.clone();
    let index_file = static_dir.join(SPA_FALLBACK_FILE);
    let entry_file = ServeFile::new(index_file);
    let spa_entry = SetResponseHeader::overriding(
        entry_file.clone(),
        SPA_FALLBACK_HEADER,
        HeaderValue::from_static("1"),
    );
    let static_files = ServeDir::new(static_dir.clone()).fallback(spa_entry);
    let spa_routes = spa::SpaRoutes::load(&static_dir).unwrap_or_else(|error| {
        tracing::warn!(
            "SPA route manifest unavailable ({error}); every fallback hit will be a 404"
        );
        spa::SpaRoutes::default()
    });
    let body_limit =
        usize::try_from(state.config.limits.request_body_limit_bytes).unwrap_or(usize::MAX);

    let hardening = state.auth.clone().zip(state.origin.clone());
    let api = Router::new()
        .route("/api/health", get(health))
        .route("/api/ready", get(health::ready))
        .route("/api/languages", get(catalogue::languages::get_languages))
        .route("/api/problems", get(catalogue::handlers::list_problems))
        .route(
            "/api/problems/{slug}",
            get(catalogue::handlers::get_problem),
        )
        .route("/api/dashboard", get(dashboard::handler::get_dashboard))
        .route("/api/reviews/due", get(reviews::handler::get_due_reviews))
        .route(
            "/api/reviews/{concept}",
            post(reviews::handler::post_review),
        )
        .route(
            "/api/hints",
            post(hints::handlers::post_hint).get(hints::handlers::get_hints),
        )
        .route("/api/runs", post(runs::create::post_run))
        .route("/api/runs/ghost", post(runs::custom::post_ghost_run))
        .route("/api/runs/{id}", get(runs::read::get_run))
        .route("/api/runs/{id}/cancel", post(runs::cancel::cancel_run))
        .route("/api/runs/{id}/events", get(runs::sse::stream_run))
        .route(
            "/api/runs/{id}/familiar-hint",
            post(familiar::hint::post_familiar_hint),
        )
        .route(
            "/api/me/familiar",
            get(familiar::api::get_prefs).patch(familiar::api::patch_prefs),
        )
        .route("/api", any(api_not_found))
        .nest("/api/breakroom", breakroom::router())
        .route("/api/{*path}", any(api_not_found))
        .with_state(state);
    let api = match hardening {
        Some((auth, origin)) => apply_api_hardening(api.merge(auth::router(auth)), origin),
        None => api,
    };

    api.fallback_service(static_files)
        .layer(middleware::from_fn_with_state(
            StaticFallback {
                routes: Arc::new(spa_routes),
                entry_file,
            },
            static_response_policy,
        ))
        .layer(RequestBodyLimitLayer::new(body_limit))
        .layer(SetResponseHeaderLayer::overriding(
            HeaderName::from_static("x-content-type-options"),
            HeaderValue::from_static("nosniff"),
        ))
}

/// The static response contract: real files may receive a cache policy, and a
/// `200.html` fallback hit keeps its 200 only for a declared SPA route — every
/// other miss is an honest `404 no-store` (the body stays the app shell so the
/// client can render its own not-found view). The fallback service marks its
/// own replies, so this layer never has to guess what the static layer served.
///
/// `entry_file` re-serves the shell when a conditional miss answered 304/206
/// without a body.
#[derive(Clone)]
struct StaticFallback {
    routes: Arc<spa::SpaRoutes>,
    entry_file: ServeFile,
}

async fn static_response_policy(
    State(fallback): State<StaticFallback>,
    request: Request<Body>,
    next: Next,
) -> Response {
    let path = request.uri().path().to_owned();
    let wants_body = request.method() != Method::HEAD;
    let mut response = next.run(request).await;

    if response.headers_mut().remove(SPA_FALLBACK_HEADER).is_some() {
        let status = response.status();
        if !fallback.routes.is_app_route(&path)
            && (status.is_success() || status == StatusCode::NOT_MODIFIED)
        {
            if wants_body && status != StatusCode::OK {
                // A conditional miss answered 304/206 without the shell body;
                // re-serve the entry file so the 404 renders the app.
                response = fallback
                    .entry_file
                    .clone()
                    .oneshot(
                        Request::get("/")
                            .body(Body::empty())
                            .expect("valid request"),
                    )
                    .await
                    .expect("serving the SPA entry file cannot fail")
                    .map(Body::new);
            }
            *response.status_mut() = StatusCode::NOT_FOUND;
            response
                .headers_mut()
                .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
        }
        return response;
    }

    if response.status().is_success() || response.status() == StatusCode::NOT_MODIFIED {
        // `ServeDir` percent-decodes before the disk lookup, so the policy
        // branches are keyed by the decoded path it actually served.
        let decoded_path = percent_encoding::percent_decode_str(&path)
            .decode_utf8()
            .map_or_else(|_| path.clone(), std::borrow::Cow::into_owned);
        // sitemaps.org registers the sitemap type as application/xml; mime_guess
        // answers text/xml for every .xml, so pin the served type here.
        if decoded_path == "/sitemap.xml" {
            response
                .headers_mut()
                .insert(header::CONTENT_TYPE, HeaderValue::from_static("application/xml"));
        }
        let policy = if decoded_path.starts_with("/_app/immutable/") {
            Some("public, max-age=31536000, immutable")
        } else if decoded_path.starts_with("/art/")
            || decoded_path.starts_with("/fonts/")
            || decoded_path.starts_with("/landing/")
        {
            Some("public, max-age=604800")
        } else if decoded_path == "/robots.txt" {
            Some("public, max-age=3600")
        } else {
            None
        };
        if let Some(policy) = policy {
            response
                .headers_mut()
                .insert(header::CACHE_CONTROL, HeaderValue::from_static(policy));
        }
    }

    response
}

async fn health() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}

async fn api_not_found() -> ApiError {
    ApiError::not_found("API route not found")
}

/// Checks that the static directory, its SPA fallback file (`200.html`) and its
/// route manifest (`routes.json`) exist, so a bad deploy fails at start-up.
///
/// # Errors
///
/// Returns an error naming the missing path when the directory, the entry file
/// or the manifest is absent or malformed.
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

    let index_file = static_dir.join(SPA_FALLBACK_FILE);
    let index_metadata = std::fs::metadata(&index_file).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "SPA fallback {} is unavailable: {error}",
                index_file.display()
            ),
        )
    })?;
    if !index_metadata.is_file() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("SPA fallback {} is not a file", index_file.display()),
        ));
    }

    let manifest_file = static_dir.join(spa::ROUTES_MANIFEST);
    let raw_manifest = std::fs::read_to_string(&manifest_file).map_err(|error| {
        io::Error::new(
            error.kind(),
            format!(
                "SPA route manifest {} is unavailable: {error}",
                manifest_file.display()
            ),
        )
    })?;
    spa::SpaRoutes::parse(&raw_manifest).map_err(|message| {
        io::Error::new(
            io::ErrorKind::InvalidData,
            format!("{}: {message}", manifest_file.display()),
        )
    })?;

    Ok(())
}
