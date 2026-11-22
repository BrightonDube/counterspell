// SPDX-License-Identifier: MIT
//! Sandbox fill (problem-catalogue C16, CAT-I14, CAT-I21, CAT-PT8, CAT-PT13, X1).
//!
//! `sandbox_fill` runs the import's code-execution checks through [`PackRunner`]:
//!
//! - SBX-01: the reference returns every authored expected value — the public
//!   examples and each curated `hidden.toml` case.
//! - SBX-02: the reference returns a well-shaped value on every generated input from
//!   the seeded [`CandidateStream`], inside the pack's own time limit.
//! - SBX-03: every known ghost reproduces its recorded failure on its `incorrect/`
//!   program.
//! - CAT-PT8/PT13: no produced hidden case hashes to a retired value, and a slug
//!   with retired values carries a rotation record for this pack version.
//!
//! On success it returns [`SandboxFill`]: the hidden cases with the expected values
//! the reference computed, held in memory only until the write step stores them in
//! one transaction. A [`RunnerError`] is infrastructure — exit 2, never a pack
//! refusal — so it propagates as `Err` while a refused pack is `Ok(Refused(..))`.

use std::collections::HashSet;
use std::time::Duration;

use counterspell_engine::{Comparator, Schema};
use counterspell_grimoire::ghosts::SandboxGhost;
use counterspell_grimoire::runner::{PackRunner, RunnerError};
use counterspell_grimoire::sandbox::{
    run_checks, Finding, Generated, HiddenSource, RetiredCheck, SandboxInputs, Scope,
};
use serde_json::Value;

use sqlx::PgPool;

use counterspell_judge::catalogue::retired::{
    canonical_hash, check_retired, RetiredFile, RetiredRefusal,
};
use counterspell_judge::catalogue::write::{
    sandbox_gate, write_sandbox, GateOutcome, SandboxImport, SandboxOutcome, WriteError,
};
pub use counterspell_judge::catalogue::write::{CaseSource, HiddenFill, SandboxFill};

/// A pack refusal from the sandbox checks. Every variant maps to a stable code;
/// none carries a hidden value, only indices, ids and hash prefixes.
#[derive(Debug, Clone, PartialEq)]
pub enum FillRefusal {
    /// SBX-01: the reference did not return an authored expected value.
    AuthoredDisagrees {
        /// Which case disagreed: a sample or a curated hidden case.
        source: CaseSource,
        /// The case's index inside its own list.
        index: usize,
    },
    /// SBX-02: the reference crashed, timed out or returned the wrong shape on a
    /// hidden input.
    ReferenceFailed {
        /// Which case failed: a curated or a generated hidden case.
        source: CaseSource,
        /// The case's index inside its own list.
        index: usize,
    },
    /// SBX-03: a known ghost did not reproduce its recorded failure.
    GhostNotReproduced {
        /// The ghost's stable id within the pack.
        ghost_id: String,
    },
    /// The generator produced fewer unique inputs than `hidden_count` asks for.
    TooFewCandidates {
        /// The requested number of generated hidden cases.
        wanted: usize,
        /// How many unique inputs the stream actually produced.
        got: usize,
    },
    /// CAT-PT8/PT13: a hidden case hashes to a retired value, or the slug has
    /// retired values without a rotation record for this version.
    Retired(RetiredRefusal),
}

impl FillRefusal {
    /// The stable refusal code, matching the design's code list.
    #[must_use]
    pub fn code(&self) -> &'static str {
        match self {
            Self::AuthoredDisagrees { .. } => "SBX-01",
            Self::ReferenceFailed { .. } => "SBX-02",
            Self::GhostNotReproduced { .. } => "SBX-03",
            Self::TooFewCandidates { .. } => "hidden_count_shortfall",
            Self::Retired(RetiredRefusal::RetiredValue { .. }) => "retired_hidden_value",
            Self::Retired(RetiredRefusal::RotationMissing { .. }) => "rotation_missing",
        }
    }
}

/// What the sandbox fill produces for one pack version.
#[derive(Debug, Clone, PartialEq)]
pub enum FillOutcome {
    /// The checks passed; the hidden cases are ready to write.
    Filled(SandboxFill),
    /// The pack was refused; nothing may be written.
    Refused(FillRefusal),
}

