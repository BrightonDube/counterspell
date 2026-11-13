//! Public run events. Only the types here may carry data to a browser.
//! No constructor accepts raw JSON for test, done or error events.

use std::fmt;

use serde::Serialize;
use serde_json::{json, Map, Value};

use crate::limits::{MAX_COMPILE_OUTPUT_BYTES, MAX_PAYLOAD_BYTES, MAX_SAMPLE_FIELD_BYTES};
use crate::status::{FailureReason, TestVerdict, Verdict};

/// One test result that may be shown to the browser.
#[derive(Debug, Clone, PartialEq)]
pub enum PublicTest {
    /// A hidden test. It has no field that could hold its input or expected value.
    Hidden {
        /// The test's position in the suite.
        ordinal: u32,
        /// The judge's verdict for this test.
        verdict: TestVerdict,
        /// Time the test took, measured by the judge.
        time_ms: u32,
    },
    /// A sample test, whose input, expected value and actual value may be shown.
    Sample {
        /// The test's position in the suite.
        ordinal: u32,
        /// The judge's verdict for this test.
        verdict: TestVerdict,
        /// Time the test took, measured by the judge.
        time_ms: u32,
        /// The values to show.
        detail: SampleDetail,
    },
}

impl PublicTest {
    fn to_json(&self) -> Value {
        match self {
            Self::Hidden {
                ordinal,
                verdict,
                time_ms,
            } => json!({
                "ordinal": ordinal,
                "verdict": verdict.as_str(),
                "time_ms": time_ms,
                "sample": false,
            }),
            Self::Sample {
                ordinal,
                verdict,
                time_ms,
                detail,
            } => json!({
                "ordinal": ordinal,
                "verdict": verdict.as_str(),
                "time_ms": time_ms,
                "sample": true,
                "input": detail.input,
                "expected": detail.expected,
                "got": detail.got,
            }),
        }
    }
}

/// The text shown for a sample test. Each value is cut to a fixed size when built.
#[derive(Debug, Clone, PartialEq)]
pub struct SampleDetail {
    input: String,
    expected: String,
    got: String,
}

impl SampleDetail {
    /// Builds the detail, cutting each value so its JSON text fits the sample budget.
    #[must_use]
    pub fn new(input: &str, expected: &str, got: &str) -> Self {
        Self {
            input: fit_text(input, MAX_SAMPLE_FIELD_BYTES),
            expected: fit_text(expected, MAX_SAMPLE_FIELD_BYTES),
            got: fit_text(got, MAX_SAMPLE_FIELD_BYTES),
        }
    }
}

/// A JSON object that is small enough to send as an event payload.
#[derive(Debug, Clone, PartialEq)]
pub struct BoundedJson(Value);

/// Why a value cannot become a [`BoundedJson`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoundedJsonError {
    /// The value could not be turned into JSON at all.
    NotSerialisable,
    /// The value is valid JSON but not an object, such as an array or a number.
    NotAnObject,
    /// The object is larger than the payload cap.
    TooLarge {
        /// The size of the object's JSON text, in bytes.
        bytes: usize,
    },
}

impl fmt::Display for BoundedJsonError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotSerialisable => f.write_str("value cannot be written as JSON"),
            Self::NotAnObject => f.write_str("event payload must be a JSON object"),
            Self::TooLarge { bytes } => write!(
                f,
                "event payload is {bytes} bytes, over the {MAX_PAYLOAD_BYTES} byte cap"
            ),
        }
    }
}

impl std::error::Error for BoundedJsonError {}

impl BoundedJson {
    /// Builds a payload from any serialisable value. Only objects within the cap are accepted.
    ///
    /// # Errors
    ///
    /// Returns an error when the value is not an object or its JSON text is over the cap.
    pub fn from_serialize<T: Serialize + ?Sized>(value: &T) -> Result<Self, BoundedJsonError> {
        let value = serde_json::to_value(value).map_err(|_| BoundedJsonError::NotSerialisable)?;
        if !value.is_object() {
            return Err(BoundedJsonError::NotAnObject);
        }
        let bytes = serde_json::to_vec(&value)
            .map_err(|_| BoundedJsonError::NotSerialisable)?
            .len();
        if bytes > MAX_PAYLOAD_BYTES {
            return Err(BoundedJsonError::TooLarge { bytes });
        }
        Ok(Self(value))
    }
}

/// The payload of a `done` event. Build it from the same value that is stored in the run's final columns.
#[derive(Debug, Clone, PartialEq)]
pub struct DonePayload {
    verdict: Verdict,
    passed: u32,
    total: u32,
    score: u32,
    compile_output: Option<String>,
}

