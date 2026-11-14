//! Model tests for one call to the driver (requirements R5.1, R5.2, R5.5, R5.9, R6.2,
//! R6.3, R9.3). They use the scripted fake backend and short real-time limits.

use std::time::Duration;

use counterspell_judge::languages::LanguageCatalog;
use counterspell_judge::sandbox::call::{run_call, CallFacts, CallIo, CallLimits, STDOUT_BUDGET};
use counterspell_judge::sandbox::config::OciRuntime;
use counterspell_judge::sandbox::session::{open_session, Opened};
use counterspell_judge::sandbox::spec::SessionSpec;
use counterspell_judge::sandbox::testkit::{Behaviour, FakeBackend, FakeSession};
use serde_json::{json, Value};
use tempfile::TempDir;
use tokio::sync::watch;
use tokio::time::Instant;
use uuid::Uuid;

mod support;
use support::shipped_catalogue;

const RUN: &str = "00000000-0000-0000-0000-000000000004";
const ARCHIVE: &[u8] = b"fixed archive bytes";
const KILL_SLACK: Duration = Duration::from_millis(500);

fn catalogue() -> (TempDir, LanguageCatalog) {
    shipped_catalogue(false)
}

async fn open(backend: &FakeBackend) -> Opened<FakeSession> {
    let (_dir, catalog) = catalogue();
    let lang = catalog.get("python").unwrap();
    let spec = SessionSpec::new(
        lang,
        Duration::from_secs(20),
        OciRuntime::Runc,
        Uuid::parse_str(RUN).unwrap(),
        "worker-1",
        1,
        1_800_000_000,
    )
    .unwrap();
    open_session(backend, &spec, lang, ARCHIVE).await.unwrap()
}

fn limits(time_ms: u64) -> CallLimits {
    CallLimits {
        time: Duration::from_millis(time_ms),
        run_deadline: Instant::now() + Duration::from_secs(10),
        kill_slack: KILL_SLACK,
    }
}

fn args() -> Vec<Value> {
    vec![json!(1)]
}

#[tokio::test]
async fn an_answer_in_time_is_returned_without_its_newline_and_keeps_the_container() {
    let backend = FakeBackend::new();
    backend.script_program([Behaviour::Answer(b"{\"t\":1,\"out\":2}\n".to_vec())]);
    let mut opened = open(&backend).await;
    let mut io = CallIo::new();
    let (_keep, mut stop) = watch::channel(false);

    let facts = run_call(&mut opened, &mut io, 1, &args(), limits(1_000), &mut stop)
        .await
        .unwrap();

    match facts {
        CallFacts::Answer { line, .. } => assert_eq!(line, b"{\"t\":1,\"out\":2}"),
        other => panic!("expected an answer, got {other:?}"),
    }
    assert_eq!(
        backend.ledger().live(),
        1,
        "a good answer leaves the driver running"
    );
    opened.session.close().await.unwrap();
}

#[tokio::test]
async fn a_silent_program_hits_the_deadline_and_its_container_is_removed() {
    let backend = FakeBackend::new();
    backend.script_program([Behaviour::Hang]);
    let mut opened = open(&backend).await;
    let mut io = CallIo::new();
    let (_keep, mut stop) = watch::channel(false);
    let started = Instant::now();

    let facts = run_call(&mut opened, &mut io, 1, &args(), limits(50), &mut stop)
        .await
        .unwrap();

    assert_eq!(facts, CallFacts::Deadline);
    assert!(
        started.elapsed() < Duration::from_millis(50) + KILL_SLACK,
        "the call must return within its limit plus the kill slack"
    );
    assert_eq!(backend.ledger().live(), 0);
}

#[tokio::test]
async fn a_stdout_flood_is_an_output_limit_and_the_container_is_removed() {
    let backend = FakeBackend::new();
    backend.script_program([Behaviour::Flood(STDOUT_BUDGET + 1)]);
    let mut opened = open(&backend).await;
    let mut io = CallIo::new();
    let (_keep, mut stop) = watch::channel(false);

    let facts = run_call(&mut opened, &mut io, 1, &args(), limits(1_000), &mut stop)
        .await
        .unwrap();

    assert_eq!(facts, CallFacts::OutputLimit);
    assert_eq!(backend.ledger().live(), 0);
}

#[tokio::test]
async fn exactly_the_stdout_budget_without_a_newline_is_not_an_overflow() {
    let backend = FakeBackend::new();
    backend.script_program([Behaviour::Flood(STDOUT_BUDGET)]);
    let mut opened = open(&backend).await;
    let mut io = CallIo::new();
    let (_keep, mut stop) = watch::channel(false);

    let facts = run_call(&mut opened, &mut io, 1, &args(), limits(50), &mut stop)
        .await
        .unwrap();

    // No line completed, so the call waits for an answer and the deadline ends it.
    assert_eq!(facts, CallFacts::Deadline);
}

#[tokio::test]
async fn a_stderr_flood_is_not_an_output_limit_and_the_deadline_still_decides() {
    let backend = FakeBackend::new();
    backend.script_program([Behaviour::Stderr(vec![b'e'; 128 * 1024]), Behaviour::Hang]);
    let mut opened = open(&backend).await;
    let mut io = CallIo::new();
    let (_keep, mut stop) = watch::channel(false);

    let facts = run_call(&mut opened, &mut io, 1, &args(), limits(50), &mut stop)
        .await
        .unwrap();

    assert_eq!(facts, CallFacts::Deadline);
    assert_eq!(backend.ledger().live(), 0);
}

