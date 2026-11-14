//! Model tests for the trusted comparator (requirements R8.1, R8.2, R8.4, property P2).
//! No Docker. The compile-time guarantee that `Pass` cannot be built outside `compare`
//! is a doc test on `counterspell_judge::sandbox::compare`.

use counterspell_engine::Comparator;
use counterspell_judge::sandbox::compare::{judge_answer, Expected, TestVerdict, WrongReason};
use counterspell_judge::sandbox::driver_protocol::ProtocolViolation;
use proptest::prelude::*;
use serde_json::{json, Value};

fn expected(value: Value) -> Expected {
    Expected::new(value)
}

fn is_pass(verdict: &TestVerdict) -> bool {
    matches!(verdict, TestVerdict::Pass(_))
}

#[test]
fn the_right_value_for_the_right_call_passes() {
    let verdict = judge_answer(
        &expected(json!([0, 1])),
        3,
        br#"{"t":3,"out":[0,1]}"#,
        Comparator::Exact,
    );
    assert!(is_pass(&verdict));
}

#[test]
fn a_different_value_is_a_wrong_answer() {
    let verdict = judge_answer(
        &expected(json!([0, 1])),
        3,
        br#"{"t":3,"out":[0,0]}"#,
        Comparator::Exact,
    );
    assert_eq!(verdict, TestVerdict::WrongAnswer(WrongReason::Different));
}

#[test]
fn an_equal_value_with_the_wrong_call_id_is_not_a_pass() {
    let verdict = judge_answer(
        &expected(json!(7)),
        3,
        br#"{"t":4,"out":7}"#,
        Comparator::Exact,
    );
    assert_eq!(
        verdict,
        TestVerdict::WrongAnswer(WrongReason::Protocol(ProtocolViolation::WrongId))
    );
}

#[test]
fn a_line_that_looks_like_a_verdict_cannot_pass() {
    let verdict = judge_answer(
        &expected(json!("pass")),
        3,
        br#"{"verdict":"pass"}"#,
        Comparator::Exact,
    );
    assert_eq!(
        verdict,
        TestVerdict::WrongAnswer(WrongReason::Protocol(ProtocolViolation::ExtraKey(
            "verdict".to_owned()
        )))
    );
}

#[test]
fn two_answer_lines_in_one_read_are_refused() {
    let verdict = judge_answer(
        &expected(json!(1)),
        3,
        b"{\"t\":3,\"out\":1}\n{\"t\":3,\"out\":1}",
        Comparator::Exact,
    );
    assert_eq!(
        verdict,
        TestVerdict::WrongAnswer(WrongReason::Protocol(ProtocolViolation::NotJson))
    );
}

#[test]
fn exact_comparison_is_order_sensitive() {
    let verdict = judge_answer(
        &expected(json!([1, 2])),
        1,
        br#"{"t":1,"out":[2,1]}"#,
        Comparator::Exact,
    );
    assert_eq!(verdict, TestVerdict::WrongAnswer(WrongReason::Different));
}

#[test]
fn unordered_comparison_accepts_the_same_multiset_in_another_order() {
    let verdict = judge_answer(
        &expected(json!([1, 2, 2])),
        1,
        br#"{"t":1,"out":[2,1,2]}"#,
        Comparator::Unordered,
    );
    assert!(is_pass(&verdict));
}

fn arbitrary_value() -> impl Strategy<Value = Value> {
    let leaf = prop_oneof![
        Just(Value::Null),
        any::<bool>().prop_map(Value::Bool),
        any::<i32>().prop_map(|n| json!(n)),
        "[a-z ]{0,6}".prop_map(Value::String),
    ];
    leaf.prop_recursive(2, 16, 4, |inner| {
        prop::collection::vec(inner, 0..4).prop_map(Value::Array)
    })
}

proptest! {
    /// P2: a test passes only when the line is `{"t": id, "out": expected}` and the
    /// comparator agrees. Arbitrary bytes never pass.
    #[test]
    fn arbitrary_bytes_never_pass(
        line in prop::collection::vec(any::<u8>(), 0..200),
        id in 0u64..10,
        want in arbitrary_value(),
    ) {
        let verdict = judge_answer(&expected(want), id, &line, Comparator::Exact);
        if is_pass(&verdict) {
            let parsed: Value = serde_json::from_slice(&line).unwrap();
            prop_assert_eq!(parsed["t"].as_u64(), Some(id));
        }
    }

    /// P2: a well-formed line with the right id passes exactly when the value matches.
    #[test]
    fn a_well_formed_line_passes_only_on_an_equal_value(
        want in arbitrary_value(),
        got in arbitrary_value(),
        id in 0u64..10,
    ) {
        let line = serde_json::to_vec(&json!({"t": id, "out": got.clone()})).unwrap();
        let verdict = judge_answer(&expected(want.clone()), id, &line, Comparator::Exact);
        prop_assert_eq!(is_pass(&verdict), want == got);
    }
}
