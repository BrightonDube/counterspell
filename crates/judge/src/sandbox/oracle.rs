// SPDX-License-Identifier: MIT
//! The ghost engine's bridge into the sandbox (`counterspell_engine::Oracle`).
//!
//! `evaluate` is synchronous by contract and runs inside `spawn_blocking`; this
//! type holds a Tokio `Handle` and drives the sandbox calls with `block_on`.
//! The user's program and the trusted reference live in **separate containers**
//! — one backend and one long-lived session per side. A call that ends
//! abnormally has already killed its container, so the next evaluate opens a
//! fresh session for that side only.
//!
//! The oracle never sees a problem's expected outputs: its inputs are the
//! public JSON values the engine generates (candidates, samples, authored
//! ghosts). What either program prints is untrusted text; only the protocol
//! answer line becomes a [`ProgramResult`]. Infrastructure faults are
//! [`OracleError`], never a user failure.

use std::time::{Duration, SystemTime, UNIX_EPOCH};

use counterspell_engine::{Observation, Oracle, OracleError, ProgramResult};
use serde_json::Value;
use tokio::runtime::Handle;
use tokio::sync::watch;
use uuid::Uuid;

use crate::languages::LanguageDefinition;

use super::backend::Backend;
use super::call::{run_call, CallFacts, CallIo, CallLimits};
use super::config::SandboxConfig;
use super::driver_protocol::parse_answer;
use super::session::{open_session, Opened, SessionError};
use super::spec::{build_tar, driver_file_name, SandboxFiles, SessionSpec};

/// A snippet's bound when output becomes evidence text (`Garbled`, `Crashed`).
const SNIPPET_BYTES: usize = 256;

/// Which side a session belongs to. Only the user side may turn a compile
/// failure into a program result; the reference is trusted code, so its
/// failures are infrastructure.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Role {
    /// The program under test.
    User,
    /// The problem's trusted reference implementation.
    Reference,
}

/// One side of the oracle: its own backend, its own container, and everything
/// needed to rebuild the session after an abnormal end.
pub(crate) struct Side<B: Backend> {
    backend: B,
    lang: LanguageDefinition,
    archive: Vec<u8>,
    run: Uuid,
    worker: String,
    run_cap: Duration,
    runtime: super::config::OciRuntime,
    orphan_age: Duration,
    opens: u32,
    opened: Option<Opened<B::Session>>,
    io: CallIo,
    next_id: u64,
    role: Role,
}

/// The sandbox-backed [`Oracle`]. Construct on any thread; call `evaluate`
/// only outside the async runtime.
pub struct SandboxOracle<B: Backend> {
    handle: Handle,
    user: Side<B>,
    reference: Side<B>,
    kill_slack: Duration,
    stop: watch::Sender<bool>,
}

impl<B: Backend> SandboxOracle<B> {
    /// Builds the oracle without opening anything — sessions open lazily on the
    /// first `evaluate`, so a hostile program's compile failure surfaces as a
    /// `Crashed` result, not a constructor error.
    ///
    /// `user_backend` and `reference_backend` are separate values (cheap clones
    /// of one client in production) so the two sides can never share a
    /// container. `signature` is the problem's public signature `Value`; it is
    /// the only problem data the oracle holds — expected outputs never reach
    /// it.
    ///
    /// # Errors
    ///
    /// Returns [`OracleError`] when the driver file cannot be read, the
    /// signature cannot be serialised, or the archive cannot be built.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        handle: Handle,
        user_backend: B,
        reference_backend: B,
        user_lang: LanguageDefinition,
        reference_lang: LanguageDefinition,
        signature: &Value,
        user_source: &str,
        reference_source: &str,
        config: &SandboxConfig,
    ) -> Result<Self, OracleError> {
        let driver_root = config.languages_file.parent().map_or_else(
            || std::path::PathBuf::from("."),
            std::path::Path::to_path_buf,
        );
        let signature_bytes = serde_json::to_vec(signature).map_err(|error| {
            OracleError::new(format!("the signature does not serialise: {error}"))
        })?;
        let user = side(
            user_backend,
            user_lang,
            &driver_root,
            &signature_bytes,
            user_source,
            config,
            Role::User,
        )?;
        let reference = side(
            reference_backend,
            reference_lang,
            &driver_root,
            &signature_bytes,
            reference_source,
            config,
            Role::Reference,
        )?;
        let (stop, _receiver) = watch::channel(false);
        Ok(Self {
            handle,
            user,
            reference,
            kill_slack: config.kill_slack,
            stop,
        })
    }
}

