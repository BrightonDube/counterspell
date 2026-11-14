//! Model tests for the container policy and its hardening checker (requirements
//! R3.1 to R3.14 and R3.16, property P3). No Docker is involved.
//!
//! Each mutation test weakens exactly one control and expects exactly one violation
//! back. That proves the checker is not a function that passes everything.

use std::time::Duration;

use counterspell_judge::sandbox::config::OciRuntime;
use counterspell_judge::sandbox::policy::{
    ContainerPolicy, PolicyViolation, ScratchExec, SeccompProfile, TmpfsMount, Ulimit,
};

mod support;
use support::shipped_catalogue;

const MIB: u64 = 1024 * 1024;

fn policy_for(scratch_exec: bool) -> ContainerPolicy {
    let (_dir, catalog) = shipped_catalogue(scratch_exec);
    let python = catalog.get("python").unwrap();
    ContainerPolicy::for_language(python, Duration::from_secs(20), OciRuntime::Runc)
}

fn valid_policy() -> ContainerPolicy {
    policy_for(false)
}

type Mutation = fn(&mut ContainerPolicy);

#[test]
fn the_production_policy_has_no_violations() {
    assert_eq!(valid_policy().hardening_violations(), Vec::new());
}

#[test]
fn the_production_policy_uses_the_default_seccomp_profile_never_unconfined() {
    let policy = valid_policy();
    assert_eq!(policy.seccomp, SeccompProfile::Default);
    assert_ne!(policy.seccomp, SeccompProfile::Unconfined);
}

#[test]
fn the_idle_pid_one_runs_sleep_for_the_run_cap_plus_five_seconds() {
    let policy = valid_policy();
    assert_eq!(policy.entrypoint, Vec::<String>::new());
    assert_eq!(policy.command, vec!["sleep".to_owned(), "25".to_owned()]);
}

#[test]
fn the_production_limits_match_the_design() {
    let policy = valid_policy();
    assert_eq!(policy.limits.memory_bytes, 256 * MIB);
    assert_eq!(policy.limits.memory_swap_bytes, 256 * MIB);
    assert_eq!(policy.limits.nano_cpus, 1_000_000_000);
    assert_eq!(policy.limits.pids_limit, 64);
    assert_eq!(policy.working_dir, "/box");
    assert_eq!(policy.user, "65534:65534");
    assert_eq!(policy.network, "none");
    assert!(policy.read_only_rootfs);
    assert_eq!(policy.log_driver, "none");
    assert_eq!(policy.restart_policy, "no");
    assert!(!policy.auto_remove);
}

#[test]
fn a_language_that_may_exec_its_scratch_directory_gets_no_noexec_flag() {
    let policy = policy_for(true);
    assert_eq!(policy.scratch, ScratchExec::Allowed);
    assert_eq!(policy.hardening_violations(), Vec::new());
    let scratch = policy.tmpfs.iter().find(|m| m.path == "/box").unwrap();
    assert!(!scratch.options.iter().any(|o| o == "noexec"));
}

#[test]
fn the_fixed_environment_is_exactly_path_lang_and_home() {
    let policy = valid_policy();
    assert_eq!(
        policy.env,
        vec![
            ("PATH".to_owned(), "/usr/local/bin:/usr/bin:/bin".to_owned()),
            ("LANG".to_owned(), "C.UTF-8".to_owned()),
            ("HOME".to_owned(), "/box".to_owned()),
        ]
    );
}

#[test]
fn the_managed_label_is_set_by_the_policy() {
    let policy = valid_policy();
    assert_eq!(
        policy
            .labels
            .get("counterspell.managed")
            .map(String::as_str),
        Some("true")
    );
}

/// Replaces the `name` ulimit with the given soft and hard value, keeping one entry.
fn set_ulimit(policy: &mut ContainerPolicy, name: &str, value: i64) {
    policy.ulimits.retain(|u| u.name != name);
    policy.ulimits.push(Ulimit {
        name: name.to_owned(),
        soft: value,
        hard: value,
    });
}

