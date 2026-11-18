// SPDX-License-Identifier: MIT
//! Network scenario S12 (judge-sandbox T29, R3.1, R16.4).
//!
//! The pass condition is out-of-band: host listeners on the loopback record
//! every TCP/UDP peer, and a hostile run must leave that count untouched —
//! what the program says about its own attempts proves nothing. A trusted
//! control first shows a network-enabled container *can* reach the listener
//! on this host; if it cannot, the test reports "control inconclusive".
//! Ignored by default.
//!
//! `cargo test -p counterspell-judge --features sandbox --test sbx_network -- --ignored`
#![cfg(feature = "sandbox")]

mod sbx;

use std::time::Duration;

use counterspell_common::status::{RunMode, Verdict};
use counterspell_judge::executor::RunOutcome;
use counterspell_judge::sandbox::bollard_backend::{
    create_started_unchecked, kill_container, remove_forced, run_exec,
};
use counterspell_judge::sandbox::preflight::probe_exec;
use counterspell_judge::sandbox::testkit::{weakened_policy, WeakenedControl};
use uuid::Uuid;

/// The hostile corpus program with its `__PORT__` and `__HOST__` templates
/// filled for this run's listener port and host address.
fn hostile_with_target(name: &str, port: u16, host: std::net::IpAddr) -> String {
    sbx::hostile(name)
        .replace("__PORT__", &port.to_string())
        .replace("__HOST__", &host.to_string())
}

/// The run's verdict, or a panic describing the failure instead.
fn verdict(observation: &sbx::RunObservation) -> Verdict {
    let outcome = observation
        .outcome
        .as_ref()
        .expect("the run produces an outcome");
    let RunOutcome::Completed(result) = outcome else {
        panic!("a network attempt still gets a completed run: {outcome:?}");
    };
    result.verdict
}

/// S12 — the control: a container with networking deliberately enabled must be
/// able to reach the host listeners, proving the listeners are observable. If
/// it cannot, the hostile results below mean nothing.
#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET"]
async fn sbx_s12_no_connection_reaches_the_host() {
    let (docker, cfg) = sbx::sandbox().await;
    let catalog = sbx::catalogue(&cfg);
    let listeners = sbx::listeners().await;
    let host = sbx::host_ip();
    eprintln!("S12 host address candidates include {host}");

    // Control: a testkit-only weakened policy gives the container a network.
    let python = catalog.get("python").expect("python is in the catalogue");
    let weakened = weakened_policy(
        python,
        cfg.run_wall_cap,
        cfg.runtime,
        &[WeakenedControl::Network],
    );
    let name = format!("counterspell-sbx-netctl-{}", Uuid::new_v4());
    let id = create_started_unchecked(&docker, &weakened, &name, cfg.api_timeout)
        .await
        .expect("the control container creates and starts");
    let control = std::fs::read_to_string(sbx::probes_dir().join("net_control.py"))
        .expect("the control script reads");
    let report = run_exec(
        &docker,
        &id,
        &probe_exec(&[
            "python3",
            "-",
            &listeners.tcp_port.to_string(),
            &listeners.udp_port.to_string(),
            &host.to_string(),
        ]),
        control.as_bytes(),
        cfg.api_timeout,
    )
    .await
    .expect("the control script runs");
    let _ = kill_container(&docker, &id, cfg.api_timeout).await;
    let _ = remove_forced(&docker, &id, cfg.api_timeout).await;

    let stdout = String::from_utf8_lossy(&report.stdout);
    let control_reached = listeners.attempts().len();
    eprintln!("S12 control: {stdout}; listener attempts so far: {control_reached}");
    if control_reached == 0 && stdout.contains("UNREACHABLE") {
        eprintln!(
            "S12 control inconclusive: a networked container could not reach the \
             host listener on this host, so 'zero connections' cannot be read as \
             isolation evidence here"
        );
        return;
    }
    assert!(
        control_reached > 0 || stdout.contains("REACHABLE"),
        "the control must prove the listener is observable: {stdout}"
    );

    // Hostile attempts, each templated with the port relevant to its channel.
    let baseline = listeners.attempts().len();
    for (name, port) in [
        ("net_tcp.py", listeners.tcp_port),
        ("net_udp.py", listeners.udp_port),
        ("net_dns.py", listeners.udp_port),
        ("net_ipv6.py", listeners.tcp_port),
        ("net_raw.py", listeners.tcp_port),
        ("net_metadata.py", listeners.tcp_port),
    ] {
        let observation = sbx::run_source(
            &docker,
            &cfg,
            &catalog,
            "python",
            "add",
            &hostile_with_target(name, port, host),
            RunMode::Submit,
        )
        .await;
        assert_ne!(
            verdict(&observation),
            Verdict::Accepted,
            "{name} must never be accepted"
        );
        sbx::drain_dead(&docker, &cfg, Duration::from_secs(20)).await;
    }

    assert_eq!(
        listeners.attempts().len(),
        baseline,
        "no connection may leave a sandboxed container: {:?}",
        listeners.attempts()
    );
}
