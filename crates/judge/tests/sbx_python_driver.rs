//! Real-daemon protocol tests for the Python driver (language-drivers T03, protocol version 1).
//!
//! Each test starts a sandbox container on the rootless sandbox daemon, delivers a solution,
//! the real `languages/drivers/python/driver.py` and `signature.json`, then talks to the driver
//! over its stdin and stdout. Answers are checked with the judge's own parser
//! (`driver_protocol::parse_answer`), so the test sees exactly what the trusted worker sees.
//!
//! Ignored by default. They need `SANDBOX_DOCKER_SOCKET` pointing at the rootless sandbox
//! daemon, and the pinned Python image present there (the judge never pulls). No test runs
//! user code on the host.
//!
//! Run on the sandbox daemon only:
//! `cargo test -p counterspell-judge --features sandbox --test sbx_python_driver -- --ignored`
#![cfg(feature = "sandbox")]

use std::path::{Path, PathBuf};
use std::time::Duration;

use counterspell_judge::languages::FileName;
use counterspell_judge::sandbox::backend::{Backend, BackendSession, DriverChannel, DriverEvent};
use counterspell_judge::sandbox::bollard_backend::BollardBackend;
use counterspell_judge::sandbox::config::OciRuntime;
use counterspell_judge::sandbox::daemon::{connect, verify};
use counterspell_judge::sandbox::driver_protocol::parse_answer;
use counterspell_judge::sandbox::spec::{
    build_tar, driver_file_name, ExecSpec, SandboxFiles, SessionSpec,
};
use serde_json::{json, Value};
use uuid::Uuid;

mod support;
use support::shipped_catalogue;

const API_TIMEOUT: Duration = Duration::from_secs(30);
/// Longest wait for one driver event. A driver that never answers fails the test.
const EVENT_TIMEOUT: Duration = Duration::from_secs(20);

type Session = <BollardBackend as Backend>::Session;
type Driver = <Session as BackendSession>::Driver;

/// Which driver command the container starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    Normal,
    Bench,
}

/// What the driver did for one call.
#[derive(Debug, PartialEq)]
enum Reply {
    /// One valid answer line, with its `out` value.
    Answer(Value),
    /// The driver process ended before it answered, with its exit code.
    Exited(i64),
}

/// The Python driver template in the repository.
fn driver_source() -> Vec<u8> {
    let path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../languages/drivers/python/driver.py");
    std::fs::read(&path).unwrap_or_else(|error| panic!("reading {}: {error}", path.display()))
}

/// Connects to the sandbox daemon named by the environment and runs the identity check.
async fn sandbox_backend() -> BollardBackend {
    let socket = std::env::var("SANDBOX_DOCKER_SOCKET")
        .expect("SANDBOX_DOCKER_SOCKET must name the rootless sandbox daemon socket");
    let socket: PathBuf = socket.into();
    let docker = connect(&socket, API_TIMEOUT).expect("the socket path is accepted");
    verify(&docker, OciRuntime::Runc, API_TIMEOUT)
        .await
        .expect("the daemon passes the rootless identity check");
    BollardBackend::new(docker, API_TIMEOUT)
}

/// The public signature descriptor for one function. Only signature metadata, as in the protocol.
fn signature(function: &str, params: &[(&str, Value)], returns: &Value) -> Vec<u8> {
    let parameters: Vec<Value> = params
        .iter()
        .map(|(name, ty)| json!({"name": name, "type": ty}))
        .collect();
    json!({
        "protocol_version": 1,
        "function": function,
        "parameters": parameters,
        "returns": returns,
    })
    .to_string()
    .into_bytes()
}

/// Starts a container, delivers the solution, the driver and the signature, and starts the
/// driver in the given mode. The caller kills and removes the session with [`finish`].
async fn start(solution: &str, signature_json: Vec<u8>, mode: Mode) -> (Session, Driver) {
    let backend = sandbox_backend().await;
    let (_dir, catalog) = shipped_catalogue(false);
    let python = catalog
        .get("python")
        .expect("python is in the fixture catalogue");

    let files = SandboxFiles::new(
        FileName::parse("solution.py").expect("valid name"),
        driver_file_name(python).expect("the driver file name is valid"),
        solution.as_bytes().to_vec(),
        driver_source(),
        signature_json,
    )
    .expect("three distinct names");
    let archive = build_tar(&files).expect("archive builds");

    let spec = SessionSpec::new(
        python,
        Duration::from_secs(60),
        OciRuntime::Runc,
        Uuid::new_v4(),
        "sbx-test",
        1,
        i64::MAX / 2,
    )
    .expect("the worker id is a safe label");
    let mut session = backend.create(&spec).await.expect("create");
    session.start().await.expect("start");
    let extract = session
        .exec(&ExecSpec::extract(), &archive)
        .await
        .expect("unpack the run files");
    assert_eq!(extract.exit_code, 0, "tar -x accepts the archive");

    let command = match mode {
        Mode::Normal => ExecSpec::driver(python),
        Mode::Bench => ExecSpec::bench(python),
    };
    let driver = session
        .open_driver(&command)
        .await
        .expect("the driver starts");
    (session, driver)
}