/// One row per control: the mutation that weakens it and the single violation it causes.
/// The table is long on purpose, because one row per control is the clearest form.
#[allow(clippy::too_many_lines)]
fn mutations() -> Vec<(&'static str, Mutation, PolicyViolation)> {
    vec![
        (
            "entrypoint replaced",
            |p| p.entrypoint = vec![String::from("python3")],
            PolicyViolation::IdlePidOne,
        ),
        (
            "command replaced",
            |p| p.command = vec![String::from("python3")],
            PolicyViolation::IdlePidOne,
        ),
        (
            "user is root",
            |p| p.user = String::from("0:0"),
            PolicyViolation::NonRootUser,
        ),
        (
            "working dir moved",
            |p| p.working_dir = String::from("/"),
            PolicyViolation::WorkingDirectory,
        ),
        (
            "network enabled",
            |p| p.network = String::from("bridge"),
            PolicyViolation::NetworkEnabled,
        ),
        (
            "root filesystem writable",
            |p| p.read_only_rootfs = false,
            PolicyViolation::WritableRootFs,
        ),
        (
            "scratch mount missing",
            |p| p.tmpfs.retain(|m| m.path != "/box"),
            PolicyViolation::ScratchMount("missing"),
        ),
        (
            "scratch mount allows setuid",
            |p| {
                let scratch = p.tmpfs.iter_mut().find(|m| m.path == "/box").unwrap();
                scratch.options.retain(|o| o != "nosuid");
            },
            PolicyViolation::ScratchMount("nosuid missing"),
        ),
        (
            "scratch mount exec when forbidden",
            |p| {
                let scratch = p.tmpfs.iter_mut().find(|m| m.path == "/box").unwrap();
                scratch.options.retain(|o| o != "noexec");
            },
            PolicyViolation::ScratchMount("noexec missing"),
        ),
        (
            "scratch mount owned by another user",
            |p| {
                let scratch = p.tmpfs.iter_mut().find(|m| m.path == "/box").unwrap();
                scratch.options.retain(|o| o != "uid=65534");
            },
            PolicyViolation::ScratchMount("owner"),
        ),
        (
            "scratch mount too big",
            |p| {
                let scratch = p.tmpfs.iter_mut().find(|m| m.path == "/box").unwrap();
                scratch.size_bytes = 128 * MIB;
            },
            PolicyViolation::ScratchMount("size"),
        ),
        (
            "tmp mount allows exec",
            |p| {
                let tmp = p.tmpfs.iter_mut().find(|m| m.path == "/tmp").unwrap();
                tmp.options.retain(|o| o != "noexec");
            },
            PolicyViolation::TmpMount("noexec missing"),
        ),
        (
            "tmp mount missing",
            |p| p.tmpfs.retain(|m| m.path != "/tmp"),
            PolicyViolation::TmpMount("missing"),
        ),
        (
            "unexpected extra mount",
            |p| {
                p.tmpfs.push(TmpfsMount {
                    path: "/data".to_owned(),
                    size_bytes: 16 * MIB,
                    options: vec!["rw".to_owned()],
                });
            },
            PolicyViolation::UnexpectedMount,
        ),
        (
            "shared memory too big",
            |p| p.shm_size_bytes = 64 * MIB,
            PolicyViolation::ShmSize,
        ),
        (
            "capabilities not all dropped",
            |p| p.cap_drop = Vec::new(),
            PolicyViolation::CapabilitiesNotDropped,
        ),
        (
            "capability added back",
            |p| p.cap_add = vec![String::from("NET_RAW")],
            PolicyViolation::CapabilityAdded,
        ),
        (
            "no-new-privileges removed",
            |p| p.security_opts.clear(),
            PolicyViolation::NoNewPrivilegesMissing,
        ),
        (
            "extra security option",
            |p| p.security_opts.push(String::from("apparmor=unconfined")),
            PolicyViolation::UnexpectedSecurityOption,
        ),
        (
            "seccomp unconfined",
            |p| p.seccomp = SeccompProfile::Unconfined,
            PolicyViolation::SeccompOverridden,
        ),
        (
            "seccomp custom profile",
            |p| p.seccomp = SeccompProfile::Custom(String::from("/tmp/p.json")),
            PolicyViolation::SeccompOverridden,
        ),
        (
            "privileged",
            |p| p.privileged = true,
            PolicyViolation::Privileged,
        ),
        (
            "host socket bound in",
            |p| p.host_resources.push(String::from("/var/run/docker.sock")),
            PolicyViolation::HostResourceMounted,
        ),
        (
            "pid namespace shared",
            |p| p.namespaces.pid = false,
            PolicyViolation::SharedNamespace("pid"),
        ),
        (
            "ipc namespace shared",
            |p| p.namespaces.ipc = false,
            PolicyViolation::SharedNamespace("ipc"),
        ),
        (
            "cgroup namespace shared",
            |p| p.namespaces.cgroup = false,
            PolicyViolation::SharedNamespace("cgroup"),
        ),
        (
            "cpu quota doubled",
            |p| p.limits.nano_cpus = 2_000_000_000,
            PolicyViolation::CpuQuota,
        ),
        (
            "memory limit missing",
            |p| {
                p.limits.memory_bytes = 0;
                p.limits.memory_swap_bytes = 0;
            },
            PolicyViolation::MemoryLimit,
        ),
        (
            "swap allowed",
            |p| p.limits.memory_swap_bytes = 512 * MIB,
            PolicyViolation::SwapAllowed,
        ),
        (
            "pid limit raised",
            |p| p.limits.pids_limit = 128,
            PolicyViolation::PidsLimit,
        ),
        (
            "open files raised",
            |p| set_ulimit(p, "nofile", 1024),
            PolicyViolation::NoFileLimit,
        ),
        (
            "file size raised",
            |p| set_ulimit(p, "fsize", 17 * 1024 * 1024),
            PolicyViolation::FileSizeLimit,
        ),
        (
            "core dumps allowed",
            |p| set_ulimit(p, "core", 1),
            PolicyViolation::CoreDumpsAllowed,
        ),
        (
            "process ulimit set",
            |p| {
                p.ulimits.push(Ulimit {
                    name: "nproc".to_owned(),
                    soft: 64,
                    hard: 64,
                });
            },
            PolicyViolation::ProcLimitSet,
        ),
        (
            "log driver enabled",
            |p| p.log_driver = String::from("json-file"),
            PolicyViolation::LogDriver,
        ),
        (
            "restart policy set",
            |p| p.restart_policy = String::from("always"),
            PolicyViolation::RestartPolicy,
        ),
        (
            "auto remove on",
            |p| p.auto_remove = true,
            PolicyViolation::AutoRemove,
        ),
        (
            "extra environment variable",
            |p| p.env.push(("SECRET".to_owned(), "x".to_owned())),
            PolicyViolation::UnexpectedEnvironment,
        ),
        (
            "managed label removed",
            |p| {
                p.labels.remove("counterspell.managed");
            },
            PolicyViolation::MissingLabel("counterspell.managed"),
        ),
    ]
}

