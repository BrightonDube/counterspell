//! Model tests for public records and redaction (requirements R13.1, R13.2, R13.6 to
//! R13.8, R8.5; properties P12 and P15). No Docker.
//!
//! Every test serialises the public record and checks the serialised text, because a
//! leak would show up there.

use counterspell_judge::sandbox::records::{
    build_result, RecordBuilder, SampleDetail, TestRun, VerdictKind,
};
use counterspell_judge::sandbox::verdict::FinalVerdict;
use proptest::prelude::*;
use serde_json::Value;

fn sample(ordinal: u32, verdict: VerdictKind, detail: SampleDetail) -> TestRun {
    TestRun {
        ordinal,
        sample: true,
        time_ms: 4,
        verdict,
        detail: Some(detail),
    }
}

fn hidden(ordinal: u32, verdict: VerdictKind, detail: Option<SampleDetail>) -> TestRun {
    TestRun {
        ordinal,
        sample: false,
        time_ms: 9,
        verdict,
        detail,
    }
}

fn detail(input: &[u8], expected: &[u8], got: &[u8], stderr: &[u8]) -> SampleDetail {
    SampleDetail {
        input: input.to_vec(),
        expected: expected.to_vec(),
        got: got.to_vec(),
        stderr: stderr.to_vec(),
    }
}

fn canary_detail() -> SampleDetail {
    detail(
        b"CANARY_INPUT",
        b"CANARY_EXPECTED",
        b"CANARY_PRINTED",
        b"CANARY_STDERR",
    )
}

fn serialised(record: &impl serde::Serialize) -> String {
    serde_json::to_string(record).unwrap()
}

#[test]
fn a_hidden_record_has_only_ordinal_verdict_and_time() {
    let mut builder = RecordBuilder::new();
    let record = builder.record(hidden(7, VerdictKind::WrongAnswer, Some(canary_detail())));
    let value: Value = serde_json::from_str(&serialised(&record)).unwrap();
    let keys: Vec<&str> = value
        .as_object()
        .unwrap()
        .keys()
        .map(String::as_str)
        .collect();
    let mut keys = keys;
    keys.sort_unstable();
    assert_eq!(keys, ["kind", "ordinal", "time_ms", "verdict"]);
    assert_eq!(value["kind"], "hidden");
    assert_eq!(value["verdict"], "wrong_answer");
    assert_eq!(value["ordinal"], 7);
    assert_eq!(value["time_ms"], 9);
}

#[test]
fn canaries_in_a_hidden_test_never_reach_any_public_record() {
    let mut builder = RecordBuilder::new();
    let record = builder.record(hidden(3, VerdictKind::Pass, Some(canary_detail())));
    let text = serialised(&record);
    for canary in [
        "CANARY_INPUT",
        "CANARY_EXPECTED",
        "CANARY_PRINTED",
        "CANARY_STDERR",
    ] {
        assert!(!text.contains(canary), "{canary} leaked into {text}");
    }
}

#[test]
fn a_sample_test_before_any_hidden_test_shows_its_detail() {
    let mut builder = RecordBuilder::new();
    let record = builder.record(sample(
        1,
        VerdictKind::WrongAnswer,
        detail(b"1 2", b"3", b"4", b""),
    ));
    let value: Value = serde_json::from_str(&serialised(&record)).unwrap();
    assert_eq!(value["kind"], "sample");
    assert_eq!(value["input"], "1 2");
    assert_eq!(value["expected"], "3");
    assert_eq!(value["got"], "4");
}

#[test]
fn a_sample_detail_is_cut_to_four_kib() {
    let long_input = vec![b'a'; 10 * 1024];
    let mut builder = RecordBuilder::new();
    let record = builder.record(sample(
        1,
        VerdictKind::Pass,
        detail(&long_input, b"", b"", b""),
    ));
    let value: Value = serde_json::from_str(&serialised(&record)).unwrap();
    assert_eq!(value["input"].as_str().unwrap().len(), 4096);
}

#[test]
fn control_characters_in_a_sample_are_neutralised() {
    let mut builder = RecordBuilder::new();
    let record = builder.record(sample(
        1,
        VerdictKind::WrongAnswer,
        detail(b"", b"", b"\x1b[31mred", b""),
    ));
    let value: Value = serde_json::from_str(&serialised(&record)).unwrap();
    let got = value["got"].as_str().unwrap();
    assert!(!got.contains('\u{1b}'));
}

#[test]
fn after_the_first_hidden_test_no_output_text_appears_even_from_a_later_sample() {
    // P15: the program can remember a hidden input and print it in a later visible test.
    let mut builder = RecordBuilder::new();
    let first = builder.record(hidden(10, VerdictKind::Pass, None));
    let later = builder.record(sample(
        2,
        VerdictKind::WrongAnswer,
        detail(b"", b"", b"LEAKED_HIDDEN_INPUT", b"LEAKED_STDERR"),
    ));
    let text = format!("{}{}", serialised(&first), serialised(&later));
    assert!(!text.contains("LEAKED_HIDDEN_INPUT"));
    assert!(!text.contains("LEAKED_STDERR"));
    let value: Value = serde_json::from_str(&serialised(&later)).unwrap();
    assert_eq!(value["kind"], "hidden");
    assert_eq!(value["verdict"], "wrong_answer");
}

#[test]
fn the_result_counts_passes_and_keeps_the_planned_total() {
    let runs = vec![
        sample(1, VerdictKind::Pass, detail(b"1", b"1", b"1", b"")),
        hidden(2, VerdictKind::Pass, None),
        hidden(3, VerdictKind::TimeLimit, None),
    ];
    let result = build_result(FinalVerdict::TimeLimit, 5, runs);
    assert_eq!(result.verdict, FinalVerdict::TimeLimit);
    assert_eq!(result.passed, 2);
    assert_eq!(result.total, 5);
    assert_eq!(result.tests.len(), 3);
}

#[test]
fn hidden_run_debug_output_has_no_canary() {
    let run = hidden(4, VerdictKind::WrongAnswer, Some(canary_detail()));
    let text = format!("{run:?}");
    for canary in [
        "CANARY_INPUT",
        "CANARY_EXPECTED",
        "CANARY_PRINTED",
        "CANARY_STDERR",
    ] {
        assert!(
            !text.contains(canary),
            "{canary} appeared in debug output: {text}"
        );
    }
}

#[test]
fn the_final_verdict_serialises_as_a_snake_case_word() {
    let result = build_result(FinalVerdict::CompileError, 1, Vec::new());
    let value: Value = serde_json::from_str(&serialised(&result)).unwrap();
    assert_eq!(value["verdict"], "compile_error");
}

proptest! {
    /// P12: whatever a hidden test's input, expected value, printed output or stderr
    /// contains, none of it appears in the serialised public result.
    #[test]
    fn hidden_test_content_never_appears_in_the_public_result(
        input in "[A-Z]{12,24}",
        expected in "[A-Z]{12,24}",
        printed in "[A-Z]{12,24}",
    ) {
        let secret = detail(input.as_bytes(), expected.as_bytes(), printed.as_bytes(), printed.as_bytes());
        let runs = vec![hidden(1, VerdictKind::WrongAnswer, Some(secret))];
        let result = build_result(FinalVerdict::WrongAnswer, 1, runs);
        let text = serialised(&result);
        prop_assert!(!text.contains(&input));
        prop_assert!(!text.contains(&expected));
        prop_assert!(!text.contains(&printed));
    }
}
