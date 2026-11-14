//! Guards `docker-compose.yml`: the `server` service must set every variable that
//! `ServerConfig::from_lookup` requires. Without `VALKEY_URL` the container exits at start-up with
//! `ConfigError::Missing("VALKEY_URL")`, and `docker compose config` cannot catch that because it
//! only validates YAML.

use std::{fs, path::PathBuf};

/// The variables `ServerConfig::from_lookup` requires. Keep this list in step with the
/// `required(&lookup, ...)` calls in `crates/server/src/config.rs`.
const REQUIRED_SERVER_VARS: [&str; 2] = ["DATABASE_URL", "VALKEY_URL"];

fn compose_text() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../docker-compose.yml");
    fs::read_to_string(&path).unwrap_or_else(|error| {
        panic!("could not read {}: {error}", path.display());
    })
}

/// The keys under `services.server.environment`, found by indentation. The compose file uses
/// two-space indents for services and six-space indents for environment entries.
fn server_environment_keys(compose: &str) -> Vec<String> {
    let mut in_server = false;
    let mut in_environment = false;
    let mut keys = Vec::new();
    for line in compose.lines() {
        if line.starts_with("  ") && !line.starts_with("   ") && line.trim_end().ends_with(':') {
            in_server = line.trim() == "server:";
            in_environment = false;
            continue;
        }
        if !in_server {
            continue;
        }
        if line.trim() == "environment:" && line.starts_with("    ") && !line.starts_with("     ") {
            in_environment = true;
            continue;
        }
        if in_environment {
            if line.starts_with("      ") && !line.starts_with("       ") {
                if let Some((key, _)) = line.trim().split_once(':') {
                    keys.push(key.to_owned());
                }
            } else if !line.trim().is_empty() && !line.starts_with("      ") {
                in_environment = false;
            }
        }
    }
    keys
}

#[test]
fn compose_server_service_sets_every_required_config_variable() {
    let keys = server_environment_keys(&compose_text());
    assert!(
        !keys.is_empty(),
        "found no environment keys under services.server; the parser or the file layout changed"
    );
    for name in REQUIRED_SERVER_VARS {
        assert!(
            keys.iter().any(|key| key == name),
            "docker-compose.yml services.server.environment does not set {name}; the server exits at start-up without it. Keys found: {keys:?}"
        );
    }
}
