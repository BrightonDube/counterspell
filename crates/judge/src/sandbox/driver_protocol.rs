//! The line protocol between the judge and the driver inside a sandbox.
//!
//! The judge writes one call per line: `{"t": <id>, "args": [...]}`. The driver answers
//! with one line: `{"t": <id>, "out": <value>, "stderr": "<optional>"}`. This module
//! only checks the shape of an answer. It never decides a verdict; the trusted
//! comparator does that.
//!
//! Named to match `language-drivers` so that crate can extend it.

use std::fmt;

use serde::de::{Deserializer, IgnoredAny, MapAccess, Visitor};
use serde::Deserialize;
use serde_json::{Map, Value};
use thiserror::Error;

const KEY_ID: &str = "t";
const KEY_OUT: &str = "out";
const KEY_ARGS: &str = "args";
const KEY_STDERR: &str = "stderr";

/// Why an answer line is not a valid answer. The protocol is the only thing it reports on.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ProtocolViolation {
    /// The call could not be encoded.
    #[error("the call could not be encoded")]
    Encode,
    /// The line is not valid UTF-8.
    #[error("the answer is not valid UTF-8")]
    NotUtf8,
    /// The line is empty or only whitespace.
    #[error("the answer is blank")]
    Blank,
    /// The line is not a single JSON value, or it has data after the value.
    #[error("the answer is not one JSON value")]
    NotJson,
    /// The JSON value is not an object.
    #[error("the answer is not a JSON object")]
    NotAnObject,
    /// The object has a key other than `t`, `out` and optional `stderr`.
    #[error("the answer has an unexpected key {0:?}")]
    ExtraKey(String),
    /// The optional `stderr` field is not a string.
    #[error("the answer has an invalid {0:?} field")]
    InvalidField(&'static str),
    /// The same key appears twice in the object.
    #[error("the answer repeats the key {0:?}")]
    DuplicateKey(String),
    /// The object lacks `t` or `out`.
    #[error("the answer has no {0:?} key")]
    MissingKey(&'static str),
    /// The answer is for a different call, or its id is not a whole number.
    #[error("the answer is for a different call")]
    WrongId,
    /// The program printed more than one line in a single read, or data after its answer.
    #[error("the driver printed more than one answer line")]
    ExtraOutput,
}

/// Encodes one call as a single JSON line that ends with a newline.
///
/// The argument values come from the problem's signature and the test input. The output
/// has no newline inside, because JSON escapes newlines in strings.
///
/// # Errors
///
/// Returns [`ProtocolViolation::Encode`] if the value cannot be serialised.
pub fn encode_call(call_id: u64, args: &[Value]) -> Result<Vec<u8>, ProtocolViolation> {
    let mut object = Map::new();
    object.insert(KEY_ID.to_owned(), Value::from(call_id));
    object.insert(KEY_ARGS.to_owned(), Value::Array(args.to_vec()));
    let mut line =
        serde_json::to_vec(&Value::Object(object)).map_err(|_| ProtocolViolation::Encode)?;
    line.push(b'\n');
    Ok(line)
}

/// Every top-level key of a JSON object, in order, including repeats. `serde_json::Value`
/// keeps only the last of two equal keys, so this type is used to catch the repeats.
struct KeyList(Vec<String>);

impl<'de> Deserialize<'de> for KeyList {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct KeyVisitor;

        impl<'de> Visitor<'de> for KeyVisitor {
            type Value = KeyList;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a JSON object")
            }

            fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<KeyList, A::Error> {
                let mut keys = Vec::new();
                while let Some((key, IgnoredAny)) = map.next_entry::<String, IgnoredAny>()? {
                    keys.push(key);
                }
                Ok(KeyList(keys))
            }
        }

        deserializer.deserialize_map(KeyVisitor)
    }
}

/// A checked answer: the `out` value for the comparator and the diagnostics the
/// driver captured inside this call.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedAnswer {
    /// The program's return value.
    pub out: Value,
    /// Bounded stderr emitted by the driver while running this call, when it wrote any.
    pub stderr: Option<String>,
}

/// Checks one answer line and returns its `out` value and optional per-call stderr.
///
/// The line must be one JSON object with the keys `t` and `out`, plus an optional
/// string `stderr` field. `t` must equal `expected`.
///
/// # Errors
///
/// Returns the [`ProtocolViolation`] for the first broken rule.
pub fn parse_answer_detail(line: &[u8], expected: u64) -> Result<ParsedAnswer, ProtocolViolation> {
    let text = std::str::from_utf8(line).map_err(|_| ProtocolViolation::NotUtf8)?;
    if text.trim().is_empty() {
        return Err(ProtocolViolation::Blank);
    }
    let value: Value = serde_json::from_str(text).map_err(|_| ProtocolViolation::NotJson)?;
    let Value::Object(mut object) = value else {
        return Err(ProtocolViolation::NotAnObject);
    };

    let keys: KeyList = serde_json::from_str(text).map_err(|_| ProtocolViolation::NotJson)?;
    let mut seen = std::collections::BTreeSet::new();
    for key in keys.0 {
        if !seen.insert(key.clone()) {
            return Err(ProtocolViolation::DuplicateKey(key));
        }
    }

    if let Some(extra) = object
        .keys()
        .find(|key| key.as_str() != KEY_ID && key.as_str() != KEY_OUT && key.as_str() != KEY_STDERR)
    {
        return Err(ProtocolViolation::ExtraKey(extra.clone()));
    }
    let id = object
        .remove(KEY_ID)
        .ok_or(ProtocolViolation::MissingKey(KEY_ID))?;
    let out = object
        .remove(KEY_OUT)
        .ok_or(ProtocolViolation::MissingKey(KEY_OUT))?;
    let stderr = object
        .remove(KEY_STDERR)
        .map(|value| match value {
            Value::String(stderr) => Ok(stderr),
            _ => Err(ProtocolViolation::InvalidField(KEY_STDERR)),
        })
        .transpose()?;

    if id.as_u64() != Some(expected) {
        return Err(ProtocolViolation::WrongId);
    }
    Ok(ParsedAnswer { out, stderr })
}

/// Checks one answer line and returns its `out` value.
///
/// The line must be one JSON object with the keys `t` and `out`, plus an optional
/// string `stderr` field. `t` must equal `expected`. The `out` value is returned as
/// it is, for the comparator to judge.
///
/// # Errors
///
/// Returns the [`ProtocolViolation`] for the first broken rule.
pub fn parse_answer(line: &[u8], expected: u64) -> Result<Value, ProtocolViolation> {
    parse_answer_detail(line, expected).map(|answer| answer.out)
}