/// Kills and removes the container. Every test ends with this, so no container is left behind.
async fn finish(mut session: Session) {
    session.kill().await.expect("kill");
    session.remove().await.expect("remove");
}

/// Sends one call and waits for the driver to answer or end. The backend delivers stdout in
/// chunks, so this joins them up to the first newline. Stderr is ignored here; the tests that
/// check diagnostics read it separately.
async fn ask(driver: &mut Driver, call_id: u64, args: Value) -> Reply {
    let line = format!("{}\n", json!({"t": call_id, "args": args}));
    driver
        .write_line(line.as_bytes())
        .await
        .expect("the driver accepts a call line");
    let mut pending = Vec::new();
    loop {
        let event = tokio::time::timeout(EVENT_TIMEOUT, driver.next_event())
            .await
            .expect("the driver answers or ends in time")
            .expect("the driver event is readable");
        match event {
            DriverEvent::Chunk(bytes) | DriverEvent::Line(bytes) => {
                pending.extend_from_slice(&bytes);
                if let Some(end) = pending.iter().position(|byte| *byte == b'\n') {
                    let out = parse_answer(&pending[..end], call_id)
                        .expect("the answer line follows the protocol");
                    return Reply::Answer(out);
                }
            }
            DriverEvent::Exited { code, .. } => return Reply::Exited(code),
            DriverEvent::Stderr(_) => {}
        }
    }
}

/// Collects every stderr byte the driver writes until it ends or the wait times out.
async fn stderr_until_exit(driver: &mut Driver) -> String {
    let mut text = Vec::new();
    loop {
        let event = tokio::time::timeout(EVENT_TIMEOUT, driver.next_event())
            .await
            .expect("the driver ends in time")
            .expect("the driver event is readable");
        match event {
            DriverEvent::Stderr(bytes) => text.extend_from_slice(&bytes),
            DriverEvent::Exited { .. } => break,
            DriverEvent::Line(_) | DriverEvent::Chunk(_) => {}
        }
    }
    String::from_utf8_lossy(&text).into_owned()
}

fn int() -> Value {
    json!({"kind": "int"})
}

