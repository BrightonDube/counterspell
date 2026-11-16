//! Starter code generator for Python and JavaScript (problem-catalogue C09, CAT-I17).
//!
//! The generator takes borrowed names and returns a new `String`. A starter is a function header
//! and a body that raises "not implemented", so a learner starts from a compiling skeleton.

use std::fmt;

/// Hard limit on the size of a starter, in bytes.
pub const MAX_STARTER_BYTES: usize = 8 * 1024;

/// A language with a starter template.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    /// Python 3.
    Python,
    /// JavaScript (Node).
    JavaScript,
}

/// The function header a starter needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signature<'a> {
    /// The function name.
    pub function_name: &'a str,
    /// The parameter names, in order.
    pub params: &'a [&'a str],
}

/// Why a starter could not be made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StarterError {
    /// A name is not a plain identifier.
    InvalidName(String),
    /// The starter would exceed [`MAX_STARTER_BYTES`].
    TooLarge,
}

impl fmt::Display for StarterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName(name) => write!(f, "\"{name}\" is not a plain identifier"),
            Self::TooLarge => write!(f, "the starter exceeds {MAX_STARTER_BYTES} bytes"),
        }
    }
}

impl std::error::Error for StarterError {}

/// Returns true for a name made of letters, digits and underscores that does not start with a digit.
fn is_identifier(name: &str) -> bool {
    let mut chars = name.chars();
    matches!(chars.next(), Some(first) if first.is_ascii_alphabetic() || first == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

/// Builds the starter code for `signature` in `lang`.
///
/// # Errors
///
/// Returns [`StarterError::InvalidName`] for a name that is not an identifier, and
/// [`StarterError::TooLarge`] when the output is over 8 KB.
pub fn starter(lang: Lang, signature: &Signature<'_>) -> Result<String, StarterError> {
    for name in std::iter::once(signature.function_name).chain(signature.params.iter().copied()) {
        if !is_identifier(name) {
            return Err(StarterError::InvalidName(name.to_string()));
        }
    }
    let params = signature.params.join(", ");
    let name = signature.function_name;
    let code = match lang {
        Lang::Python => {
            format!("def {name}({params}):\n    raise NotImplementedError(\"not implemented\")\n")
        }
        Lang::JavaScript => {
            format!("function {name}({params}) {{\n  throw new Error(\"not implemented\");\n}}\n")
        }
    };
    if code.len() > MAX_STARTER_BYTES {
        return Err(StarterError::TooLarge);
    }
    Ok(code)
}