#[tokio::test]
async fn a_stderr_flood_does_not_change_a_later_answer() {
    let backend = FakeBackend::new();
    backend.script_program([
        Behaviour::Stderr(vec![b'e'; 128 * 1024]),
        Behaviour::Answer(b"{\"t\":1,\"out\":2}\n".to_vec()),
    ]);
    let mut opened = open(&backend).await;
    let mut io = CallIo::new();
    let (_keep, mut stop) = watch::channel(false);

    let facts = run_call(&mut opened, &mut io, 1, &args(), limits(1_000), &mut stop)
        .await
        .unwrap();

    assert!(matches!(facts, CallFacts::Answer { .. }));
    assert_eq!(backend.ledger().live(), 1);
    opened.session.close().await.unwrap();
}

#[tokio::test]
async fn an_exit_with_oom_evidence_keeps_the_last_stderr_line_after_truncation() {
    let backend = FakeBackend::new();
    let mut stderr = vec![b'j'; 100 * 1024];
    stderr.extend_from_slice(b"\nKeyError: 'k'\n");
    backend.script_program([
        Behaviour::Stderr(stderr),
        Behaviour::Exit {
            code: 137,
            oom_kill: 1,
        },
    ]);
    let mut opened = open(&backend).await;
    let mut io = CallIo::new();
    let (_keep, mut stop) = watch::channel(false);

    let facts = run_call(&mut opened, &mut io, 1, &args(), limits(1_000), &mut stop)
        .await
        .unwrap();

    assert_eq!(
        facts,
        CallFacts::DriverExited {
            code: 137,
            oom_kill: 1,
            last_stderr_line: Some(b"KeyError: 'k'".to_vec()),
        }
    );
    assert_eq!(backend.ledger().live(), 0);
}

#[tokio::test]
async fn an_exit_without_oom_evidence_reports_no_last_line_when_stderr_is_empty() {
    let backend = FakeBackend::new();
    backend.script_program([Behaviour::Exit {
        code: 1,
        oom_kill: 0,
    }]);
    let mut opened = open(&backend).await;
    let mut io = CallIo::new();
    let (_keep, mut stop) = watch::channel(false);

    let facts = run_call(&mut opened, &mut io, 1, &args(), limits(1_000), &mut stop)
        .await
        .unwrap();

    assert_eq!(
        facts,
        CallFacts::DriverExited {
            code: 1,
            oom_kill: 0,
            last_stderr_line: None,
        }
    );
}

#[tokio::test]
async fn a_stop_signal_ends_the_call_as_stopped_and_removes_the_container() {
    let backend = FakeBackend::new();
    backend.script_program([Behaviour::Hang]);
    let mut opened = open(&backend).await;
    let mut io = CallIo::new();
    let (keep, mut stop) = watch::channel(false);
    keep.send(true).unwrap();

    let facts = run_call(&mut opened, &mut io, 1, &args(), limits(10_000), &mut stop)
        .await
        .unwrap();

    assert_eq!(facts, CallFacts::Stopped);
    assert_eq!(backend.ledger().live(), 0);
}

#[tokio::test]
async fn a_write_the_program_never_reads_is_bounded_by_the_deadline() {
    let backend = FakeBackend::new();
    backend.set_block_writes(true);
    let mut opened = open(&backend).await;
    let mut io = CallIo::new();
    let (_keep, mut stop) = watch::channel(false);

    let facts = run_call(&mut opened, &mut io, 1, &args(), limits(50), &mut stop)
        .await
        .unwrap();

    assert_eq!(facts, CallFacts::Deadline);
    assert_eq!(backend.ledger().live(), 0);
}

#[tokio::test]
async fn each_call_gets_a_fresh_stdout_budget() {
    let backend = FakeBackend::new();
    let mut answer = vec![b'x'; 700 * 1024];
    answer.push(b'\n');
    backend.script_program([Behaviour::Answer(answer.clone()), Behaviour::Answer(answer)]);
    let mut opened = open(&backend).await;
    let mut io = CallIo::new();
    let (_keep, mut stop) = watch::channel(false);

    // Two calls of 700 KiB each would exceed one 1 MiB budget, so the second must not overflow.
    for id in 1..=2 {
        let facts = run_call(&mut opened, &mut io, id, &args(), limits(1_000), &mut stop)
            .await
            .unwrap();
        assert!(
            matches!(facts, CallFacts::Answer { .. }),
            "call {id}: {facts:?}"
        );
    }
    opened.session.close().await.unwrap();
}

#[tokio::test]
async fn a_second_line_read_with_the_first_answers_the_next_call_without_reading_again() {
    let backend = FakeBackend::new();
    backend.script_program([Behaviour::Answer(b"first\nsecond\n".to_vec())]);
    let mut opened = open(&backend).await;
    let mut io = CallIo::new();
    let (_keep, mut stop) = watch::channel(false);

    let first = run_call(&mut opened, &mut io, 1, &args(), limits(1_000), &mut stop)
        .await
        .unwrap();
    let second = run_call(&mut opened, &mut io, 2, &args(), limits(1_000), &mut stop)
        .await
        .unwrap();

    assert!(matches!(&first, CallFacts::Answer { line, .. } if line == b"first"));
    assert!(matches!(second, CallFacts::Answer { ref line, .. } if line == b"second"));
    opened.session.close().await.unwrap();
}
