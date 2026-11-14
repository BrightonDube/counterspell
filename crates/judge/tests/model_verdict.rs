//! Model tests for the final verdict (requirements R8.3, R8.6, R9.1, R9.6; properties
//! P1 and P10). No Docker.
//!
//! A passing test is produced through `judge_answer`, the only code allowed to make one.

use counterspell_engine::Comparator;
use counterspell_judge::sandbox::compare::{judge_answer, Expected, TestVerdict, WrongReason};
use counterspell_judge::sandbox::verdict::{decide, ExecutedTest, FinalVerdict, RunFacts};
use proptest::prelude::*;
use serde_json::json;

/// A real passing verdict, built the only way the judge allows.
fn pass() -> TestVerdict {
    judge_answer(
        &Expected::new(json!(1)),
        1,
        br#"{"t":1,"out":1}"#,
        Comparator::Exact,
    )
}

/// The non-passing verdict kinds, indexed 1 to 5 (index 0 is a pass).
fn failing(kind: u8) -> TestVerdict {
    match kind {
        1 => TestVerdict::WrongAnswer(WrongReason::Different),
        2 => TestVerdict::RuntimeError,
        3 => TestVerdict::TimeLimit,
        4 => TestVerdict::MemoryLimit,
        _ => TestVerdict::OutputLimit,
    }
}

fn verdict_of(kind: u8) -> TestVerdict {
    if kind == 0 {
        pass()
    } else {
        failing(kind)
    }
}

fn facts(compiled: bool, planned: u32, cut_by_cap: bool, stopped: bool) -> RunFacts {
    RunFacts {
        compiled,
        planned,
        cut_by_cap,
        stopped,
    }
}

fn executed(ordinal: u32, kind: u8) -> ExecutedTest {
    ExecutedTest::new(ordinal, verdict_of(kind))
}

fn is_pass(test: &ExecutedTest) -> bool {
    matches!(test.verdict(), TestVerdict::Pass(_))
}

#[test]
fn every_test_passing_on_a_complete_compiled_run_is_accepted() {
    let tests = [executed(1, 0), executed(2, 0)];
    assert_eq!(
        decide(&tests, &facts(true, 2, false, false)),
        FinalVerdict::Accepted
    );
}

#[test]
fn a_compile_failure_wins_over_every_test_result() {
    let tests = [executed(1, 0)];
    assert_eq!(
        decide(&tests, &facts(false, 1, false, false)),
        FinalVerdict::CompileError
    );
}

#[test]
fn the_first_failing_test_in_ordinal_order_decides_the_verdict() {
    // Test 2 times out and test 5 gives a wrong answer. Ordinal 2 comes first.
    let tests = [executed(5, 1), executed(2, 3), executed(1, 0)];
    assert_eq!(
        decide(&tests, &facts(true, 3, false, false)),
        FinalVerdict::TimeLimit
    );
}

#[test]
fn a_cut_run_is_time_limit_when_no_test_failed() {
    let tests = [executed(1, 0)];
    assert_eq!(
        decide(&tests, &facts(true, 3, true, false)),
        FinalVerdict::TimeLimit
    );
}

#[test]
fn a_failing_test_wins_over_the_run_cap() {
    let tests = [executed(1, 1)];
    assert_eq!(
        decide(&tests, &facts(true, 3, true, false)),
        FinalVerdict::WrongAnswer
    );
}

#[test]
fn a_stopped_run_is_never_accepted() {
    let tests = [executed(1, 0)];
    assert_eq!(
        decide(&tests, &facts(true, 1, false, true)),
        FinalVerdict::Stopped
    );
}

#[test]
fn a_run_with_no_planned_tests_is_not_accepted() {
    assert_eq!(
        decide(&[], &facts(true, 0, false, false)),
        FinalVerdict::Incomplete
    );
}

#[test]
fn an_unexplained_short_run_is_incomplete_not_accepted() {
    let tests = [executed(1, 0)];
    assert_eq!(
        decide(&tests, &facts(true, 2, false, false)),
        FinalVerdict::Incomplete
    );
}

