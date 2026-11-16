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

use counterspell_engine::classify::{classify, Classification, Observation};
use counterspell_engine::generate::{CandidateStream, GeneratorConfig};
use counterspell_engine::{Comparator, ProgramResult, Schema};
use counterspell_grimoire::ghosts::SandboxGhost;
use counterspell_grimoire::runner::{PackRunner, RunnerError, Target};
use counterspell_grimoire::samples::accepts_return;
use counterspell_grimoire::signature::LaunchType;
use serde_json::Value;

use sqlx::PgPool;

use crate::catalogue::retired::{
    canonical_hash, canonical_json, check_retired, RetiredFile, RetiredRefusal,
};
use crate::catalogue::write::{
    sandbox_gate, write_sandbox, GateOutcome, SandboxImport, SandboxOutcome, WriteError,
};

/// Which authored or produced case a refusal names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaseSource {
    /// A public `[[samples]]` example.
    Sample,
    /// A curated hidden case from `hidden.toml`.
    Curated,
    /// A generated hidden case from `CandidateStream`.
    Generated,
}

/// One hidden case the fill produced: the input and the expected value the reference
/// computed inside the sandbox.
#[derive(Debug, Clone, PartialEq)]
pub struct HiddenFill {
    /// `curated` (authored input, verified expected) or `generated`.
    pub source: CaseSource,
    /// The case input.
    pub input: Value,
    /// The reference's own answer. For a curated case it equals the authored value,
    /// or the case was refused instead.
    pub expected: Value,
}

/// The hidden data one sandbox import writes, minus anything already static.
#[derive(Debug, Clone, PartialEq)]
pub struct SandboxFill {
    /// Hidden cases in write order: curated first, then generated.
    pub hidden: Vec<HiddenFill>,
    /// The sandbox image digest that computed the expected values.
    pub digest: String,
}

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

/// Whether a reference result is a well-shaped return for the declared launch type.
fn shaped(return_kind: Option<LaunchType>, result: &ProgramResult) -> bool {
    matches!(
        result,
        ProgramResult::Returned(value)
            if return_kind.is_some_and(|kind| accepts_return(kind, value))
    )
}

