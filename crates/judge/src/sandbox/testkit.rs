//! A scripted sandbox backend for tests. It needs no Docker.
//!
//! Every backend call (create, start, exec, open, write, read, kill, remove) counts as
//! one call, starting at 1. A fault can be set for one call number, so a test can make
//! exactly the n-th daemon call fail. The ledger records every container created,
//! killed and removed, and every line written to a driver, so tests can check cleanup
//! and what the program was given.
//!
//! Scripted driver behaviour is pure data. A `Hang` never produces an event, so the
//! caller must bring its own deadline, as the real call runner does.

use std::collections::VecDeque;
use std::future::{pending, ready, Future};
use std::sync::{Arc, Mutex, MutexGuard, PoisonError};
use std::time::Duration;

use super::backend::{
    Backend, BackendSession, DriverChannel, DriverEvent, ExecOutput, InfraError, ManagedContainer,
};
use super::config::OciRuntime;
use super::policy::{ContainerPolicy, SeccompProfile};
use super::spec::{ExecSpec, SessionSpec};
use crate::languages::LanguageDefinition;

/// What the scripted program does on its next read.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Behaviour {
    /// Answers with these bytes as one line.
    Answer(Vec<u8>),
    /// Writes this many bytes with no newline, as a flood would.
    Flood(usize),
    /// Exits with this code and this kernel OOM-kill count.
    Exit {
        /// The exit code.
        code: i64,
        /// The OOM-kill count.
        oom_kill: u64,
    },
    /// Writes these bytes to stderr, then continues with the next behaviour.
    Stderr(Vec<u8>),
    /// Never produces output.
    Hang,
    /// Panics inside the worker's read, as a bug in the worker would.
    Panic,
}

/// A record of what the fake backend did.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Ledger {
    created: Vec<String>,
    removed: Vec<String>,
    killed: Vec<String>,
    inputs: Vec<Vec<u8>>,
}

impl Ledger {
    /// How many containers were created.
    #[must_use]
    pub fn created(&self) -> usize {
        self.created.len()
    }

    /// The names of the containers created, in order.
    #[must_use]
    pub fn created_names(&self) -> Vec<String> {
        self.created.clone()
    }

    /// How many containers were removed.
    #[must_use]
    pub fn removed(&self) -> usize {
        self.removed.len()
    }

    /// How many containers were killed.
    #[must_use]
    pub fn killed(&self) -> usize {
        self.killed.len()
    }

    /// Containers created and not yet removed. Zero means every container was cleaned up.
    #[must_use]
    pub fn live(&self) -> usize {
        self.created
            .iter()
            .filter(|name| !self.removed.contains(name))
            .count()
    }

    /// Every line written to a driver's stdin, in order.
    #[must_use]
    pub fn inputs(&self) -> Vec<Vec<u8>> {
        self.inputs.clone()
    }
}

/// A fault that fires on one call number.
#[derive(Debug)]
struct Fault {
    at_call: usize,
    error: InfraError,
}

#[derive(Debug, Default)]
struct State {
    calls: usize,
    fault: Option<Fault>,
    program: VecDeque<Behaviour>,
    exec_exits: VecDeque<i64>,
    exec_exit: i64,
    exec_stdout: VecDeque<Vec<u8>>,
    exec_overflow: bool,
    readback: Option<ContainerPolicy>,
    block_writes: bool,
    fired_on: Option<&'static str>,
    ledger: Ledger,
    /// The unix time stamped on created containers, for sweeper tests.
    clock: i64,
    /// Live managed containers: name -> creation time. Only containers created with the
    /// `counterspell.managed=true` label are listed, as the daemon's label filter does.
    live: std::collections::BTreeMap<String, i64>,
}

/// A backend that follows a script. Clones share the same state.
#[derive(Debug, Clone, Default)]
pub struct FakeBackend {
    state: Arc<Mutex<State>>,
}

impl FakeBackend {
    /// A fresh backend with no faults, an empty program script, and exec exit code 0.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// A copy of the ledger as it is now.
    #[must_use]
    pub fn ledger(&self) -> Ledger {
        self.lock().ledger.clone()
    }