#[test]
fn each_failing_kind_maps_to_its_own_final_verdict() {
    let cases = [
        (1, FinalVerdict::WrongAnswer),
        (2, FinalVerdict::RuntimeError),
        (3, FinalVerdict::TimeLimit),
        (4, FinalVerdict::MemoryLimit),
        (5, FinalVerdict::OutputLimit),
    ];
    for (kind, expected) in cases {
        let tests = [executed(1, kind)];
        assert_eq!(
            decide(&tests, &facts(true, 1, false, false)),
            expected,
            "kind {kind}"
        );
    }
}

/// Exhaustive table over a small domain: every combination of compile, stop, cap, plan
/// size and the kinds of up to two executed tests. Checks the rules that must hold for
/// every row, including the soundness of `Accepted`.
#[test]
fn the_decision_table_is_sound_for_every_small_combination() {
    let kinds = 0u8..=5;
    let mut lists: Vec<Vec<u8>> = vec![Vec::new()];
    for a in kinds.clone() {
        lists.push(vec![a]);
        for b in kinds.clone() {
            lists.push(vec![a, b]);
        }
    }
    for compiled in [false, true] {
        for stopped in [false, true] {
            for cut in [false, true] {
                for planned in 0u32..=2 {
                    for list in &lists {
                        let tests: Vec<ExecutedTest> = list
                            .iter()
                            .enumerate()
                            .map(|(i, kind)| executed(u32::try_from(i + 1).unwrap(), *kind))
                            .collect();
                        let verdict = decide(&tests, &facts(compiled, planned, cut, stopped));
                        if verdict == FinalVerdict::Accepted {
                            assert!(compiled && !stopped && !cut);
                            assert_eq!(u32::try_from(tests.len()).unwrap(), planned);
                            assert!(planned > 0);
                            assert!(tests.iter().all(is_pass));
                        }
                        if !compiled && !stopped {
                            assert_eq!(verdict, FinalVerdict::CompileError);
                        }
                    }
                }
            }
        }
    }
}

proptest! {
    /// P10: adding a failing test never turns a run into `Accepted`.
    #[test]
    fn a_failing_test_never_makes_a_run_accepted(
        passing in 1u32..6,
        kind in 1u8..=5,
    ) {
        let mut tests: Vec<ExecutedTest> = (1..=passing).map(|o| executed(o, 0)).collect();
        tests.push(executed(passing + 1, kind));
        let planned = passing + 1;
        let verdict = decide(&tests, &facts(true, planned, false, false));
        prop_assert_ne!(verdict, FinalVerdict::Accepted);
    }

    /// P10: the verdict does not depend on the order the tests arrived in.
    #[test]
    fn the_verdict_is_independent_of_input_order(
        kinds in prop::collection::vec(0u8..=5, 1..6),
    ) {
        let forward: Vec<ExecutedTest> = kinds
            .iter()
            .enumerate()
            .map(|(i, k)| executed(u32::try_from(i + 1).unwrap(), *k))
            .collect();
        let backward: Vec<ExecutedTest> = forward.iter().rev().map(|t| executed(t.ordinal(), kind_of(t))).collect();
        let planned = u32::try_from(kinds.len()).unwrap();
        prop_assert_eq!(
            decide(&forward, &facts(true, planned, false, false)),
            decide(&backward, &facts(true, planned, false, false))
        );
    }

    /// P1: when nothing failed, compiled, not stopped and complete, the run is accepted.
    #[test]
    fn an_all_passing_complete_run_is_accepted(count in 1u32..8) {
        let tests: Vec<ExecutedTest> = (1..=count).map(|o| executed(o, 0)).collect();
        prop_assert_eq!(
            decide(&tests, &facts(true, count, false, false)),
            FinalVerdict::Accepted
        );
    }
}

/// Maps a test back to its kind index, for the order test above.
fn kind_of(test: &ExecutedTest) -> u8 {
    match test.verdict() {
        TestVerdict::Pass(_) => 0,
        TestVerdict::WrongAnswer(_) => 1,
        TestVerdict::RuntimeError => 2,
        TestVerdict::TimeLimit => 3,
        TestVerdict::MemoryLimit => 4,
        TestVerdict::OutputLimit => 5,
    }
}