impl<B: Backend> Oracle for SandboxOracle<B> {
    fn evaluate(&mut self, input: &Value, limit: Duration) -> Result<Observation, OracleError> {
        let args: Vec<Value> = match input {
            Value::Array(items) => items.clone(),
            single => vec![single.clone()],
        };
        let handle = self.handle.clone();
        let stop = &self.stop;
        let kill_slack = self.kill_slack;
        let (user, reference) = (&mut self.user, &mut self.reference);
        // Both sides run concurrently so `evaluate` returns within `limit +
        // KILL_SLACK`, not twice it. `block_on` is the sanctioned bridge from a
        // blocking thread — but it panics on a runtime worker thread, and the
        // engine must get an honest error there, not a crash.
        let call = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            handle.block_on(async move {
                let user_call = evaluate_side(user, &args, limit, kill_slack, stop.subscribe());
                let reference_call =
                    evaluate_side(reference, &args, limit, kill_slack, stop.subscribe());
                let (user_result, reference_result) = tokio::join!(user_call, reference_call);
                Ok(Observation {
                    user: user_result?,
                    reference: reference_result?,
                })
            })
        }));
        call.unwrap_or_else(|_panic| {
            Err(OracleError::new(
                "SandboxOracle::evaluate cannot run on an async worker thread (use spawn_blocking)",
            ))
        })
    }
}

/// Builds one side's archive and spec material. Returns `Err` when the driver
/// file cannot be read or the archive cannot be assembled.
pub(crate) fn side<B: Backend>(
    backend: B,
    lang: LanguageDefinition,
    driver_root: &std::path::Path,
    signature_bytes: &[u8],
    source: &str,
    config: &SandboxConfig,
    role: Role,
) -> Result<Side<B>, OracleError> {
    let driver_path = driver_root.join(lang.driver().as_str());
    let driver_code = std::fs::read(&driver_path).map_err(|error| {
        OracleError::new(format!(
            "the driver file cannot be read ({}): {error}",
            driver_path.display()
        ))
    })?;
    let driver_name = driver_file_name(&lang)
        .ok_or_else(|| OracleError::new("the driver file name is invalid"))?;
    let files = SandboxFiles::new(
        lang.file().clone(),
        driver_name,
        source.as_bytes().to_vec(),
        driver_code,
        signature_bytes.to_vec(),
    )
    .map_err(|error| OracleError::new(format!("the archive files conflict: {error}")))?;
    let archive =
        build_tar(&files).map_err(|error| OracleError::new(format!("the archive: {error}")))?;
    Ok(Side {
        backend,
        lang,
        archive,
        run: Uuid::new_v4(),
        worker: "sbx-oracle".to_owned(),
        run_cap: config.run_wall_cap,
        runtime: config.runtime,
        orphan_age: config.orphan_age,
        opens: 0,
        opened: None,
        io: CallIo::new(),
        next_id: 1,
        role,
    })
}

/// Opens the side's session when none is live, then runs one call and maps the
/// facts onto [`ProgramResult`]. Any end other than an answer has already
/// killed the container, so the handle is dropped and the next call reopens.
pub(crate) async fn evaluate_side<B: Backend>(
    side: &mut Side<B>,
    args: &[Value],
    limit: Duration,
    kill_slack: Duration,
    mut stop: watch::Receiver<bool>,
) -> Result<ProgramResult, OracleError> {
    if side.opened.is_none() {
        side.opens += 1;
        let spec = SessionSpec::new(
            &side.lang,
            side.run_cap,
            side.runtime,
            side.run,
            &side.worker,
            side.opens,
            expiry(side.orphan_age),
        )
        .map_err(|error| OracleError::new(format!("the session spec: {error}")))?;
        match open_session(&side.backend, &spec, &side.lang, &side.archive).await {
            Ok(opened) => {
                side.opened = Some(opened);
                // A fresh driver answers a fresh call sequence.
                side.io = CallIo::new();
                side.next_id = 1;
            }
            // A compile failure of the user's program is the program's own
            // result, not infrastructure. The reference is trusted code.
            Err(SessionError::CompileFailed(message)) if side.role == Role::User => {
                return Ok(ProgramResult::Crashed {
                    exception_class: "CompileError".to_owned(),
                    message: bounded(&format!("{message:?}")),
                });
            }
            Err(error) => return Err(OracleError::new(error.to_string())),
        }
    }
    let Some(opened) = side.opened.as_mut() else {
        return Err(OracleError::new("no open container for the call"));
    };
    let id = side.next_id;
    side.next_id += 1;
    let limits = CallLimits {
        time: limit,
        run_deadline: tokio::time::Instant::now() + limit + kill_slack,
        kill_slack,
    };
    match run_call(opened, &mut side.io, id, args, limits, &mut stop).await {
        Err(infra) => {
            side.opened = None;
            Err(OracleError::new(infra.to_string()))
        }
        Ok(facts) => {
            if !matches!(facts, CallFacts::Answer { .. }) {
                side.opened = None;
            }
            Ok(map_facts(facts, id))
        }
    }
}