    /// Makes call number `at_call` (counting from 1) fail with `error`. The fault fires once.
    pub fn fail_call(&self, at_call: usize, error: InfraError) {
        self.lock().fault = Some(Fault { at_call, error });
    }

    /// Queues behaviours for the scripted program, consumed one per driver read.
    pub fn script_program(&self, behaviours: impl IntoIterator<Item = Behaviour>) {
        self.lock().program.extend(behaviours);
    }

    /// Sets the exit code that every `exec` returns, once the scripted exits are used up.
    pub fn set_exec_exit(&self, code: i64) {
        self.lock().exec_exit = code;
    }

    /// Queues exit codes for the next `exec` calls, in order. An exec after the queue
    /// is empty returns the code set by [`FakeBackend::set_exec_exit`].
    pub fn script_exec_exits(&self, codes: impl IntoIterator<Item = i64>) {
        self.lock().exec_exits.extend(codes);
    }

    /// Queues stdout payloads for the next `exec` calls, in order. An exec after the
    /// queue is empty returns no stdout.
    pub fn script_exec_stdout(&self, payloads: impl IntoIterator<Item = Vec<u8>>) {
        self.lock().exec_stdout.extend(payloads);
    }

    /// The operation an injected fault fired on, or `None` if no fault has fired.
    /// Operations are named after the backend method: `create`, `remove`, `write`, and so on.
    #[must_use]
    pub fn fault_fired_on(&self) -> Option<&'static str> {
        self.lock().fired_on
    }

    /// Makes every driver write wait forever, as a program that never reads its stdin does.
    pub fn set_block_writes(&self, blocked: bool) {
        self.lock().block_writes = blocked;
    }

    /// Makes every exec report that its output overflowed the cap, as a flooding compile would.
    pub fn set_exec_overflow(&self, overflowed: bool) {
        self.lock().exec_overflow = overflowed;
    }

    /// Makes every read-back return `policy` instead of the policy the container was created with.
    pub fn set_readback(&self, policy: ContainerPolicy) {
        self.lock().readback = Some(policy);
    }

    /// Sets the unix time stamped on containers created after this call, so sweeper
    /// tests control container ages without waiting.
    pub fn set_clock(&self, unix_seconds: i64) {
        self.lock().clock = unix_seconds;
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        // A poisoned lock only happens after a test panic. The state is still readable.
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Counts one call and returns the injected fault if this is its number.
    fn tick(&self, operation: &'static str) -> Result<(), InfraError> {
        let mut state = self.lock();
        state.calls += 1;
        let calls = state.calls;
        match &state.fault {
            Some(fault) if fault.at_call == calls => {
                let error = fault.error.clone();
                state.fault = None;
                state.fired_on = Some(operation);
                Err(error)
            }
            _ => Ok(()),
        }
    }
}

impl Backend for FakeBackend {
    type Session = FakeSession;

    fn create(
        &self,
        spec: &SessionSpec,
    ) -> impl Future<Output = Result<FakeSession, InfraError>> + Send {
        let result = self.tick("create").map(|()| {
            let mut state = self.lock();
            state.ledger.created.push(spec.name.clone());
            if spec
                .policy
                .labels
                .get("counterspell.managed")
                .map(String::as_str)
                == Some("true")
            {
                let clock = state.clock;
                state.live.insert(spec.name.clone(), clock);
            }
            FakeSession {
                backend: self.clone(),
                name: spec.name.clone(),
                policy: spec.policy.clone(),
            }
        });
        ready(result)
    }

    fn list_managed(
        &self,
    ) -> impl Future<Output = Result<Vec<ManagedContainer>, InfraError>> + Send {
        let result = self.tick("list_managed").map(|()| {
            self.lock()
                .live
                .iter()
                .map(|(name, created_at)| ManagedContainer {
                    id: name.clone(),
                    name: name.clone(),
                    created_at: *created_at,
                })
                .collect()
        });
        ready(result)
    }