impl DonePayload {
    /// Builds the payload. Compile output is cut to its fixed size.
    #[must_use]
    pub fn new(
        verdict: Verdict,
        passed: u32,
        total: u32,
        score: u32,
        compile_output: Option<&str>,
    ) -> Self {
        Self {
            verdict,
            passed,
            total,
            score,
            compile_output: compile_output.map(|text| fit_text(text, MAX_COMPILE_OUTPUT_BYTES)),
        }
    }

    fn to_json(&self) -> Value {
        let mut object = Map::new();
        object.insert("verdict".to_owned(), json!(self.verdict.as_str()));
        object.insert("passed".to_owned(), json!(self.passed));
        object.insert("total".to_owned(), json!(self.total));
        object.insert("score".to_owned(), json!(self.score));
        if let Some(output) = &self.compile_output {
            object.insert("compile_output".to_owned(), json!(output));
        }
        Value::Object(object)
    }
}

/// The payload of an `error` event. Its message comes from the fixed table on [`FailureReason`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ErrorPayload {
    code: FailureReason,
}

impl ErrorPayload {
    /// Builds the error payload for a reason. The message is looked up, never typed in.
    #[must_use]
    pub const fn for_reason(code: FailureReason) -> Self {
        Self { code }
    }

    /// The reason this error ends the run.
    #[must_use]
    pub const fn reason(self) -> FailureReason {
        self.code
    }

    fn to_json(self) -> Value {
        json!({
            "code": self.code.as_str(),
            "message": self.code.public_message(),
        })
    }
}

/// Every event a browser can receive for a run. The enum has no variant for raw JSON.
#[derive(Debug, Clone, PartialEq)]
pub enum PublicEvent {
    /// The run is starting again after a lost attempt. The UI clears its rows.
    Restarted {
        /// The attempt number that is starting.
        attempt: u16,
    },
    /// The program is being prepared.
    Compiling,
    /// One test result.
    Test(PublicTest),
    /// The ghost engine has started.
    GhostHunting,
    /// One recorded shrink step.
    ShrinkStep(BoundedJson),
    /// The ghost input.
    Ghost(BoundedJson),
    /// One timing point from bench mode.
    BenchPoint(BoundedJson),
    /// The measured complexity class.
    Complexity(BoundedJson),
    /// The mood of the efficiency mascot.
    Familiar(BoundedJson),
    /// The final result.
    Done(DonePayload),
    /// The run ended without a verdict.
    Error(ErrorPayload),
}

impl PublicEvent {
    /// The SSE event name. These eleven names are the whole public contract.
    #[must_use]
    pub fn event_type(&self) -> &'static str {
        match self {
            Self::Restarted { .. } => "restarted",
            Self::Compiling => "compiling",
            Self::Test(_) => "test",
            Self::GhostHunting => "ghost_hunting",
            Self::ShrinkStep(_) => "shrink_step",
            Self::Ghost(_) => "ghost",
            Self::BenchPoint(_) => "bench_point",
            Self::Complexity(_) => "complexity",
            Self::Familiar(_) => "familiar",
            Self::Done(_) => "done",
            Self::Error(_) => "error",
        }
    }

    /// The JSON object sent as the event's data.
    #[must_use]
    pub fn payload(&self) -> Value {
        match self {
            Self::Restarted { attempt } => json!({ "attempt": attempt }),
            Self::Compiling | Self::GhostHunting => json!({}),
            Self::Test(test) => test.to_json(),
            Self::ShrinkStep(data)
            | Self::Ghost(data)
            | Self::BenchPoint(data)
            | Self::Complexity(data)
            | Self::Familiar(data) => data.0.clone(),
            Self::Done(done) => done.to_json(),
            Self::Error(error) => error.to_json(),
        }
    }
}

/// The JSON wrapper around one event on the wire: its sequence number, attempt and payload.
#[derive(Debug, Clone, PartialEq)]
pub struct EventEnvelope {
    seq: u32,
    attempt: u16,
    payload: Value,
}

impl EventEnvelope {
    /// Wraps an event with its per-run sequence number and attempt.
    #[must_use]
    pub fn new(seq: u32, attempt: u16, event: &PublicEvent) -> Self {
        Self {
            seq,
            attempt,
            payload: event.payload(),
        }
    }

    /// The JSON object sent to the browser.
    #[must_use]
    pub fn to_json(&self) -> Value {
        json!({
            "seq": self.seq,
            "attempt": self.attempt,
            "payload": self.payload,
        })
    }
}

/// Cuts `text` to the longest prefix whose JSON string form fits in `max_bytes`.
/// The cut never splits a character.
fn fit_text(text: &str, max_bytes: usize) -> String {
    let mut used = 0;
    let mut kept = String::new();
    for ch in text.chars() {
        let width = json_width(ch);
        if used + width > max_bytes {
            break;
        }
        used += width;
        kept.push(ch);
    }
    kept
}

