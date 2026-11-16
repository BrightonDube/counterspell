//! Model tests for the preflight evaluation (judge-sandbox T22, R4.1 to R4.6).
//!
//! The probe outputs here are synthetic and written from the policy, so the tests need no
//! daemon. They show that each control in the probe is read and compared. The real probe
//! output from the rootless daemon is checked by the `sbx_` tests (T22, T33) and recorded
//! in `deploy/sandbox-verification.md`.
#![cfg(feature = "sandbox")]

use std::sync::Arc;
use std::time::Duration;

use counterspell_judge::sandbox::backend::{ExecOutput, InfraError};
use counterspell_judge::sandbox::config::OciRuntime;
use counterspell_judge::sandbox::policy::ContainerPolicy;
use counterspell_judge::sandbox::preflight::{
    evaluate, parse_size, probe_outputs, probe_spec, refresh, run as refresh_loop, run_probe,
    PreflightState, ProbeOutputs, PROBE_COMMANDS,
};
use counterspell_judge::sandbox::spec::SessionSpec;
use counterspell_judge::sandbox::testkit::FakeBackend;

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

// --- The probe path and `PreflightState` (T22) ---

/// The session spec a probe container is built from, for fake-backend tests.
fn spec() -> SessionSpec {
    SessionSpec {
        name: "model-probe".to_owned(),
        policy: policy(),
    }
}

/// One bounded `cat`/`touch` result, in the order `probe_outputs` expects.
fn exec_output(exit_code: i64, stdout: String) -> ExecOutput {
    ExecOutput {
        exit_code,
        stdout: stdout.into_bytes(),
        stderr: Vec::new(),
        overflowed: false,
    }
}

/// The seven text outputs plus the touch result of a good probe, as exec results.
fn good_execs(policy: &ContainerPolicy) -> Vec<ExecOutput> {
    let outputs = good(policy);
    vec![
        exec_output(0, outputs.proc_status),
        exec_output(0, outputs.net_dev),
        exec_output(0, outputs.memory_max),
        exec_output(0, outputs.memory_swap_max),
        exec_output(0, outputs.pids_max),
        exec_output(0, outputs.cpu_max),
        exec_output(0, outputs.mounts),
        exec_output(outputs.touch_exit, String::new()),
    ]
}

#[test]
fn exec_results_fold_in_probe_command_order() {
    let policy = policy();
    let expected = good(&policy);
    let folded = probe_outputs(good_execs(&policy)).expect("eight results fold");
    assert_eq!(folded, expected);
    assert_eq!(folded.proc_status, expected.proc_status);
}

#[test]
fn an_unreadable_probe_file_fails_its_own_check() {
    let policy = policy();
    let mut results = good_execs(&policy);
    results[2] = exec_output(1, String::new()); // cat memory.max failed
    let outputs = probe_outputs(results).expect("eight results fold");
    assert_eq!(failing_names(&outputs, &policy), vec!["memory_limit"]);
}

#[test]
fn a_wrong_result_count_is_an_infrastructure_error() {
    let policy = policy();
    let results = good_execs(&policy)[..PROBE_COMMANDS.len() - 1].to_vec();
    assert!(matches!(
        probe_outputs(results),
        Err(InfraError::Unexpected(_))
    ));
}

#[test]
fn the_report_serializes_as_json() {
    let policy = policy();
    let report = evaluate(&good(&policy), &policy);
    let text = serde_json::to_string(&report).expect("the report serializes");
    assert!(text.contains("\"no_new_privs\""), "{text}");
    assert!(text.contains("\"ok\":true"), "{text}");
}

#[test]
fn probe_spec_builds_a_hardened_managed_container() {
    let (_dir, catalog) = shipped_catalogue(false);
    let python = catalog.get("python").expect("python is in the fixture");
    let spec = probe_spec(python, Duration::from_secs(20), OciRuntime::Runc)
        .expect("the probe spec builds");
    assert!(spec.name.starts_with("cs-sbx-"), "{}", spec.name);
    assert_eq!(spec.policy.hardening_violations(), []);
    assert_eq!(
        spec.policy
            .labels
            .get("counterspell.worker")
            .map(String::as_str),
        Some("preflight")
    );
}

#[test]
fn no_report_counts_as_failing() {
    let state = PreflightState::new();
    assert!(!state.passed());
    assert!(state.latest().is_none());
}

#[tokio::test]
async fn refresh_stores_a_passing_probe() {
    let spec = spec();
    let backend = FakeBackend::new();
    backend.script_exec_stdout(good_execs(&spec.policy).into_iter().map(|out| out.stdout));
    backend.script_exec_exits([0, 0, 0, 0, 0, 0, 0, 1]);
    let state = PreflightState::new();
    refresh(&state, &backend, &spec)
        .await
        .expect("the refresh ran");
    assert!(state.passed(), "failures: {:?}", state.latest());
    assert_eq!(state.latest().expect("a report exists").checks.len(), 12);
}

#[tokio::test]
async fn a_probe_error_stores_a_failing_report() {
    let backend = FakeBackend::new();
    backend.fail_call(1, InfraError::DaemonUnreachable("down".to_owned()));
    let state = PreflightState::new();
    let result = refresh(&state, &backend, &spec()).await;
    assert!(result.is_err());
    assert!(!state.passed());
    let report = state.latest().expect("a report exists");
    assert_eq!(
        report
            .failures()
            .iter()
            .map(|check| check.name)
            .collect::<Vec<_>>(),
        vec!["sandbox_probe"]
    );
}

#[tokio::test]
async fn a_probe_failure_still_removes_the_container() {
    let backend = FakeBackend::new();
    // Call 3 is the first probe exec; the run_probe path must still kill and remove.
    backend.fail_call(3, InfraError::DaemonUnreachable("gone".to_owned()));
    let result = run_probe(&backend, &spec()).await;
    assert!(result.is_err());
    let ledger = backend.ledger();
    assert_eq!(ledger.created(), 1);
    assert_eq!(ledger.removed(), 1, "the probe container is removed");
}

#[tokio::test]
async fn the_refresh_loop_repeats_until_stopped() {
    let backend = FakeBackend::new();
    let state = Arc::new(PreflightState::new());
    let spec = spec();
    let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
    let loop_backend = backend.clone();
    let loop_state = state.clone();
    let task = tokio::spawn(refresh_loop(
        loop_state,
        loop_backend,
        move || Ok(spec.clone()),
        Duration::from_millis(10),
        stop_rx,
    ));
    tokio::time::sleep(Duration::from_millis(55)).await;
    stop_tx.send(true).expect("the stop signal sends");
    task.await.expect("the loop exits on stop");
    assert!(
        backend.ledger().created() >= 2,
        "the loop refreshed more than once"
    );
    assert!(state.latest().is_some(), "every round stores a report");
}