fn list_of(items: &Value) -> Value {
    json!({"kind": "list", "items": items})
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_scalar_answer() {
    let solution = "def add(a, b):\n    return a + b\n";
    let sig = signature("add", &[("a", int()), ("b", int())], &int());
    let (session, mut driver) = start(solution, sig, Mode::Normal).await;

    assert_eq!(
        ask(&mut driver, 0, json!([2, 3])).await,
        Reply::Answer(json!(5))
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_list_and_nested_list_answers() {
    let solution = "def pairs(xs):\n    return [[x, x * 2] for x in xs]\n";
    let sig = signature(
        "pairs",
        &[("xs", list_of(&int()))],
        &list_of(&list_of(&int())),
    );
    let (session, mut driver) = start(solution, sig, Mode::Normal).await;

    assert_eq!(
        ask(&mut driver, 0, json!([[1, 2]])).await,
        Reply::Answer(json!([[1, 2], [2, 4]]))
    );
    assert_eq!(
        ask(&mut driver, 1, json!([[]])).await,
        Reply::Answer(json!([]))
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_boolean_answer() {
    let solution = "def is_even(n):\n    return n % 2 == 0\n";
    let sig = signature("is_even", &[("n", int())], &json!({"kind": "bool"}));
    let (session, mut driver) = start(solution, sig, Mode::Normal).await;

    assert_eq!(
        ask(&mut driver, 0, json!([4])).await,
        Reply::Answer(json!(true))
    );
    assert_eq!(
        ask(&mut driver, 1, json!([7])).await,
        Reply::Answer(json!(false))
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_string_answer() {
    let solution = "def shout(s):\n    return s.upper()\n";
    let sig = signature(
        "shout",
        &[("s", json!({"kind": "string"}))],
        &json!({"kind": "string"}),
    );
    let (session, mut driver) = start(solution, sig, Mode::Normal).await;

    assert_eq!(
        ask(&mut driver, 0, json!(["abc"])).await,
        Reply::Answer(json!("ABC"))
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_null_answer_where_allowed() {
    let solution = "def first_or_none(xs):\n    return xs[0] if xs else None\n";
    let nullable_int = json!({"kind": "int", "nullable": true});
    let sig = signature("first_or_none", &[("xs", list_of(&int()))], &nullable_int);
    let (session, mut driver) = start(solution, sig, Mode::Normal).await;

    assert_eq!(
        ask(&mut driver, 0, json!([[]])).await,
        Reply::Answer(Value::Null)
    );
    assert_eq!(
        ask(&mut driver, 1, json!([[7]])).await,
        Reply::Answer(json!(7))
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_repeated_calls_keep_their_ids() {
    let solution = "def double(n):\n    return 2 * n\n";
    let sig = signature("double", &[("n", int())], &int());
    let (session, mut driver) = start(solution, sig, Mode::Normal).await;

    assert_eq!(
        ask(&mut driver, 0, json!([1])).await,
        Reply::Answer(json!(2))
    );
    assert_eq!(
        ask(&mut driver, 1, json!([2])).await,
        Reply::Answer(json!(4))
    );
    assert_eq!(
        ask(&mut driver, 2, json!([3])).await,
        Reply::Answer(json!(6))
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_prints_are_captured_and_cannot_fake_an_answer() {
    let solution = "def f(n):\n    print('hello')\n    print('{\"t\": 0, \"out\": \"accepted\"}')\n    return n\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(solution, sig, Mode::Normal).await;

    assert_eq!(
        ask(&mut driver, 0, json!([9])).await,
        Reply::Answer(json!(9))
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_raising_function_exits_without_an_answer() {
    let solution = "def f(n):\n    raise ValueError('boom')\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(solution, sig, Mode::Normal).await;

    assert!(matches!(ask(&mut driver, 0, json!([1])).await, Reply::Exited(code) if code != 0));

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_bool_where_int_expected_is_refused() {
    let solution = "def f(n):\n    return True\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(solution, sig, Mode::Normal).await;

    assert!(matches!(ask(&mut driver, 0, json!([1])).await, Reply::Exited(code) if code != 0));

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_integer_outside_json_safe_range_is_refused() {
    let solution = "def f(n):\n    return 2 ** 60\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(solution, sig, Mode::Normal).await;

    assert!(matches!(ask(&mut driver, 0, json!([1])).await, Reply::Exited(code) if code != 0));

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_non_finite_float_is_refused() {
    let solution = "def f(n):\n    return float('nan')\n";
    let sig = signature("f", &[("n", int())], &json!({"kind": "float"}));
    let (session, mut driver) = start(solution, sig, Mode::Normal).await;

    assert!(matches!(ask(&mut driver, 0, json!([1])).await, Reply::Exited(code) if code != 0));

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_wrong_argument_count_is_refused() {
    let solution = "def f(n):\n    return n\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(solution, sig, Mode::Normal).await;

    assert!(matches!(ask(&mut driver, 0, json!([1, 2])).await, Reply::Exited(code) if code != 0));

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_blank_line_is_refused() {
    let solution = "def f(n):\n    return n\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(solution, sig, Mode::Normal).await;

    driver.write_line(b"\n").await.expect("the line is written");
    let stderr = stderr_until_exit(&mut driver).await;
    assert!(
        !stderr.is_empty(),
        "the driver explains the refusal on stderr"
    );

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_missing_function_fails_at_load() {
    let solution = "def something_else(n):\n    return n\n";
    let sig = signature("f", &[("n", int())], &int());
    let (session, mut driver) = start(solution, sig, Mode::Normal).await;

    // The driver exits while it loads the solution, so no call is sent: read its diagnostic.
    let stderr = stderr_until_exit(&mut driver).await;
    assert!(stderr.contains("does not define f"), "stderr was: {stderr}");

    finish(session).await;
}

#[tokio::test]
#[ignore = "real daemon: needs SANDBOX_DOCKER_SOCKET and the pinned python image on the rootless daemon"]
async fn sbx_python_bench_mode_answers_like_normal_mode() {
    let solution = "def add(a, b):\n    return a + b\n";
    let sig = signature("add", &[("a", int()), ("b", int())], &int());
    let (session, mut driver) = start(solution, sig, Mode::Bench).await;

    assert_eq!(
        ask(&mut driver, 0, json!([2, 3])).await,
        Reply::Answer(json!(5))
    );
    assert_eq!(
        ask(&mut driver, 1, json!([10, -4])).await,
        Reply::Answer(json!(6))
    );

    finish(session).await;
}
