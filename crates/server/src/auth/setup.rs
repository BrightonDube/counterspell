//! Builds the auth state from validated configuration and starts its background tasks.
//!
//! This is the only place that chooses concrete adapters (OAuth providers and the
//! clock). The rest of the auth code receives them through [`AuthState`]. Nothing here touches
//! the database or the network, so the state can be built in tests.

use std::{sync::Arc, time::Duration as StdDuration};

use sqlx::PgPool;
use thiserror::Error;
use tokio::task::JoinHandle;

use crate::{
    auth::{
        cleanup::spawn_cleanup,
        clock::{Clock, SystemClock},
        config::AuthConfig,
        cookies::CookieSettings,
        demo::{seed_demo, spawn_daily_reset},
        handlers::config::PublicSettings,
        limits::AuthLimiter,
        oauth::{github::GithubProvider, google::GoogleProvider, OAuthProvider},
        password::HashGate,
        security::SecuritySettings,
        AuthState,
    },
    client_ip::TrustedProxies,
    ratelimit::Limiter,
};

/// Why the auth state could not be built. Each message names the problem and never a secret.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SetupError {
    /// An OAuth provider adapter could not be built.
    #[error("an OAuth provider could not be built")]
    OAuth,
}

/// Builds the auth state for the server. Only the providers with client settings are enabled.
///
/// # Errors
///
/// Returns a [`SetupError`] naming the setting that could not be used.
pub fn build_state(
    pool: PgPool,
    config: &AuthConfig,
    limiter: Limiter,
    trusted: TrustedProxies,
) -> Result<AuthState, SetupError> {
    let clock: Arc<dyn Clock> = Arc::new(SystemClock);
    let limits = AuthLimiter::new(limiter, &config.session_secret);
    let mut state = AuthState::new(
        pool,
        clock,
        CookieSettings::for_base_url(&config.public_base_url),
        limits,
        HashGate::new(4, StdDuration::from_secs(2)),
        trusted,
    )
    .with_oauth(config.public_base_url.clone(), oauth_providers(config)?)
    .with_settings(PublicSettings::from_config(config));
    state.security = Some(SecuritySettings::new(
        &config.public_base_url,
        config.session_secret.expose(),
    ));
    Ok(state)
}

/// One adapter for each provider that has client settings.
fn oauth_providers(config: &AuthConfig) -> Result<Vec<Arc<dyn OAuthProvider>>, SetupError> {
    let mut providers: Vec<Arc<dyn OAuthProvider>> = Vec::new();
    if let Some(github) = &config.github {
        let adapter =
            GithubProvider::production(github.client_id.clone(), github.client_secret.clone())
                .map_err(|_| SetupError::OAuth)?;
        providers.push(Arc::new(adapter));
    }
    if let Some(google) = &config.google {
        let adapter =
            GoogleProvider::production(google.client_id.clone(), google.client_secret.clone())
                .map_err(|_| SetupError::OAuth)?;
        providers.push(Arc::new(adapter));
    }
    Ok(providers)
}

/// Seeds the demo account and starts the background tasks: the hourly cleanup, and the daily demo
/// reset when the demo account is configured. A seed failure is logged, and the reset task retries
/// on its own schedule.
pub async fn start_background(state: &AuthState, config: &AuthConfig) -> Vec<JoinHandle<()>> {
    let mut tasks = vec![spawn_cleanup(state.pool.clone(), state.clock.clone())];
    if let Some(demo) = &config.demo {
        if seed_demo(&state.pool, demo, &state.hashes).await.is_err() {
            tracing::error!("the demo account could not be seeded; the daily reset will retry");
        }
        tasks.push(spawn_daily_reset(
            state.pool.clone(),
            state.clock.clone(),
            config.demo_reset_hour_utc,
        ));
    }
    tasks
}
