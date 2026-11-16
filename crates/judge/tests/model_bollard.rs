//! Model tests for the bollard conversion (judge-sandbox T20, R2.2, R3.15, R1.9).
//!
//! No daemon is involved. A policy is converted into the create request, the request
//! is turned into the inspect document a daemon would return, and that document is read
//! back into a policy. Each mutation weakens one field of that document, and the test
//! expects exactly one violation back.
#![cfg(feature = "sandbox")]

use std::time::Duration;

use bollard::models::{
    ContainerConfig, ContainerInspectResponse, HostConfig, HostConfigCgroupnsModeEnum,
    ResourcesUlimits,
};
use counterspell_judge::languages::PinnedImage;
use counterspell_judge::sandbox::bollard_backend::{
    from_inspect, image_digest_matches, to_create_body, without_image_env, ReadbackError,
};
use counterspell_judge::sandbox::config::OciRuntime;
use counterspell_judge::sandbox::policy::{ContainerPolicy, PolicyViolation};

mod support;
use support::shipped_catalogue;

const PINNED: &str =
    "python:3.13-slim@sha256:e48f59cdba69a3cbd9ed2cf0f348d90d45a8d350d98134cf1070bc106ee9ed90";
const MIB_SIGNED: i64 = 1024 * 1024;

fn production_policy() -> ContainerPolicy {
    let (_dir, catalog) = shipped_catalogue(false);
    let python = catalog.get("python").expect("python is in the fixture");
    ContainerPolicy::for_language(python, Duration::from_secs(20), OciRuntime::Runc)
}

/// The inspect document a daemon would return for a container created from `policy`.
/// The create request and the inspect model share their field names in the Docker API.
fn daemon_view(policy: &ContainerPolicy) -> ContainerInspectResponse {
    let body = to_create_body(policy);
    let mut config = serde_json::to_value(&body).expect("create request serialises");
    let host = config
        .as_object_mut()
        .and_then(|fields| fields.remove("HostConfig"))
        .unwrap_or_default();
    let document = serde_json::json!({ "Config": config, "HostConfig": host });
    serde_json::from_value(document).expect("the daemon view matches the inspect model")
}

fn host(view: &mut ContainerInspectResponse) -> &mut HostConfig {
    view.host_config.as_mut().expect("host config is present")
}

fn config(view: &mut ContainerInspectResponse) -> &mut ContainerConfig {
    view.config.as_mut().expect("config is present")
}

#[test]
fn the_create_request_sets_every_control() {
    let body = to_create_body(&production_policy());
    let host = body.host_config.as_ref().expect("host config is set");

    assert_eq!(body.image.as_deref(), Some(PINNED));
    assert_eq!(body.user.as_deref(), Some("65534:65534"));
    assert_eq!(body.working_dir.as_deref(), Some("/box"));
    assert_eq!(body.cmd, Some(vec!["sleep".to_owned(), "25".to_owned()]));
    assert_eq!(body.entrypoint, Some(Vec::new()));
    assert_eq!(host.network_mode.as_deref(), Some("none"));
    assert_eq!(host.readonly_rootfs, Some(true));
    assert_eq!(host.cap_drop, Some(vec!["ALL".to_owned()]));
    assert_eq!(
        host.security_opt,
        Some(vec!["no-new-privileges:true".to_owned()])
    );
    assert_eq!(host.privileged, Some(false));
    assert_eq!(host.memory, Some(256 * MIB_SIGNED));
    assert_eq!(host.memory_swap, Some(256 * MIB_SIGNED));
    assert_eq!(host.nano_cpus, Some(1_000_000_000));
    assert_eq!(host.pids_limit, Some(64));
    assert_eq!(host.auto_remove, Some(false));
    assert_eq!(host.runtime.as_deref(), Some("runc"));
    assert_eq!(
        host.cgroupns_mode,
        Some(HostConfigCgroupnsModeEnum::PRIVATE)
    );
    let log = host.log_config.as_ref().expect("log config is set");
    assert_eq!(log.typ.as_deref(), Some("none"));
}

