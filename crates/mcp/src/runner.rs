//! The runner slot that code-running tools use.
//!
//! The typed contract is Grimoire's [`PackRunner`]: a trusted runner executes the
//! reference or one incorrect program on one input. Tests inject a fake runner. The
//! production path needs the sandbox socket to pass the guard in [`crate::config`].
//! The judge's production runner is not built yet, so the production slot stays
//! unconfigured and every code-running tool returns `sandbox_not_configured`.

use crate::config::{sandbox_socket, ConfigError};
use counterspell_grimoire::runner::PackRunner;

/// Holds the runner for code-running tools, or nothing when no runner is available.
pub struct RunnerSlot {
    runner: Option<Box<dyn PackRunner + Send>>,
}

impl RunnerSlot {
    /// A slot with no runner. Code-running tools fail closed with `NotConfigured`.
    #[must_use]
    pub fn unconfigured() -> Self {
        Self { runner: None }
    }

    /// A slot holding `runner`. Used by tests with a fake runner.
    #[must_use]
    pub fn injected(runner: Box<dyn PackRunner + Send>) -> Self {
        Self {
            runner: Some(runner),
        }
    }

    /// Returns the runner for one code-running call.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigError::NotConfigured`] when the slot holds no runner.
    pub fn get(&mut self) -> Result<&mut dyn PackRunner, ConfigError> {
        match self.runner.as_mut() {
            Some(runner) => Ok(runner.as_mut()),
            None => Err(ConfigError::NotConfigured),
        }
    }
}

/// Builds the production slot. The raw socket must pass the sandbox guard first.
///
/// Until the judge provides a production runner, the returned slot is unconfigured.
///
/// # Errors
///
/// Returns [`ConfigError`] when the socket is missing or refused by the guard.
pub fn production_slot(raw_socket: Option<&str>) -> Result<RunnerSlot, ConfigError> {
    sandbox_socket(raw_socket)?;
    Ok(RunnerSlot::unconfigured())
}

#[cfg(test)]
mod tests {
    use super::*;
    use counterspell_engine::ProgramResult;
    use counterspell_grimoire::runner::{RunnerError, Target};
    use serde_json::Value;
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

    #[test]
    fn production_needs_an_accepted_sandbox_socket() {
        assert!(matches!(
            production_slot(None),
            Err(ConfigError::NotConfigured)
        ));
        assert!(matches!(
            production_slot(Some("/var/run/docker.sock")),
            Err(ConfigError::Rejected(_))
        ));
    }

    #[test]
    fn production_with_an_accepted_socket_still_fails_closed_until_the_judge_runner_exists() {
        let mut slot =
            production_slot(Some("/run/user/1000/docker.sock")).expect("accepted socket");
        assert!(matches!(slot.get(), Err(ConfigError::NotConfigured)));
    }
}
