//! Model tests for run orchestration (requirements R5.3 to R5.8, R7.3, R9.2, R13.7). They
//! use the scripted fake backend. Property P15 (sample tests before hidden tests) is at
//! the end.

use std::time::Duration;

use counterspell_engine::Comparator;
use counterspell_judge::languages::LanguageCatalog;
use counterspell_judge::sandbox::backend::InfraError;
use counterspell_judge::sandbox::compare::Expected;
use counterspell_judge::sandbox::config::OciRuntime;
use counterspell_judge::sandbox::run::{
    execute_run, schedule, PlannedTest, RunResult, RunSettings,
};
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

const RUN: &str = "00000000-0000-0000-0000-000000000005";
const ARCHIVE: &[u8] = b"fixed archive bytes";

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

fn sample(ordinal: u32, expected: Value) -> PlannedTest {
    PlannedTest {
        sample: true,
        ..hidden(ordinal, expected, 1_000)
    }
}

/// The answer line a correct program prints for call `ordinal`.
fn answer(ordinal: u32, out: &Value) -> Behaviour {
    Behaviour::Answer(format!("{{\"t\":{ordinal},\"out\":{out}}}\n").into_bytes())
}

fn settings(run_for: Duration) -> RunSettings {
    RunSettings {
        run_deadline: Instant::now() + run_for,
        kill_slack: Duration::from_millis(500),
        max_consecutive_abnormal: 3,
    }
}

async fn run(
    backend: &FakeBackend,
    plan: Vec<PlannedTest>,
    settings: RunSettings,
    stop: &mut watch::Receiver<bool>,
) -> Result<RunResult, InfraError> {
    let (_dir, catalog) = catalogue();
    let lang = catalog.get("python").unwrap();
    let make_spec = |ordinal: u32| {
        SessionSpec::new(
            lang,
            Duration::from_secs(20),
            OciRuntime::Runc,
            Uuid::parse_str(RUN).unwrap(),
            "worker-1",
            ordinal,
            1_800_000_000,
        )
        .map_err(|error| InfraError::Unexpected(error.to_string()))
    };
    execute_run(backend, lang, ARCHIVE, make_spec, plan, settings, stop).await
}

#[tokio::test]
async fn a_run_where_every_test_passes_is_accepted_and_the_container_is_removed() {
    let backend = FakeBackend::new();
    backend.script_program([
        answer(1, &json!(1)),
        answer(2, &json!(2)),
        answer(3, &json!(3)),
    ]);
    let (_keep, mut stop) = watch::channel(false);
    let plan = vec![
        hidden(1, json!(1), 1_000),
        hidden(2, json!(2), 1_000),
        hidden(3, json!(3), 1_000),
    ];

    let result = run(&backend, plan, settings(Duration::from_secs(10)), &mut stop)
        .await
        .unwrap();

    assert_eq!(result.executed.len(), 3);
    assert_eq!(result.verdict(), FinalVerdict::Accepted);
    assert_eq!(backend.ledger().live(), 0);
}

#[tokio::test]
async fn a_wrong_answer_is_recorded_and_the_same_container_keeps_running() {
    let backend = FakeBackend::new();
    backend.script_program([answer(1, &json!(99)), answer(2, &json!(2))]);
    let (_keep, mut stop) = watch::channel(false);
    let plan = vec![hidden(1, json!(1), 1_000), hidden(2, json!(2), 1_000)];

    let result = run(&backend, plan, settings(Duration::from_secs(10)), &mut stop)
        .await
        .unwrap();

    assert_eq!(
        result.executed.len(),
        2,
        "a wrong answer does not stop the run"
    );
    assert_eq!(result.verdict(), FinalVerdict::WrongAnswer);
    assert_eq!(
        backend.ledger().created(),
        1,
        "a wrong answer does not restart the program"
    );
}

#[tokio::test]
async fn timeouts_restart_in_a_new_container_and_fail_fast_after_the_consecutive_limit() {
    let backend = FakeBackend::new();
    // No scripted behaviour: every read waits, so every call hits its 50 ms limit.
    let (_keep, mut stop) = watch::channel(false);
    let plan: Vec<PlannedTest> = (1..=6).map(|n| hidden(n, json!(n), 50)).collect();

    let result = run(&backend, plan, settings(Duration::from_secs(10)), &mut stop)
        .await
        .unwrap();

    assert_eq!(result.facts.planned, 6);
    assert_eq!(
        result.executed.len(),
        3,
        "fail-fast stops after 3 abnormal tests in a row"
    );
    assert_eq!(result.verdict(), FinalVerdict::TimeLimit);
    let ledger = backend.ledger();
    assert_eq!(
        ledger.created(),
        3,
        "each abnormal test gets a fresh container"
    );
    assert_eq!(ledger.live(), 0);
}

#[tokio::test]
async fn a_run_cap_that_has_already_passed_runs_no_test_and_is_a_time_limit() {
    let backend = FakeBackend::new();
    let (_keep, mut stop) = watch::channel(false);
    let plan = vec![hidden(1, json!(1), 1_000)];

    let result = run(&backend, plan, settings(Duration::ZERO), &mut stop)
        .await
        .unwrap();

    assert!(result.executed.is_empty());
    assert!(result.facts.cut_by_cap);
    assert_eq!(result.verdict(), FinalVerdict::TimeLimit);
    assert_eq!(backend.ledger().created(), 0);
}

