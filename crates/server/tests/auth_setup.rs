//! Building the auth state from configuration (auth T33). No database or network is used: the pool
//! is created lazily and never connects.
//!
//! Run with `cargo test -p counterspell-server auth_setup`.

use std::collections::HashMap;

use counterspell_server::{
    auth::{config::AuthConfig, email::EmailAddress, oauth::ProviderId, setup::build_state},
    client_ip::TrustedProxies,
    ratelimit::{Limiter, MemoryLimiter},
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

/// A development configuration with the log transport, which needs no mail key.
fn development(extra: &[(&str, &str)]) -> HashMap<String, String> {
    let mut vars: HashMap<String, String> = [
        ("APP_ENV", "development"),
        ("PUBLIC_BASE_URL", "http://localhost:8080"),
        ("DATABASE_URL", "postgres://server:fake-pw@127.0.0.1:1/cs"),
        ("VALKEY_URL", "redis://127.0.0.1:1"),
        ("SESSION_SECRET", "fake-session-secret-value-0123456789"),
        ("MAIL_TRANSPORT", "log"),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_owned(), value.to_owned()))
    .collect();
    for (name, value) in extra {
        vars.insert((*name).to_owned(), (*value).to_owned());
    }
    vars
}

fn lazy_pool() -> sqlx::PgPool {
    PgPoolOptions::new()
        .connect_lazy("postgres://server:fake-pw@127.0.0.1:1/cs")
        .expect("a lazy pool never connects")
}

fn limiter() -> Limiter {
    Limiter::Memory(Arc::new(MemoryLimiter::default()))
}

#[tokio::test]
async fn auth_setup_builds_mail_and_no_providers_without_oauth_settings() {
    let config = AuthConfig::from_map(&development(&[])).expect("a valid development config");

    let state = build_state(lazy_pool(), &config, limiter(), TrustedProxies::default())
        .expect("the state builds");

    assert!(state.mail.is_some(), "mail is wired");
    let port = state.oauth_port().expect("the OAuth port is set");
    assert!(
        port.providers.is_empty(),
        "no provider without its client settings"
    );
}

#[tokio::test]
async fn auth_setup_enables_only_the_providers_that_have_credentials() {
    let config = AuthConfig::from_map(&development(&[
        ("GITHUB_OAUTH_CLIENT_ID", "fake-github-client"),
        ("GITHUB_OAUTH_CLIENT_SECRET", "fake-github-secret"),
    ]))
    .expect("a valid development config");

    let state = build_state(lazy_pool(), &config, limiter(), TrustedProxies::default())
        .expect("the state builds");

    let port = state.oauth_port().expect("the OAuth port is set");
    assert!(port.providers.contains_key(&ProviderId::GitHub));
    assert!(!port.providers.contains_key(&ProviderId::Google));
}

#[tokio::test]
async fn auth_setup_leaves_mail_unwired_when_no_provider_is_configured() {
    // Production with no mail variables at all: OAuth-only sign-in.
    let vars: std::collections::HashMap<String, String> = [
        ("APP_ENV", "production"),
        ("PUBLIC_BASE_URL", "https://counterspell.example"),
        ("DATABASE_URL", "postgres://server:fake-pw@127.0.0.1:1/cs"),
        ("VALKEY_URL", "redis://127.0.0.1:1"),
        ("SESSION_SECRET", "fake-session-secret-value-0123456789"),
    ]
    .into_iter()
    .map(|(name, value)| (name.to_owned(), value.to_owned()))
    .collect();
    let config = AuthConfig::from_map(&vars).expect("production without mail is valid");

    let state = build_state(lazy_pool(), &config, limiter(), TrustedProxies::default())
        .expect("the state builds without mail");

    assert!(
        state.mail.is_none(),
        "no mail provider means mail stays unwired"
    );
    assert!(
        matches!(
            state.mail_port(),
            Err(counterspell_server::auth::errors::AuthError::EmailUnavailable)
        ),
        "mail routes answer email_unavailable"
    );
}

#[tokio::test]
async fn auth_setup_keeps_the_demo_address_out_of_mail() {
    let config = AuthConfig::from_map(&development(&[
        ("DEMO_ACCOUNT_EMAIL", "demo@example.co"),
        ("DEMO_ACCOUNT_PASSWORD", "fake-demo-passphrase-0123"),
    ]))
    .expect("a valid development config with a demo account");

    let state = build_state(lazy_pool(), &config, limiter(), TrustedProxies::default())
        .expect("the state builds");

    let mail = state.mail_port().expect("mail is wired");
    assert_eq!(
        mail.demo_email.as_ref().map(EmailAddress::as_str),
        Some("demo@example.co")
    );
}