#[test]
fn weakening_exactly_one_control_reports_exactly_that_violation() {
    for (label, mutate, expected) in mutations() {
        let mut policy = valid_policy();
        mutate(&mut policy);
        assert_eq!(
            policy.hardening_violations(),
            vec![expected.clone()],
            "mutation `{label}` should report only {expected:?}"
        );
    }
}

#[test]
fn every_checked_control_has_a_mutation_test() {
    // Guards the table above: a control the checker can report must have a mutation row.
    let covered: Vec<PolicyViolation> = mutations().into_iter().map(|(_, _, v)| v).collect();
    for expected in [
        PolicyViolation::IdlePidOne,
        PolicyViolation::NonRootUser,
        PolicyViolation::NetworkEnabled,
        PolicyViolation::WritableRootFs,
        PolicyViolation::CapabilitiesNotDropped,
        PolicyViolation::NoNewPrivilegesMissing,
        PolicyViolation::SeccompOverridden,
        PolicyViolation::Privileged,
        PolicyViolation::CpuQuota,
        PolicyViolation::MemoryLimit,
        PolicyViolation::SwapAllowed,
        PolicyViolation::PidsLimit,
        PolicyViolation::ProcLimitSet,
        PolicyViolation::LogDriver,
        PolicyViolation::RestartPolicy,
        PolicyViolation::AutoRemove,
        PolicyViolation::UnexpectedEnvironment,
    ] {
        assert!(
            covered.contains(&expected),
            "no mutation covers {expected:?}"
        );
    }
}
