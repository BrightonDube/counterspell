//! Builds the auth state from validated configuration and starts its background tasks.
//!
//! This is the only place that chooses concrete adapters (mail transport, OAuth providers and the
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
        config::{AuthConfig, MailTransport},
        cookies::CookieSettings,
        demo::{seed_demo, spawn_daily_reset},
        email::{EmailAddress, EmailSender, GuardedSender, LogSender, ResendSender},
        handlers::config::PublicSettings,
        limits::AuthLimiter,
        oauth::{github::GithubProvider, google::GoogleProvider, OAuthProvider},
        password::HashGate,
        AuthState, MailPort,
    },
    client_ip::TrustedProxies,
    ratelimit::Limiter,
};

/// Why the auth state could not be built. Each message names the problem and never a secret.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SetupError {
    /// Mail is set to Resend, but its key or sender address is missing.
    #[error("MAIL_TRANSPORT is resend but RESEND_API_KEY or MAIL_FROM is missing")]
    MissingMailSettings,
    /// The mail sender or its HTTP client could not be built.
    #[error("the mail sender could not be built")]
    Mail,
    /// An OAuth provider adapter could not be built.
    #[error("an OAuth provider could not be built")]
    OAuth,
    /// The demo account address is not a valid email address.
    #[error("DEMO_ACCOUNT_EMAIL is not a valid email address")]
    DemoEmail,
}

/// Builds the auth state for the server. Mail is guarded by the daily cap and the demo address,
/// and only the providers with client settings are enabled.
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
    let demo_email = config
        .demo
        .as_ref()
        .map(|demo| EmailAddress::parse(&demo.email))
        .transpose()
        .map_err(|_| SetupError::DemoEmail)?;
    let transport = mail_transport(config)?;
    let sender = Arc::new(GuardedSender::new(
        transport,
        limits.clone(),
        config.email_daily_cap,
        demo_email.clone(),
        clock.clone(),
    ));
    let state = AuthState::new(
        pool,
        clock,
        CookieSettings::for_base_url(&config.public_base_url),
        limits,
        HashGate::new(4, StdDuration::from_secs(2)),
        trusted,
    )
    .with_mail(MailPort {
        sender,
        public_base_url: config.public_base_url.clone(),
        demo_email,
    })
    .with_oauth(config.public_base_url.clone(), oauth_providers(config)?)
    .with_settings(PublicSettings::from_config(config));
    Ok(state)
}

/// The mail sender for the configured transport. The log transport is refused in production by
/// [`LogSender::new`].
fn mail_transport(config: &AuthConfig) -> Result<Arc<dyn EmailSender>, SetupError> {
    match config.mail.transport {
        MailTransport::Resend => {
            let key = config
                .mail
                .resend_api_key
                .clone()
                .ok_or(SetupError::MissingMailSettings)?;
            let from = config
                .mail
                .mail_from
                .clone()
                .ok_or(SetupError::MissingMailSettings)?;
            let sender = ResendSender::production(key, from).map_err(|_| SetupError::Mail)?;
            Ok(Arc::new(sender))
        }
        MailTransport::Log => {
            let sender = LogSender::new(config.app_env).map_err(|_| SetupError::Mail)?;
            Ok(Arc::new(sender))
        }
    }
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
