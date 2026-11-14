//! The seam between the session logic and the Docker daemon.
//!
//! The session, call and run code is written against these traits. The real
//! implementation (`bollard_backend.rs`, feature `sandbox`) talks to the rootless
//! sandbox daemon. The scripted fake (`testkit.rs`) lets every session rule be tested
//! without Docker. The traits use generics rather than `dyn`, so calls are static.

use std::future::Future;

use thiserror::Error;

use super::spec::{ExecSpec, SessionSpec};

/// A failure of the sandbox infrastructure. It is never a verdict on the user's code:
/// the run ends as an internal error, not as a wrong answer (requirement R9.4).
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum InfraError {
    /// The sandbox daemon could not be reached.
    #[error("sandbox daemon unreachable: {0}")]
    DaemonUnreachable(String),
    /// A single daemon call did not finish within the API timeout.
    #[error("sandbox API call timed out")]
    ApiTimeout,
    /// The pinned image is not present on the sandbox daemon. Images are never pulled at run time.
    #[error("sandbox image missing: {0}")]
    ImageMissing(String),
    /// The image on the daemon has a different digest from the pinned one.
    #[error("sandbox image digest mismatch: {0}")]
    DigestMismatch(String),
    /// The container's configuration, read back from the daemon, weakens the policy.
    #[error("container policy read-back mismatch: {0}")]
    PolicyReadbackMismatch(String),
    /// The container could not be killed, or it still existed after the kill.
    #[error("container kill failed: {0}")]
    KillFailed(String),
    /// A command could not be started inside the container.
    #[error("exec setup failed: {0}")]
    ExecSetup(String),
    /// The daemon answered in a shape the worker does not understand.
    #[error("unexpected daemon response: {0}")]
    Unexpected(String),
}

/// The result of a bounded command that ran to completion (compile, probe, file load).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecOutput {
    /// The process exit code.
    pub exit_code: i64,
    /// Bytes the command wrote to stdout.
    pub stdout: Vec<u8>,
    /// Bytes the command wrote to stderr.
    pub stderr: Vec<u8>,
}

/// One thing the driver process did while the worker waited for it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DriverEvent {
    /// A complete line on stdout.
    Line(Vec<u8>),
    /// A chunk of stdout that does not end a line yet.
    Chunk(Vec<u8>),
    /// The driver process ended, with its exit code and the kernel's OOM-kill count.
    Exited {
        /// The exit code.
        code: i64,
        /// The `oom_kill` count from the container's memory events.
        oom_kill: u64,
    },
}

/// Creates sandbox containers.
pub trait Backend {
    /// The live container type this backend returns.
    type Session: BackendSession;

    /// Creates one container from the spec. Nothing is started yet.
    fn create(
        &self,
        spec: &SessionSpec,
    ) -> impl Future<Output = Result<Self::Session, InfraError>> + Send;
}

/// One live container.
pub trait BackendSession: Send {
    /// The driver channel this session opens.
    type Driver: DriverChannel;

    /// The container name, as given to the daemon.
    fn name(&self) -> &str;

    /// Starts the container's idle PID 1.
    fn start(&mut self) -> impl Future<Output = Result<(), InfraError>> + Send;

    /// Runs a bounded command to completion, with `stdin` as its input.
    fn exec(
        &mut self,
        spec: &ExecSpec,
        stdin: &[u8],
    ) -> impl Future<Output = Result<ExecOutput, InfraError>> + Send;

    /// Starts the driver command with stdin and stdout attached.
    fn open_driver(
        &mut self,
        spec: &ExecSpec,
    ) -> impl Future<Output = Result<Self::Driver, InfraError>> + Send;

    /// Kills the container's processes.
    fn kill(&mut self) -> impl Future<Output = Result<(), InfraError>> + Send;

    /// Removes the container, forcibly.
    fn remove(&mut self) -> impl Future<Output = Result<(), InfraError>> + Send;
}

/// The driver process's stdin and stdout.
pub trait DriverChannel: Send {
    /// Writes one call line to the driver's stdin.
    fn write_line(&mut self, line: &[u8]) -> impl Future<Output = Result<(), InfraError>> + Send;

    /// Waits for the driver's next output event.
    fn next_event(&mut self) -> impl Future<Output = Result<DriverEvent, InfraError>> + Send;
}
