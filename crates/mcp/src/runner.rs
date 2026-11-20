//! The runner slot that code-running tools use.
//!
//! The typed contract is Grimoire's [`PackRunner`]: a trusted runner executes the
//! reference or one incorrect program on one input. Tests inject a fake runner. The
//! production path needs the sandbox socket to pass the guard in [`crate::config`].
//! The judge's production runner is not built yet, so the production slot stays
//! unconfigured and every code-running tool returns `sandbox_not_configured`.

use crate::config::{sandbox_socket, ConfigError, SANDBOX_SOCKET_VAR};
use crate::errors::ToolError;
use counterspell_grimoire::runner::{PackRunner, RunnerError};
use counterspell_judge::sandbox::config::SandboxConfig;
use counterspell_judge::sandbox::pack_runner::PackRunnerFactory;
use serde_json::Value;
use std::collections::BTreeMap;
use std::path::Path;

/// What a tool tells the slot about the pack it is about to run: the program texts and the
/// driver-protocol signature. Only the reference and the one program under test are included,
/// never hidden tests or other private files.
#[derive(Clone, Debug)]
pub struct PackJob {
    /// Source texts keyed by `reference` or by the `incorrect/...` path.
    pub sources: BTreeMap<String, String>,
    /// The driver-protocol signature written to `signature.json` in the sandbox.
    pub signature: Value,
}

/// Builds a runner for one pack. The production factory wraps the judge's sandbox runner.
pub trait RunnerFactory: Send {
    /// Builds a runner for `job`. Nothing runs until the runner is called.
    ///
    /// # Errors
    ///
    /// Returns [`RunnerError::Infrastructure`] when the sandbox side cannot be set up.
    fn build(&self, job: &PackJob) -> Result<Box<dyn PackRunner + Send>, RunnerError>;
}

/// Why a slot could not hand out a runner.
#[derive(Debug)]
pub enum SlotError {
    /// No runner is configured, or the sandbox endpoint was refused.
    Config(ConfigError),
    /// The pack could not be read into a job.
    Job(ToolError),
    /// The factory could not build the runner. This is infrastructure, never a verdict.
    Runner(RunnerError),
}

/// Where a slot gets its runner.
enum Source {
    None,
    Injected(Box<dyn PackRunner + Send>),
    Factory {
        // Declared before `factory`: fields drop in order, and the runner's container cleanup
        // is spawned on the runtime the factory owns, so the runner has to go first.
        built: Option<Box<dyn PackRunner + Send>>,
        factory: Box<dyn RunnerFactory>,
    },
}

/// Holds the runner for code-running tools, or nothing when no runner is available.
pub struct RunnerSlot {
    source: Source,
}

impl RunnerSlot {
    /// A slot with no runner. Code-running tools fail closed with `NotConfigured`.
    #[must_use]
    pub fn unconfigured() -> Self {
        Self {
            source: Source::None,
        }
    }

    /// A slot holding `runner`. Used by tests with a fake runner.
    #[must_use]
    pub fn injected(runner: Box<dyn PackRunner + Send>) -> Self {
        Self {
            source: Source::Injected(runner),
        }
    }

    /// A slot that builds a fresh runner for each call from the pack the tool names.
    #[must_use]
    pub fn with_factory(factory: Box<dyn RunnerFactory>) -> Self {
        Self {
            source: Source::Factory {
                factory,
                built: None,
            },
        }
    }

    /// Drops the runner built for the last call, which closes its sandbox container. A tool calls
    /// this when it finishes, so no container stays open between calls.
    pub fn release(&mut self) {
        if let Source::Factory { built, .. } = &mut self.source {
            *built = None;
        }
    }

    /// Returns a runner that needs no per-pack job: only an injected one. A factory slot needs a
    /// job, so it answers `NotConfigured` here.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::NotConfigured`] when the slot holds no ready runner.
    pub fn get(&mut self) -> Result<&mut dyn PackRunner, ConfigError> {
        match &mut self.source {
            Source::Injected(runner) => Ok(runner.as_mut()),
            Source::None | Source::Factory { .. } => Err(ConfigError::NotConfigured),
        }
    }