/// Runs the SBX-01..03 checks on `check` and returns the produced hidden cases.
///
/// # Errors
///
/// Returns [`RunnerError`] only for an infrastructure fault in the runner. A refused
/// pack is `Ok(FillOutcome::Refused(..))`, not an error.
pub fn sandbox_fill(
    check: &SandboxCheck<'_>,
    runner: &mut dyn PackRunner,
) -> Result<FillOutcome, RunnerError> {
    let return_kind = LaunchType::parse(check.returns);

    // SBX-01 for the public samples: the reference must return each authored value.
    for (index, (input, expected)) in check.examples.iter().enumerate() {
        let result = runner.run(Target::Reference, input, check.limit)?;
        let agrees = matches!(
            &result,
            ProgramResult::Returned(got) if check.comparator.equal(expected, got)
        );
        if !agrees {
            return Ok(FillOutcome::Refused(FillRefusal::AuthoredDisagrees {
                source: CaseSource::Sample,
                index,
            }));
        }
    }

    let mut hidden: Vec<HiddenFill> = Vec::with_capacity(check.curated.len() + check.hidden_count);
    let mut seen: HashSet<String> = HashSet::new();

    // Curated cases are authored like samples (SBX-01 verifies the claimed answer);
    // the reference must also produce a well-shaped value (SBX-02). The stored
    // expected is the reference's own answer, not the authored claim.
    for (index, (input, authored)) in check.curated.iter().enumerate() {
        let result = runner.run(Target::Reference, input, check.limit)?;
        if !shaped(return_kind, &result) {
            return Ok(FillOutcome::Refused(FillRefusal::ReferenceFailed {
                source: CaseSource::Curated,
                index,
            }));
        }
        let ProgramResult::Returned(expected) = result else {
            unreachable!("shaped() guarantees Returned")
        };
        if !check.comparator.equal(authored, &expected) {
            return Ok(FillOutcome::Refused(FillRefusal::AuthoredDisagrees {
                source: CaseSource::Curated,
                index,
            }));
        }
        seen.insert(canonical_json(input));
        hidden.push(HiddenFill {
            source: CaseSource::Curated,
            input: input.clone(),
            expected,
        });
    }

    // SBX-02 generated cases: the seeded stream yields unique candidates; a curated
    // duplicate is skipped so no hidden case repeats.
    match fill_generated(check, runner, return_kind, &mut seen) {
        Ok(cases) => hidden.extend(cases),
        Err(FillFault::Runner(error)) => return Err(error),
        Err(FillFault::Refuse(refusal)) => return Ok(FillOutcome::Refused(refusal)),
    }

    // SBX-03: every known ghost must reproduce its recorded failure on its program.
    for ghost in check.ghosts {
        let known = &ghost.known;
        let user = runner.run(Target::Program(&ghost.program), &known.input, check.limit)?;
        let reference = runner.run(Target::Reference, &known.input, check.limit)?;
        let observation = Observation { user, reference };
        let reproduces = match classify(&observation, &check.comparator) {
            Classification::Fail(predicate) => {
                known.matches(&known.input, &predicate, &observation.user)
            }
            _ => false,
        };
        if !reproduces {
            return Ok(FillOutcome::Refused(FillRefusal::GhostNotReproduced {
                ghost_id: known.id.clone(),
            }));
        }
    }

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

/// Why a fill step stopped: a runner fault, or a refusal the caller reports.
enum FillFault {
    /// `RunnerError`, propagated as `Err` for the exit-2 path.
    Runner(RunnerError),
    /// A pack refusal, reported as `Ok(Refused(..))`.
    Refuse(FillRefusal),
}

impl From<RunnerError> for FillFault {
    fn from(error: RunnerError) -> Self {
        Self::Runner(error)
    }
}

/// SBX-02: runs the reference on `hidden_count` unique generated inputs and returns
/// the produced cases. `seen` carries the curated inputs so a generated value that
/// repeats one is skipped instead of duplicated.
fn fill_generated(
    check: &SandboxCheck<'_>,
    runner: &mut dyn PackRunner,
    return_kind: Option<LaunchType>,
    seen: &mut HashSet<String>,
) -> Result<Vec<HiddenFill>, FillFault> {
    let stream = CandidateStream::new(
        check.schema.clone(),
        check.seed,
        Vec::new(),
        Vec::new(),
        GeneratorConfig::default(),
    );
    let mut cases: Vec<HiddenFill> = Vec::with_capacity(check.hidden_count);
    // The stream yields unique candidates; a tiny domain could otherwise loop on
    // duplicates forever, so the attempt count is bounded before the shortfall check.
    let attempts = check.hidden_count.saturating_mul(8).max(16);
    for candidate in stream.take(attempts) {
        if cases.len() == check.hidden_count {
            break;
        }
        let input = candidate.value;
        if !seen.insert(canonical_json(&input)) {
            continue;
        }
        let result = runner.run(Target::Reference, &input, check.limit)?;
        if !shaped(return_kind, &result) {
            return Err(FillFault::Refuse(FillRefusal::ReferenceFailed {
                source: CaseSource::Generated,
                index: cases.len(),
            }));
        }
        let ProgramResult::Returned(expected) = result else {
            unreachable!("shaped() guarantees Returned")
        };
        cases.push(HiddenFill {
            source: CaseSource::Generated,
            input,
            expected,
        });
    }
    if cases.len() < check.hidden_count {
        return Err(FillFault::Refuse(FillRefusal::TooFewCandidates {
            wanted: check.hidden_count,
            got: cases.len(),
        }));
    }
    Ok(cases)
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