/// How many bytes one character takes inside a JSON string, after escaping.
fn json_width(ch: char) -> usize {
    match ch {
        '"' | '\\' | '\n' | '\r' | '\t' | '\u{8}' | '\u{c}' => 2,
        c if (c as u32) < 0x20 => 6,
        c => c.len_utf8(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::limits::MAX_PAYLOAD_BYTES;
    use crate::status::{FailureReason, TestVerdict, Verdict};
    use proptest::prelude::*;
    use serde_json::{json, Value};
    use std::collections::BTreeSet;

    fn keys(value: &Value) -> BTreeSet<String> {
        value
            .as_object()
            .expect("payload must be a JSON object")
            .keys()
            .cloned()
            .collect()
    }

    fn set(items: &[&str]) -> BTreeSet<String> {
        items.iter().map(|item| (*item).to_owned()).collect()
    }

    fn some_bounded() -> BoundedJson {
        BoundedJson::from_serialize(&json!({})).expect("an empty object is accepted")
    }

    #[test]
    fn hidden_test_payload_has_exactly_the_public_keys() {
        let event = PublicEvent::Test(PublicTest::Hidden {
            ordinal: 7,
            verdict: TestVerdict::WrongAnswer,
            time_ms: 12,
        });
        let payload = event.payload();
        assert_eq!(
            keys(&payload),
            set(&["ordinal", "verdict", "time_ms", "sample"])
        );
        assert_eq!(
            payload,
            json!({"ordinal": 7, "verdict": "wrong_answer", "time_ms": 12, "sample": false})
        );
    }

    #[test]
    fn sample_test_payload_adds_input_expected_and_got() {
        let event = PublicEvent::Test(PublicTest::Sample {
            ordinal: 1,
            verdict: TestVerdict::WrongAnswer,
            time_ms: 3,
            detail: SampleDetail::new("[2,7]", "[0,1]", "[0,0]"),
        });
        assert_eq!(
            event.payload(),
            json!({
                "ordinal": 1, "verdict": "wrong_answer", "time_ms": 3, "sample": true,
                "input": "[2,7]", "expected": "[0,1]", "got": "[0,0]"
            })
        );
    }

    #[test]
    fn sample_fields_are_cut_by_their_escaped_length() {
        // Each control character is written as six bytes (\u0001) in JSON.
        // A 2048-byte budget therefore holds 341 of them.
        let detail = SampleDetail::new(&"\u{1}".repeat(1000), "x", "y");
        let payload = PublicEvent::Test(PublicTest::Sample {
            ordinal: 1,
            verdict: TestVerdict::RuntimeError,
            time_ms: 0,
            detail,
        })
        .payload();
        let input = payload["input"].as_str().expect("input is a string");
        assert_eq!(input.chars().count(), 341);
    }

    #[test]
    fn bounded_json_accepts_an_object_and_rejects_other_shapes() {
        assert!(BoundedJson::from_serialize(&json!({"a": 1})).is_ok());
        assert!(BoundedJson::from_serialize(&vec![1, 2]).is_err());
        assert!(BoundedJson::from_serialize(&5).is_err());
        assert!(BoundedJson::from_serialize(&"text").is_err());
    }

    #[test]
    fn bounded_json_size_limit_is_exactly_sixteen_kib() {
        // `{"blob":"` plus `"}` is 11 bytes of JSON around the string.
        let fits = json!({"blob": "a".repeat(16384 - 11)});
        let too_big = json!({"blob": "a".repeat(16384 - 10)});
        assert!(BoundedJson::from_serialize(&fits).is_ok());
        assert!(BoundedJson::from_serialize(&too_big).is_err());
    }

    #[test]
    fn error_payload_message_comes_from_the_fixed_table() {
        let payload =
            PublicEvent::Error(ErrorPayload::for_reason(FailureReason::Cancelled)).payload();
        assert_eq!(
            payload,
            json!({
                "code": "cancelled",
                "message": "The run was cancelled before it produced a verdict."
            })
        );
    }

    #[test]
    fn every_failure_reason_has_its_own_error_payload() {
        for reason in FailureReason::ALL {
            let payload = PublicEvent::Error(ErrorPayload::for_reason(*reason)).payload();
            assert_eq!(payload["code"], reason.as_str());
            assert_eq!(payload["message"], reason.public_message());
        }
    }

    #[test]
    fn done_payload_cuts_compile_output_to_eight_kib() {
        let payload = PublicEvent::Done(DonePayload::new(
            Verdict::CompileError,
            0,
            3,
            0,
            Some(&"a".repeat(9000)),
        ))
        .payload();
        let output = payload["compile_output"].as_str().expect("compile output");
        assert_eq!(output.len(), 8192);
    }

    #[test]
    fn done_payload_without_compile_output_has_no_such_key() {
        let payload =
            PublicEvent::Done(DonePayload::new(Verdict::Accepted, 3, 3, 100, None)).payload();
        assert_eq!(
            keys(&payload),
            set(&["verdict", "passed", "total", "score"])
        );
        assert_eq!(
            payload,
            json!({"verdict": "accepted", "passed": 3, "total": 3, "score": 100})
        );
    }

    #[test]
    fn event_names_are_exactly_the_eleven_allowed_names() {
        let events = [
            PublicEvent::Restarted { attempt: 2 },
            PublicEvent::Compiling,
            PublicEvent::Test(PublicTest::Hidden {
                ordinal: 1,
                verdict: TestVerdict::Pass,
                time_ms: 1,
            }),
            PublicEvent::GhostHunting,
            PublicEvent::ShrinkStep(some_bounded()),
            PublicEvent::Ghost(some_bounded()),
            PublicEvent::BenchPoint(some_bounded()),
            PublicEvent::Complexity(some_bounded()),
            PublicEvent::Familiar(some_bounded()),
            PublicEvent::Done(DonePayload::new(Verdict::Accepted, 1, 1, 100, None)),
            PublicEvent::Error(ErrorPayload::for_reason(FailureReason::Expired)),
        ];
        let names: BTreeSet<&str> = events.iter().map(PublicEvent::event_type).collect();
        assert_eq!(
            names,
            [
                "compiling",
                "test",
                "ghost_hunting",
                "shrink_step",
                "ghost",
                "bench_point",
                "complexity",
                "familiar",
                "restarted",
                "done",
                "error",
            ]
            .into_iter()
            .collect()
        );
    }

    #[test]
    fn envelope_carries_seq_attempt_and_payload() {
        let envelope = EventEnvelope::new(3, 2, &PublicEvent::Compiling);
        assert_eq!(
            envelope.to_json(),
            json!({"seq": 3, "attempt": 2, "payload": {}})
        );
    }

    fn any_text() -> impl Strategy<Value = String> {
        prop::collection::vec(any::<char>(), 0..9000)
            .prop_map(|chars| chars.into_iter().collect::<String>())
    }

    fn any_public_event() -> impl Strategy<Value = PublicEvent> {
        let hidden = (
            any::<u32>(),
            prop::sample::select(TestVerdict::ALL.to_vec()),
            any::<u32>(),
        )
            .prop_map(|(ordinal, verdict, time_ms)| {
                PublicEvent::Test(PublicTest::Hidden {
                    ordinal,
                    verdict,
                    time_ms,
                })
            });
        let sample = (
            any::<u32>(),
            prop::sample::select(TestVerdict::ALL.to_vec()),
            any::<u32>(),
            any_text(),
            any_text(),
            any_text(),
        )
            .prop_map(|(ordinal, verdict, time_ms, input, expected, got)| {
                PublicEvent::Test(PublicTest::Sample {
                    ordinal,
                    verdict,
                    time_ms,
                    detail: SampleDetail::new(&input, &expected, &got),
                })
            });
        let done = (
            prop::sample::select(Verdict::ALL.to_vec()),
            any::<u32>(),
            any::<u32>(),
            any::<u32>(),
            prop::option::of(any_text()),
        )
            .prop_map(|(verdict, passed, total, score, output)| {
                PublicEvent::Done(DonePayload::new(
                    verdict,
                    passed,
                    total,
                    score,
                    output.as_deref(),
                ))
            });
        let error = prop::sample::select(FailureReason::ALL.to_vec())
            .prop_map(|reason| PublicEvent::Error(ErrorPayload::for_reason(reason)));
        let bounded = "[a-z]{0,200}".prop_map(|text| {
            PublicEvent::Ghost(
                BoundedJson::from_serialize(&json!({ "text": text }))
                    .expect("a short object is under the cap"),
            )
        });
        prop_oneof![
            any::<u16>().prop_map(|attempt| PublicEvent::Restarted { attempt }),
            Just(PublicEvent::Compiling),
            Just(PublicEvent::GhostHunting),
            hidden,
            sample,
            done,
            error,
            bounded,
        ]
    }

    proptest! {
        #[test]
        fn every_public_event_payload_is_an_object_under_the_cap(event in any_public_event()) {
            let payload = event.payload();
            prop_assert!(payload.is_object());
            let size = serde_json::to_vec(&payload).expect("payload serialises").len();
            prop_assert!(size <= MAX_PAYLOAD_BYTES, "payload was {} bytes", size);
        }
    }
}