/// Maps what a call observed onto the engine's result vocabulary.
fn map_facts(facts: CallFacts, id: u64) -> ProgramResult {
    match facts {
        CallFacts::Answer { line, .. } => match parse_answer(&line, id) {
            Ok(value) => ProgramResult::Returned(value),
            Err(_violation) => ProgramResult::Garbled {
                snippet: bounded_bytes(&line),
            },
        },
        CallFacts::Deadline => ProgramResult::TimedOut,
        CallFacts::ExtraOutput => ProgramResult::Garbled {
            snippet: "the driver wrote more than its one answer line".to_owned(),
        },
        CallFacts::OutputLimit => ProgramResult::Garbled {
            snippet: "the driver exceeded the output limit".to_owned(),
        },
        CallFacts::DriverExited {
            code,
            oom_kill,
            last_stderr_line,
        } => crash(code, oom_kill, last_stderr_line),
        // The worker asked to stop; from the engine's view that is an
        // infrastructure condition, not the program's answer.
        CallFacts::Stopped => ProgramResult::Crashed {
            exception_class: "Stopped".to_owned(),
            message: "the call was stopped".to_owned(),
        },
    }
}

/// Turns a driver exit into `Crashed`: the kernel's OOM counter first, then the
/// last stderr line parsed as `ExceptionClass: message` (Python's traceback
/// tail), and finally the bare exit code.
fn crash(code: i64, oom_kill: u64, last_stderr_line: Option<Vec<u8>>) -> ProgramResult {
    if oom_kill > 0 {
        return ProgramResult::Crashed {
            exception_class: "OutOfMemory".to_owned(),
            message: "killed by the memory limit".to_owned(),
        };
    }
    let text = last_stderr_line.map_or_else(String::new, |line| bounded_bytes(&line));
    let text = text.trim();
    if let Some((class, message)) = text.split_once(": ") {
        if !class.is_empty()
            && class
                .chars()
                .all(|ch| ch.is_ascii_alphanumeric() || ch == '_' || ch == '.')
        {
            return ProgramResult::Crashed {
                exception_class: class.to_owned(),
                message: message.to_owned(),
            };
        }
    }
    ProgramResult::Crashed {
        exception_class: if text.is_empty() {
            format!("Exit{code}")
        } else {
            "Exit".to_owned()
        },
        message: if text.is_empty() {
            format!("the driver exited with code {code}")
        } else {
            text.to_owned()
        },
    }
}

/// A bounded lossy view of untrusted bytes for evidence snippets.
fn bounded_bytes(bytes: &[u8]) -> String {
    let text = String::from_utf8_lossy(bytes);
    bounded(&text)
}

/// Truncates on a char boundary so snippets never grow past `SNIPPET_BYTES`.
fn bounded(text: &str) -> String {
    let mut end = text.len().min(SNIPPET_BYTES);
    while !text.is_char_boundary(end) {
        end -= 1;
    }
    text[..end].to_owned()
}

/// The `counterspell.expires` label value: orphan age from now.
fn expiry(orphan_age: Duration) -> i64 {
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |since| {
            i64::try_from(since.as_secs()).unwrap_or(i64::MAX)
        });
    now.saturating_add(i64::try_from(orphan_age.as_secs()).unwrap_or(i64::MAX))
}