    fn remove_managed(&self, id: &str) -> impl Future<Output = Result<(), InfraError>> + Send {
        let result = self.tick("remove_managed").map(|()| {
            let mut state = self.lock();
            state.live.remove(id);
            state.ledger.removed.push(id.to_owned());
        });
        ready(result)
    }
}

/// One fake container.
#[derive(Debug)]
pub struct FakeSession {
    backend: FakeBackend,
    name: String,
    policy: ContainerPolicy,
}

impl BackendSession for FakeSession {
    type Driver = FakeDriver;

    fn name(&self) -> &str {
        &self.name
    }

    fn read_policy(&mut self) -> impl Future<Output = Result<ContainerPolicy, InfraError>> + Send {
        let result = self.backend.tick("read_policy").map(|()| {
            self.backend
                .lock()
                .readback
                .clone()
                .unwrap_or_else(|| self.policy.clone())
        });
        ready(result)
    }

    fn start(&mut self) -> impl Future<Output = Result<(), InfraError>> + Send {
        ready(self.backend.tick("start"))
    }

    fn exec(
        &mut self,
        _spec: &ExecSpec,
        _stdin: &[u8],
    ) -> impl Future<Output = Result<ExecOutput, InfraError>> + Send {
        let result = self.backend.tick("exec").map(|()| {
            let mut state = self.backend.lock();
            let exit_code = state.exec_exits.pop_front().unwrap_or(state.exec_exit);
            ExecOutput {
                exit_code,
                stdout: state.exec_stdout.pop_front().unwrap_or_default(),
                stderr: Vec::new(),
                overflowed: state.exec_overflow,
            }
        });
        ready(result)
    }

    fn open_driver(
        &mut self,
        _spec: &ExecSpec,
    ) -> impl Future<Output = Result<FakeDriver, InfraError>> + Send {
        let result = self.backend.tick("open_driver").map(|()| FakeDriver {
            backend: self.backend.clone(),
        });
        ready(result)
    }

    fn kill(&mut self) -> impl Future<Output = Result<(), InfraError>> + Send {
        let result = self.backend.tick("kill").map(|()| {
            let mut state = self.backend.lock();
            state.ledger.killed.push(self.name.clone());
            state.live.remove(&self.name);
        });
        ready(result)
    }

    fn remove(&mut self) -> impl Future<Output = Result<(), InfraError>> + Send {
        let result = self.backend.tick("remove").map(|()| {
            let mut state = self.backend.lock();
            state.ledger.removed.push(self.name.clone());
            state.live.remove(&self.name);
        });
        ready(result)
    }
}

/// The fake driver's stdin and stdout, driven by the scripted program.
#[derive(Debug)]
pub struct FakeDriver {
    backend: FakeBackend,
}

impl DriverChannel for FakeDriver {
    async fn write_line(&mut self, line: &[u8]) -> Result<(), InfraError> {
        self.backend.tick("write")?;
        if self.backend.lock().block_writes {
            return pending().await;
        }
        self.backend.lock().ledger.inputs.push(line.to_vec());
        Ok(())
    }

    async fn next_event(&mut self) -> Result<DriverEvent, InfraError> {
        self.backend.tick("read")?;
        match self.take_behaviour() {
            Some(event) => Ok(event),
            // A hang, or an empty script, never produces output. The caller's deadline decides.
            None => pending().await,
        }
    }
}

impl FakeDriver {
    /// Pops the next scripted behaviour. `None` means the program hangs.
    fn take_behaviour(&self) -> Option<DriverEvent> {
        let behaviour = self.backend.lock().program.pop_front();
        match behaviour? {
            Behaviour::Answer(bytes) => Some(DriverEvent::Line(bytes)),
            Behaviour::Flood(size) => Some(DriverEvent::Chunk(vec![b'x'; size])),
            Behaviour::Exit { code, oom_kill } => Some(DriverEvent::Exited { code, oom_kill }),
            Behaviour::Stderr(bytes) => Some(DriverEvent::Stderr(bytes)),
            Behaviour::Hang => None,
            Behaviour::Panic => panic!("scripted panic in the driver read"),
        }
    }
}

