//! The line protocol between the judge and the driver inside a sandbox.
//!
//! The judge writes one call per line: `{"t": <id>, "args": [...]}`. The driver answers
//! with one line: `{"t": <id>, "out": <value>}`. This module only checks the shape of
//! an answer. It never decides a verdict; the trusted comparator does that.
//!
//! Named to match `language-drivers` so that crate can extend it.

use serde_json::{Map, Value};
use thiserror::Error;

const KEY_ID: &str = "t";
const KEY_OUT: &str = "out";
const KEY_ARGS: &str = "args";

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
    /// The object has a key other than `t` and `out`.
    #[error("the answer has an unexpected key {0:?}")]
    ExtraKey(String),
    /// The object lacks `t` or `out`.
    #[error("the answer has no {0:?} key")]
    MissingKey(&'static str),
    /// The answer is for a different call, or its id is not a whole number.
    #[error("the answer is for a different call")]
    WrongId,
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

/// Checks one answer line and returns its `out` value.
///
/// The line must be one JSON object with exactly the keys `t` and `out`, and `t` must
/// equal `expected`. The `out` value is returned as it is, for the comparator to judge.
///
/// # Errors
///
/// Returns the [`ProtocolViolation`] for the first broken rule.
pub fn parse_answer(line: &[u8], expected: u64) -> Result<Value, ProtocolViolation> {
    let text = std::str::from_utf8(line).map_err(|_| ProtocolViolation::NotUtf8)?;
    if text.trim().is_empty() {
        return Err(ProtocolViolation::Blank);
    }
    let value: Value = serde_json::from_str(text).map_err(|_| ProtocolViolation::NotJson)?;
    let Value::Object(mut object) = value else {
        return Err(ProtocolViolation::NotAnObject);
    };

    if let Some(extra) = object
        .keys()
        .find(|key| key.as_str() != KEY_ID && key.as_str() != KEY_OUT)
    {
        return Err(ProtocolViolation::ExtraKey(extra.clone()));
    }
    let id = object
        .remove(KEY_ID)
        .ok_or(ProtocolViolation::MissingKey(KEY_ID))?;
    let out = object
        .remove(KEY_OUT)
        .ok_or(ProtocolViolation::MissingKey(KEY_OUT))?;

    if id.as_u64() != Some(expected) {
        return Err(ProtocolViolation::WrongId);
    }
    Ok(out)
}
