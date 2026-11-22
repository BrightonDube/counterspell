//! Authentication settings, read from environment variables once at start-up.
//!
//! All parsing and every rule lives in [`AuthConfig::from_map`], which takes a plain map, so
//! tests never touch the process environment. [`AuthConfig::from_env`] only collects the
//! variables it knows about. Secrets are wrapped in [`Redacted`], so `Debug` and `Display`
//! never show them, and [`ConfigError`] names a variable without echoing its value.

use std::{collections::HashMap, fmt};

use thiserror::Error;
use url::Url;

/// The fewest bytes a session secret may have.
pub const MIN_SESSION_SECRET_BYTES: usize = 32;
/// The fewest characters the demo account password may have.
pub const MIN_DEMO_PASSWORD_CHARS: usize = 16;
/// Every variable this module reads. [`AuthConfig::from_env`] collects only these names.
pub const ENV_NAMES: [&str; 12] = [
    "PUBLIC_BASE_URL",
    "DATABASE_URL",
    "VALKEY_URL",
    "SESSION_SECRET",
    "APP_ENV",
    "GITHUB_OAUTH_CLIENT_ID",
    "GITHUB_OAUTH_CLIENT_SECRET",
    "GOOGLE_OAUTH_CLIENT_ID",
    "GOOGLE_OAUTH_CLIENT_SECRET",
    "DEMO_ACCOUNT_EMAIL",
    "DEMO_ACCOUNT_PASSWORD",
    "DEMO_RESET_HOUR_UTC",
];

/// A value that must not appear in logs or error text. `Debug` and `Display` print `[redacted]`.
#[derive(Clone, PartialEq, Eq)]
pub struct Redacted<T>(T);

impl<T> Redacted<T> {
    /// Wraps a secret value.
    #[must_use]
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// Returns the secret. Call this only where the value is used, never to print it.
    #[must_use]
    pub fn expose(&self) -> &T {
        &self.0
    }
}

impl<T> fmt::Debug for Redacted<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[redacted]")
    }
}

impl<T> fmt::Display for Redacted<T> {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("[redacted]")
    }
}

/// A setting is missing or unusable. The message names the variable and never its value.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ConfigError {
    /// A required variable is not set.
    #[error("required environment variable {0} is not set")]
    Missing(&'static str),
    /// A variable is set but cannot be used.
    #[error("environment variable {name} is invalid: {reason}")]
    Invalid {
        /// The variable name.
        name: &'static str,
        /// What is wrong, in plain words.
        reason: String,
    },
}

/// Whether the server runs in production or on a developer machine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppEnv {
    /// The deployed server. Strict rules apply.
    Production,
    /// A developer machine. Local plain-HTTP is allowed.
    Development,
}

/// Client id and secret for one OAuth provider. The provider is enabled only when both exist.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OAuthClient {
    /// Public client id from the provider's console.
    pub client_id: String,
    /// Client secret. Never printed.
    pub client_secret: Redacted<String>,
}

/// The demo account's sign-in details.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DemoConfig {
    /// The demo address, lower-cased and trimmed.
    pub email: String,
    /// The demo password. Never printed.
    pub password: Redacted<String>,
}

/// Every setting the auth module needs. Built once at start-up.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AuthConfig {
    /// Production or development.
    pub app_env: AppEnv,
    /// The origin every emitted URL uses, for example `https://counterspell.example`.
    pub public_base_url: String,
    /// Postgres URL for the server role. Never printed.
    pub database_url: Redacted<String>,
    /// Valkey URL for counters. Never printed.
    pub valkey_url: Redacted<String>,
    /// Key material for hashing rate-limit and log identifiers. Never printed.
    pub session_secret: Redacted<Vec<u8>>,
    /// GitHub sign-in, when both of its variables are set.
    pub github: Option<OAuthClient>,
    /// Google sign-in, when both of its variables are set.
    pub google: Option<OAuthClient>,
    /// The demo account, when its variables are set.
    pub demo: Option<DemoConfig>,
    /// The UTC hour (0 to 23) at which the demo account resets.
    pub demo_reset_hour_utc: u8,
}

impl AuthConfig {
    /// Collects the variables this module reads from the process environment.
    ///
    /// Only the map is built here. Every rule is applied by [`AuthConfig::from_map`].
    ///
    /// # Errors
    ///
    /// Returns the same errors as [`AuthConfig::from_map`].
    pub fn from_env() -> Result<Self, ConfigError> {
        let vars: HashMap<String, String> = ENV_NAMES
            .iter()
            .filter_map(|name| {
                std::env::var(name)
                    .ok()
                    .map(|value| ((*name).to_owned(), value))
            })
            .collect();
        Self::from_map(&vars)
    }

