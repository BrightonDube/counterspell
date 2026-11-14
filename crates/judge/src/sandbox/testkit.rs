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

use super::backend::{Backend, BackendSession, DriverChannel, DriverEvent, ExecOutput, InfraError};
use super::spec::{ExecSpec, SessionSpec};

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
    /// Never produces output.
    Hang,
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
    exec_exit: i64,
    ledger: Ledger,
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

    /// Sets the exit code that every `exec` returns.
    pub fn set_exec_exit(&self, code: i64) {
        self.lock().exec_exit = code;
    }

    fn lock(&self) -> MutexGuard<'_, State> {
        // A poisoned lock only happens after a test panic. The state is still readable.
        self.state.lock().unwrap_or_else(PoisonError::into_inner)
    }

    /// Counts one call and returns the injected fault if this is its number.
    fn tick(&self) -> Result<(), InfraError> {
        let mut state = self.lock();
        state.calls += 1;
        let calls = state.calls;
        match &state.fault {
            Some(fault) if fault.at_call == calls => {
                let error = fault.error.clone();
                state.fault = None;
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
        let result = self.tick().map(|()| {
            self.lock().ledger.created.push(spec.name.clone());
            FakeSession {
                backend: self.clone(),
                name: spec.name.clone(),
            }
        });
        ready(result)
    }
}

/// One fake container.
#[derive(Debug)]
pub struct FakeSession {
    backend: FakeBackend,
    name: String,
}

impl BackendSession for FakeSession {
    type Driver = FakeDriver;

    fn name(&self) -> &str {
        &self.name
    }

    fn start(&mut self) -> impl Future<Output = Result<(), InfraError>> + Send {
        ready(self.backend.tick())
    }

    fn exec(
        &mut self,
        _spec: &ExecSpec,
        _stdin: &[u8],
    ) -> impl Future<Output = Result<ExecOutput, InfraError>> + Send {
        let result = self.backend.tick().map(|()| ExecOutput {
            exit_code: self.backend.lock().exec_exit,
            stdout: Vec::new(),
            stderr: Vec::new(),
        });
        ready(result)
    }

    fn open_driver(
        &mut self,
        _spec: &ExecSpec,
    ) -> impl Future<Output = Result<FakeDriver, InfraError>> + Send {
        let result = self.backend.tick().map(|()| FakeDriver {
            backend: self.backend.clone(),
        });
        ready(result)
    }

    fn kill(&mut self) -> impl Future<Output = Result<(), InfraError>> + Send {
        let result = self.backend.tick().map(|()| {
            self.backend.lock().ledger.killed.push(self.name.clone());
        });
        ready(result)
    }

    fn remove(&mut self) -> impl Future<Output = Result<(), InfraError>> + Send {
        let result = self.backend.tick().map(|()| {
            self.backend.lock().ledger.removed.push(self.name.clone());
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
    fn write_line(&mut self, line: &[u8]) -> impl Future<Output = Result<(), InfraError>> + Send {
        let result = self.backend.tick().map(|()| {
            self.backend.lock().ledger.inputs.push(line.to_vec());
        });
        ready(result)
    }

    async fn next_event(&mut self) -> Result<DriverEvent, InfraError> {
        self.backend.tick()?;
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
            Behaviour::Hang => None,
        }
    }
}
