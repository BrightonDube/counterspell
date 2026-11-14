//! Model tests for the sandbox settings (requirement R17.1). No Docker is involved.
//!
//! The settings are read through a lookup closure, so these tests never touch the
//! real process environment.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use counterspell_judge::config::ConfigError;
use counterspell_judge::sandbox::config::{OciRuntime, SandboxConfig};

const SOCKET_NAME: &str = "SANDBOX_DOCKER_SOCKET";
const SOCKET_VALUE: &str = "/srv/sandbox/docker.sock";

fn from_pairs(pairs: &[(&str, &str)]) -> Result<SandboxConfig, ConfigError> {
    let map: HashMap<String, String> = pairs
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect();
    SandboxConfig::from_lookup(|name| map.get(name).cloned())
}

#[test]
fn defaults_match_the_design_when_only_the_socket_is_set() {
    let config = from_pairs(&[(SOCKET_NAME, SOCKET_VALUE)]).unwrap();

    assert_eq!(config.docker_socket, PathBuf::from(SOCKET_VALUE));
    assert_eq!(config.runtime, OciRuntime::Runc);
    assert_eq!(config.max_containers, 4);
    assert_eq!(config.reaper_interval, Duration::from_secs(10));
    assert_eq!(config.orphan_age, Duration::from_secs(40));
    assert_eq!(config.kill_slack, Duration::from_millis(1000));
    assert_eq!(config.run_wall_cap, Duration::from_secs(20));
    assert_eq!(config.max_consecutive_abnormal, 3);
    assert_eq!(config.cleanup_budget, Duration::from_secs(5));
    assert_eq!(config.api_timeout, Duration::from_secs(10));
    assert_eq!(
        config.languages_file,
        PathBuf::from("languages/languages.toml")
    );
}

#[test]
fn the_container_cap_follows_the_judge_concurrency_by_default() {
    let config = from_pairs(&[(SOCKET_NAME, SOCKET_VALUE), ("JUDGE_CONCURRENCY", "4")]).unwrap();
    assert_eq!(config.max_containers, 8);
}

#[test]
fn a_missing_socket_path_is_refused_because_there_is_no_default() {
    let error = from_pairs(&[]).unwrap_err();
    assert_eq!(
        error,
        ConfigError::MissingRequired {
            name: "SANDBOX_DOCKER_SOCKET"
        }
    );
}

#[test]
fn a_blank_socket_path_counts_as_missing() {
    let error = from_pairs(&[(SOCKET_NAME, "   ")]).unwrap_err();
    assert_eq!(
        error,
        ConfigError::MissingRequired {
            name: "SANDBOX_DOCKER_SOCKET"
        }
    );
}

#[test]
fn runsc_is_accepted_as_a_runtime_name_and_other_names_are_refused() {
    let runsc = from_pairs(&[(SOCKET_NAME, SOCKET_VALUE), ("SANDBOX_RUNTIME", "runsc")]).unwrap();
    assert_eq!(runsc.runtime, OciRuntime::Runsc);

    let error =
        from_pairs(&[(SOCKET_NAME, SOCKET_VALUE), ("SANDBOX_RUNTIME", "crun")]).unwrap_err();
    assert_eq!(error, ConfigError::UnknownRuntime);
}

#[test]
fn a_kill_slack_below_its_range_is_out_of_range() {
    let error =
        from_pairs(&[(SOCKET_NAME, SOCKET_VALUE), ("SANDBOX_KILL_SLACK_MS", "50")]).unwrap_err();
    assert_eq!(
        error,
        ConfigError::OutOfRange {
            name: "SANDBOX_KILL_SLACK_MS",
            min: 100,
            max: 10_000
        }
    );
}

#[test]
fn a_value_that_is_not_a_whole_number_names_the_variable() {
    let error = from_pairs(&[
        (SOCKET_NAME, SOCKET_VALUE),
        ("SANDBOX_MAX_CONTAINERS", "lots"),
    ])
    .unwrap_err();
    assert_eq!(
        error,
        ConfigError::NotANumber {
            name: "SANDBOX_MAX_CONTAINERS"
        }
    );
}

#[test]
fn the_orphan_age_must_exceed_the_run_cap_plus_fifteen_seconds() {
    // Default run cap is 20 s, so the orphan age must be more than 35 s.
    let too_young = from_pairs(&[
        (SOCKET_NAME, SOCKET_VALUE),
        ("SANDBOX_ORPHAN_AGE_SECS", "35"),
    ])
    .unwrap_err();
    assert_eq!(too_young, ConfigError::OrphanAgeTooShort);

    let just_old_enough = from_pairs(&[
        (SOCKET_NAME, SOCKET_VALUE),
        ("SANDBOX_ORPHAN_AGE_SECS", "36"),
    ])
    .unwrap();
    assert_eq!(just_old_enough.orphan_age, Duration::from_secs(36));
}

#[test]
fn raising_the_run_cap_raises_the_orphan_age_it_needs() {
    // A 60 s cap needs an orphan age above 75 s, so the default 40 s is refused.
    let error =
        from_pairs(&[(SOCKET_NAME, SOCKET_VALUE), ("RUN_WALL_CAP_SECS", "60")]).unwrap_err();
    assert_eq!(error, ConfigError::OrphanAgeTooShort);
}
