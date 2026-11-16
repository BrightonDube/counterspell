//! Readiness: checks that Postgres and Valkey answer before the deploy script sends traffic.
//!
//! `GET /api/ready` is internal only. Caddy answers it with 404 (deploy-ops R6.6), and the deploy
//! script calls it from inside the edge network. Liveness stays on `GET /api/health`.

use std::{sync::Arc, time::Duration};

use async_trait::async_trait;
use axum::{
    extract::State,
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use sqlx::PgPool;

use crate::{ratelimit::Limiter, AppState};

/// How long one dependency may take to answer before readiness counts it as down.
pub const PROBE_TIMEOUT: Duration = Duration::from_secs(2);

/// Says that a dependency did not answer. It carries no detail, so no error text, host or
/// connection string can reach a client or a log line.
#[derive(Debug)]
pub struct ProbeFailure;

/// One dependency that readiness pings.
#[async_trait]
pub trait Probe: Send + Sync {
    /// Short name for logs, for example `postgres`. Never a URL.
    fn name(&self) -> &'static str;

    /// Pings the dependency once. The caller applies the timeout.
    async fn ping(&self) -> Result<(), ProbeFailure>;
}

/// Pings Postgres with `SELECT 1` through the server's pool.
pub struct PostgresProbe {
    pool: PgPool,
}

impl PostgresProbe {
    /// Wraps the server's pool.
    #[must_use]
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Probe for PostgresProbe {
    fn name(&self) -> &'static str {
        "postgres"
    }

    async fn ping(&self) -> Result<(), ProbeFailure> {
        sqlx::query("SELECT 1")
            .execute(&self.pool)
            .await
            .map(|_| ())
            .map_err(|_| ProbeFailure)
    }
}

/// Pings the rate-limit store. In production that is Valkey.
pub struct LimiterProbe {
    limiter: Limiter,
}

impl LimiterProbe {
    /// Wraps the limiter the server already uses.
    #[must_use]
    pub fn new(limiter: Limiter) -> Self {
        Self { limiter }
    }
}

#[async_trait]
impl Probe for LimiterProbe {
    fn name(&self) -> &'static str {
        "valkey"
    }

    async fn ping(&self) -> Result<(), ProbeFailure> {
        if self.limiter.is_reachable().await {
            Ok(())
        } else {
            Err(ProbeFailure)
        }
    }
}

/// The set of dependencies that readiness checks.
pub struct Readiness {
    probes: Vec<Arc<dyn Probe>>,
}

impl Readiness {
    /// Builds the check over the given dependencies.
    #[must_use]
    pub fn new(probes: Vec<Arc<dyn Probe>>) -> Self {
        Self { probes }
    }

    /// Pings every dependency at the same time, each within [`PROBE_TIMEOUT`], and returns the names
    /// of those that did not answer in time. An empty result means the server is ready.
    pub async fn failing(&self) -> Vec<&'static str> {
        let checks = self.probes.iter().map(|probe| async move {
            let answered = matches!(
                tokio::time::timeout(PROBE_TIMEOUT, probe.ping()).await,
                Ok(Ok(()))
            );
            (probe.name(), answered)
        });
        futures_util::future::join_all(checks)
            .await
            .into_iter()
            .filter_map(|(name, answered)| (!answered).then_some(name))
            .collect()
    }
}

/// `GET /api/ready`: `200 {"status":"ok"}` when every dependency answers, otherwise
/// `503 {"status":"unavailable"}`. The failing names go to the log at `warn`, never to the client.
pub async fn ready(State(state): State<AppState>) -> Response {
    let failing = state.readiness.failing().await;
    if failing.is_empty() {
        return (StatusCode::OK, Json(json!({ "status": "ok" }))).into_response();
    }
    for name in &failing {
        tracing::warn!(dependency = name, "readiness check failed");
    }
    (
        StatusCode::SERVICE_UNAVAILABLE,
        Json(json!({ "status": "unavailable" })),
    )
        .into_response()
}