/// Everything the sandbox checks need for one pack version. The caller assembles it
/// from the public pack model and the private tree; nothing here reads files.
pub struct SandboxCheck<'a> {
    /// The problem slug.
    pub slug: &'a str,
    /// The pack version, used by the retired-rotation check.
    pub pack_version: i32,
    /// The pack comparator.
    pub comparator: Comparator,
    /// The launch return-type name (`int`, `list[int]`, …) for the shape check.
    pub returns: &'a str,
    /// The per-call time limit from the pack's `limits.time_ms`.
    pub limit: Duration,
    /// The public examples `(input, expected)` — SBX-01.
    pub examples: &'a [(Value, Value)],
    /// The curated hidden cases `(input, authored expected)` — verified like
    /// samples, then stored with the reference's own answer.
    pub curated: &'a [(Value, Value)],
    /// The generator seed from `generator.toml`.
    pub seed: u64,
    /// How many generated hidden cases to produce.
    pub hidden_count: usize,
    /// The engine input schema the candidates are drawn from.
    pub schema: Schema,
    /// The known ghosts to reproduce — SBX-03.
    pub ghosts: &'a [SandboxGhost],
    /// The parsed `retired-values.toml`.
    pub retired: &'a RetiredFile,
    /// The sandbox image digest the runner executes under; stamped into the fill so
    /// the write step can record it as `sandbox_image_digest`.
    pub image_digest: &'a str,
}

/// Runs the SBX-01..03 checks on `check` and returns the produced hidden cases.
///
/// The checks are Grimoire's [`run_checks`] under [`Scope::FirstError`], the same code
/// `validate` runs, so the hidden set stored here is the one the validator checked.
/// After a recorded rotation, a generated case whose hash is retired is drawn again.
///
/// # Errors
///
/// Returns [`RunnerError`] only for an infrastructure fault in the runner. A refused
/// pack is `Ok(FillOutcome::Refused(..))`, not an error.
pub fn sandbox_fill(
    check: &SandboxCheck<'_>,
    runner: &mut dyn PackRunner,
) -> Result<FillOutcome, RunnerError> {
    let rotated = check
        .retired
        .rotated
        .iter()
        .any(|record| record.slug == check.slug && record.pack_version == check.pack_version);
    let retired: HashSet<&str> = check
        .retired
        .retired
        .iter()
        .filter(|entry| entry.slug == check.slug)
        .map(|entry| entry.sha256.as_str())
        .collect();
    // An approved rotation can resample a leaked generated value. Curated cases and the
    // finished set still pass through the fail-closed retired-value check below.
    let is_retired = |input: &Value, expected: &Value| {
        retired.contains(hex(&canonical_hash(input, expected)).as_str())
    };
    let inputs = SandboxInputs {
        comparator: check.comparator,
        returns: check.returns,
        limit: check.limit,
        schema: &check.schema,
        samples: check.examples,
        curated: check.curated,
        generated: Some(Generated {
            seed: check.seed,
            count: check.hidden_count,
            retired: rotated.then_some(&is_retired as RetiredCheck<'_>),
        }),
        ghosts: check.ghosts,
    };
    let outcome = run_checks(&inputs, Scope::FirstError, runner)?;
    if let Some(refusal) = outcome.findings.iter().find_map(refusal_for) {
        return Ok(FillOutcome::Refused(refusal));
    }
    let hidden: Vec<HiddenFill> = outcome
        .hidden
        .into_iter()
        .map(|case| HiddenFill {
            source: case_source(case.source),
            input: case.input,
            expected: case.expected,
        })
        .collect();

    // CAT-PT8/PT13: every produced case is hashed and checked against the retired
    // list. A refusal names only a hash prefix.
    let case_hashes: Vec<String> = hidden
        .iter()
        .map(|case| hex(&canonical_hash(&case.input, &case.expected)))
        .collect();
    if let Err(refusal) = check_retired(check.retired, check.slug, check.pack_version, &case_hashes)
    {
        return Ok(FillOutcome::Refused(FillRefusal::Retired(refusal)));
    }

    Ok(FillOutcome::Filled(SandboxFill {
        hidden,
        digest: check.image_digest.to_owned(),
    }))
}

/// The import refusal for a finding. The SBX-04 warning never refuses a pack.
fn refusal_for(finding: &Finding) -> Option<FillRefusal> {
    Some(match finding {
        Finding::SampleDisagrees { index } => FillRefusal::AuthoredDisagrees {
            source: CaseSource::Sample,
            index: *index,
        },
        Finding::CuratedDisagrees { index } => FillRefusal::AuthoredDisagrees {
            source: CaseSource::Curated,
            index: *index,
        },
        Finding::ReferenceFailed { source, index } => FillRefusal::ReferenceFailed {
            source: case_source(*source),
            index: *index,
        },
        Finding::TooFewGenerated { wanted, got } => FillRefusal::TooFewCandidates {
            wanted: *wanted,
            got: *got,
        },
        Finding::GhostNotReproduced { id, .. } => FillRefusal::GhostNotReproduced {
            ghost_id: id.clone(),
        },
        Finding::GhostNotMinimal { .. } => return None,
    })
}

/// The catalogue case source for a Grimoire hidden source.
fn case_source(source: HiddenSource) -> CaseSource {
    match source {
        HiddenSource::Curated => CaseSource::Curated,
        HiddenSource::Generated => CaseSource::Generated,
    }
}

/// A private-tree snapshot for `validate-sandbox` (C18, CAT-I16, CAT-PT10).
///
/// The caller reads the pack files, recomputes the combined hash and assembles
/// the check; this struct carries only what the sandbox run and the write need.
pub struct SandboxPack<'a> {
    /// The recomputed combined hash of every allowed file (public and private).
    pub content_sha256: [u8; 32],
    /// The check inputs for the pack version on disk.
    pub check: SandboxCheck<'a>,
    /// The generator version recorded into `hidden_generation`.
    pub generator_version: String,
    /// Private-tree file hashes `(private/<path>, sha256)` for `hidden_sources`.
    pub sources: Vec<(String, [u8; 32])>,
}

