//! Fault-injection properties for run orchestration (requirements R9.4, R9.5, R10.1,
//! R10.7, R11.3). P8: the ledger balances after every way a run can end. P9: a fault at any
//! daemon call is an infrastructure error and never a verdict. All runs use the scripted
//! fake backend, so no Docker is involved.

use std::time::Duration;

use counterspell_engine::Comparator;
use counterspell_judge::languages::LanguageCatalog;
use counterspell_judge::sandbox::backend::InfraError;
use counterspell_judge::sandbox::compare::Expected;
use counterspell_judge::sandbox::config::OciRuntime;
use counterspell_judge::sandbox::run::{execute_run, PlannedTest, RunResult, RunSettings};
use counterspell_judge::sandbox::spec::SessionSpec;
use counterspell_judge::sandbox::testkit::{Behaviour, FakeBackend};
use counterspell_judge::sandbox::verdict::FinalVerdict;
use proptest::prelude::*;
use serde_json::{json, Value};
use tempfile::TempDir;
use tokio::sync::watch;
use tokio::time::Instant;
use uuid::Uuid;

mod support;
use support::shipped_catalogue;

const ARCHIVE: &[u8] = b"fixed archive bytes";
const STDOUT_BUDGET: usize = 1024 * 1024;

fn catalogue() -> (TempDir, LanguageCatalog) {
    shipped_catalogue(false)
}

fn hidden(ordinal: u32, expected: Value, time_ms: u64) -> PlannedTest {
    PlannedTest {
        ordinal,
        sample: false,
        args: vec![json!(ordinal)],
        expected: Expected::new(expected),
        comparator: Comparator::Exact,
        time_limit: Duration::from_millis(time_ms),
    }
}

fn correct_answer(ordinal: u32) -> Behaviour {
    Behaviour::Answer(format!("{{\"t\":{ordinal},\"out\":{ordinal}}}\n").into_bytes())
}

/// Runs `plan` on `backend` as the run `run`.
async fn execute(
    backend: &FakeBackend,
    plan: Vec<PlannedTest>,
    run: Uuid,
    stop: &mut watch::Receiver<bool>,
) -> Result<RunResult, InfraError> {
    let (_dir, catalog) = catalogue();
    let lang = catalog.get("python").unwrap();
    let make_spec = |ordinal: u32| {
        SessionSpec::new(
            lang,
            Duration::from_secs(20),
            OciRuntime::Runc,
            run,
            "worker-1",
            ordinal,
            1_800_000_000,
        )
        .map_err(|error| InfraError::Unexpected(error.to_string()))
    };
    let settings = RunSettings {
        run_deadline: Instant::now() + Duration::from_secs(10),
        kill_slack: Duration::from_millis(500),
        max_consecutive_abnormal: 3,
    };
    execute_run(backend, lang, ARCHIVE, make_spec, plan, settings, stop).await
}

