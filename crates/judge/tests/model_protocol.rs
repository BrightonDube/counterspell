//! Model tests for the driver protocol framing (requirement R8.4). No Docker.
//!
//! The parser checks the shape of an answer only. It never decides a verdict; the
//! comparator does that (design section 10).

use counterspell_judge::sandbox::driver_protocol::{encode_call, parse_answer, ProtocolViolation};
use proptest::prelude::*;
use serde_json::{json, Value};

#[test]
fn a_call_is_one_json_line_with_its_id_and_arguments() {
    let line = encode_call(3, &[json!(1), json!("a")]).unwrap();
    assert_eq!(line.last(), Some(&b'\n'));
    let parsed: Value = serde_json::from_slice(&line[..line.len() - 1]).unwrap();
    assert_eq!(parsed, json!({"t": 3, "args": [1, "a"]}));
}

#[test]
fn a_well_formed_answer_returns_its_out_value() {
    let out = parse_answer(br#"{"t":3,"out":[1,2]}"#, 3).unwrap();
    assert_eq!(out, json!([1, 2]));
}

#[test]
fn an_answer_for_another_call_is_refused() {
    assert_eq!(
        parse_answer(br#"{"t":4,"out":1}"#, 3).unwrap_err(),
        ProtocolViolation::WrongId
    );
}

#[test]
fn an_id_too_large_for_u64_is_refused_as_the_wrong_id() {
    assert_eq!(
        parse_answer(br#"{"t":18446744073709551616,"out":1}"#, 3).unwrap_err(),
        ProtocolViolation::WrongId
    );
}

#[test]
fn a_repeated_key_is_refused_even_when_the_values_agree() {
    // Value keeps only the last `t`, so without this check the first id would be lost.
    assert_eq!(
        parse_answer(br#"{"t":2,"t":1,"out":1}"#, 1).unwrap_err(),
        ProtocolViolation::DuplicateKey("t".to_owned())
    );
}

#[test]
fn an_extra_key_is_refused() {
    assert_eq!(
        parse_answer(br#"{"t":3,"out":1,"extra":true}"#, 3).unwrap_err(),
        ProtocolViolation::ExtraKey("extra".to_owned())
    );
}

#[test]
fn a_missing_key_is_refused() {
    assert_eq!(
        parse_answer(br#"{"t":3}"#, 3).unwrap_err(),
        ProtocolViolation::MissingKey("out")
    );
    assert_eq!(
        parse_answer(br#"{"out":1}"#, 3).unwrap_err(),
        ProtocolViolation::MissingKey("t")
    );
}

#[test]
fn a_line_that_is_not_an_object_is_refused() {
    assert_eq!(
        parse_answer(b"[1,2]", 3).unwrap_err(),
        ProtocolViolation::NotAnObject
    );
    assert_eq!(
        parse_answer(b"5", 3).unwrap_err(),
        ProtocolViolation::NotAnObject
    );
}

#[test]
fn a_blank_line_is_refused() {
    assert_eq!(parse_answer(b"", 3).unwrap_err(), ProtocolViolation::Blank);
    assert_eq!(
        parse_answer(b"   ", 3).unwrap_err(),
        ProtocolViolation::Blank
    );
}

#[test]
fn a_huge_number_outside_json_range_is_not_json() {
    assert_eq!(
        parse_answer(br#"{"t":1e400,"out":1}"#, 3).unwrap_err(),
        ProtocolViolation::NotJson
    );
}

#[test]
fn trailing_data_after_the_object_is_refused() {
    assert_eq!(
        parse_answer(br#"{"t":3,"out":1} {}"#, 3).unwrap_err(),
        ProtocolViolation::NotJson
    );
}

#[test]
fn bytes_that_are_not_utf8_are_refused() {
    assert_eq!(
        parse_answer(b"\xff\xfe", 3).unwrap_err(),
        ProtocolViolation::NotUtf8
    );
}

#[test]
fn a_line_that_looks_like_a_verdict_is_just_an_extra_key() {
    assert_eq!(
        parse_answer(br#"{"verdict":"pass"}"#, 3).unwrap_err(),
        ProtocolViolation::ExtraKey("verdict".to_owned())
    );
}

#[test]
fn a_string_out_value_that_reads_pass_is_only_an_answer_value() {
    // The parser returns the value for the comparator. It does not grant a verdict.
    let out = parse_answer(br#"{"t":3,"out":"pass"}"#, 3).unwrap();
    assert_eq!(out, json!("pass"));
}

proptest! {
    /// The encoded call has exactly one newline, at the end, even when the arguments
    /// contain newlines, quotes and control characters.
    #[test]
    fn an_encoded_call_has_no_newline_except_its_terminator(
        texts in prop::collection::vec(any::<String>(), 0..6),
        id in any::<u32>(),
    ) {
        let args: Vec<Value> = texts.into_iter().map(Value::String).collect();
        let line = encode_call(u64::from(id), &args).unwrap();
        let (last, inside) = line.split_last().unwrap();
        prop_assert_eq!(*last, b'\n');
        prop_assert!(!inside.contains(&b'\n'));
    }
}
