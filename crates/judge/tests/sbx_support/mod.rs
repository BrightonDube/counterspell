//! Harness for the real-daemon driver tests (language-drivers T03 and T04).
//!
//! Each test starts a sandbox container on the rootless sandbox daemon, delivers a solution,
//! the real driver template from `languages/drivers/<lang>/` and `signature.json`, then talks to
//! the driver over its stdin and stdout. Answers are checked with the judge's own parser
//! (`driver_protocol::parse_answer`), so a test sees exactly what the trusted worker sees.
//!
//! Only compiled with the `sandbox` feature. Nothing here runs user code on the host.

#![cfg(feature = "sandbox")]
#![allow(dead_code)]

use std::path::{Path, PathBuf};
use std::time::Duration;

use counterspell_judge::languages::{FileName, LanguageCatalog, LanguageDefinition};
use counterspell_judge::sandbox::backend::{Backend, BackendSession, DriverChannel, DriverEvent};
use counterspell_judge::sandbox::bollard_backend::BollardBackend;
use counterspell_judge::sandbox::config::OciRuntime;
use counterspell_judge::sandbox::daemon::{connect, verify};
use counterspell_judge::sandbox::driver_protocol::parse_answer;
use counterspell_judge::sandbox::spec::{
    build_tar, driver_file_name, ExecSpec, SandboxFiles, SessionSpec,
};
use serde_json::{json, Value};
use tempfile::TempDir;
use uuid::Uuid;

const API_TIMEOUT: Duration = Duration::from_secs(30);
/// Longest wait for one driver event. A driver that never answers fails the test.
const EVENT_TIMEOUT: Duration = Duration::from_secs(20);

/// The live container type the bollard backend returns.
pub type Session = <BollardBackend as Backend>::Session;
/// The driver channel of a live container.
pub type Driver = <Session as BackendSession>::Driver;

/// Which driver command the container starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Bench,
}

/// What the driver did for one call.
#[derive(Debug, PartialEq)]
pub enum Reply {
    /// One valid answer line, with its `out` value.
    Answer(Value),
    /// The driver process ended before it answered, with its exit code.
    Exited(i64),
}

/// The repository root, found from this crate's manifest directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

/// Loads the shipped `languages/languages.toml` unchanged. Each enabled language's driver
/// file is copied from the repository, or stubbed when it does not exist yet, so the loader
/// accepts the catalogue. The sandbox run itself always uses the real driver file.
pub fn catalogue() -> (TempDir, LanguageCatalog) {
    let text = include_str!("../../../../languages/languages.toml");
    catalogue_text(text)
}

/// The shipped catalogue with the named languages flipped to `enabled = true`, for tests
/// that run ahead of a language's public enablement.
pub fn catalogue_enabled(extra: &[&str]) -> (TempDir, LanguageCatalog) {
    let mut text = include_str!("../../../../languages/languages.toml").to_string();
    for lang in extra {
        let section = format!("[{lang}]");
        let start = text
            .find(&section)
            .unwrap_or_else(|| panic!("a [{lang}] section"));
        let end = text[start..]
            .find("\n[")
            .map_or(text.len(), |offset| start + offset);
        let block = text[start..end].replace("enabled = false", "enabled = true");
        assert!(block.contains("enabled = true"), "[{lang}] becomes enabled");
        text.replace_range(start..end, &block);
    }
    catalogue_text(&text)
}

fn catalogue_text(text: &str) -> (TempDir, LanguageCatalog) {
    let dir = TempDir::new().expect("a temp dir for the stub drivers");
    for driver in [
        "drivers/python/driver.py",
        "drivers/javascript/driver.js",
        "drivers/java/Driver.java",
        "drivers/cpp/driver.cpp",
    ] {
        let target = dir.path().join(driver);
        std::fs::create_dir_all(target.parent().expect("driver path has a parent"))
            .expect("driver directory");
        let source = repo_root().join("languages").join(driver);
        let contents = std::fs::read(&source).unwrap_or_else(|_| b"// stub\n".to_vec());
        std::fs::write(&target, contents).expect("driver stub written");
    }
    let catalog = LanguageCatalog::parse(text, dir.path()).expect("the shipped catalogue loads");
    (dir, catalog)
}

