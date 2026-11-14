//! The lifecycle of one sandbox container: create, read back, start, load the files,
//! compile, start the driver, and close.
//!
//! [`Session`] owns the container. On the normal path the caller awaits
//! [`Session::close`]. On every other path, including `?`, task cancellation and panic,
//! the `Drop` impl spawns the same kill and remove on the runtime. `Drop` cannot await,
//! so that cleanup is best effort, and the sweeper removes anything it misses.

use std::time::Duration;

use thiserror::Error;

use super::backend::{Backend, BackendSession, InfraError};
use super::policy::PolicyViolation;
use super::spec::{ExecSpec, SessionSpec};
use crate::languages::LanguageDefinition;

/// The most output (stdout and stderr together) a compile step may produce.
const COMPILE_OUTPUT_CAP: usize = 64 * 1024;

/// Why the compile step failed. Only the worker decides this, from what it observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CompileFailure {
    /// The compiler exited with this non-zero code.
    NonZeroExit(i64),
    /// The compile did not finish within the language's compile budget.
    TimedOut,
    /// The compile wrote more than 64 KiB of output.
    OutputTooLarge,
}

/// Why a session could not be opened. Compile failures are the user's fault. The other
/// variants are infrastructure problems.
#[derive(Debug, PartialEq, Eq, Error)]
pub enum SessionError {
    /// The policy failed its own hardening check before anything was created.
    #[error("container policy was weakened before create: {0:?}")]
    PolicyViolations(Vec<PolicyViolation>),
    /// The sandbox failed.
    #[error(transparent)]
    Infrastructure(#[from] InfraError),
    /// The submitted source did not compile.
    #[error("compile failed: {0:?}")]
    CompileFailed(CompileFailure),
}

/// One live container. Dropping it without `close` starts a best-effort removal.
pub struct Session<S: BackendSession + 'static> {
    container: Option<S>,
}

impl<S: BackendSession + 'static> Session<S> {
    fn new(container: S) -> Self {
        Self {
            container: Some(container),
        }
    }

    fn container_mut(&mut self) -> Result<&mut S, InfraError> {
        self.container
            .as_mut()
            .ok_or_else(|| InfraError::Unexpected("session already closed".to_owned()))
    }

    /// Kills and removes the container, and waits for the removal.
    ///
    /// A failed kill is logged and not returned: a forced removal stops a running container
    /// as well. A failed removal is returned, and the sweeper removes the container later.
    ///
    /// # Errors
    ///
    /// Returns the daemon error when the removal fails.
    pub async fn close(mut self) -> Result<(), InfraError> {
        match self.container.take() {
            Some(mut container) => finish(&mut container).await,
            None => Ok(()),
        }
    }
}

/// Kills, then force-removes. Shared by `close` and the `Drop` guard.
async fn finish<S: BackendSession>(container: &mut S) -> Result<(), InfraError> {
    if let Err(error) = container.kill().await {
        tracing::warn!(%error, "kill before removal failed; forced removal follows");
    }
    container.remove().await
}

impl<S: BackendSession + 'static> Drop for Session<S> {
    fn drop(&mut self) {
        let Some(mut container) = self.container.take() else {
            return;
        };
        match tokio::runtime::Handle::try_current() {
            Ok(handle) => {
                handle.spawn(async move {
                    if let Err(error) = finish(&mut container).await {
                        tracing::error!(%error, "sandbox cleanup failed; the sweeper will remove it");
                    }
                });
            }
            Err(_) => {
                tracing::error!(
                    "no runtime to clean up a sandbox container; the sweeper will remove it"
                );
            }
        }
    }
}

/// A session whose driver has started and is ready for calls.
pub struct Opened<S: BackendSession + 'static> {
    /// The container. The caller closes it when the run ends.
    pub session: Session<S>,
    /// The driver's stdin and stdout.
    pub driver: S::Driver,
}

/// Creates a container and brings it to the state where the driver can take calls.
///
/// The steps are: pre-check the policy, create, read the configuration back, start,
/// unpack `archive` (the source, driver and signature), compile, and start the driver.
/// Every failure closes the container before the error is returned.
///
/// # Errors
///
/// Returns [`SessionError::PolicyViolations`] if the spec's policy is weakened,
/// [`SessionError::CompileFailed`] if the compile fails, and
/// [`SessionError::Infrastructure`] for any sandbox failure.
pub async fn open_session<B: Backend>(
    backend: &B,
    spec: &SessionSpec,
    lang: &LanguageDefinition,
    archive: &[u8],
) -> Result<Opened<B::Session>, SessionError> {
    let violations = spec.policy.hardening_violations();
    if !violations.is_empty() {
        return Err(SessionError::PolicyViolations(violations));
    }
    let mut session = Session::new(backend.create(spec).await?);
    match prepare(&mut session, lang, archive).await {
        Ok(driver) => Ok(Opened { session, driver }),
        Err(error) => {
            if let Err(close_error) = session.close().await {
                tracing::warn!(%close_error, "closing a failed session did not confirm removal");
            }
            Err(error)
        }
    }
}

/// Runs the steps after create. Returns the driver channel once the driver has started.
async fn prepare<S: BackendSession + 'static>(
    session: &mut Session<S>,
    lang: &LanguageDefinition,
    archive: &[u8],
) -> Result<S::Driver, SessionError> {
    let container = session.container_mut()?;

    let readback = container.read_policy().await?;
    let violations = readback.hardening_violations();
    if !violations.is_empty() {
        return Err(InfraError::PolicyReadbackMismatch(format!("{violations:?}")).into());
    }

    container.start().await?;

    let loaded = container.exec(&ExecSpec::extract(), archive).await?;
    if loaded.exit_code != 0 {
        return Err(InfraError::ExecSetup(format!(
            "unpacking the run files exited with {}",
            loaded.exit_code
        ))
        .into());
    }

    if let Some(compile) = ExecSpec::compile(lang) {
        let budget = Duration::from_millis(lang.compile_budget_ms());
        compile_step(container, &compile, budget).await?;
    }

    let driver = container.open_driver(&ExecSpec::driver(lang)).await?;
    Ok(driver)
}

/// Runs the compile command within its budget and checks its exit code and output size.
async fn compile_step<S: BackendSession>(
    container: &mut S,
    compile: &ExecSpec,
    budget: Duration,
) -> Result<(), SessionError> {
    let output = match tokio::time::timeout(budget, container.exec(compile, &[])).await {
        Err(_elapsed) => return Err(SessionError::CompileFailed(CompileFailure::TimedOut)),
        Ok(result) => result?,
    };
    if output.stdout.len() + output.stderr.len() > COMPILE_OUTPUT_CAP {
        return Err(SessionError::CompileFailed(CompileFailure::OutputTooLarge));
    }
    if output.exit_code != 0 {
        return Err(SessionError::CompileFailed(CompileFailure::NonZeroExit(
            output.exit_code,
        )));
    }
    Ok(())
}
