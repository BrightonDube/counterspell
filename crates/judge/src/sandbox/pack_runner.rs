// SPDX-License-Identifier: MIT
//! The judge's production [`PackRunner`]: pack programs run inside the
//! rootless sandbox, one lazily-opened session per target.
//!
//! It lives in the judge so the catalogue importer and `counterspell-mcp` share one runner (same
//! images, limits, driver and cleanup) without the judge depending on Grimoire. The catalogue
//! importer's sandbox path is a caller. The sources it holds
//! are the pack's public `reference/` and `incorrect/` files only — hidden
//! inputs and private files never enter a runner archive. Construction happens
//! once per pack (sources differ between packs); a session then opens on the
//! first call to a target and rebuilds after an abnormal end, exactly like
//! [`SandboxOracle`](super::oracle::SandboxOracle)'s sides.

use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

use counterspell_engine::runner::{PackRunner, RunnerError, Target};
use counterspell_engine::ProgramResult;
use serde_json::Value;
use tokio::runtime::Handle;
use tokio::sync::watch;

use super::backend::Backend;
use super::bench::SUBMIT_BENCH_BUDGET;
use super::config::SandboxConfig;
use super::oracle::{bench_side, evaluate_side, side, Role, Side};
use crate::familiar::BenchOutcome;
use crate::languages::LanguageDefinition;

#[cfg(feature = "sandbox")]
use super::bollard_backend::BollardBackend;
#[cfg(feature = "sandbox")]
use super::daemon::connect;
#[cfg(feature = "sandbox")]
use crate::languages::LanguageCatalog;
#[cfg(feature = "sandbox")]
use tokio::runtime::Runtime;

/// The source-map key for the pack's trusted reference.
pub const REFERENCE_SOURCE: &str = "reference";

/// Runs pack programs inside sandbox containers, one side per target.
///
/// `run` is synchronous by contract, so the runner drives the sandbox through
/// `block_on` on a [`Handle`] the caller keeps alive — the same bridge the
/// oracle uses from `spawn_blocking`.
pub struct PackSandboxRunner<B: Backend> {
    handle: Handle,
    backend: B,
    lang: LanguageDefinition,
    driver_root: PathBuf,
    signature_bytes: Vec<u8>,
    sources: BTreeMap<String, String>,
    sides: BTreeMap<String, Side<B>>,
    config: SandboxConfig,
    stop: watch::Sender<bool>,
}

impl<B: Backend + Clone + Send> PackSandboxRunner<B> {
    /// Builds the runner without opening anything — sessions open lazily on
    /// the first call to a target, so a bad program surfaces as a
    /// [`ProgramResult`], not a constructor error.
    ///
    /// `sources` maps [`REFERENCE_SOURCE`] to the pack's trusted reference and
    /// each `incorrect/<path>` to that program's text.
    ///
    /// # Errors
    ///
    /// Returns [`RunnerError::Infrastructure`] when the signature cannot be
    /// serialised for `signature.json`.
    pub fn new(
        handle: Handle,
        backend: B,
        lang: LanguageDefinition,
        signature: &Value,
        sources: BTreeMap<String, String>,
        config: &SandboxConfig,
    ) -> Result<Self, RunnerError> {
        let signature_bytes = serde_json::to_vec(signature).map_err(|error| {
            RunnerError::Infrastructure(format!("the signature does not serialise: {error}"))
        })?;
        let driver_root = config
            .languages_file
            .parent()
            .map_or_else(|| PathBuf::from("."), std::path::Path::to_path_buf);
        let (stop, _receiver) = watch::channel(false);
        Ok(Self {
            handle,
            backend,
            lang,
            driver_root,
            signature_bytes,
            sources,
            sides: BTreeMap::new(),
            config: config.clone(),
            stop,
        })
    }

    /// Returns the target's side, building it on first use. A missing source
    /// means the caller named a program the pack does not carry.
    fn side_for(&mut self, key: &str, role: Role) -> Result<&mut Side<B>, RunnerError> {
        if !self.sides.contains_key(key) {
            let Some(source) = self.sources.get(key) else {
                return Err(RunnerError::Infrastructure(format!(
                    "the pack has no source for {key}"
                )));
            };
            let built = side(
                self.backend.clone(),
                self.lang.clone(),
                &self.driver_root,
                &self.signature_bytes,
                source,
                &self.config,
                role,
            )
            .map_err(|error| RunnerError::Infrastructure(error.to_string()))?;
            self.sides.insert(key.to_owned(), built);
        }
        self.sides.get_mut(key).ok_or_else(|| {
            RunnerError::Infrastructure("the sandbox side was not stored".to_owned())
        })
    }
}

impl<B: Backend + Clone + Send> PackRunner for PackSandboxRunner<B> {
    fn run(
        &mut self,
        target: Target<'_>,
        input: &Value,
        limit: Duration,
    ) -> Result<ProgramResult, RunnerError> {
        let (key, role) = match target {
            Target::Reference => (REFERENCE_SOURCE, Role::Reference),
            // Incorrect programs are untrusted code: a compile failure is the
            // program's own result, never an infrastructure fault.
            Target::Program(path) => (path, Role::User),
        };
        let handle = self.handle.clone();
        let kill_slack = self.config.kill_slack;
        let stop = self.stop.subscribe();
        let side = self.side_for(key, role)?;
        let args: Vec<Value> = match input {
            Value::Array(items) => items.clone(),
            single => vec![single.clone()],
        };
        // `run` is sync but its callers are `async` — `block_on` panics on a
        // runtime worker thread no matter which runtime it belongs to, so the
        // call is driven from a scoped plain thread instead.
        let call = std::thread::scope(|scope| {
            scope
                .spawn(|| handle.block_on(evaluate_side(side, &args, limit, kill_slack, stop)))
                .join()
        });
        match call {
            Ok(result) => result.map_err(|error| RunnerError::Infrastructure(error.to_string())),
            Err(_panic) => Err(RunnerError::Infrastructure(
                "the pack runner's sandbox call panicked".to_owned(),
            )),
        }
    }