#[test]
fn the_create_request_adds_no_host_resource_or_extra_field() {
    let body = to_create_body(&production_policy());
    let host = body.host_config.as_ref().expect("host config is set");

    assert!(body.hostname.is_none());
    assert!(body.network_disabled.is_none());
    assert!(host.binds.is_none());
    assert!(host.devices.is_none());
    assert!(host.extra_hosts.is_none());
    assert!(host.links.is_none());
    assert!(host.volumes_from.is_none());
    assert!(host.mounts.is_none());
    assert_eq!(
        host.cap_add.clone().unwrap_or_default(),
        Vec::<String>::new()
    );
}

#[test]
fn a_daemon_view_reads_back_as_the_same_policy() {
    let policy = production_policy();

    let read = from_inspect(&daemon_view(&policy)).expect("the view reads back");

    assert_eq!(read, policy);
    assert_eq!(read.hardening_violations(), Vec::new());
}

#[test]
fn a_view_without_a_runtime_is_refused_rather_than_guessed() {
    let mut view = daemon_view(&production_policy());
    host(&mut view).runtime = None;

    assert_eq!(
        from_inspect(&view),
        Err(ReadbackError::Unrecognised("runtime"))
    );
}

#[test]
fn a_view_without_host_config_is_refused() {
    let mut view = daemon_view(&production_policy());
    view.host_config = None;

    assert_eq!(
        from_inspect(&view),
        Err(ReadbackError::Missing("host config"))
    );
}

type Mutation = fn(&mut ContainerInspectResponse);

