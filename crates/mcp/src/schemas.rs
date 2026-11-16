//! Request types for the six tools. Each one denies unknown fields, so a misspelled or
//! smuggled argument is rejected instead of ignored.

use schemars::JsonSchema;
use serde::Deserialize;
use serde_json::Value;

/// The most samples `fit_complexity` accepts. Matches the engine's `MAX_POINTS`.
pub const MAX_FIT_POINTS: usize = 32;

/// Arguments for `validate_pack`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ValidatePackArgs {
    /// Workspace-relative pack directory, such as `packs/count-pairs`.
    pub pack_path: String,
    /// When true, any warning makes `passed` false.
    #[serde(default)]
    pub deny_warnings: bool,
}

/// Arguments for `explain_verdict`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ExplainVerdictArgs {
    /// Workspace-relative pack directory.
    pub pack_path: String,
    /// The input that was tried.
    pub input: Value,
    /// The expected value, for a wrong answer.
    #[serde(default)]
    pub expected: Option<Value>,
    /// The value the program returned, for a wrong answer.
    #[serde(default)]
    pub actual: Option<Value>,
    /// One of `wrong`, `crash` or `timeout`.
    pub kind: String,
    /// The exception class, for a crash.
    #[serde(default)]
    pub exception_class: Option<String>,
}

/// One timing sample for `fit_complexity`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct SampleArg {
    /// The input size. At least 2.
    pub n: u64,
    /// The median time in nanoseconds. Above zero.
    pub median_ns: u64,
}

/// Arguments for `fit_complexity`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FitComplexityArgs {
    /// Timing samples, with strictly increasing sizes.
    pub samples: Vec<SampleArg>,
    /// The declared class, such as `n` or `n log n`.
    pub reference_complexity: String,
}

/// Arguments for `run_reference`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct RunReferenceArgs {
    /// Workspace-relative pack directory.
    pub pack_path: String,
    /// The argument list, in parameter order.
    pub input: Value,
}

/// Search and shrink limits for `find_ghost_input`. Every field is optional.
#[derive(Debug, Default, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct BudgetArgs {
    /// Most candidates to try in the search stage.
    #[serde(default)]
    pub max_candidates: Option<u32>,
    /// Search time in milliseconds.
    #[serde(default)]
    pub search_ms: Option<u64>,
    /// Most evaluations in the shrink stage.
    #[serde(default)]
    pub max_shrink_evals: Option<u32>,
    /// Shrink time in milliseconds.
    #[serde(default)]
    pub shrink_ms: Option<u64>,
    /// Whole-hunt time in milliseconds.
    #[serde(default)]
    pub total_ms: Option<u64>,
}

/// Arguments for `find_ghost_input`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct FindGhostArgs {
    /// Workspace-relative pack directory.
    pub pack_path: String,
    /// The incorrect program inside the pack, such as `incorrect/same-index.py`.
    pub program: String,
    /// Replaces the derived seed, for replay.
    #[serde(default)]
    pub seed_override: Option<u64>,
    /// Optional search and shrink limits.
    #[serde(default)]
    pub budget: Option<BudgetArgs>,
}

/// The failure a shrink must keep.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ShrinkTargetArg {
    /// One of `wrong` or `crash`. Timeouts are not supported by this tool yet.
    pub kind: String,
}

/// Arguments for `shrink_input`.
#[derive(Debug, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ShrinkInputArgs {
    /// Workspace-relative pack directory.
    pub pack_path: String,
    /// The incorrect program inside the pack.
    pub program: String,
    /// The failing input to shrink. It must satisfy the pack schema.
    pub input: Value,
    /// The failure the shrink must keep.
    pub target: ShrinkTargetArg,
    /// Most shrink evaluations.
    pub max_evals: u32,
    /// Shrink time in milliseconds.
    pub wall_ms: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unknown_field_is_refused() {
        let text = r#"{"pack_path":"packs/x","sneaky":true}"#;
        assert!(serde_json::from_str::<ValidatePackArgs>(text).is_err());
    }

    #[test]
    fn deny_warnings_defaults_to_false() {
        let args: ValidatePackArgs =
            serde_json::from_str(r#"{"pack_path":"packs/x"}"#).expect("valid request");
        assert!(!args.deny_warnings);
    }

    #[test]
    fn a_fit_request_without_samples_is_refused() {
        assert!(
            serde_json::from_str::<FitComplexityArgs>(r#"{"reference_complexity":"n"}"#).is_err()
        );
    }
}