    /// Times the reference through the same bench collector a learner's Submit uses: two
    /// warm-ups and five timed calls per size, the worker clock and the Submit budget. Returns
    /// one `(n, median_ns)` pair per size. A reference that fails, or a bench that runs out of
    /// budget, means the pack's bench is wrong for its own reference, so it is reported as
    /// infrastructure and the caller refuses the pack.
    fn bench(
        &mut self,
        sized_inputs: &[(u64, Value)],
        limit: Duration,
    ) -> Result<Vec<(u64, u64)>, RunnerError> {
        let handle = self.handle.clone();
        let stop = self.stop.subscribe();
        let side = self.side_for(REFERENCE_SOURCE, Role::Reference)?;
        // `bench` is sync but its callers are `async`; drive it from a scoped plain thread,
        // exactly like `run`.
        let call = std::thread::scope(|scope| {
            scope
                .spawn(|| {
                    handle.block_on(bench_side(
                        side,
                        sized_inputs,
                        limit,
                        SUBMIT_BENCH_BUDGET,
                        stop,
                    ))
                })
                .join()
        });
        let outcome = match call {
            Ok(result) => result.map_err(|error| RunnerError::Infrastructure(error.to_string()))?,
            Err(_panic) => {
                return Err(RunnerError::Infrastructure(
                    "the pack runner's bench panicked".to_owned(),
                ))
            }
        };
        match outcome {
            BenchOutcome::Complete(points) => Ok(points
                .iter()
                .map(|point| (point.n, point.median_ns))
                .collect()),
            BenchOutcome::Incomplete(points) => Err(RunnerError::Infrastructure(format!(
                "the bench budget ran out after {} of {} sizes",
                points.len(),
                sized_inputs.len()
            ))),
            BenchOutcome::Failed => Err(RunnerError::Infrastructure(
                "the reference failed during the bench".to_owned(),
            )),
        }
    }
}

/// Everything a catalogue command needs to build a [`PackSandboxRunner`] per
/// pack: the daemon client, a private runtime the runner's `block_on` calls
/// drive, and the reference language (`python` — pack programs are always the
/// reference language).
#[cfg(feature = "sandbox")]
pub struct PackRunnerFactory {
    /// `Option` so `drop` can detach the runtime — `Runtime`'s own drop panics
    /// inside an async context, and the commands hold the factory in `async fn`s.
    runtime: Option<Runtime>,
    config: SandboxConfig,
    backend: BollardBackend,
    lang: LanguageDefinition,
}

#[cfg(feature = "sandbox")]
impl PackRunnerFactory {
    /// Reads `SANDBOX_DOCKER_SOCKET` and the rest of [`SandboxConfig`]. Any
    /// missing or unusable value, an unreadable language catalogue, or a
    /// disabled `python` entry means `None` — the commands report `SBX-07`.
    #[must_use]
    pub fn from_env() -> Option<Self> {
        let config = SandboxConfig::from_env().ok()?;
        let docker = connect(&config.docker_socket, config.api_timeout).ok()?;
        let backend = BollardBackend::new(docker, config.api_timeout);
        let catalog = LanguageCatalog::load(&config.languages_file).ok()?;
        let lang = catalog
            .get("python")
            .filter(|lang| lang.is_enabled())?
            .clone();
        // A current_thread runtime does not drive its I/O or timer drivers
        // from `Handle::block_on` on another thread, which is how `run`
        // reaches the daemon; a single multi-thread worker does.
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .ok()?;
        Some(Self {
            runtime: Some(runtime),
            config,
            backend,
            lang,
        })
    }

    /// Builds the runner for one pack's public sources.
    ///
    /// # Errors
    ///
    /// Returns [`RunnerError::Infrastructure`] when the signature cannot be
    /// serialised.
    pub fn build(
        &self,
        sources: BTreeMap<String, String>,
        signature: &Value,
    ) -> Result<PackSandboxRunner<BollardBackend>, RunnerError> {
        let Some(runtime) = self.runtime.as_ref() else {
            return Err(RunnerError::Infrastructure(
                "the pack runner factory has shut down".to_owned(),
            ));
        };
        PackSandboxRunner::new(
            runtime.handle().clone(),
            self.backend.clone(),
            self.lang.clone(),
            signature,
            sources,
            &self.config,
        )
    }
}

#[cfg(feature = "sandbox")]
impl Drop for PackRunnerFactory {
    /// Detaches the runtime's shutdown: `Runtime::drop` blocks and panics inside
    /// an async context, while the commands keep the factory across `.await`s.
    fn drop(&mut self) {
        if let Some(runtime) = self.runtime.take() {
            runtime.shutdown_background();
        }
    }
}
