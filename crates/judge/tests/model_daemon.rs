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