/// A named control that [`weakened_policy`] removes. Only under `testkit`: no production
/// path can ask for a weakened policy.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WeakenedControl {
    /// The container keeps a network interface.
    Network,
    /// The root filesystem stays writable.
    WritableRoot,
    /// No memory limit is set.
    MemoryLimit,
    /// No PID limit is set.
    PidsLimit,
    /// No CPU quota is set.
    CpuQuota,
    /// No seccomp filter applies.
    Seccomp,
    /// Capabilities are not dropped.
    Capabilities,
    /// `no-new-privileges` is not set.
    NoNewPrivileges,
}

/// A production-shaped policy with the named controls weakened, for tests that must show
/// the probe and the hardening checks actually fail (for example
/// `sbx_preflight_fails_when_policy_weakened` and the S12 network control run).
#[must_use]
pub fn weakened_policy(
    lang: &LanguageDefinition,
    run_cap: Duration,
    runtime: OciRuntime,
    controls: &[WeakenedControl],
) -> ContainerPolicy {
    let mut policy = ContainerPolicy::for_language(lang, run_cap, runtime);
    for control in controls {
        match control {
            WeakenedControl::Network => "bridge".clone_into(&mut policy.network),
            WeakenedControl::WritableRoot => policy.read_only_rootfs = false,
            WeakenedControl::MemoryLimit => {
                policy.limits.memory_bytes = 0;
                policy.limits.memory_swap_bytes = 0;
            }
            WeakenedControl::PidsLimit => policy.limits.pids_limit = 0,
            WeakenedControl::CpuQuota => policy.limits.nano_cpus = 0,
            WeakenedControl::Seccomp => policy.seccomp = SeccompProfile::Unconfined,
            WeakenedControl::Capabilities => policy.cap_drop = Vec::new(),
            WeakenedControl::NoNewPrivileges => policy.security_opts = Vec::new(),
        }
    }
    policy
}

/// Problems seeded in memory for executor tests. The `testkit` feature is off in
/// release builds, so this loader never runs in production.
#[derive(Debug, Clone, Default)]
pub struct StaticProblemTests {
    specs: std::collections::HashMap<String, super::problem::ProblemSpec>,
}

impl StaticProblemTests {
    /// A loader holding `specs`, keyed by slug.
    #[must_use]
    pub fn new(specs: impl IntoIterator<Item = super::problem::ProblemSpec>) -> Self {
        Self {
            specs: specs
                .into_iter()
                .map(|spec| (spec.slug.clone(), spec))
                .collect(),
        }
    }
}

impl super::problem::ProblemTests for StaticProblemTests {
    fn load(
        &self,
        slug: &str,
        version: u32,
        mode: counterspell_common::status::RunMode,
    ) -> impl Future<
        Output = Result<Option<super::problem::ProblemSpec>, super::problem::ProblemError>,
    > + Send {
        // One spec per slug, so a pinned version that does not match is simply absent,
        // the same answer a real store gives for a version it does not have.
        let spec = self
            .specs
            .get(slug)
            .filter(|spec| spec.version == version)
            .map(|spec| super::problem::tests_for_mode(spec, mode));
        ready(Ok(spec))
    }
}

/// Loads problems from `<dir>/<slug>.json` fixtures, for tests that want the real file
/// shape instead of a seeded struct. Only file names that are valid slugs are read, so a
/// slug can never escape the directory.
#[derive(Debug, Clone)]
pub struct DirProblemTests {
    dir: std::path::PathBuf,
}

impl DirProblemTests {
    /// A loader rooted at `dir`, for example `tests/fixtures/problems`.
    #[must_use]
    pub fn new(dir: impl Into<std::path::PathBuf>) -> Self {
        Self { dir: dir.into() }
    }
}