/// The real driver template for a language, read from the repository.
fn driver_source(driver: &str) -> Vec<u8> {
    let path = repo_root().join("languages").join(driver);
    std::fs::read(&path).unwrap_or_else(|error| panic!("reading {}: {error}", path.display()))
}

/// Connects to the sandbox daemon named by the environment and runs the identity check.
pub async fn sandbox_backend() -> BollardBackend {
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
pub fn signature(function: &str, params: &[(&str, Value)], returns: &Value) -> Vec<u8> {
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

/// A JSON integer type descriptor.
pub fn int() -> Value {
    json!({"kind": "int"})
}

/// A JSON list type descriptor with the given item type.
pub fn list_of(items: &Value) -> Value {
    json!({"kind": "list", "items": items})
}

/// Builds the archive the sandbox unpacks: the solution, the real driver template and the
/// public signature. Only these three files reach the container.
pub fn run_archive(
    language: &LanguageDefinition,
    solution: &str,
    signature_json: Vec<u8>,
) -> Vec<u8> {
    let files = SandboxFiles::new(
        FileName::parse(language.file().as_str()).expect("valid solution name"),
        driver_file_name(language).expect("the driver file name is valid"),
        solution.as_bytes().to_vec(),
        driver_source(language.driver().as_str()),
        signature_json,
    )
    .expect("three distinct names");
    build_tar(&files).expect("archive builds")
}

/// Starts a container for `lang`, delivers the solution, the real driver template and the
/// signature, and starts the driver in the given mode. The caller ends the session with
/// [`finish`].
pub async fn start(
    lang: &str,
    solution: &str,
    signature_json: Vec<u8>,
    mode: Mode,
) -> (Session, Driver) {
    let (_dir, catalog) = catalogue();
    let (session, driver) = start_on(&catalog, lang, solution, signature_json, mode).await;
    (session, driver.expect("the shipped languages compile"))
}

/// Like [`start`] but on a caller-supplied catalogue (the stretch tests enable Java and
/// C++). Runs the compile step when the language defines one and reports its exit code to
/// the caller, so a compile-failure case can observe it instead of panicking.
pub async fn start_on(
    catalog: &LanguageCatalog,
    lang: &str,
    solution: &str,
    signature_json: Vec<u8>,
    mode: Mode,
) -> (Session, Result<Driver, String>) {
    let backend = sandbox_backend().await;
    let language = catalog.get(lang).expect("the language is in the catalogue");
    let archive = run_archive(language, solution, signature_json);

    let spec = SessionSpec::new(
        language,
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

    if let Some(compile) = ExecSpec::compile(language) {
        let built = session
            .exec(&compile, &[])
            .await
            .expect("the compile step runs");
        if built.exit_code != 0 {
            return (
                session,
                Err(String::from_utf8_lossy(&built.stderr).into_owned()),
            );
        }
    }

    let command = match mode {
        Mode::Normal => ExecSpec::driver(language),
        Mode::Bench => ExecSpec::bench(language),
    };
    let driver = session
        .open_driver(&command)
        .await
        .expect("the driver starts");
    (session, Ok(driver))
}

/// Kills and removes the container. Every test ends with this, so no container is left behind.
pub async fn finish(mut session: Session) {
    session.kill().await.expect("kill");
    session.remove().await.expect("remove");
}

/// Sends one call and waits for the driver to answer or end. The backend delivers stdout in
/// chunks, so this joins them up to the first newline. Stderr is ignored here; the tests that
/// check diagnostics read it separately.
pub async fn ask(driver: &mut Driver, call_id: u64, args: Value) -> Reply {
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
                    let line = String::from_utf8_lossy(&pending[..end]).into_owned();
                    let out = parse_answer(&pending[..end], call_id).unwrap_or_else(|e| {
                        panic!("the answer line follows the protocol: {e:?} — line: {line:?}")
                    });
                    return Reply::Answer(out);
                }
            }
            DriverEvent::Exited { code, .. } => return Reply::Exited(code),
            DriverEvent::Stderr(_) => {}
        }
    }
}

/// Collects every stderr byte the driver writes until it ends or the wait times out.
pub async fn stderr_until_exit(driver: &mut Driver) -> String {
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