#[tokio::test]
async fn a_compile_failure_runs_no_test_and_removes_the_container() {
    let backend = FakeBackend::new();
    // Exit 0 for the file load, then 1 for the compile.
    backend.script_exec_exits([0, 1]);
    let (_keep, mut stop) = watch::channel(false);
    let plan = vec![hidden(1, json!(1), 1_000)];

    let result = run(&backend, plan, settings(Duration::from_secs(10)), &mut stop)
        .await
        .unwrap();

    assert!(result.executed.is_empty());
    assert!(!result.facts.compiled);
    assert_eq!(result.verdict(), FinalVerdict::CompileError);
    assert_eq!(backend.ledger().live(), 0);
}

#[tokio::test]
async fn a_stop_before_the_run_gives_stopped_and_no_test() {
    let backend = FakeBackend::new();
    let (keep, mut stop) = watch::channel(false);
    keep.send(true).unwrap();
    let plan = vec![hidden(1, json!(1), 1_000)];

    let result = run(&backend, plan, settings(Duration::from_secs(10)), &mut stop)
        .await
        .unwrap();

    assert!(result.executed.is_empty());
    assert_eq!(result.verdict(), FinalVerdict::Stopped);
}

#[tokio::test]
async fn an_infrastructure_fault_mid_run_is_an_error_and_the_container_is_removed() {
    let backend = FakeBackend::new();
    // Calls: 1 create, 2 read-back, 3 start, 4 load, 5 compile, 6 open driver, 7 first write.
    backend.fail_call(7, InfraError::DaemonUnreachable("socket gone".to_owned()));
    let (_keep, mut stop) = watch::channel(false);
    let plan = vec![hidden(1, json!(1), 1_000)];

    let result = run(&backend, plan, settings(Duration::from_secs(10)), &mut stop).await;

    assert_eq!(
        result.unwrap_err(),
        InfraError::DaemonUnreachable("socket gone".to_owned())
    );
    assert_eq!(
        backend.ledger().live(),
        0,
        "the container must be removed before the error is returned"
    );
}

#[tokio::test]
async fn a_dropped_stop_sender_counts_as_a_stop_like_the_call_runner() {
    let backend = FakeBackend::new();
    let (keep, mut stop) = watch::channel(false);
    drop(keep);
    let plan = vec![hidden(1, json!(1), 1_000)];

    let result = run(&backend, plan, settings(Duration::from_secs(10)), &mut stop)
        .await
        .unwrap();

    assert!(result.executed.is_empty());
    assert_eq!(result.verdict(), FinalVerdict::Stopped);
}

#[tokio::test]
async fn a_zero_consecutive_abnormal_limit_is_refused() {
    let backend = FakeBackend::new();
    let (_keep, mut stop) = watch::channel(false);
    let plan = vec![hidden(1, json!(1), 1_000)];
    let mut zero = settings(Duration::from_secs(10));
    zero.max_consecutive_abnormal = 0;

    let result = run(&backend, plan, zero, &mut stop).await;

    assert!(matches!(result, Err(InfraError::Unexpected(_))));
    assert_eq!(backend.ledger().created(), 0);
}

#[tokio::test]
async fn an_extra_output_line_is_a_wrong_answer_and_the_next_test_gets_a_new_container() {
    let backend = FakeBackend::new();
    backend.script_program([
        Behaviour::Answer(b"{\"t\":1,\"out\":1}\n{\"t\":1,\"out\":1}\n".to_vec()),
        answer(2, &json!(2)),
    ]);
    let (_keep, mut stop) = watch::channel(false);
    let plan = vec![hidden(1, json!(1), 1_000), hidden(2, json!(2), 1_000)];

    let result = run(&backend, plan, settings(Duration::from_secs(10)), &mut stop)
        .await
        .unwrap();

    assert_eq!(result.executed.len(), 2);
    assert_eq!(result.verdict(), FinalVerdict::WrongAnswer);
    assert_eq!(backend.ledger().created(), 2);
    assert_eq!(backend.ledger().live(), 0);
}

proptest! {
    /// P15: for any plan, every sample test is scheduled before the first hidden test, and
    /// no sample test is scheduled after it. Within each group the ordinals ascend.
    #[test]
    fn samples_always_come_before_hidden_tests(
        plan in proptest::collection::vec((0u32..40, any::<bool>()), 0..25),
    ) {
        let tests: Vec<PlannedTest> = plan
            .iter()
            .map(|&(ordinal, is_sample)| PlannedTest {
                sample: is_sample,
                ..hidden(ordinal, Value::Null, 1_000)
            })
            .collect();

        let scheduled = schedule(tests);

        let first_hidden = scheduled.iter().position(|test| !test.sample);
        if let Some(cut) = first_hidden {
            prop_assert!(scheduled[cut..].iter().all(|test| !test.sample));
        }
        let samples: Vec<u32> = scheduled.iter().filter(|t| t.sample).map(|t| t.ordinal).collect();
        let hiddens: Vec<u32> = scheduled.iter().filter(|t| !t.sample).map(|t| t.ordinal).collect();
        prop_assert!(samples.windows(2).all(|pair| pair[0] <= pair[1]));
        prop_assert!(hiddens.windows(2).all(|pair| pair[0] <= pair[1]));
    }
}

#[test]
fn a_sample_is_always_scheduled_before_a_hidden_test_with_a_lower_ordinal() {
    let scheduled = schedule(vec![hidden(1, Value::Null, 1), sample(9, Value::Null)]);

    let order: Vec<(bool, u32)> = scheduled.iter().map(|t| (t.sample, t.ordinal)).collect();
    assert_eq!(order, vec![(true, 9), (false, 1)]);
}
