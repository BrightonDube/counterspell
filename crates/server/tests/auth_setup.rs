//! Building the auth state from configuration (auth T33). No database or network is used: the pool
//! is created lazily and never connects.
//!
//! Run with `cargo test -p counterspell-server auth_setup`.

use std::collections::HashMap;

use counterspell_server::{
    auth::{config::AuthConfig, oauth::ProviderId, setup::build_state},
    client_ip::TrustedProxies,
    ratelimit::{Limiter, MemoryLimiter},
};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;

/// A development configuration, which needs no provider credentials.
// test-change: AUTH-EMAIL-REMOVAL
fn development(extra: &[(&str, &str)]) -> HashMap<String, String> {
    let mut vars: HashMap<String, String> = [
        ("APP_ENV", "development"),
        ("PUBLIC_BASE_URL", "http://localhost:8080"),
        ("DATABASE_URL", "postgres://server:fake-pw@127.0.0.1:1/cs"),
        ("VALKEY_URL", "redis://127.0.0.1:1"),
        ("SESSION_SECRET", "fake-session-secret-value-0123456789"),
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
// test-change: AUTH-EMAIL-REMOVAL
async fn auth_setup_builds_state_and_no_providers_without_oauth_settings() {
    let config = AuthConfig::from_map(&development(&[])).expect("a valid development config");

    let state = build_state(lazy_pool(), &config, limiter(), TrustedProxies::default())
        .expect("the state builds");

    let port = state.oauth_port().expect("the OAuth port is set");
    assert!(
        port.providers.is_empty(),
        "no provider without its client settings"
    );
    assert_eq!(state.settings.demo_email(), None);
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
// test-change: AUTH-EMAIL-REMOVAL
async fn auth_setup_reports_the_demo_account_in_the_public_settings() {
    let config = AuthConfig::from_map(&development(&[
        ("DEMO_ACCOUNT_EMAIL", "demo@example.co"),
        ("DEMO_ACCOUNT_PASSWORD", "fake-demo-passphrase-0123"),
    ]))
    .expect("a valid development config with a demo account");

    let state = build_state(lazy_pool(), &config, limiter(), TrustedProxies::default())
        .expect("the state builds");

    assert_eq!(state.settings.demo_email(), Some("demo@example.co"));
}
