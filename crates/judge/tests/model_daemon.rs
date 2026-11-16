//! Daemon identity checks (judge-sandbox T19, R1.1 to R1.3, R4.3).
//!
//! The judge may only talk to the rootless sandbox daemon through an explicit
//! socket path. These tests use hand-written facts. T02 replaces them with
//! recorded `docker info` output from the VPS.

use std::path::PathBuf;

use counterspell_judge::sandbox::config::OciRuntime;
use counterspell_judge::sandbox::daemon::{
    check_facts, validate_socket_path, DaemonCheckError, DaemonFacts, SocketPathError,
};

fn good_facts() -> DaemonFacts {
    DaemonFacts {
        rootless: true,
        data_root: "/home/sandbox/.local/share/docker".to_owned(),
        cgroup_v2: true,
        cgroup_driver: "systemd".to_owned(),
        runtimes: vec!["runc".to_owned()],
        warnings: Vec::new(),
    }
}

/// P14: the daemon module never reads the environment, so `DOCKER_HOST` cannot redirect
/// the judge. This scans the source instead of setting process variables in a test.
/// Doc comments may name `DOCKER_HOST`; only code that reads environment is refused.
#[test]
fn daemon_module_never_reads_the_environment() {
    let source = include_str!("../src/sandbox/daemon.rs");
    let code: String = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    assert!(!code.contains("env::"), "daemon.rs reads the environment");
    assert!(!code.contains("var_os"), "daemon.rs reads the environment");
}

#[test]
fn empty_and_blank_socket_paths_are_refused() {
    assert_eq!(validate_socket_path(""), Err(SocketPathError::Empty));
    assert_eq!(validate_socket_path("   "), Err(SocketPathError::Empty));
}

#[test]
fn relative_socket_path_is_refused() {
    assert_eq!(
        validate_socket_path("docker.sock"),
        Err(SocketPathError::NotAbsolute)
    );
}

#[test]
fn urls_are_refused_so_tcp_and_unix_schemes_never_pass() {
    assert_eq!(
        validate_socket_path("tcp://127.0.0.1:2375"),
        Err(SocketPathError::NotPlainPath)
    );
    assert_eq!(
        validate_socket_path("unix:///run/user/1000/docker.sock"),
        Err(SocketPathError::NotPlainPath)
    );
}

#[test]
fn main_daemon_sockets_are_refused() {
    assert_eq!(
        validate_socket_path("/var/run/docker.sock"),
        Err(SocketPathError::MainDaemonSocket)
    );
    assert_eq!(
        validate_socket_path("/run/docker.sock"),
        Err(SocketPathError::MainDaemonSocket)
    );
}

#[test]
fn rootless_sandbox_socket_is_accepted_and_trimmed() {
    assert_eq!(
        validate_socket_path("  /run/user/1000/docker.sock  "),
        Ok(PathBuf::from("/run/user/1000/docker.sock"))
    );
}

#[test]
fn good_rootless_daemon_passes_the_checks() {
    assert_eq!(check_facts(&good_facts(), OciRuntime::Runc), Ok(()));
}

#[test]
fn a_root_daemon_is_refused() {
    let facts = DaemonFacts {
        rootless: false,
        ..good_facts()
    };
    assert_eq!(
        check_facts(&facts, OciRuntime::Runc),
        Err(DaemonCheckError::NotRootless)
    );
}

#[test]
fn the_main_data_directory_is_refused() {
    let facts = DaemonFacts {
        data_root: "/var/lib/docker".to_owned(),
        ..good_facts()
    };
    assert_eq!(
        check_facts(&facts, OciRuntime::Runc),
        Err(DaemonCheckError::MainDataRoot)
    );
}

#[test]
fn cgroup_v1_is_refused() {
    let facts = DaemonFacts {
        cgroup_v2: false,
        ..good_facts()
    };
    assert_eq!(
        check_facts(&facts, OciRuntime::Runc),
        Err(DaemonCheckError::NotCgroupV2)
    );
}

#[test]
fn driver_none_is_refused() {
    let facts = DaemonFacts {
        cgroup_driver: "none".to_owned(),
        ..good_facts()
    };
    assert_eq!(
        check_facts(&facts, OciRuntime::Runc),
        Err(DaemonCheckError::NoCgroupDriver)
    );
}

#[test]
fn a_requested_runtime_must_be_registered() {
    assert_eq!(
        check_facts(&good_facts(), OciRuntime::Runsc),
        Err(DaemonCheckError::RuntimeMissing("runsc".to_owned()))
    );
}

#[test]
fn a_limit_support_warning_is_refused() {
    let facts = DaemonFacts {
        warnings: vec!["WARNING: No swap limit support".to_owned()],
        ..good_facts()
    };
    assert_eq!(
        check_facts(&facts, OciRuntime::Runc),
        Err(DaemonCheckError::LimitSupport(
            "WARNING: No swap limit support".to_owned()
        ))
    );
}

#[cfg(feature = "sandbox")]
mod from_bollard {
    use super::*;
    use bollard::models::SystemInfo;

    fn synthetic_info(overrides: &serde_json::Value) -> SystemInfo {
        let mut base = serde_json::json!({
            "SecurityOptions": ["name=seccomp,profile=default", "name=rootless"],
            "DockerRootDir": "/home/sandbox/.local/share/docker",
            "CgroupVersion": "2",
            "CgroupDriver": "systemd",
            "Runtimes": {"runc": {"path": "runc"}},
            "Warnings": []
        });
        if let (Some(target), Some(extra)) = (base.as_object_mut(), overrides.as_object()) {
            for (key, value) in extra {
                target.insert(key.clone(), value.clone());
            }
        }
        serde_json::from_value(base).expect("synthetic fixture matches SystemInfo")
    }

    #[test]
    fn a_rootless_v2_info_converts_to_passing_facts() {
        let facts = DaemonFacts::from(&synthetic_info(&serde_json::json!({})));
        assert_eq!(check_facts(&facts, OciRuntime::Runc), Ok(()));
    }

    #[test]
    fn missing_security_options_fail_closed_as_not_rootless() {
        let info = synthetic_info(&serde_json::json!({"SecurityOptions": null}));
        let facts = DaemonFacts::from(&info);
        assert_eq!(
            check_facts(&facts, OciRuntime::Runc),
            Err(DaemonCheckError::NotRootless)
        );
    }

    /// Recorded 14 Nov 2026 from the local WSL Docker daemon (rootful, not the sandbox
    /// daemon): `docker info --format` gave `["name=seccomp,profile=builtin","name=cgroupns"]`,
    /// cgroup version 2, driver systemd, root dir `/var/lib/docker`. A real rejected fixture.
    #[test]
    fn recorded_rootful_local_daemon_is_rejected() {
        let info: SystemInfo = serde_json::from_value(serde_json::json!({
            "SecurityOptions": ["name=seccomp,profile=builtin", "name=cgroupns"],
            "CgroupVersion": "2",
            "CgroupDriver": "systemd",
            "DockerRootDir": "/var/lib/docker"
        }))
        .expect("recorded fixture matches SystemInfo");
        let facts = DaemonFacts::from(&info);
        assert_eq!(
            check_facts(&facts, OciRuntime::Runc),
            Err(DaemonCheckError::NotRootless)
        );
    }

    #[test]
    fn a_v1_cgroup_field_fails_closed() {
        let info = synthetic_info(&serde_json::json!({"CgroupVersion": "1"}));
        let facts = DaemonFacts::from(&info);
        assert_eq!(
            check_facts(&facts, OciRuntime::Runc),
            Err(DaemonCheckError::NotCgroupV2)
        );
    }
}
