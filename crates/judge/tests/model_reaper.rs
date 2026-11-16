// SPDX-License-Identifier: MIT
//! Model tests for the sweeper (judge-sandbox T23, R10.3 and R10.4).
//!
//! The fake backend scripts container ages, so no daemon and no waiting is needed. The
//! real daemon check is `sbx_sweeper.rs`.

use std::time::Duration;

use counterspell_judge::sandbox::backend::{Backend, InfraError};
use counterspell_judge::sandbox::config::OciRuntime;
use counterspell_judge::sandbox::policy::ContainerPolicy;
use counterspell_judge::sandbox::reaper::{run_sweeper, sweep_once};
use counterspell_judge::sandbox::spec::SessionSpec;
use counterspell_judge::sandbox::testkit::FakeBackend;

mod support;
use support::shipped_catalogue;

const ORPHAN_AGE: Duration = Duration::from_secs(40);

fn policy() -> ContainerPolicy {
    let (_dir, catalog) = shipped_catalogue(false);
    let python = catalog.get("python").expect("python is in the fixture");
    ContainerPolicy::for_language(python, Duration::from_secs(20), OciRuntime::Runc)
}

/// A spec for a managed container (carries `counterspell.managed=true`).
fn managed_spec(name: &str) -> SessionSpec {
    SessionSpec {
        name: name.to_owned(),
        policy: policy(),
    }
}

/// A spec for a container that does not carry the managed label, like any unrelated
/// container on the host. The sweeper must never see it.
fn unlabelled_spec(name: &str) -> SessionSpec {
    let mut policy = policy();
    policy.labels.remove("counterspell.managed");
    SessionSpec {
        name: name.to_owned(),
        policy,
    }
}

#[tokio::test]
async fn an_old_managed_container_is_removed() {
    let backend = FakeBackend::new();
    backend.set_clock(1_000);
    let _session = backend.create(&managed_spec("old")).await.expect("create");
    let outcome = sweep_once(&backend, ORPHAN_AGE, 1_041)
        .await
        .expect("sweep");
    assert_eq!(outcome.listed, 1);
    assert_eq!(outcome.removed, 1);
    assert_eq!(outcome.failed, 0);
    assert_eq!(backend.ledger().live(), 0, "the orphan is gone");
}

#[tokio::test]
async fn a_young_container_is_kept() {
    let backend = FakeBackend::new();
    backend.set_clock(1_000);
    let _session = backend
        .create(&managed_spec("young"))
        .await
        .expect("create");
    // Exactly the orphan age old: only a container strictly older is removed.
    let outcome = sweep_once(&backend, ORPHAN_AGE, 1_040)
        .await
        .expect("sweep");
    assert_eq!(outcome.listed, 1);
    assert_eq!(outcome.removed, 0, "exactly orphan-age old is kept");
    assert_eq!(backend.ledger().live(), 1);
}

#[tokio::test]
async fn an_unlabelled_container_is_never_listed_or_removed() {
    let backend = FakeBackend::new();
    backend.set_clock(0); // ancient, so age can never be the reason it survives
    let _session = backend
        .create(&unlabelled_spec("stray"))
        .await
        .expect("create");
    let outcome = sweep_once(&backend, ORPHAN_AGE, i64::MAX)
        .await
        .expect("sweep");
    assert_eq!(outcome.listed, 0);
    assert_eq!(outcome.removed, 0);
    assert_eq!(backend.ledger().live(), 1, "the stray still exists");
}

#[tokio::test]
async fn a_removal_error_is_counted_and_the_sweep_continues() {
    let backend = FakeBackend::new();
    backend.set_clock(0);
    let _one = backend.create(&managed_spec("one")).await.expect("create"); // call 1
    let _two = backend.create(&managed_spec("two")).await.expect("create"); // call 2
                                                                            // Call 3 is the list, call 4 the removal of "one", call 5 the removal of "two".
    backend.fail_call(4, InfraError::KillFailed("stuck".to_owned()));
    let outcome = sweep_once(&backend, ORPHAN_AGE, i64::MAX)
        .await
        .expect("sweep");
    assert_eq!(outcome.listed, 2);
    assert_eq!(outcome.failed, 1);
    assert_eq!(outcome.removed, 1, "the next orphan is still removed");
    assert_eq!(backend.fault_fired_on(), Some("remove_managed"));
    assert_eq!(backend.ledger().live(), 1);
}

#[tokio::test]
async fn a_list_error_is_returned_and_nothing_is_removed() {
    let backend = FakeBackend::new();
    backend.fail_call(1, InfraError::DaemonUnreachable("down".to_owned()));
    let result = sweep_once(&backend, ORPHAN_AGE, i64::MAX).await;
    assert!(matches!(result, Err(InfraError::DaemonUnreachable(_))));
    assert_eq!(backend.fault_fired_on(), Some("list_managed"));
}

#[tokio::test]
async fn the_sweeper_runs_until_stopped() {
    let backend = FakeBackend::new();
    backend.set_clock(0); // the orphan is ancient against the real clock
    let _session = backend
        .create(&managed_spec("orphan"))
        .await
        .expect("create");
    let (stop_tx, stop_rx) = tokio::sync::watch::channel(false);
    let loop_backend = backend.clone();
    let task = tokio::spawn(run_sweeper(
        loop_backend,
        Duration::from_millis(10),
        ORPHAN_AGE,
        stop_rx,
    ));
    tokio::time::sleep(Duration::from_millis(55)).await;
    stop_tx.send(true).expect("the stop signal sends");
    task.await.expect("the loop exits on stop");
    assert_eq!(backend.ledger().live(), 0, "the orphan was swept");
}
