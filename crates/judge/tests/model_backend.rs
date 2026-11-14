//! Model tests for the backend seam and the scripted fake backend (requirements R7 and
//! R9.4). No Docker is involved: every behaviour here comes from the fake.

use std::time::Duration;

use counterspell_judge::languages::LanguageCatalog;
use counterspell_judge::sandbox::backend::{
    Backend, BackendSession, DriverChannel, DriverEvent, InfraError,
};
use counterspell_judge::sandbox::config::OciRuntime;
use counterspell_judge::sandbox::spec::{ExecSpec, SessionSpec};
use counterspell_judge::sandbox::testkit::{Behaviour, FakeBackend};
use uuid::Uuid;

mod support;
use support::shipped_catalogue;

const RUN: &str = "00000000-0000-0000-0000-000000000002";

fn catalogue() -> (tempfile::TempDir, LanguageCatalog) {
    shipped_catalogue(false)
}

fn session_spec(ordinal: u32) -> SessionSpec {
    let (_dir, catalog) = catalogue();
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

fn driver_exec() -> ExecSpec {
    let (_dir, catalog) = catalogue();
    ExecSpec::driver(catalog.get("python").unwrap())
}

#[tokio::test]
async fn the_ledger_counts_each_created_and_removed_container() {
    let backend = FakeBackend::new();
    let mut first = backend.create(&session_spec(1)).await.unwrap();
    let mut second = backend.create(&session_spec(2)).await.unwrap();

    first.remove().await.unwrap();

    let ledger = backend.ledger();
    assert_eq!(ledger.created(), 2);
    assert_eq!(ledger.removed(), 1);
    assert_eq!(ledger.live(), 1);

    second.remove().await.unwrap();
    assert_eq!(backend.ledger().live(), 0);
}

#[tokio::test]
async fn the_ledger_records_the_container_name_from_the_spec() {
    let backend = FakeBackend::new();
    let spec = session_spec(7);
    let session = backend.create(&spec).await.unwrap();

    assert_eq!(session.name(), spec.name);
    assert_eq!(backend.ledger().created_names(), vec![spec.name]);
}

#[tokio::test]
async fn an_injected_fault_fires_on_exactly_the_chosen_call_and_only_once() {
    let backend = FakeBackend::new();
    // Call 1 is create, call 2 is start.
    backend.fail_call(2, InfraError::DaemonUnreachable("socket gone".to_owned()));

    let mut session = backend.create(&session_spec(1)).await.unwrap();
    let failed = session.start().await;
    assert_eq!(
        failed,
        Err(InfraError::DaemonUnreachable("socket gone".to_owned()))
    );

    // Call 3 is after the fault: it succeeds.
    session.start().await.unwrap();
}

#[tokio::test]
async fn an_injected_fault_on_create_leaves_nothing_in_the_ledger() {
    let backend = FakeBackend::new();
    backend.fail_call(1, InfraError::ImageMissing("python".to_owned()));

    let result = backend.create(&session_spec(1)).await;

    assert!(matches!(result, Err(InfraError::ImageMissing(_))));
    assert_eq!(backend.ledger().created(), 0);
}

#[tokio::test]
async fn a_scripted_answer_comes_back_as_one_line_and_the_input_is_recorded() {
    let backend = FakeBackend::new();
    backend.script_program([Behaviour::Answer(b"{\"t\":1,\"out\":2}\n".to_vec())]);
    let mut session = backend.create(&session_spec(1)).await.unwrap();
    let mut driver = session.open_driver(&driver_exec()).await.unwrap();

    driver
        .write_line(b"{\"t\":1,\"args\":[1]}\n")
        .await
        .unwrap();

    assert_eq!(
        driver.next_event().await.unwrap(),
        DriverEvent::Line(b"{\"t\":1,\"out\":2}\n".to_vec())
    );
    assert_eq!(
        backend.ledger().inputs(),
        vec![b"{\"t\":1,\"args\":[1]}\n".to_vec()]
    );
}

#[tokio::test]
async fn a_scripted_flood_arrives_as_one_chunk_of_the_scripted_size() {
    let backend = FakeBackend::new();
    backend.script_program([Behaviour::Flood(70_000)]);
    let mut session = backend.create(&session_spec(1)).await.unwrap();
    let mut driver = session.open_driver(&driver_exec()).await.unwrap();

    match driver.next_event().await.unwrap() {
        DriverEvent::Chunk(bytes) => assert_eq!(bytes.len(), 70_000),
        other => panic!("expected a chunk, got {other:?}"),
    }
}

#[tokio::test]
async fn a_scripted_exit_reports_its_code_and_oom_evidence() {
    let backend = FakeBackend::new();
    backend.script_program([Behaviour::Exit {
        code: 137,
        oom_kill: 1,
    }]);
    let mut session = backend.create(&session_spec(1)).await.unwrap();
    let mut driver = session.open_driver(&driver_exec()).await.unwrap();

    assert_eq!(
        driver.next_event().await.unwrap(),
        DriverEvent::Exited {
            code: 137,
            oom_kill: 1
        }
    );
}

#[tokio::test]
async fn a_scripted_hang_never_answers_so_the_caller_must_bring_its_own_deadline() {
    let backend = FakeBackend::new();
    backend.script_program([Behaviour::Hang]);
    let mut session = backend.create(&session_spec(1)).await.unwrap();
    let mut driver = session.open_driver(&driver_exec()).await.unwrap();

    let waited = tokio::time::timeout(Duration::from_millis(50), driver.next_event()).await;

    assert!(
        waited.is_err(),
        "a hanging program must not produce an event"
    );
}

#[tokio::test]
async fn kill_and_remove_are_both_recorded_against_the_container() {
    let backend = FakeBackend::new();
    let mut session = backend.create(&session_spec(1)).await.unwrap();

    session.kill().await.unwrap();
    session.remove().await.unwrap();

    let ledger = backend.ledger();
    assert_eq!(ledger.killed(), 1);
    assert_eq!(ledger.removed(), 1);
    assert_eq!(ledger.live(), 0);
}

#[tokio::test]
async fn a_scripted_exec_returns_the_configured_exit_code() {
    let backend = FakeBackend::new();
    backend.set_exec_exit(3);
    let mut session = backend.create(&session_spec(1)).await.unwrap();

    let output = session.exec(&driver_exec(), b"").await.unwrap();

    assert_eq!(output.exit_code, 3);
}