/// Each row weakens exactly one control in the daemon's answer. The checker must report
/// exactly that control, and nothing else. The table is one list on purpose: one row per
/// control is easier to audit than several helpers.
#[allow(clippy::too_many_lines)]
#[test]
fn each_weakened_daemon_field_is_reported_as_its_own_violation() {
    let cases: Vec<(&str, Mutation, PolicyViolation)> = vec![
        (
            "root filesystem writable",
            (|v| host(v).readonly_rootfs = Some(false)) as Mutation,
            PolicyViolation::WritableRootFs,
        ),
        (
            "network on",
            (|v| host(v).network_mode = Some("bridge".to_owned())) as Mutation,
            PolicyViolation::NetworkEnabled,
        ),
        (
            "privileged",
            (|v| host(v).privileged = Some(true)) as Mutation,
            PolicyViolation::Privileged,
        ),
        (
            "capability not dropped",
            (|v| host(v).cap_drop = Some(Vec::new())) as Mutation,
            PolicyViolation::CapabilitiesNotDropped,
        ),
        (
            "seccomp unconfined",
            (|v| {
                host(v).security_opt = Some(vec![
                    "no-new-privileges:true".to_owned(),
                    "seccomp=unconfined".to_owned(),
                ]);
            }) as Mutation,
            PolicyViolation::SeccompOverridden,
        ),
        (
            "extra security option",
            (|v| {
                host(v).security_opt = Some(vec![
                    "no-new-privileges:true".to_owned(),
                    "apparmor=unconfined".to_owned(),
                ]);
            }) as Mutation,
            PolicyViolation::UnexpectedSecurityOption,
        ),
        (
            "pid limit raised",
            (|v| host(v).pids_limit = Some(128)) as Mutation,
            PolicyViolation::PidsLimit,
        ),
        (
            "swap allowed",
            (|v| host(v).memory_swap = Some(512 * MIB_SIGNED)) as Mutation,
            PolicyViolation::SwapAllowed,
        ),
        (
            "cpu quota changed",
            (|v| host(v).nano_cpus = Some(2_000_000_000)) as Mutation,
            PolicyViolation::CpuQuota,
        ),
        (
            "nofile limit removed",
            (|v| {
                let kept: Vec<ResourcesUlimits> = host(v)
                    .ulimits
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|u| u.name.as_deref() != Some("nofile"))
                    .collect();
                host(v).ulimits = Some(kept);
            }) as Mutation,
            PolicyViolation::NoFileLimit,
        ),
        (
            "core dumps allowed",
            (|v| {
                let kept: Vec<ResourcesUlimits> = host(v)
                    .ulimits
                    .clone()
                    .unwrap_or_default()
                    .into_iter()
                    .filter(|u| u.name.as_deref() != Some("core"))
                    .collect();
                host(v).ulimits = Some(kept);
            }) as Mutation,
            PolicyViolation::CoreDumpsAllowed,
        ),
        (
            "process limit set",
            (|v| {
                let mut ulimits = host(v).ulimits.clone().unwrap_or_default();
                ulimits.push(ResourcesUlimits {
                    name: Some("nproc".to_owned()),
                    soft: Some(64),
                    hard: Some(64),
                });
                host(v).ulimits = Some(ulimits);
            }) as Mutation,
            PolicyViolation::ProcLimitSet,
        ),
        (
            "log driver",
            (|v| {
                host(v).log_config = Some(bollard::models::HostConfigLogConfig {
                    typ: Some("json-file".to_owned()),
                    config: None,
                });
            }) as Mutation,
            PolicyViolation::LogDriver,
        ),
        (
            "auto remove on",
            (|v| host(v).auto_remove = Some(true)) as Mutation,
            PolicyViolation::AutoRemove,
        ),
        (
            "host bind mount",
            (|v| host(v).binds = Some(vec!["/etc:/host-etc".to_owned()])) as Mutation,
            PolicyViolation::HostResourceMounted,
        ),
        (
            "cgroup shared with the host",
            (|v| host(v).cgroupns_mode = Some(HostConfigCgroupnsModeEnum::HOST)) as Mutation,
            PolicyViolation::SharedNamespace("cgroup"),
        ),
        (
            "pid namespace shared with the host",
            (|v| host(v).pid_mode = Some("host".to_owned())) as Mutation,
            PolicyViolation::SharedNamespace("pid"),
        ),
        (
            "idle command replaced",
            (|v| config(v).cmd = Some(vec!["sh".to_owned()])) as Mutation,
            PolicyViolation::IdlePidOne,
        ),
        (
            "running as root",
            (|v| config(v).user = Some("0:0".to_owned())) as Mutation,
            PolicyViolation::NonRootUser,
        ),
        (
            "extra environment variable",
            (|v| {
                let mut env = config(v).env.clone().unwrap_or_default();
                env.push("HTTP_PROXY=http://example.invalid".to_owned());
                config(v).env = Some(env);
            }) as Mutation,
            PolicyViolation::UnexpectedEnvironment,
        ),
        (
            "managed label removed",
            (|v| config(v).labels = Some(std::collections::HashMap::new())) as Mutation,
            PolicyViolation::MissingLabel("counterspell.managed"),
        ),
        (
            "/tmp without noexec",
            (|v| {
                let mut tmpfs = host(v).tmpfs.clone().unwrap_or_default();
                tmpfs.insert(
                    "/tmp".to_owned(),
                    "rw,nosuid,nodev,size=16777216".to_owned(),
                );
                host(v).tmpfs = Some(tmpfs);
            }) as Mutation,
            PolicyViolation::TmpMount("noexec missing"),
        ),
        (
            "/box without nosuid",
            (|v| {
                let mut tmpfs = host(v).tmpfs.clone().unwrap_or_default();
                tmpfs.insert(
                    "/box".to_owned(),
                    "rw,nodev,noexec,uid=65534,gid=65534,mode=0700,size=67108864".to_owned(),
                );
                host(v).tmpfs = Some(tmpfs);
            }) as Mutation,
            PolicyViolation::ScratchMount("nosuid missing"),
        ),
        (
            "/box mount removed",
            (|v| {
                let mut tmpfs = host(v).tmpfs.clone().unwrap_or_default();
                tmpfs.remove("/box");
                host(v).tmpfs = Some(tmpfs);
            }) as Mutation,
            PolicyViolation::ScratchMount("missing"),
        ),
        (
            "shared memory enlarged",
            (|v| host(v).shm_size = Some(1024 * MIB_SIGNED)) as Mutation,
            PolicyViolation::ShmSize,
        ),
    ];

    for (name, mutate, expected) in cases {
        let mut view = daemon_view(&production_policy());
        mutate(&mut view);
        let read = from_inspect(&view).unwrap_or_else(|error| panic!("{name}: {error}"));
        assert_eq!(
            read.hardening_violations(),
            vec![expected],
            "mutation: {name}"
        );
    }
}

