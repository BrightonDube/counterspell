//! The judge's own re-check of a submitted source (requirements VAL-1 and VAL-3).
//!
//! The server checks first, and the browser checks for feedback only. The judge checks
//! again because it must not trust what it reads from the database. This is not a
//! parser, and it does not look for keywords. The sandbox is the real boundary.

use thiserror::Error;

/// Why a source is refused before it reaches the sandbox.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SourceRejection {
    /// The source is larger than the language's limit. Both numbers are in bytes.
    #[error("the source is {actual} bytes; the limit is {limit} bytes")]
    TooLarge {
        /// The largest allowed size in bytes.
        limit: u64,
        /// The size of the submitted source in bytes.
        actual: u64,
    },
    /// The source is not valid UTF-8.
    #[error("the source is not valid UTF-8")]
    NotUtf8,
    /// The source contains a NUL byte.
    #[error("the source contains a NUL byte")]
    NulByte,
    /// The source contains a control character other than tab, newline or carriage return.
    #[error("the source contains a control character")]
    ControlCharacter,
}

/// Checks a submitted source and returns it as text.
///
/// The size is counted in bytes, and it is checked first, before any decoding. Then the
/// bytes must be UTF-8, with no NUL and no control character except tab, LF and CR.
///
/// # Errors
///
/// Returns the [`SourceRejection`] for the first rule the source breaks.
pub fn precheck(bytes: &[u8], max_bytes: u64) -> Result<&str, SourceRejection> {
    let actual = u64::try_from(bytes.len()).unwrap_or(u64::MAX);
    if actual > max_bytes {
        return Err(SourceRejection::TooLarge {
            limit: max_bytes,
            actual,
        });
    }
    let text = std::str::from_utf8(bytes).map_err(|_| SourceRejection::NotUtf8)?;
    for ch in text.chars() {
        if ch == '\0' {
            return Err(SourceRejection::NulByte);
        }
        if ch.is_control() && !matches!(ch, '\t' | '\n' | '\r') {
            return Err(SourceRejection::ControlCharacter);
        }
    }
    Ok(text)
}
