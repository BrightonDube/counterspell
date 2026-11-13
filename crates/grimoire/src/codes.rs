//! Diagnostic codes. Every code the validator emits is a constant here.
//!
//! Other modules must use these constants and must not invent a code. Each code
//! carries its severity, so a diagnostic can never disagree with the table.

use crate::report::Severity;

/// A stable diagnostic code and its severity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Code {
    /// The printed identifier, such as `MET-02`.
    pub id: &'static str,
    /// Whether the code is an error or a warning.
    pub severity: Severity,
}

const fn error(id: &'static str) -> Code {
    Code {
        id,
        severity: Severity::Error,
    }
}

const fn warning(id: &'static str) -> Code {
    Code {
        id,
        severity: Severity::Warning,
    }
}

/// A required file is missing.
pub const LAY_01: Code = error("LAY-01");
/// A file is outside the allow-list, or its name starts with a dot.
pub const LAY_02: Code = error("LAY-02");
/// A file is a symbolic link, resolves outside the pack, or has a non-UTF-8 name.
pub const LAY_03: Code = error("LAY-03");
/// A file or the whole pack is over its size cap.
pub const LAY_04: Code = error("LAY-04");
/// A text file is not UTF-8, or contains a NUL or a forbidden control character.
pub const LAY_05: Code = error("LAY-05");
/// The directory name is not a valid slug, or differs from `meta.toml` `slug`.
pub const LAY_06: Code = error("LAY-06");

/// `meta.toml` has a TOML syntax error or an unknown key.
pub const MET_01: Code = error("MET-01");
/// A required field is missing, has the wrong type, or is out of range.
pub const MET_02: Code = error("MET-02");
/// `origin` is not `original`, `licence` is not `MIT`, or a third-party field exists.
pub const MET_03: Code = error("MET-03");
/// A parameter or return type is not in the launch set.
pub const MET_04: Code = error("MET-04");
/// `input_schema` does not match `params` in count or kind.
pub const MET_05: Code = error("MET-05");
/// The engine's `check_definition` rejects the schema.
pub const MET_06: Code = error("MET-06");
/// The comparator is not allowed for this return type.
pub const MET_07: Code = error("MET-07");
/// `limits` are outside the launch caps.
pub const MET_08: Code = error("MET-08");
/// `reference_complexity` or `[bench]` is invalid, or only one of them is present.
pub const MET_09: Code = error("MET-09");

/// The title line or the heading order in `problem.md` is wrong.
pub const STM_01: Code = error("STM-01");
/// The statement is too long, or contains raw HTML.
pub const STM_02: Code = error("STM-02");

/// The sample count is not 2 to 5.
pub const SMP_01: Code = error("SMP-01");
/// A sample input fails the schema, or its expected value does not match `returns`.
pub const SMP_02: Code = error("SMP-02");
/// Two samples have the same input.
pub const SMP_03: Code = error("SMP-03");

/// The generator seed or hidden count is out of range.
pub const GEN_01: Code = error("GEN-01");
/// The engine's first 200 candidates include an invalid or repeated value.
pub const GEN_02: Code = error("GEN-02");

/// `hints.toml` does not have exactly four hints in rung order 1 to 4.
pub const HNT_01: Code = error("HNT-01");
/// A hint is too long, or has a code fence longer than 3 lines.
pub const HNT_02: Code = error("HNT-02");

/// There are more than 10 known ghosts, or an `id` is duplicated or not kebab-case.
pub const KNG_01: Code = error("KNG-01");
/// A ghost input fails the schema, or its `program` is missing or does not define the function.
pub const KNG_02: Code = error("KNG-02");
/// A ghost's `kind` does not match its `got` or `exception_class` fields.
pub const KNG_03: Code = error("KNG-03");
/// A ghost explanation is empty, too long, or uses a forbidden phrase.
pub const KNG_04: Code = error("KNG-04");
/// A ghost concept is not in the pack's `concepts`.
pub const KNG_05: Code = error("KNG-05");
/// `dogfood = true` appears in a pack whose name does not start with `_`.
pub const KNG_06: Code = error("KNG-06");

/// The reference is missing, or does not contain a column-0 `def <function>(` line.
pub const REF_01: Code = error("REF-01");

/// The reference disagrees with an authored sample.
pub const SBX_01: Code = error("SBX-01");
/// The reference crashes, times out, or returns the wrong shape on a hidden input.
pub const SBX_02: Code = error("SBX-02");
/// A known ghost does not reproduce its failure on its program.
pub const SBX_03: Code = error("SBX-03");
/// A known ghost input is not minimal; a smaller failing input was found.
pub const SBX_04: Code = warning("SBX-04");
/// The measured complexity differs from `reference_complexity`.
pub const SBX_05: Code = warning("SBX-05");
/// A runner infrastructure failure occurred (exit 2, not a pack failure).
pub const SBX_06: Code = error("SBX-06");
/// No sandbox runner is configured (exit 2).
pub const SBX_07: Code = error("SBX-07");
/// Complexity is not checked yet, because the bench fit is not available.
pub const SBX_08: Code = warning("SBX-08");

/// Every code, in the order the design lists them.
pub const ALL: &[Code] = &[
    LAY_01, LAY_02, LAY_03, LAY_04, LAY_05, LAY_06, MET_01, MET_02, MET_03, MET_04, MET_05, MET_06,
    MET_07, MET_08, MET_09, STM_01, STM_02, SMP_01, SMP_02, SMP_03, GEN_01, GEN_02, HNT_01, HNT_02,
    KNG_01, KNG_02, KNG_03, KNG_04, KNG_05, KNG_06, REF_01, SBX_01, SBX_02, SBX_03, SBX_04, SBX_05,
    SBX_06, SBX_07, SBX_08,
];

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn every_code_id_is_unique() {
        let ids: HashSet<&str> = ALL.iter().map(|code| code.id).collect();

        assert_eq!(ids.len(), ALL.len());
    }

    #[test]
    fn only_three_codes_are_warnings() {
        let warnings: Vec<&str> = ALL
            .iter()
            .filter(|code| code.severity == Severity::Warning)
            .map(|code| code.id)
            .collect();

        assert_eq!(warnings, vec!["SBX-04", "SBX-05", "SBX-08"]);
    }
}
