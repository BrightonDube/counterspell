//! Server settings, read from environment variables.
//!
//! Secrets (the database and Valkey URLs) come only from the environment. They are
//! never printed: `Debug` on [`ServerConfig`] hides them, and [`ConfigError`] names a
//! variable without echoing its value.

use std::{fmt, net::SocketAddr, path::PathBuf};

use thiserror::Error;

use crate::client_ip::TrustedProxies;

/// The window, in seconds, that per-user and per-IP run limits are counted over.
pub const RUN_WINDOW_SECS: u64 = 600;

/// The default trusted proxies: loopback and the private ranges Compose and Caddy use.
const DEFAULT_TRUSTED_PROXIES: &str = "127.0.0.1/32,10.0.0.0/8,172.16.0.0/12";

/// A setting is missing or cannot be used. Messages name the variable and never echo its value.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConfigError {
    /// A required variable is not set.
    #[error("required environment variable {0} is not set")]
    Missing(&'static str),
    /// A variable is set but is not a usable value.
    #[error("environment variable {name} is invalid: {reason}")]
    Invalid {
        /// The variable name.
        name: &'static str,
        /// What is wrong with it, in plain words.
        reason: String,
    },
}

/// Limits on run submissions, counted per user and per IP.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RunLimits {
    /// Runs one user may start per [`RUN_WINDOW_SECS`].
    pub run_rate_user: u64,
    /// Runs one IP address may start per [`RUN_WINDOW_SECS`].
    pub run_rate_ip: u64,
    /// Runs one user may have queued or active at once.
    pub run_active_cap: u64,
    /// Queued runs allowed across all users before new submissions get `queue_full`.
    pub queue_depth_max: u64,
    /// Largest accepted source size, in bytes.
    pub max_source_bytes: u64,
    /// Largest accepted HTTP request body, in bytes.
    pub request_body_limit_bytes: u64,
}

/// Everything the server needs at start-up.
#[derive(Clone, PartialEq, Eq)]
pub struct ServerConfig {
    /// Interface to bind, for example `0.0.0.0`.
    pub host: String,
    /// Port to bind.
    pub port: u16,
    /// Directory with the built static web app.
    pub static_dir: PathBuf,
    /// Postgres connection URL. Secret: never printed.
    pub database_url: String,
    /// Valkey connection URL. Secret: never printed.
    pub valkey_url: String,
    /// Submission limits.
    pub limits: RunLimits,
    /// Proxies whose `X-Forwarded-For` header is believed for the per-IP limit.
    pub trusted_proxies: TrustedProxies,
}

impl fmt::Debug for ServerConfig {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ServerConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("static_dir", &self.static_dir)
            .field("database_url", &"<redacted>")
            .field("valkey_url", &"<redacted>")
            .field("limits", &self.limits)
            .field("trusted_proxies", &self.trusted_proxies)
            .finish()
    }
}

impl ServerConfig {
    /// Reads the configuration from the process environment.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] when a required variable is missing or a value is invalid.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    /// Builds the configuration from any lookup function. Tests use this to avoid the real environment.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError`] when a required variable is missing or a value is invalid.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let database_url = required(&lookup, "DATABASE_URL")?;
        let valkey_url = required(&lookup, "VALKEY_URL")?;
        let host = lookup("HOST").unwrap_or_else(|| "0.0.0.0".to_owned());
        let port = ranged(&lookup, "PORT", 8080, 1, 65_535)?;
        let port = u16::try_from(port).map_err(|_| invalid("PORT", "must be 65535 or lower"))?;
        let static_dir =
            PathBuf::from(lookup("STATIC_DIR").unwrap_or_else(|| "web/build".to_owned()));

        let limits = RunLimits {
            run_rate_user: ranged(&lookup, "RUN_RATE_USER", 30, 1, 10_000)?,
            run_rate_ip: ranged(&lookup, "RUN_RATE_IP", 100, 1, 100_000)?,
            run_active_cap: ranged(&lookup, "RUN_ACTIVE_CAP_PER_USER", 3, 1, 100)?,
            queue_depth_max: ranged(&lookup, "RUN_QUEUE_DEPTH_MAX", 200, 1, 100_000)?,
            max_source_bytes: ranged(&lookup, "MAX_SOURCE_BYTES", 65_536, 1, 1_048_576)?,
            request_body_limit_bytes: ranged(
                &lookup,
                "REQUEST_BODY_LIMIT_BYTES",
                131_072,
                1024,
                4_194_304,
            )?,
        };

        let proxies_text = lookup("TRUSTED_PROXIES");
        let trusted_proxies =
            TrustedProxies::parse(proxies_text.as_deref().unwrap_or(DEFAULT_TRUSTED_PROXIES))
                .map_err(|reason| invalid("TRUSTED_PROXIES", reason))?;

        Ok(Self {
            host,
            port,
            static_dir,
            database_url,
            valkey_url,
            limits,
            trusted_proxies,
        })
    }

    /// The address to bind, built from `host` and `port`.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Invalid`] when `host` is not an IP address.
    pub fn address(&self) -> Result<SocketAddr, ConfigError> {
        let text = format!("{}:{}", self.host, self.port);
        text.parse()
            .map_err(|_| invalid("HOST", "must be an IP address such as 0.0.0.0 or 127.0.0.1"))
    }
}

fn required(
    lookup: &impl Fn(&str) -> Option<String>,
    name: &'static str,
) -> Result<String, ConfigError> {
    lookup(name)
        .filter(|value| !value.trim().is_empty())
        .ok_or(ConfigError::Missing(name))
}

fn ranged(
    lookup: &impl Fn(&str) -> Option<String>,
    name: &'static str,
    default: u64,
    min: u64,
    max: u64,
) -> Result<u64, ConfigError> {
    let Some(text) = lookup(name) else {
        return Ok(default);
    };
    let value: u64 = text
        .trim()
        .parse()
        .map_err(|_| invalid(name, "must be a whole number"))?;
    if value < min || value > max {
        return Err(invalid(name, &format!("must be from {min} to {max}")));
    }
    Ok(value)
}

fn invalid(name: &'static str, reason: &str) -> ConfigError {
    ConfigError::Invalid {
        name,
        reason: reason.to_owned(),
    }
}