    /// Builds the settings from a map of variable names to values.
    ///
    /// An empty value counts as not set.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::Missing`] for a required variable that is absent, and
    /// [`ConfigError::Invalid`] for a value that breaks a rule. Neither error echoes a value.
    pub fn from_map(vars: &HashMap<String, String>) -> Result<Self, ConfigError> {
        let app_env = parse_app_env(optional(vars, "APP_ENV"))?;
        let public_base_url = parse_base_url(required(vars, "PUBLIC_BASE_URL")?, app_env)?;
        let database_url = Redacted::new(required(vars, "DATABASE_URL")?.to_owned());
        let valkey_url = Redacted::new(required(vars, "VALKEY_URL")?.to_owned());
        let session_secret = parse_session_secret(required(vars, "SESSION_SECRET")?)?;
        Ok(Self {
            app_env,
            public_base_url,
            database_url,
            valkey_url,
            session_secret,
            github: oauth_client(vars, "GITHUB_OAUTH_CLIENT_ID", "GITHUB_OAUTH_CLIENT_SECRET"),
            google: oauth_client(vars, "GOOGLE_OAUTH_CLIENT_ID", "GOOGLE_OAUTH_CLIENT_SECRET"),
            demo: parse_demo(vars)?,
            demo_reset_hour_utc: parse_reset_hour(optional(vars, "DEMO_RESET_HOUR_UTC"))?,
        })
    }
}

/// Reads a variable, treating an empty value as not set.
fn optional<'a>(vars: &'a HashMap<String, String>, name: &str) -> Option<&'a str> {
    vars.get(name)
        .map(String::as_str)
        .filter(|value| !value.is_empty())
}

/// Reads a variable that must be set.
fn required<'a>(
    vars: &'a HashMap<String, String>,
    name: &'static str,
) -> Result<&'a str, ConfigError> {
    optional(vars, name).ok_or(ConfigError::Missing(name))
}

/// Builds an [`ConfigError::Invalid`] with a plain reason that never contains the value.
fn invalid(name: &'static str, reason: &str) -> ConfigError {
    ConfigError::Invalid {
        name,
        reason: reason.to_owned(),
    }
}

fn parse_app_env(raw: Option<&str>) -> Result<AppEnv, ConfigError> {
    match raw {
        None | Some("production") => Ok(AppEnv::Production),
        Some("development") => Ok(AppEnv::Development),
        Some(_) => Err(invalid("APP_ENV", "must be production or development")),
    }
}

/// Checks the public base URL: a bare origin, https in production, and plain http only on
/// localhost in development. Returns the origin without a trailing slash.
fn parse_base_url(raw: &str, app_env: AppEnv) -> Result<String, ConfigError> {
    let url = Url::parse(raw).map_err(|_| invalid("PUBLIC_BASE_URL", "must be an absolute URL"))?;
    let bare_origin = matches!(url.path(), "" | "/")
        && url.query().is_none()
        && url.fragment().is_none()
        && url.username().is_empty()
        && url.password().is_none();
    if !bare_origin {
        return Err(invalid(
            "PUBLIC_BASE_URL",
            "must be an origin with no path, query or fragment",
        ));
    }
    let local = matches!(url.host_str(), Some("localhost" | "127.0.0.1"));
    match (url.scheme(), app_env) {
        ("https", _) => {}
        ("http", AppEnv::Development) if local => {}
        ("http", _) => {
            return Err(invalid(
                "PUBLIC_BASE_URL",
                "must use https outside local development",
            ))
        }
        _ => return Err(invalid("PUBLIC_BASE_URL", "must use https")),
    }
    Ok(url.origin().ascii_serialization())
}

/// The session secret is key material. It must be at least [`MIN_SESSION_SECRET_BYTES`] bytes.
fn parse_session_secret(raw: &str) -> Result<Redacted<Vec<u8>>, ConfigError> {
    if raw.len() < MIN_SESSION_SECRET_BYTES {
        return Err(invalid("SESSION_SECRET", "must be at least 32 bytes"));
    }
    Ok(Redacted::new(raw.as_bytes().to_vec()))
}

/// A provider is enabled only when both its client id and its secret are set.
fn oauth_client(
    vars: &HashMap<String, String>,
    id_name: &str,
    secret_name: &str,
) -> Option<OAuthClient> {
    match (optional(vars, id_name), optional(vars, secret_name)) {
        (Some(id), Some(secret)) => Some(OAuthClient {
            client_id: id.to_owned(),
            client_secret: Redacted::new(secret.to_owned()),
        }),
        _ => None,
    }
}

fn parse_reset_hour(raw: Option<&str>) -> Result<u8, ConfigError> {
    let Some(raw) = raw else {
        return Ok(0);
    };
    let hour: u8 = raw
        .parse()
        .map_err(|_| invalid("DEMO_RESET_HOUR_UTC", "must be a whole number from 0 to 23"))?;
    if hour > 23 {
        return Err(invalid("DEMO_RESET_HOUR_UTC", "must be from 0 to 23"));
    }
    Ok(hour)
}

