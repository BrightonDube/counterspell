//! Model tests for the session lifecycle and its cleanup guard (requirements R3.15, R7.1,
//! R7.2, R7.6, R10.1). They run against the scripted fake backend, so no Docker is used.

use std::future::pending;
use std::time::Duration;

use counterspell_judge::languages::LanguageCatalog;
use counterspell_judge::sandbox::backend::InfraError;
use counterspell_judge::sandbox::config::OciRuntime;
use counterspell_judge::sandbox::session::{open_session, CompileFailure, SessionError};
use counterspell_judge::sandbox::spec::SessionSpec;
use counterspell_judge::sandbox::testkit::FakeBackend;
use tempfile::TempDir;
use uuid::Uuid;

mod support;
use support::shipped_catalogue;

const RUN: &str = "00000000-0000-0000-0000-000000000003";
const ARCHIVE: &[u8] = b"not really a tar, the fake never reads it";

fn catalogue() -> (TempDir, LanguageCatalog) {
    shipped_catalogue(false)
}

fn session_spec(catalog: &LanguageCatalog, ordinal: u32) -> SessionSpec {
    SessionSpec::new(
        catalog.get("python").unwrap(),
        Duration::from_secs(20),
        OciRuntime::Runc,
        Uuid::parse_str(RUN).unwrap(),
        "worker-1",
        ordinal,
        1_800_000_000,
    )
    .unwrap()
}

/// Waits until the fake's ledger shows no live container. Cleanup from `Drop` runs on a
/// spawned task, so the test polls for it rather than assuming an order.
async fn wait_for_no_live_containers(backend: &FakeBackend) {
    for _ in 0..400 {
        if backend.ledger().live() == 0 {
            return;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    panic!(
        "a container was still live after cleanup: {:?}",
        backend.ledger()
    );
}

async fn wait_for_created(backend: &FakeBackend, count: usize) {
    for _ in 0..400 {
        if backend.ledger().created() >= count {
            return;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    panic!("the expected container was never created");
}

#[tokio::test]
async fn a_good_session_opens_a_driver_and_keeps_its_container_until_closed() {
    let (_dir, catalog) = catalogue();
    let backend = FakeBackend::new();

    let opened = open_session(
        &backend,
        &session_spec(&catalog, 1),
        catalog.get("python").unwrap(),
        ARCHIVE,
    )
    .await
    .unwrap();

    assert_eq!(
        backend.ledger().live(),
        1,
        "the container stays up while the run uses it"
    );
    opened.session.close().await.unwrap();
    assert_eq!(backend.ledger().live(), 0);
    assert_eq!(backend.ledger().removed(), 1);
}

#[tokio::test]
async fn a_compile_that_exits_non_zero_is_a_compile_failure_and_the_container_is_removed() {
    let (_dir, catalog) = catalogue();
    let backend = FakeBackend::new();
    // The first exec is the file load (exit 0). The second is the compile (exit 1).
    backend.script_exec_exits([0, 1]);

    let result = open_session(
        &backend,
        &session_spec(&catalog, 1),
        catalog.get("python").unwrap(),
        ARCHIVE,
    )
    .await;

    assert!(matches!(
        result,
        Err(SessionError::CompileFailed(CompileFailure::NonZeroExit(1)))
    ));
    wait_for_no_live_containers(&backend).await;
    assert_eq!(backend.ledger().removed(), 1);
}

#[tokio::test]
async fn a_read_back_that_weakens_the_policy_removes_the_container() {
    let (_dir, catalog) = catalogue();
    let backend = FakeBackend::new();
    let spec = session_spec(&catalog, 1);
    let mut weakened = spec.policy.clone();
    weakened.network = "bridge".to_owned();
    backend.set_readback(weakened);

    let result = open_session(&backend, &spec, catalog.get("python").unwrap(), ARCHIVE).await;

    assert!(matches!(
        result,
        Err(SessionError::Infrastructure(
            InfraError::PolicyReadbackMismatch(_)
        ))
    ));
    assert_eq!(backend.ledger().removed(), 1);
    assert_eq!(backend.ledger().live(), 0);
}

#[tokio::test]
async fn a_failed_file_load_removes_the_container() {
    let (_dir, catalog) = catalogue();
    let backend = FakeBackend::new();
    backend.script_exec_exits([2]);

    let result = open_session(
        &backend,
        &session_spec(&catalog, 1),
        catalog.get("python").unwrap(),
        ARCHIVE,
    )
    .await;

    assert!(matches!(
        result,
        Err(SessionError::Infrastructure(InfraError::ExecSetup(_)))
    ));
    assert_eq!(backend.ledger().live(), 0);
}

#[tokio::test]
async fn a_fault_while_starting_the_driver_removes_the_container() {
    let (_dir, catalog) = catalogue();
    let backend = FakeBackend::new();
    // Calls: 1 create, 2 read-back, 3 start, 4 load, 5 compile, 6 open driver.
    backend.fail_call(
        6,
        InfraError::ExecSetup("driver would not start".to_owned()),
    );

    let result = open_session(
        &backend,
        &session_spec(&catalog, 1),
        catalog.get("python").unwrap(),
        ARCHIVE,
    )
    .await;

    assert!(matches!(result, Err(SessionError::Infrastructure(_))));
    assert_eq!(backend.ledger().live(), 0);
}

#[tokio::test]
async fn an_early_return_with_question_mark_still_removes_the_container() {
    let (_dir, catalog) = catalogue();
    let backend = FakeBackend::new();
    let spec = session_spec(&catalog, 1);
    let lang = catalog.get("python").unwrap();

    let caller = async {
        let _opened = open_session(&backend, &spec, lang, ARCHIVE).await?;
        // Pretend a later step failed and returned early.
        Err::<(), SessionError>(SessionError::Infrastructure(InfraError::Unexpected(
            "later step".to_owned(),
        )))
    };
    assert!(caller.await.is_err());

    wait_for_no_live_containers(&backend).await;
    assert_eq!(backend.ledger().removed(), 1);
}

#[tokio::test]
async fn an_aborted_task_holding_a_session_still_removes_the_container() {
    let (_dir, catalog) = catalogue();
    let backend = FakeBackend::new();
    let spec = session_spec(&catalog, 1);
    let lang = catalog.get("python").unwrap().clone();
    let task_backend = backend.clone();

    let task = tokio::spawn(async move {
        let _opened = open_session(&task_backend, &spec, &lang, ARCHIVE)
            .await
            .unwrap();
        pending::<()>().await;
    });
    wait_for_created(&backend, 1).await;
    task.abort();

    assert!(task.await.unwrap_err().is_cancelled());
    wait_for_no_live_containers(&backend).await;
    assert_eq!(backend.ledger().removed(), 1);
}

#[tokio::test]
async fn a_panic_while_holding_a_session_still_removes_the_container() {
    let (_dir, catalog) = catalogue();
    let backend = FakeBackend::new();
    let spec = session_spec(&catalog, 1);
    let lang = catalog.get("python").unwrap().clone();
    let task_backend = backend.clone();

    let task = tokio::spawn(async move {
        let _opened = open_session(&task_backend, &spec, &lang, ARCHIVE)
            .await
            .unwrap();
        panic!("scripted panic while the session is open");
    });

    assert!(task.await.unwrap_err().is_panic());
    wait_for_no_live_containers(&backend).await;
    assert_eq!(backend.ledger().removed(), 1);
}
