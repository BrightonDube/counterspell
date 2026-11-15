//! Model tests for the pure helpers behind the bollard session (judge-sandbox T21, R5.2,
//! R7.1, R7.2, R9.3, R9.4, R10.1).
//!
//! No daemon is involved. The collector enforces the 64 KiB exec cap, the parser reads the
//! kernel's OOM counter, and the error mapper decides which daemon failures are
//! infrastructure errors. Each daemon failure must stay an `InfraError`, never a verdict.
#![cfg(feature = "sandbox")]

use bollard::errors::Error as DaemonError;
use counterspell_judge::sandbox::backend::InfraError;
use counterspell_judge::sandbox::bollard_backend::{
    map_error, parse_oom_kill, OutputCollector, OutputStream,
};

const CAP: usize = 64 * 1024;

#[test]
fn a_write_that_ends_exactly_at_the_cap_is_kept() {
    let mut collector = OutputCollector::new(CAP);
    assert!(collector.push(OutputStream::Stdout, &vec![b'a'; CAP]));
    let output = collector.into_output(0);
    assert!(!output.overflowed);
    assert_eq!(output.stdout.len(), CAP);
    assert_eq!(output.stderr, b"" as &[u8]);
}

#[test]
fn one_byte_over_the_cap_overflows_and_keeps_only_the_cap() {
    let mut collector = OutputCollector::new(CAP);
    assert!(collector.push(OutputStream::Stdout, &vec![b'a'; CAP - 1]));
    assert!(
        !collector.push(OutputStream::Stdout, b"bc"),
        "the second write crosses the cap"
    );
    let output = collector.into_output(0);
    assert!(output.overflowed);
    assert_eq!(
        output.stdout.len(),
        CAP,
        "bytes past the cap are never stored"
    );
    assert_eq!(output.stdout.last(), Some(&b'b'));
}

#[test]
fn stdout_and_stderr_share_one_cap() {
    let mut collector = OutputCollector::new(10);
    assert!(collector.push(OutputStream::Stdout, b"12345"));
    assert!(collector.push(OutputStream::Stderr, b"67890"));
    assert!(!collector.push(OutputStream::Stderr, b"x"));
    let output = collector.into_output(7);
    assert!(output.overflowed);
    assert_eq!(output.stdout, b"12345");
    assert_eq!(output.stderr, b"67890");
    assert_eq!(output.exit_code, 7);
}

#[test]
fn nothing_is_stored_after_an_overflow() {
    let mut collector = OutputCollector::new(4);
    assert!(!collector.push(OutputStream::Stdout, b"123456"));
    assert!(!collector.push(OutputStream::Stdout, b"more"));
    let output = collector.into_output(0);
    assert_eq!(output.stdout, b"1234");
}

#[test]
fn the_oom_counter_is_read_from_the_kernel_text() {
    let text = "low 0\nhigh 0\nmax 2\noom 1\noom_kill 3\noom_group_kill 0\n";
    assert_eq!(parse_oom_kill(text).expect("oom_kill is present"), 3);
}

#[test]
fn a_missing_oom_counter_is_an_infrastructure_error() {
    let error = parse_oom_kill("low 0\nmax 2\n").expect_err("no oom_kill line");
    assert!(matches!(error, InfraError::Unexpected(_)));
}

#[test]
fn a_non_numeric_oom_counter_is_an_infrastructure_error() {
    let error = parse_oom_kill("oom_kill many\n").expect_err("not a number");
    assert!(matches!(error, InfraError::Unexpected(_)));
}

#[test]
fn a_timed_out_daemon_call_maps_to_api_timeout() {
    assert_eq!(
        map_error(&DaemonError::RequestTimeoutError),
        InfraError::ApiTimeout
    );
}

#[test]
fn a_missing_socket_maps_to_daemon_unreachable() {
    let error = map_error(&DaemonError::SocketNotFoundError("/run/x.sock".to_owned()));
    assert!(matches!(error, InfraError::DaemonUnreachable(_)));
}

#[test]
fn a_connection_failure_maps_to_daemon_unreachable() {
    let io = std::io::Error::other("connection refused");
    let error = map_error(&DaemonError::IOError { err: io });
    assert!(matches!(error, InfraError::DaemonUnreachable(_)));
}

#[test]
fn a_server_refusal_maps_to_unexpected_and_keeps_the_message() {
    let error = map_error(&DaemonError::DockerResponseServerError {
        status_code: 500,
        message: "no space left".to_owned(),
    });
    match error {
        InfraError::Unexpected(text) => assert!(text.contains("no space left")),
        other => panic!("expected Unexpected, got {other:?}"),
    }
}