    /// Returns the runner for one code-running call. `job` is called only when a factory has to
    /// build a runner, so an injected fake never reads pack files it does not use. A factory
    /// slot replaces its previous runner, which drops that runner's containers.
    ///
    /// # Errors
    ///
    /// Returns [`SlotError`] when nothing is configured, the job cannot be built, or the factory
    /// fails.
    pub fn get_for(
        &mut self,
        job: impl FnOnce() -> Result<PackJob, ToolError>,
    ) -> Result<&mut dyn PackRunner, SlotError> {
        match &mut self.source {
            Source::None => Err(SlotError::Config(ConfigError::NotConfigured)),
            Source::Injected(runner) => Ok(runner.as_mut()),
            Source::Factory { factory, built } => {
                // Drop the old runner first so its containers are released before a new one opens.
                *built = None;
                let job = job().map_err(SlotError::Job)?;
                let runner = factory.build(&job).map_err(SlotError::Runner)?;
                Ok(built.insert(runner).as_mut())
            }
        }
    }
}

/// Builds the production slot. The raw socket must pass the sandbox guard first, then the daemon
/// behind it must pass the judge's identity check (rootless, its own data root, cgroup limits
/// delegated). Only then does the slot get a factory that builds the judge's sandbox runner.
///
/// The configuration is the judge's own `SandboxConfig`, so limits, images, the driver and
/// cleanup are exactly the production ones. The socket is the guarded value, never a second
/// source, and the language catalogue is read from the workspace.
///
/// # Errors
///
/// Returns [`ConfigError::NotConfigured`] without a socket, [`ConfigError::Rejected`] when the
/// guard refuses it, and [`ConfigError::Unavailable`] when the settings are unusable or the daemon
/// fails the identity check.
pub async fn production_slot(
    root: &Path,
    raw_socket: Option<&str>,
) -> Result<RunnerSlot, ConfigError> {
    let socket = sandbox_socket(raw_socket)?;
    let socket = socket.to_string_lossy().into_owned();
    let languages = root.join("languages/languages.toml");
    let languages = languages.to_string_lossy().into_owned();
    let config = SandboxConfig::from_lookup(|name| match name {
        SANDBOX_SOCKET_VAR => Some(socket.clone()),
        "LANGUAGES_FILE" => Some(languages.clone()),
        other => std::env::var(other).ok(),
    })
    .map_err(|error| ConfigError::Unavailable(error.to_string()))?;
    let factory = PackRunnerFactory::verified(config)
        .await
        .map_err(ConfigError::Unavailable)?;
    Ok(RunnerSlot::with_factory(Box::new(JudgeRunnerFactory(
        factory,
    ))))
}

/// Builds the judge's sandbox runner for a pack. Each runner opens its own containers on the
/// sandbox daemon the factory was verified against.
struct JudgeRunnerFactory(PackRunnerFactory);