#[test]
fn a_digest_in_the_repo_digests_matches_the_pinned_digest() {
    let pinned = PinnedImage::parse(PINNED).expect("the pinned fixture is valid");
    let digests = vec![
        "python@sha256:e48f59cdba69a3cbd9ed2cf0f348d90d45a8d350d98134cf1070bc106ee9ed90".to_owned(),
    ];

    assert!(image_digest_matches(&digests, &pinned));
}

#[test]
fn a_different_digest_does_not_match() {
    let pinned = PinnedImage::parse(PINNED).expect("the pinned fixture is valid");
    let digests = vec![
        "python@sha256:0000000000000000000000000000000000000000000000000000000000000000".to_owned(),
    ];

    assert!(!image_digest_matches(&digests, &pinned));
}

#[test]
fn an_empty_digest_list_does_not_match() {
    let pinned = PinnedImage::parse(PINNED).expect("the pinned fixture is valid");

    assert!(!image_digest_matches(&[], &pinned));
}

#[test]
fn a_tag_only_entry_does_not_match() {
    let pinned = PinnedImage::parse(PINNED).expect("the pinned fixture is valid");
    let digests = vec!["python:3.13-slim".to_owned()];

    assert!(!image_digest_matches(&digests, &pinned));
}

fn env_pairs(entries: &[(&str, &str)]) -> Vec<(String, String)> {
    entries
        .iter()
        .map(|(key, value)| ((*key).to_owned(), (*value).to_owned()))
        .collect()
}

fn image_env() -> Vec<String> {
    [
        "PATH=/usr/local/bin:/usr/local/sbin:/usr/bin:/bin",
        "LANG=C.UTF-8",
        "GPG_KEY=image-declared-value",
        "PYTHON_VERSION=3.13.18",
    ]
    .iter()
    .map(|entry| (*entry).to_owned())
    .collect()
}

#[test]
fn variables_the_pinned_image_declares_are_tolerated_on_read_back() {
    // The daemon merges the image's own variables into the container. Ours are PATH, LANG
    // and HOME; GPG_KEY and PYTHON_VERSION come from the image with the same values.
    let read = env_pairs(&[
        ("PATH", "/usr/local/bin:/usr/bin:/bin"),
        ("LANG", "C.UTF-8"),
        ("HOME", "/box"),
        ("GPG_KEY", "image-declared-value"),
        ("PYTHON_VERSION", "3.13.18"),
    ]);

    let kept = without_image_env(read, &image_env());

    assert_eq!(
        kept,
        env_pairs(&[
            ("PATH", "/usr/local/bin:/usr/bin:/bin"),
            ("LANG", "C.UTF-8"),
            ("HOME", "/box"),
        ])
    );
}

#[test]
fn an_injected_variable_the_image_does_not_declare_is_kept_so_the_check_fails() {
    let read = env_pairs(&[("HTTP_PROXY", "http://example.invalid")]);

    let kept = without_image_env(read.clone(), &image_env());

    assert_eq!(kept, read);
}

#[test]
fn an_image_variable_with_a_changed_value_is_kept_so_the_check_fails() {
    let read = env_pairs(&[("GPG_KEY", "ATTACKER")]);

    let kept = without_image_env(read.clone(), &image_env());

    assert_eq!(kept, read);
}

#[test]
fn a_fixed_key_is_never_dropped_even_when_the_image_declares_it() {
    // PATH is ours. The image's PATH differs, so a daemon that kept the image value must fail.
    let read = env_pairs(&[("PATH", "/usr/local/bin:/usr/local/sbin:/usr/bin:/bin")]);

    let kept = without_image_env(read.clone(), &image_env());

    assert_eq!(kept, read);
}
