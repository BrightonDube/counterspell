//! Checks on submitted source, run before a run is queued (VAL-1 size, VAL-2 parse, VAL-3 plain text).
//!
//! These checks are a first filter only. They do not try to spot hostile code by keyword, because
//! keyword lists are easy to bypass. The sandbox is the real security boundary. Browser checks are
//! for feedback, and the server repeats every one of these.

use axum::http::StatusCode;
use thiserror::Error;
use tree_sitter::{Language as Grammar, Node, Parser};

use crate::error::ApiError;

/// Why a submitted source was refused. Messages are safe to show to the learner and never echo the
/// source or a control character.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum ValidationError {
    /// The source is larger than the language's byte limit (VAL-1).
    #[error("Source is {actual} bytes. The limit for this language is {limit} bytes.")]
    TooLarge {
        /// The limit in bytes.
        limit: u64,
        /// The size of the submitted source in bytes.
        actual: u64,
    },
    /// The bytes are not valid UTF-8 (VAL-3).
    #[error("Source must be plain text encoded as UTF-8.")]
    NotUtf8,
    /// The source has a NUL or control character other than tab, newline or carriage return (VAL-3).
    #[error("Source contains a character that is not allowed, at line {line}, column {column}.")]
    ForbiddenCharacter {
        /// Line of the first forbidden character, counting from 1.
        line: usize,
        /// Column of that character in characters, counting from 1.
        column: usize,
    },
    /// The source does not parse as the selected language (VAL-2).
    #[error("Source does not parse as {language}: error at line {line}, column {column}.")]
    DoesNotParse {
        /// The language the source was checked against.
        language: String,
        /// Line of the first parse error, counting from 1.
        line: usize,
        /// Column of that error in bytes, counting from 1.
        column: usize,
    },
    /// The language name is not one the server can check.
    #[error("Language is not supported.")]
    UnsupportedLanguage,
    /// The grammar could not be loaded. This is a server fault, not a learner mistake.
    #[error("Source checking is unavailable.")]
    Unavailable,
}

impl From<ValidationError> for ApiError {
    fn from(error: ValidationError) -> Self {
        let message = error.to_string();
        match error {
            ValidationError::TooLarge { .. } => {
                ApiError::new(StatusCode::PAYLOAD_TOO_LARGE, "source_too_large", message)
            }
            ValidationError::NotUtf8 | ValidationError::ForbiddenCharacter { .. } => {
                ApiError::bad_request("invalid_source", message)
            }
            ValidationError::DoesNotParse { .. } => {
                ApiError::bad_request("source_does_not_parse", message)
            }
            ValidationError::UnsupportedLanguage => {
                ApiError::bad_request("invalid_request", message)
            }
            ValidationError::Unavailable => ApiError::internal("grammar could not be loaded"),
        }
    }
}

/// Checks the bytes of a request body are valid UTF-8 and returns them as text.
///
/// # Errors
///
/// Returns [`ValidationError::NotUtf8`] when the bytes are not valid UTF-8.
pub fn decode_source(bytes: &[u8]) -> Result<&str, ValidationError> {
    std::str::from_utf8(bytes).map_err(|_| ValidationError::NotUtf8)
}

/// Checks one submitted source file for `language` against `max_source_bytes`.
///
/// Checks run in this order, so the cheap ones come first and the parser only sees bounded input:
/// the language name, the byte size, the forbidden characters, then the parse.
///
/// # Errors
///
/// Returns the first [`ValidationError`] found.
pub fn validate_source(
    language: &str,
    code: &str,
    max_source_bytes: u64,
) -> Result<(), ValidationError> {
    let grammar = grammar_for(language)?;

    let actual = u64::try_from(code.len()).unwrap_or(u64::MAX);
    if actual > max_source_bytes {
        return Err(ValidationError::TooLarge {
            limit: max_source_bytes,
            actual,
        });
    }

    check_characters(code)?;
    check_parses(language, &grammar, code)
}

fn grammar_for(language: &str) -> Result<Grammar, ValidationError> {
    match language {
        "python" => Ok(tree_sitter_python::LANGUAGE.into()),
        "javascript" => Ok(tree_sitter_javascript::LANGUAGE.into()),
        _ => Err(ValidationError::UnsupportedLanguage),
    }
}

/// Refuses NUL and control characters, except tab, newline and carriage return.
fn check_characters(code: &str) -> Result<(), ValidationError> {
    let mut line = 1;
    let mut column = 1;
    for character in code.chars() {
        let allowed = matches!(character, '\t' | '\n' | '\r') || !character.is_control();
        if !allowed {
            return Err(ValidationError::ForbiddenCharacter { line, column });
        }
        if character == '\n' {
            line += 1;
            column = 1;
        } else {
            column += 1;
        }
    }
    Ok(())
}

fn check_parses(language: &str, grammar: &Grammar, code: &str) -> Result<(), ValidationError> {
    let mut parser = Parser::new();
    parser
        .set_language(grammar)
        .map_err(|_| ValidationError::Unavailable)?;
    let Some(tree) = parser.parse(code, None) else {
        return Err(parse_error(language, (1, 1)));
    };
    let root = tree.root_node();
    if !root.has_error() {
        return Ok(());
    }
    Err(parse_error(language, first_error(root).unwrap_or((1, 1))))
}

fn parse_error(language: &str, (line, column): (usize, usize)) -> ValidationError {
    ValidationError::DoesNotParse {
        language: language.to_owned(),
        line,
        column,
    }
}

/// Finds the position (line and byte column, both from 1) of the first error or missing node.
///
/// The walk uses a cursor, not recursion, so a deeply nested tree cannot overflow the stack.
fn first_error(root: Node<'_>) -> Option<(usize, usize)> {
    let mut cursor = root.walk();
    loop {
        let node = cursor.node();
        if node.is_error() || node.is_missing() {
            let point = node.start_position();
            return Some((point.row + 1, point.column + 1));
        }
        if node.has_error() && cursor.goto_first_child() {
            continue;
        }
        loop {
            if cursor.goto_next_sibling() {
                break;
            }
            if !cursor.goto_parent() {
                return None;
            }
        }
    }
}