impl RunnerFactory for JudgeRunnerFactory {
    fn build(&self, job: &PackJob) -> Result<Box<dyn PackRunner + Send>, RunnerError> {
        let runner = self.0.build(job.sources.clone(), &job.signature)?;
        Ok(Box::new(runner))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::errors::{ErrorCategory, ToolError};
    use counterspell_engine::ProgramResult;
    use counterspell_grimoire::runner::{RunnerError, Target};
    use serde_json::{json, Value};
    use std::collections::BTreeMap;
    use std::path::Path;
    use std::time::Duration;

    /// A fake runner that always returns the same result. It never runs anything.
    struct FixedRunner;

    impl PackRunner for FixedRunner {
        fn run(
            &mut self,
            _target: Target<'_>,
            _input: &Value,
            _limit: Duration,
        ) -> Result<ProgramResult, RunnerError> {
            Err(RunnerError::Infrastructure("fixed fake".to_string()))
        }

        fn bench(
            &mut self,
            _sized_inputs: &[(u64, Value)],
            _limit: Duration,
        ) -> Result<Vec<(u64, u64)>, RunnerError> {
            Ok(Vec::new())
        }
    }

    /// What a factory saw, so a test can check the job reached it unchanged.
    #[derive(Default)]
    struct Seen {
        builds: usize,
        reference: Option<String>,
        signature: Option<Value>,
    }

    /// A factory that records its inputs and hands out `FixedRunner`, or fails on demand.
    struct RecordingFactory {
        seen: std::sync::Arc<std::sync::Mutex<Seen>>,
        fail: bool,
    }

    impl RunnerFactory for RecordingFactory {
        fn build(&self, job: &PackJob) -> Result<Box<dyn PackRunner + Send>, RunnerError> {
            let mut seen = self.seen.lock().expect("lock");
            seen.builds += 1;
            seen.reference = job.sources.get("reference").cloned();
            seen.signature = Some(job.signature.clone());
            if self.fail {
                return Err(RunnerError::Infrastructure("daemon down".to_owned()));
            }
            Ok(Box::new(FixedRunner))
        }
    }

    fn job() -> PackJob {
        PackJob {
            sources: BTreeMap::from([(
                "reference".to_owned(),
                "def f(x):\n    return x\n".to_owned(),
            )]),
            signature: json!({"function": "f"}),
        }
    }

    fn factory_slot(fail: bool) -> (RunnerSlot, std::sync::Arc<std::sync::Mutex<Seen>>) {
        let seen = std::sync::Arc::new(std::sync::Mutex::new(Seen::default()));
        let factory = RecordingFactory {
            seen: seen.clone(),
            fail,
        };
        (RunnerSlot::with_factory(Box::new(factory)), seen)
    }

    /// Breaks if a per-pack runner is built from anything but the job the tool described.
    #[test]
    fn a_factory_slot_builds_the_runner_from_the_job() {
        let (mut slot, seen) = factory_slot(false);
        assert!(slot.get_for(|| Ok(job())).is_ok());
        let seen = seen.lock().expect("lock");
        assert_eq!(seen.builds, 1);
        assert_eq!(seen.reference.as_deref(), Some("def f(x):\n    return x\n"));
        assert_eq!(seen.signature, Some(json!({"function": "f"})));
    }

    /// Breaks if a call reuses the previous pack's runner, which holds that pack's sources.
    #[test]
    fn every_call_gets_a_freshly_built_runner() {
        let (mut slot, seen) = factory_slot(false);
        assert!(slot.get_for(|| Ok(job())).is_ok());
        assert!(slot.get_for(|| Ok(job())).is_ok());
        assert_eq!(seen.lock().expect("lock").builds, 2);
    }

    /// Breaks if a pack is read for a slot that cannot run anything (the job is only needed by a
    /// factory), or if an injected fake needs files it does not use.
    #[test]
    fn the_job_is_only_built_for_a_factory() {
        let never = || -> Result<PackJob, ToolError> { panic!("the job must not be built") };
        assert!(matches!(
            RunnerSlot::unconfigured().get_for(never),
            Err(SlotError::Config(ConfigError::NotConfigured))
        ));
        assert!(RunnerSlot::injected(Box::new(FixedRunner))
            .get_for(never)
            .is_ok());
    }

    /// Breaks if a sandbox fault while building becomes a verdict instead of an infrastructure error.
    #[test]
    fn a_factory_fault_is_a_runner_error() {
        let (mut slot, _seen) = factory_slot(true);
        assert!(matches!(
            slot.get_for(|| Ok(job())),
            Err(SlotError::Runner(RunnerError::Infrastructure(_)))
        ));
    }

    /// Breaks if a pack that cannot be read still reaches the sandbox.
    #[test]
    fn a_job_error_stops_before_the_factory_is_called() {
        let (mut slot, seen) = factory_slot(false);
        let broken = || {
            Err(ToolError::new(
                ErrorCategory::NotFound,
                "no reference",
                Path::new("/w"),
            ))
        };
        assert!(matches!(slot.get_for(broken), Err(SlotError::Job(_))));
        assert_eq!(seen.lock().expect("lock").builds, 0);
    }

    #[test]
    fn an_unconfigured_slot_refuses_to_give_a_runner() {
        let mut slot = RunnerSlot::unconfigured();
        assert!(matches!(slot.get(), Err(ConfigError::NotConfigured)));
    }

    #[test]
    fn an_injected_fake_runner_is_handed_out() {
        let mut slot = RunnerSlot::injected(Box::new(FixedRunner));
        assert!(slot.get().is_ok());
    }

    /// Logs `runner` or `factory` when dropped, so a test can see the order.
    struct Logged {
        name: &'static str,
        log: std::sync::Arc<std::sync::Mutex<Vec<&'static str>>>,
    }

    impl Drop for Logged {
        fn drop(&mut self) {
            self.log.lock().expect("lock").push(self.name);
        }
    }

    struct LoggedRunner {
        _guard: Logged,
    }

    impl PackRunner for LoggedRunner {
        fn run(
            &mut self,
            _target: Target<'_>,
            _input: &Value,
            _limit: Duration,
        ) -> Result<ProgramResult, RunnerError> {
            Ok(ProgramResult::Returned(json!(0)))
        }

        fn bench(
            &mut self,
            _sized_inputs: &[(u64, Value)],
            _limit: Duration,
        ) -> Result<Vec<(u64, u64)>, RunnerError> {
            Ok(Vec::new())
        }
    }

    struct LoggedFactory {
        _guard: Logged,
        log: std::sync::Arc<std::sync::Mutex<Vec<&'static str>>>,
    }

    impl RunnerFactory for LoggedFactory {
        fn build(&self, _job: &PackJob) -> Result<Box<dyn PackRunner + Send>, RunnerError> {
            Ok(Box::new(LoggedRunner {
                _guard: Logged {
                    name: "runner",
                    log: self.log.clone(),
                },
            }))
        }
    }

    fn logged_slot() -> (
        RunnerSlot,
        std::sync::Arc<std::sync::Mutex<Vec<&'static str>>>,
    ) {
        let log = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));
        let factory = LoggedFactory {
            _guard: Logged {
                name: "factory",
                log: log.clone(),
            },
            log: log.clone(),
        };
        (RunnerSlot::with_factory(Box::new(factory)), log)
    }

    /// Breaks if a finished tool call leaves its runner (and so its sandbox container) alive
    /// until the next call or until the process ends.
    #[test]
    fn release_drops_the_built_runner_at_once() {
        let (mut slot, log) = logged_slot();
        assert!(slot.get_for(|| Ok(job())).is_ok());
        assert!(log.lock().expect("lock").is_empty(), "still held");

        slot.release();

        assert_eq!(*log.lock().expect("lock"), vec!["runner"]);
    }

    /// Breaks if the factory (which owns the runtime the cleanup is spawned on) goes away before
    /// the runner it built, which would lose the container cleanup.
    #[test]
    fn dropping_a_slot_drops_the_runner_before_the_factory() {
        let (mut slot, log) = logged_slot();
        assert!(slot.get_for(|| Ok(job())).is_ok());

        drop(slot);

        assert_eq!(*log.lock().expect("lock"), vec!["runner", "factory"]);
    }

    // test-change: MCP-1. The signature gained a workspace root and became async because the slot
    // now builds the judge's sandbox runner and checks the daemon identity. Assertions unchanged.
    #[tokio::test]
    async fn production_needs_an_accepted_sandbox_socket() {
        let root = Path::new("/w");
        assert!(matches!(
            production_slot(root, None).await,
            Err(ConfigError::NotConfigured)
        ));
        assert!(matches!(
            production_slot(root, Some("/var/run/docker.sock")).await,
            Err(ConfigError::Rejected(_))
        ));
    }

    // test-change: MCP-1. The old test asserted that an accepted socket still gave an
    // unconfigured slot "until the judge runner exists". The runner exists now (judge
    // `PackRunnerFactory`), so an accepted socket whose daemon does not answer must be refused by
    // the identity check, which keeps the fail-closed rule: no runner, no code runs.
    #[tokio::test]
    async fn an_accepted_socket_with_no_daemon_behind_it_is_refused() {
        let error = production_slot(
            Path::new("/w"),
            Some("/run/user/1/counterspell-no-such.sock"),
        )
        .await
        .err()
        .expect("an unreachable daemon is refused");
        assert!(matches!(error, ConfigError::Unavailable(_)), "{error:?}");
    }
}
