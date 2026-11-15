//! Model tests for the preflight evaluation (judge-sandbox T22, R4.1 to R4.6).
//!
//! The probe outputs here are synthetic and written from the policy, so the tests need no
//! daemon. They show that each control in the probe is read and compared. The real probe
//! output from the rootless daemon is checked by the `sbx_` tests (T22, T33) and recorded
//! in `deploy/sandbox-verification.md`.
#![cfg(feature = "sandbox")]

use std::time::Duration;

use counterspell_judge::sandbox::config::OciRuntime;
use counterspell_judge::sandbox::policy::ContainerPolicy;
use counterspell_judge::sandbox::preflight::{evaluate, parse_size, ProbeOutputs};

mod support;
use support::shipped_catalogue;

fn policy() -> ContainerPolicy {
    let (_dir, catalog) = shipped_catalogue(false);
    let python = catalog.get("python").expect("python is in the fixture");
    ContainerPolicy::for_language(python, Duration::from_secs(20), OciRuntime::Runc)
}

/// What a container built from `policy` would report, if every control held.
fn good(policy: &ContainerPolicy) -> ProbeOutputs {
    let box_size = policy
        .tmpfs
        .iter()
        .find(|mount| mount.path == "/box")
        .map_or(0, |mount| mount.size_bytes);
    let quota = policy.limits.nano_cpus * 100_000 / 1_000_000_000;
    ProbeOutputs {
        proc_status:
            "Name:\tsleep\nUid:\t65534\t65534\t65534\t65534\nNoNewPrivs:\t1\nSeccomp:\t2\n\
            CapInh:\t0000000000000000\nCapPrm:\t0000000000000000\nCapEff:\t0000000000000000\n\
            CapBnd:\t0000000000000000\nCapAmb:\t0000000000000000\n"
                .to_owned(),
        net_dev: "Inter-|   Receive\n face |bytes packets\n    lo:       0       0\n".to_owned(),
        memory_max: format!("{}\n", policy.limits.memory_bytes),
        memory_swap_max: "0\n".to_owned(),
        pids_max: format!("{}\n", policy.limits.pids_limit),
        cpu_max: format!("{quota} 100000\n"),
        mounts: format!(
            "overlay / overlay ro,relatime 0 0\n\
             tmpfs /box tmpfs rw,nosuid,nodev,size={box_size} 0 0\n\
             tmpfs /tmp tmpfs rw,nosuid,nodev,noexec,size=16777216 0 0\n"
        ),
        touch_exit: 1,
    }
}

/// One named weakening of a good probe.
type Weaken = fn(&mut ProbeOutputs);

fn failing_names(outputs: &ProbeOutputs, policy: &ContainerPolicy) -> Vec<&'static str> {
    evaluate(outputs, policy)
        .checks
        .iter()
        .filter(|check| !check.ok)
        .map(|check| check.name)
        .collect()
}

#[test]
fn a_probe_that_shows_every_control_passes() {
    let policy = policy();
    let report = evaluate(&good(&policy), &policy);
    assert!(report.passed(), "failed: {:?}", report.failures());
    assert_eq!(report.checks.len(), 12, "one entry per control");
}

#[test]
fn each_weakened_control_fails_exactly_its_own_check() {
    let policy = policy();
    let cases: Vec<(&str, Weaken)> = vec![
        ("no_new_privs", |o| {
            o.proc_status = o.proc_status.replace("NoNewPrivs:\t1", "NoNewPrivs:\t0");
        }),
        ("seccomp_filter", |o| {
            o.proc_status = o.proc_status.replace("Seccomp:\t2", "Seccomp:\t0");
        }),
        ("capabilities_empty", |o| {
            o.proc_status = o
                .proc_status
                .replace("CapEff:\t0000000000000000", "CapEff:\t0000000000000001");
        }),
        ("sandbox_uid", |o| {
            o.proc_status = o.proc_status.replace("Uid:\t65534", "Uid:\t0");
        }),
        ("network_interfaces", |o| {
            o.net_dev.push_str("  eth0:       0       0\n");
        }),
        ("memory_limit", |o| o.memory_max = "max\n".to_owned()),
        ("swap_disabled", |o| o.memory_swap_max = "max\n".to_owned()),
        ("pids_limit", |o| o.pids_max = "max\n".to_owned()),
        ("cpu_quota", |o| o.cpu_max = "max 100000\n".to_owned()),
        ("root_read_only", |o| {
            o.mounts = o.mounts.replace("ro,relatime", "rw,relatime");
        }),
        ("scratch_tmpfs", |o| {
            o.mounts = o.mounts.replace("tmpfs /box tmpfs", "ext4 /box ext4");
        }),
        ("read_only_write_refused", |o| o.touch_exit = 0),
    ];
    for (expected, weaken) in cases {
        let mut outputs = good(&policy);
        weaken(&mut outputs);
        assert_eq!(
            failing_names(&outputs, &policy),
            vec![expected],
            "weakening {expected} must fail only {expected}"
        );
    }
}

#[test]
fn a_probe_checked_against_a_different_policy_fails() {
    // A probe from a container that was not built from this policy (no memory limit at all)
    // must not pass for the production policy.
    let production = policy();
    let mut weaker = production.clone();
    weaker.limits.memory_bytes = 512 * 1024 * 1024;
    let outputs = good(&weaker);
    assert_eq!(failing_names(&outputs, &production), vec!["memory_limit"]);
}

#[test]
fn sizes_are_read_with_their_unit_suffix() {
    assert_eq!(parse_size("16777216"), Some(16_777_216));
    assert_eq!(parse_size("64k"), Some(65_536));
    assert_eq!(parse_size("64M"), Some(64 * 1024 * 1024));
    assert_eq!(parse_size("1g"), Some(1024 * 1024 * 1024));
    assert_eq!(parse_size("many"), None);
    assert_eq!(parse_size(""), None);
}

#[test]
fn a_size_too_large_for_u64_is_not_read() {
    assert_eq!(parse_size("99999999999999999999g"), None);
}