/// Waits (for up to about 1 s) until no container is live. Cleanup from `Drop` runs on a
/// spawned task, so the check polls. A remaining container is reported to the caller.
async fn settle(backend: &FakeBackend) -> usize {
    for _ in 0..200 {
        if backend.ledger().live() == 0 {
            return 0;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    backend.ledger().live()
}

fn runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// P9: a fault at any daemon call ends the run with that infrastructure error, or the
    /// run is unaffected and accepted. A fault never produces another verdict. A container
    /// is left behind only when the fault itself hit a removal, which the sweeper handles.
    #[test]
    fn a_fault_at_any_daemon_call_is_an_infrastructure_error_never_a_verdict(
        tests in 1u32..=3,
        fault_at in 1usize..=20,
    ) {
        runtime().block_on(async {
            let backend = FakeBackend::new();
            let injected = InfraError::DaemonUnreachable("injected".to_owned());
            backend.fail_call(fault_at, injected.clone());
            backend.script_program((1..=tests).map(correct_answer));
            let (_keep, mut stop) = watch::channel(false);
            let plan = (1..=tests).map(|n| hidden(n, json!(n), 1_000)).collect();

            let outcome = execute(&backend, plan, Uuid::new_v4(), &mut stop).await;

            match outcome {
                Err(error) => {
                    assert_eq!(error, injected, "only the injected fault may fail the run");
                    assert!(backend.fault_fired_on().is_some());
                }
                Ok(result) => {
                    assert_eq!(result.verdict(), FinalVerdict::Accepted);
                    assert_eq!(result.executed.len(), tests as usize);
                }
            }
            let left_behind = settle(&backend).await;
            if left_behind > 0 {
                assert_eq!(backend.fault_fired_on(), Some("remove"));
            }
        });
    }
}

/// The program behaviours P8 draws from. Each test consumes exactly one.
#[derive(Debug, Clone, Copy)]
enum Choice {
    Correct,
    Wrong,
    Hang,
    Flood,
    OomExit,
    PlainExit,
}

fn behaviour_for(choice: Choice, ordinal: u32) -> Behaviour {
    match choice {
        Choice::Correct => correct_answer(ordinal),
        Choice::Wrong => {
            Behaviour::Answer(format!("{{\"t\":{ordinal},\"out\":99}}\n").into_bytes())
        }
        Choice::Hang => Behaviour::Hang,
        Choice::Flood => Behaviour::Flood(STDOUT_BUDGET + 1),
        Choice::OomExit => Behaviour::Exit {
            code: 137,
            oom_kill: 1,
        },
        Choice::PlainExit => Behaviour::Exit {
            code: 1,
            oom_kill: 0,
        },
    }
}

fn choice() -> impl Strategy<Value = Choice> {
    prop_oneof![
        Just(Choice::Correct),
        Just(Choice::Wrong),
        Just(Choice::Hang),
        Just(Choice::Flood),
        Just(Choice::OomExit),
        Just(Choice::PlainExit),
    ]
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(24))]

    /// P8: for any mix of answers, hangs, floods and exits, every container the run
    /// created is removed by the time it returns, and none is removed twice.
    #[test]
    fn every_container_is_removed_once_for_any_program_behaviour(
        choices in proptest::collection::vec(choice(), 1..=4),
    ) {
        runtime().block_on(async {
            let backend = FakeBackend::new();
            backend.script_program((1u32..).zip(choices.iter()).map(|(ordinal, &c)| behaviour_for(c, ordinal)));
            let (_keep, mut stop) = watch::channel(false);
            let count = u32::try_from(choices.len()).unwrap();
            let plan = (1..=count).map(|n| hidden(n, json!(n), 20)).collect();

            let _result = execute(&backend, plan, Uuid::new_v4(), &mut stop).await;

            let ledger = backend.ledger();
            assert_eq!(ledger.live(), 0, "a run leaves no container behind");
            assert_eq!(ledger.created(), ledger.removed());
        });
    }
}

#[test]
fn a_run_task_that_is_cancelled_removes_its_container() {
    runtime().block_on(async {
        let backend = FakeBackend::new();
        // The program never answers and the test limit is long, so the run is still waiting.
        backend.script_program([Behaviour::Hang]);
        let task_backend = backend.clone();
        let task = tokio::spawn(async move {
            let (_keep, mut stop) = watch::channel(false);
            let plan = vec![hidden(1, json!(1), 5_000)];
            execute(&task_backend, plan, Uuid::new_v4(), &mut stop).await
        });
        for _ in 0..400 {
            if backend.ledger().created() >= 1 {
                break;
            }
            tokio::time::sleep(Duration::from_millis(5)).await;
        }

        task.abort();

        assert!(task.await.unwrap_err().is_cancelled());
        assert_eq!(settle(&backend).await, 0);
        assert_eq!(backend.ledger().created(), backend.ledger().removed());
    });
}

#[test]
fn a_panic_inside_the_run_removes_its_container() {
    runtime().block_on(async {
        let backend = FakeBackend::new();
        backend.script_program([Behaviour::Panic]);
        let task_backend = backend.clone();
        let task = tokio::spawn(async move {
            let (_keep, mut stop) = watch::channel(false);
            let plan = vec![hidden(1, json!(1), 1_000)];
            execute(&task_backend, plan, Uuid::new_v4(), &mut stop).await
        });

        assert!(task.await.unwrap_err().is_panic());
        assert_eq!(settle(&backend).await, 0);
        assert_eq!(backend.ledger().created(), backend.ledger().removed());
    });
}

#[test]
fn two_runs_never_share_a_container_name() {
    runtime().block_on(async {
        let backend = FakeBackend::new();
        backend.script_program([correct_answer(1), correct_answer(1)]);
        let first_run = Uuid::new_v4();
        let second_run = Uuid::new_v4();
        let (_keep, mut stop) = watch::channel(false);

        execute(
            &backend,
            vec![hidden(1, json!(1), 1_000)],
            first_run,
            &mut stop,
        )
        .await
        .unwrap();
        execute(
            &backend,
            vec![hidden(1, json!(1), 1_000)],
            second_run,
            &mut stop,
        )
        .await
        .unwrap();

        let names = backend.ledger().created_names();
        assert_eq!(names.len(), 2);
        assert_ne!(names[0], names[1]);
        assert!(names[0].contains(&first_run.to_string()));
        assert!(names[1].contains(&second_run.to_string()));
        assert_eq!(backend.ledger().live(), 0);
    });
}
