// SPDX-License-Identifier: MIT
//! Starter code generator for the catalogue languages (problem-catalogue C09, CAT-I17,
//! language-drivers T16).
//!
//! The generator takes borrowed names and returns a new `String`. A starter is a function
//! header and a body that raises "not implemented", so a learner starts from a compiling
//! skeleton. Java and C++ starters are typed, so each parameter and the return value carry
//! the pack's launch type name; the C++ starter also emits the `counterspell_call` shim the
//! driver uses to reach the learner's function.

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
    /// Java 21 (temurin).
    Java,
    /// C++20 (gcc).
    Cpp,
}

/// One starter parameter: the name the learner calls it, plus its launch type name
/// (`int`, `str`, `list[int]` or `list[list[int]]`). Python and JavaScript ignore `kind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Param<'a> {
    /// The parameter name.
    pub name: &'a str,
    /// The launch type name from the pack's `meta.toml`.
    pub kind: &'a str,
}

/// The function header a starter needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Signature<'a> {
    /// The function name.
    pub function_name: &'a str,
    /// The parameters, in order.
    pub params: &'a [Param<'a>],
    /// The return type's launch type name. Python and JavaScript ignore it.
    pub returns: &'a str,
}

/// Why a starter could not be made.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StarterError {
    /// A name is not a plain identifier.
    InvalidName(String),
    /// A parameter or return type name is not a launch type.
    UnknownType(String),
    /// The starter would exceed [`MAX_STARTER_BYTES`].
    TooLarge,
}

impl fmt::Display for StarterError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidName(name) => write!(f, "\"{name}\" is not a plain identifier"),
            Self::UnknownType(name) => write!(f, "\"{name}\" is not a launch type"),
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

/// The Java type for a launch type name.
fn java_type(kind: &str) -> Result<&'static str, StarterError> {
    match kind {
        "int" => Ok("long"),
        "str" => Ok("String"),
        "list[int]" => Ok("long[]"),
        "list[list[int]]" => Ok("long[][]"),
        _ => Err(StarterError::UnknownType(kind.to_owned())),
    }
}

/// The C++ type for a launch type name.
fn cpp_type(kind: &str) -> Result<&'static str, StarterError> {
    match kind {
        "int" => Ok("long long"),
        "str" => Ok("std::string"),
        "list[int]" => Ok("std::vector<long long>"),
        "list[list[int]]" => Ok("std::vector<std::vector<long long>>"),
        _ => Err(StarterError::UnknownType(kind.to_owned())),
    }
}

fn java_starter(signature: &Signature<'_>) -> Result<String, StarterError> {
    let mut params = Vec::with_capacity(signature.params.len());
    for param in signature.params {
        params.push(format!("{} {}", java_type(param.kind)?, param.name));
    }
    Ok(format!(
        "public class Solution {{\n    public static {} {}({}) {{\n        throw new UnsupportedOperationException(\"not implemented\");\n    }}\n}}\n",
        java_type(signature.returns)?,
        signature.function_name,
        params.join(", ")
    ))
}

fn cpp_starter(signature: &Signature<'_>) -> Result<String, StarterError> {
    let mut params = Vec::with_capacity(signature.params.len());
    let mut unwrapped = Vec::with_capacity(signature.params.len());
    for (index, param) in signature.params.iter().enumerate() {
        let ty = cpp_type(param.kind)?;
        params.push(format!("{ty} {}", param.name));
        unwrapped.push(format!("from_json<{ty}>(args[{index}])"));
    }
    let name = signature.function_name;
    Ok(format!(
        "{} {}({}) {{\n    throw std::runtime_error(\"not implemented\");\n}}\n\nJSONValue counterspell_call(const std::vector<JSONValue>& args) {{\n    return to_json({}({}));\n}}\n",
        cpp_type(signature.returns)?,
        name,
        params.join(", "),
        name,
        unwrapped.join(", ")
    ))
}

/// Builds the starter code for `signature` in `lang`.
///
/// # Errors
///
/// Returns [`StarterError::InvalidName`] for a name that is not an identifier,
/// [`StarterError::UnknownType`] for a type name outside the launch set (Java and C++
/// starters only; Python and JavaScript ignore type names), and
/// [`StarterError::TooLarge`] when the output is over 8 KB.
pub fn starter(lang: Lang, signature: &Signature<'_>) -> Result<String, StarterError> {
    for name in std::iter::once(signature.function_name)
        .chain(signature.params.iter().map(|param| param.name))
    {
        if !is_identifier(name) {
            return Err(StarterError::InvalidName(name.to_string()));
        }
    }
    let params: Vec<&str> = signature.params.iter().map(|param| param.name).collect();
    let name = signature.function_name;
    let code = match lang {
        Lang::Python => {
            format!(
                "def {name}({}):\n    raise NotImplementedError(\"not implemented\")\n",
                params.join(", ")
            )
        }
        Lang::JavaScript => {
            format!(
                "function {name}({}) {{\n  throw new Error(\"not implemented\");\n}}\n",
                params.join(", ")
            )
        }
        Lang::Java => java_starter(signature)?,
        Lang::Cpp => cpp_starter(signature)?,
    };
    if code.len() > MAX_STARTER_BYTES {
        return Err(StarterError::TooLarge);
    }
    Ok(code)
}