/// The demo account is configured only when both of its variables are set.
fn parse_demo(vars: &HashMap<String, String>) -> Result<Option<DemoConfig>, ConfigError> {
    match (
        optional(vars, "DEMO_ACCOUNT_EMAIL"),
        optional(vars, "DEMO_ACCOUNT_PASSWORD"),
    ) {
        (None, None) => Ok(None),
        (None, Some(_)) => Err(ConfigError::Missing("DEMO_ACCOUNT_EMAIL")),
        (Some(_), None) => Err(ConfigError::Missing("DEMO_ACCOUNT_PASSWORD")),
        (Some(email), Some(password)) => {
            let email = normalise_demo_email(email)?;
            if password.chars().count() < MIN_DEMO_PASSWORD_CHARS {
                return Err(invalid(
                    "DEMO_ACCOUNT_PASSWORD",
                    "must be at least 16 characters",
                ));
            }
            Ok(Some(DemoConfig {
                email,
                password: Redacted::new(password.to_owned()),
            }))
        }
    }
}

/// Trims and lower-cases the demo address and parses it the same way the login route does, so a
/// shape the route refuses fails here at startup instead of advertising a dead account.
fn normalise_demo_email(raw: &str) -> Result<String, ConfigError> {
    let email = raw.trim().to_lowercase();
    let plausible = super::email::EmailAddress::parse(&email).is_ok();
    if plausible {
        Ok(email)
    } else {
        Err(invalid("DEMO_ACCOUNT_EMAIL", "must be an email address"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid() -> HashMap<String, String> {
        let pairs = [
            ("PUBLIC_BASE_URL", "https://counterspell.example"),
            ("DATABASE_URL", "postgres://server:fake-pw@db.internal/cs"),
            ("VALKEY_URL", "redis://:fake-valkey@cache.internal:6379"),
            ("SESSION_SECRET", "fake-session-secret-value-0123456789"),
        ];
        pairs
            .into_iter()
            .map(|(name, value)| (name.to_owned(), value.to_owned()))
            .collect()
    }

    fn with(overrides: &[(&str, &str)]) -> HashMap<String, String> {
        let mut vars = valid();
        for (name, value) in overrides {
            vars.insert((*name).to_owned(), (*value).to_owned());
        }
        vars
    }

    fn without(name: &str) -> HashMap<String, String> {
        let mut vars = valid();
        vars.remove(name);
        vars
    }

    fn invalid_name(result: Result<AuthConfig, ConfigError>) -> &'static str {
        match result {
            Err(ConfigError::Invalid { name, .. }) => name,
            other => panic!("expected an invalid setting, got {other:?}"),
        }
    }

    #[test]
    fn a_complete_production_map_is_accepted() {
        let config = AuthConfig::from_map(&valid()).expect("valid production map");
        assert_eq!(config.app_env, AppEnv::Production);
        // test-change: ACC-1
        assert_eq!(config.public_base_url, "https://counterspell.example");
        assert_eq!(config.demo_reset_hour_utc, 0);
        assert!(config.github.is_none() && config.google.is_none() && config.demo.is_none());
    }

    #[test]
    fn each_required_variable_is_named_when_missing() {
        for name in [
            "PUBLIC_BASE_URL",
            "DATABASE_URL",
            "SESSION_SECRET",
            "VALKEY_URL",
        ] {
            assert_eq!(
                AuthConfig::from_map(&without(name)),
                Err(ConfigError::Missing(
                    ENV_NAMES
                        .iter()
                        .find(|candidate| **candidate == name)
                        .copied()
                        .unwrap()
                )),
                "missing {name}"
            );
        }
    }

    #[test]
    fn an_empty_value_counts_as_missing() {
        let vars = with(&[("SESSION_SECRET", "")]);
        assert_eq!(
            AuthConfig::from_map(&vars),
            Err(ConfigError::Missing("SESSION_SECRET"))
        );
    }

    #[test]
    fn a_short_session_secret_is_refused_without_echoing_it() {
        let vars = with(&[("SESSION_SECRET", "short-secret-value")]);
        let error = AuthConfig::from_map(&vars).expect_err("too short");
        assert_eq!(invalid_name(Err(error.clone())), "SESSION_SECRET");
        assert!(!error.to_string().contains("short-secret-value"));
    }

    #[test]
    fn production_refuses_plain_http_base_url() {
        let vars = with(&[("PUBLIC_BASE_URL", "http://counterspell.example")]);
        assert_eq!(invalid_name(AuthConfig::from_map(&vars)), "PUBLIC_BASE_URL");
    }

    #[test]
    // test-change: ACC-1
    fn development_allows_http_on_localhost_only() {
        let local = with(&[
            ("APP_ENV", "development"),
            ("PUBLIC_BASE_URL", "http://localhost:8080/"),
        ]);
        let config = AuthConfig::from_map(&local).expect("localhost in development");
        assert_eq!(config.public_base_url, "http://localhost:8080");

        let remote = with(&[
            ("APP_ENV", "development"),
            ("PUBLIC_BASE_URL", "http://counterspell.example"),
        ]);
        assert_eq!(
            invalid_name(AuthConfig::from_map(&remote)),
            "PUBLIC_BASE_URL"
        );
    }

    #[test]
    fn base_url_must_be_a_bare_origin() {
        for raw in [
            "https://counterspell.example/app",
            "https://counterspell.example/?x=1",
            "https://counterspell.example/#frag",
            "not a url",
        ] {
            let vars = with(&[("PUBLIC_BASE_URL", raw)]);
            assert_eq!(
                invalid_name(AuthConfig::from_map(&vars)),
                "PUBLIC_BASE_URL",
                "{raw}"
            );
        }
    }

    #[test]
    fn an_unknown_app_env_is_refused() {
        assert_eq!(
            invalid_name(AuthConfig::from_map(&with(&[("APP_ENV", "staging")]))),
            "APP_ENV"
        );
    }

    #[test]
    fn a_provider_is_enabled_only_with_both_of_its_values() {
        let both = with(&[
            ("GITHUB_OAUTH_CLIENT_ID", "gh-client"),
            ("GITHUB_OAUTH_CLIENT_SECRET", "gh-secret-value"),
        ]);
        let config = AuthConfig::from_map(&both).expect("github enabled");
        let github = config.github.expect("github client");
        assert_eq!(github.client_id, "gh-client");

        let only_id = with(&[("GOOGLE_OAUTH_CLIENT_ID", "google-client")]);
        let config = AuthConfig::from_map(&only_id).expect("google disabled");
        assert!(config.google.is_none());
    }

    #[test]
    fn the_demo_account_needs_both_variables_and_a_long_password() {
        let only_email = with(&[("DEMO_ACCOUNT_EMAIL", "judge@counterspell.example")]);
        assert_eq!(
            AuthConfig::from_map(&only_email),
            Err(ConfigError::Missing("DEMO_ACCOUNT_PASSWORD"))
        );

        let short = with(&[
            ("DEMO_ACCOUNT_EMAIL", "judge@counterspell.example"),
            ("DEMO_ACCOUNT_PASSWORD", "fifteen-chars!!"),
        ]);
        assert_eq!(
            invalid_name(AuthConfig::from_map(&short)),
            "DEMO_ACCOUNT_PASSWORD"
        );

        let ok = with(&[
            ("DEMO_ACCOUNT_EMAIL", "  Judge@Counterspell.example "),
            ("DEMO_ACCOUNT_PASSWORD", "sixteen-chars-ok"),
        ]);
        let demo = AuthConfig::from_map(&ok)
            .expect("demo")
            .demo
            .expect("demo config");
        assert_eq!(demo.email, "judge@counterspell.example");
    }

    #[test]
    // test-change: ACC-1
    fn reset_hour_has_a_safe_range() {
        assert_eq!(
            invalid_name(AuthConfig::from_map(&with(&[(
                "DEMO_RESET_HOUR_UTC",
                "24"
            )]))),
            "DEMO_RESET_HOUR_UTC"
        );
        let config =
            AuthConfig::from_map(&with(&[("DEMO_RESET_HOUR_UTC", "23")])).expect("in range");
        assert_eq!(config.demo_reset_hour_utc, 23);
    }

    #[test]
    fn debug_output_never_shows_a_secret() {
        let vars = with(&[
            ("DATABASE_URL", "postgres://server:db-password-xyz@db/cs"),
            ("VALKEY_URL", "redis://:valkey-password-xyz@cache:6379"),
            ("GITHUB_OAUTH_CLIENT_ID", "gh-client"),
            ("GITHUB_OAUTH_CLIENT_SECRET", "gh-secret-xyz"),
            ("DEMO_ACCOUNT_EMAIL", "judge@counterspell.example"),
            ("DEMO_ACCOUNT_PASSWORD", "demo-password-xyz-long"),
        ]);
        let config = AuthConfig::from_map(&vars).expect("valid");
        let text = format!("{config:?}");
        for secret in [
            "db-password-xyz",
            "valkey-password-xyz",
            "gh-secret-xyz",
            "demo-password-xyz-long",
            "re_fake_key_value",
            "fake-session-secret-value-0123456789",
        ] {
            assert!(!text.contains(secret), "Debug leaked {secret}");
        }
        assert!(text.contains("[redacted]"));
        assert_eq!(format!("{}", Redacted::new("x")), "[redacted]");
    }
}