/// What a `validate-sandbox` run did.
#[derive(Debug, Clone, PartialEq)]
pub enum ValidateOutcome {
    /// The fill ran clean and the version was stamped.
    Validated {
        /// The stamped version number.
        version: i32,
    },
    /// The latest version was already validated; refused, no write (CAT-S2).
    AlreadyValidated {
        /// The stamped version number.
        version: i32,
    },
    /// The slug has no stored version.
    Missing,
    /// The private tree changed since the static import (CAT-PT10).
    PrivateTreeChanged {
        /// The latest stored version number.
        version: i32,
    },
    /// The sandbox checks refused the pack; nothing was written.
    Refused(FillRefusal),
}

/// A `validate-sandbox` run that failed at the infrastructure layer. Both
/// variants map to exit 2; a refused pack is an outcome, never an error.
#[derive(Debug, thiserror::Error)]
pub enum ValidateError {
    /// The gate or the write failed.
    #[error(transparent)]
    Write(#[from] WriteError),
    /// The sandbox runner failed.
    #[error("sandbox runner failed: {0:?}")]
    Runner(RunnerError),
}

/// Runs `validate-sandbox` for `slug` (C18): the gate, the fill, then the write.
///
/// The gate holds the problem lock only long enough to read the latest version's
/// state, so the sandbox run does not execute code under a transaction. The
/// write re-checks the stamp under the same lock convention, so a runner that
/// stamped the version between the two calls surfaces as
/// [`ValidateOutcome::AlreadyValidated`] rather than a second write.
///
/// # Errors
///
/// Returns [`ValidateError::Runner`] for a runner fault and
/// [`ValidateError::Write`] for a database fault. Every pack-level refusal is an
/// outcome, not an error.
pub async fn validate_sandbox(
    pool: &PgPool,
    slug: &str,
    pack: &SandboxPack<'_>,
    runner: &mut dyn PackRunner,
    operator: &str,
) -> Result<ValidateOutcome, ValidateError> {
    let version = match sandbox_gate(pool, slug, &pack.content_sha256).await? {
        GateOutcome::Ready { version } => version,
        GateOutcome::AlreadyValidated { version } => {
            return Ok(ValidateOutcome::AlreadyValidated { version });
        }
        GateOutcome::Missing => return Ok(ValidateOutcome::Missing),
        GateOutcome::PrivateTreeChanged { version } => {
            return Ok(ValidateOutcome::PrivateTreeChanged { version });
        }
    };
    let fill = match sandbox_fill(&pack.check, runner).map_err(ValidateError::Runner)? {
        FillOutcome::Filled(fill) => fill,
        FillOutcome::Refused(refusal) => return Ok(ValidateOutcome::Refused(refusal)),
    };
    let import = SandboxImport {
        slug: slug.to_owned(),
        version,
        fill,
        // The seed is a 64-bit pattern, not a magnitude; bit-preserving so the
        // stored bigint reproduces the stream's seed exactly when read back.
        generator_seed: i64::from_ne_bytes(pack.check.seed.to_ne_bytes()),
        generator_version: pack.generator_version.clone(),
        sources: pack.sources.clone(),
    };
    Ok(match write_sandbox(pool, &import, operator).await? {
        SandboxOutcome::Stamped { version } => ValidateOutcome::Validated { version },
        SandboxOutcome::Unchanged { version } => ValidateOutcome::AlreadyValidated { version },
        SandboxOutcome::Missing { .. } => ValidateOutcome::Missing,
    })
}

/// Lower-case hex of a 32-byte digest.
fn hex(bytes: &[u8; 32]) -> String {
    use std::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut out, byte| {
        let _ = write!(out, "{byte:02x}");
        out
    })
}