/// The on-disk shape of a fixture problem.
#[derive(serde::Deserialize)]
struct FixtureSpec {
    slug: String,
    version: u32,
    signature: serde_json::Value,
    comparator: String,
    time_limit_ms: Option<u32>,
    tests: Vec<FixtureTest>,
    reference_python: Option<String>,
    bench: Option<FixtureBench>,
    /// The input schema a submit ghost hunt generates candidates from.
    #[serde(default)]
    input_schema: Option<counterspell_engine::Schema>,
    /// Authored known ghosts the fixture wants the hunt to seed and match.
    #[serde(default)]
    known_ghosts: Vec<counterspell_engine::KnownGhost>,
    /// Concept tags copied into a found ghost's autopsy.
    #[serde(default)]
    concept_tags: Vec<String>,
}

/// A fixture's bench declaration: sizes, the growing parameter, the declared class
/// and the generator schema — atomic, so a partial declaration cannot load.
#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct FixtureBench {
    sizes: Vec<u64>,
    size_param: usize,
    reference: counterspell_common::familiar::class::ClassId,
    schema: counterspell_engine::Schema,
}

/// One fixture test.
#[derive(serde::Deserialize)]
struct FixtureTest {
    ordinal: u32,
    is_sample: bool,
    args: Vec<serde_json::Value>,
    expected: serde_json::Value,
}

/// True when `slug` is safe to use as a file name inside the fixture directory.
fn safe_slug(slug: &str) -> bool {
    !slug.is_empty()
        && slug
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_')
}

impl super::problem::ProblemTests for DirProblemTests {
    fn load(
        &self,
        slug: &str,
        version: u32,
        mode: counterspell_common::status::RunMode,
    ) -> impl Future<
        Output = Result<Option<super::problem::ProblemSpec>, super::problem::ProblemError>,
    > + Send {
        ready(self.load_sync(slug, version, mode))
    }
}

impl DirProblemTests {
    /// The synchronous body of `load`; kept separate so `load` stays an `impl Future`.
    fn load_sync(
        &self,
        slug: &str,
        version: u32,
        mode: counterspell_common::status::RunMode,
    ) -> Result<Option<super::problem::ProblemSpec>, super::problem::ProblemError> {
        if !safe_slug(slug) {
            return Ok(None);
        }
        let path = self.dir.join(format!("{slug}.json"));
        let text = match std::fs::read_to_string(&path) {
            Ok(text) => text,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => {
                return Err(super::problem::ProblemError::Store(error.to_string()));
            }
        };
        let raw: FixtureSpec = serde_json::from_str(&text)
            .map_err(|error| super::problem::ProblemError::Store(error.to_string()))?;
        // An unsupported comparator is an absent problem, never a guess.
        let comparator = match raw.comparator.as_str() {
            "exact" => counterspell_engine::Comparator::Exact,
            "unordered" => counterspell_engine::Comparator::Unordered,
            other => {
                tracing::warn!(%other, slug, "fixture uses a comparator the judge does not support");
                return Ok(None);
            }
        };
        let spec = super::problem::ProblemSpec {
            slug: raw.slug,
            version: raw.version,
            signature: raw.signature,
            comparator,
            time_limit_ms: raw.time_limit_ms,
            tests: raw
                .tests
                .into_iter()
                .map(|test| super::problem::TestCase {
                    ordinal: test.ordinal,
                    is_sample: test.is_sample,
                    args: test.args,
                    expected: test.expected,
                })
                .collect(),
            reference_python: raw.reference_python,
            bench: raw.bench.as_ref().map(|bench| super::problem::BenchDecl {
                sizes: bench.sizes.clone(),
                size_param: bench.size_param,
            }),
            generator_schema: raw.bench.as_ref().map(|bench| bench.schema.clone()),
            reference_complexity: raw.bench.map(|bench| bench.reference),
            input_schema: raw.input_schema,
            known_ghosts: raw.known_ghosts,
            concept_tags: raw.concept_tags,
        };
        // A fixture pins one version; a run pinned to another version finds nothing,
        // exactly like a store that only holds the versions it imported.
        if spec.version != version {
            return Ok(None);
        }
        Ok(Some(super::problem::tests_for_mode(&spec, mode)))
    }
}
