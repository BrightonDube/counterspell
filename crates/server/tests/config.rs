//! Settings are read from environment variables. These tests pass a fake lookup so no real environment is touched.

use counterspell_server::config::{ConfigError, ServerConfig};

fn lookup<'a>(pairs: &'a [(&'a str, &'a str)]) -> impl Fn(&str) -> Option<String> + 'a {
    move |name| {
        pairs
            .iter()
            .find(|(key, _)| *key == name)
            .map(|(_, value)| (*value).to_owned())
    }
}

const REQUIRED: [(&str, &str); 2] = [
    ("DATABASE_URL", "postgres://app@db.internal/counterspell"),
    ("VALKEY_URL", "redis://cache.internal:6379"),
];

#[test]
fn defaults_apply_when_only_required_urls_are_set() {
    let config = ServerConfig::from_lookup(lookup(&REQUIRED)).expect("config loads");

    assert_eq!(config.host, "0.0.0.0");
    assert_eq!(config.port, 8080);
    assert_eq!(config.static_dir.to_string_lossy(), "web/build");
    assert_eq!(config.limits.run_rate_user, 30);
    assert_eq!(config.limits.run_rate_ip, 100);
    assert_eq!(config.limits.run_active_cap, 3);
    assert_eq!(config.limits.queue_depth_max, 200);
    assert_eq!(
        config.limits.max_source_bytes, None,
        "unset MAX_SOURCE_BYTES is no ceiling — each driver's own limit applies"
    );
    assert_eq!(config.limits.request_body_limit_bytes, 131_072);
}

#[test]
fn a_configured_max_source_bytes_is_the_operator_ceiling() {
    let mut pairs = REQUIRED.to_vec();
    pairs.push(("MAX_SOURCE_BYTES", "32768"));
    let config = ServerConfig::from_lookup(lookup(&pairs)).expect("config loads");

    assert_eq!(config.limits.max_source_bytes, Some(32_768));
}

#[test]
fn missing_database_url_is_named_without_a_value() {
    let pairs = [("VALKEY_URL", "redis://cache.internal:6379")];
    let error = ServerConfig::from_lookup(lookup(&pairs)).expect_err("DATABASE_URL is required");

    assert_eq!(error, ConfigError::Missing("DATABASE_URL"));
}

#[test]
fn missing_valkey_url_is_named() {
    let pairs = [("DATABASE_URL", "postgres://app@db.internal/counterspell")];
    let error = ServerConfig::from_lookup(lookup(&pairs)).expect_err("VALKEY_URL is required");

    assert_eq!(error, ConfigError::Missing("VALKEY_URL"));
}

#[test]
fn port_outside_the_valid_range_is_rejected() {
    let mut pairs = REQUIRED.to_vec();
    pairs.push(("PORT", "70000"));
    let error = ServerConfig::from_lookup(lookup(&pairs)).expect_err("70000 is not a port");

    assert!(matches!(error, ConfigError::Invalid { name: "PORT", .. }));
}

#[test]
fn non_numeric_limit_is_rejected() {
    let mut pairs = REQUIRED.to_vec();
    pairs.push(("RUN_RATE_USER", "many"));
    let error = ServerConfig::from_lookup(lookup(&pairs)).expect_err("a limit must be a number");

    assert!(matches!(
        error,
        ConfigError::Invalid {
            name: "RUN_RATE_USER",
            ..
        }
    ));
}

#[test]
fn zero_active_cap_is_rejected() {
    let mut pairs = REQUIRED.to_vec();
    pairs.push(("RUN_ACTIVE_CAP_PER_USER", "0"));
    let error =
        ServerConfig::from_lookup(lookup(&pairs)).expect_err("a cap of zero blocks all runs");

    assert!(matches!(
        error,
        ConfigError::Invalid {
            name: "RUN_ACTIVE_CAP_PER_USER",
            ..
        }
    ));
}

#[test]
fn debug_output_hides_connection_urls() {
    let config = ServerConfig::from_lookup(lookup(&REQUIRED)).expect("config loads");
    let printed = format!("{config:?}");

    assert!(
        !printed.contains("db.internal"),
        "database host leaked: {printed}"
    );
    assert!(
        !printed.contains("cache.internal"),
        "valkey host leaked: {printed}"
    );
}

#[test]
fn bind_address_combines_host_and_port() {
    let mut pairs = REQUIRED.to_vec();
    pairs.push(("HOST", "127.0.0.1"));
    pairs.push(("PORT", "9000"));
    let config = ServerConfig::from_lookup(lookup(&pairs)).expect("config loads");

    assert_eq!(
        config.address().expect("valid address").to_string(),
        "127.0.0.1:9000"
    );
}

#[test]
fn sse_settings_have_the_design_defaults() {
    let config = ServerConfig::from_lookup(lookup(&REQUIRED)).expect("config loads");

    assert_eq!(config.sse.poll_ms, 250);
    assert_eq!(config.sse.heartbeat_secs, 15);
    assert_eq!(config.sse.max_age_secs, 900);
    assert_eq!(config.sse.max_streams_per_user, 5);
}

#[test]
fn sse_poll_interval_below_the_minimum_is_rejected() {
    let mut pairs = REQUIRED.to_vec();
    pairs.push(("SSE_POLL_MS", "10"));
    let error = ServerConfig::from_lookup(lookup(&pairs)).expect_err("10 ms is too fast");

    assert!(matches!(
        error,
        ConfigError::Invalid {
            name: "SSE_POLL_MS",
            ..
        }
    ));
}
