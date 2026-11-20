//! Pack loading (problem-catalogue C19, CAT-I4, CAT-PT1, CAT-PT6).
//!
//! Reads a pack's public and private trees from disk and assembles the import
//! models. This is the only catalogue module that walks pack directories; every
//! other stage works on the bytes collected here. No file contents are ever
//! echoed into reports or logs — private bytes feed the manifest hashes and the
//! sandbox write transaction only.

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::Path;
use std::time::Duration;

use counterspell_engine::{Comparator, FailureKind, Schema, ENGINE_VERSION, GENERATOR_VERSION};
use counterspell_grimoire::ghosts::{sandbox_ghosts, SandboxGhost};
use counterspell_grimoire::hints::AuthoredLadder;
use counterspell_grimoire::meta::Meta;
use counterspell_grimoire::record::ProblemRecord;
use serde::Deserialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use thiserror::Error;

use crate::sandbox_fill::{SandboxCheck, SandboxPack};
use counterspell_judge::catalogue::manifest::{build_manifest, Manifest, Tree};
use counterspell_judge::catalogue::retired::RetiredFile;
use counterspell_judge::catalogue::starter::{starter, Lang, Signature as StarterSignature};
use counterspell_judge::catalogue::statement::constraints;
use counterspell_judge::catalogue::write::{KnownGhost, StaticImport};

/// Sub-directories whose files belong to the pack, at one level of nesting.
const PACK_DIRS: [&str; 2] = ["reference", "incorrect"];

/// The approval file is read for the gate check but never hashed (CAT-PT6).
const APPROVAL_FILE: &str = "approval.toml";

/// One pack's files from both trees: relative path → bytes.
///
/// `approval.toml` appears in the map it lives in (normally `private`) so the
/// approval gate can read it; the manifest builders exclude it from hashing.
#[derive(Debug, Default)]
pub struct PackFiles {
    /// Public-tree files, sorted by relative path.
    pub public: BTreeMap<String, Vec<u8>>,
    /// Private-tree files, sorted by relative path.
    pub private: BTreeMap<String, Vec<u8>>,
}

/// A pack could not be assembled into an import model.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum AssembleError {
    /// The pack passed static checks but `meta.toml` did not parse as a `Meta`.
    #[error("meta.toml did not parse")]
    MetaInvalid,
    /// `reference/solution.py` is missing from both trees.
    #[error("reference/solution.py missing")]
    ReferenceMissing,
    /// A required file held bytes that are not valid UTF-8.
    #[error("{0} is not valid UTF-8")]
    BadUtf8(String),
    /// `hints.toml` does not describe the four-rung ladder.
    #[error("hints.toml does not yield four rungs")]
    HintsInvalid,
    /// The signature could not be rendered as a starter.
    #[error("starter generation failed for {0}")]
    StarterInvalid(&'static str),
    /// A signature type string is not a launch type.
    #[error("signature type {0} is not a launch type")]
    SignatureTypeInvalid(String),
    /// `hidden.toml` does not parse into `[[case]]` entries.
    #[error("hidden.toml does not parse")]
    HiddenInvalid,
    /// `generator.toml` does not parse or holds an out-of-range seed.
    #[error("generator.toml does not parse")]
    GeneratorInvalid,
    /// JSON serialization failed while assembling.
    #[error("json assembly failed")]
    JsonFailed,
}

impl AssembleError {
    /// The stable refusal code reported for this failure.
    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::MetaInvalid => "meta_invalid",
            Self::ReferenceMissing => "reference_missing",
            Self::BadUtf8(_) => "bad_utf8",
            Self::HintsInvalid => "hints_invalid",
            Self::StarterInvalid(_) => "starter_invalid",
            Self::SignatureTypeInvalid(_) => "signature_type_invalid",
            Self::HiddenInvalid => "hidden_invalid",
            Self::GeneratorInvalid => "generator_invalid",
            Self::JsonFailed => "json_failed",
        }
    }
}

/// Reads every pack file under `public_dir` and `private_dir`.
///
/// A `private_dir` of `None` yields an empty private map; callers that require a
/// paired private part check that separately (`private_part_missing`).
///
/// # Errors
///
/// Returns the first `io::Error` from reading a directory or file.
pub fn read_pack(public_dir: &Path, private_dir: Option<&Path>) -> io::Result<PackFiles> {
    let public = read_tree(public_dir)?;
    let private = match private_dir {
        Some(dir) => read_tree(dir)?,
        None => BTreeMap::new(),
    };
    Ok(PackFiles { public, private })
}

/// Reads one tree: top-level files plus one level inside `reference/` and
/// `incorrect/`. Dot-names (`.gitkeep`, `.env`) and symlinks are skipped.
fn read_tree(dir: &Path) -> io::Result<BTreeMap<String, Vec<u8>>> {
    let mut files = BTreeMap::new();
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let file_type = entry.file_type()?;
        if file_type.is_symlink() {
            continue;
        }
        let Some(name) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if name.starts_with('.') {
            continue;
        }
        if file_type.is_dir() && PACK_DIRS.contains(&name.as_str()) {
            read_pack_dir(&entry.path(), &name, &mut files)?;
        } else if file_type.is_file() {
            files.insert(name, fs::read(entry.path())?);
        }
    }
    Ok(files)
}

/// Reads the files directly inside a `reference/` or `incorrect/` directory.
fn read_pack_dir(dir: &Path, name: &str, files: &mut BTreeMap<String, Vec<u8>>) -> io::Result<()> {
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let Some(inner) = entry.file_name().to_str().map(str::to_owned) else {
            continue;
        };
        if inner.starts_with('.') {
            continue;
        }
        files.insert(format!("{name}/{inner}"), fs::read(entry.path())?);
    }
    Ok(())
}

impl PackFiles {
    /// Returns the manifest over both trees. `approval.toml` is excluded (CAT-PT6).
    ///
    /// # Errors
    ///
    /// Returns `ManifestError` on a duplicate prefixed path.
    pub fn manifest(
        &self,
    ) -> Result<Manifest, counterspell_judge::catalogue::manifest::ManifestError> {
        let files: Vec<(Tree, &str, &[u8])> = self
            .public
            .iter()
            .map(|(name, bytes)| (Tree::Public, name.as_str(), bytes.as_slice()))
            .chain(
                self.private
                    .iter()
                    .map(|(name, bytes)| (Tree::Private, name.as_str(), bytes.as_slice())),
            )
            .collect();
        build_manifest(&files)
    }

    /// Returns the approval file bytes, private tree first then public.
    #[must_use]
    pub fn approval_text(&self) -> Option<&str> {
        self.text_any(APPROVAL_FILE)
    }

    /// Returns a file's text, private tree first then public. Files that hold
    /// hidden data live only in the private tree; during the layout migration a
    /// public copy still counts.
    #[must_use]
    pub fn text_any(&self, name: &str) -> Option<&str> {
        self.private
            .get(name)
            .or_else(|| self.public.get(name))
            .and_then(|bytes| std::str::from_utf8(bytes).ok())
    }

    /// Returns a file's text, erroring on non-UTF-8 bytes.
    fn text_any_utf8(&self, name: &str) -> Result<Option<&str>, AssembleError> {
        match self.private.get(name).or_else(|| self.public.get(name)) {
            Some(bytes) => std::str::from_utf8(bytes)
                .map(Some)
                .map_err(|_| AssembleError::BadUtf8(name.to_string())),
            None => Ok(None),
        }
    }

    /// Returns `(private/<path>, sha256 bytes)` for each private file, excluding
    /// `approval.toml`. These become `hidden_sources` rows.
    #[must_use]
    pub fn private_sources(&self) -> Vec<(String, [u8; 32])> {
        self.private
            .iter()
            .filter(|(name, _)| name.as_str() != APPROVAL_FILE)
            .map(|(name, bytes)| {
                let hash: [u8; 32] = Sha256::digest(bytes).into();
                (format!("private/{name}"), hash)
            })
            .collect()
    }
}

/// Builds the driver-protocol signature descriptor the column stores.
///
/// The judge writes `signature` verbatim to `/box/signature.json` when a run
/// launches, so `parameters` carry `{"kind": …}` descriptors, not meta.toml
/// type strings.
///
/// # Errors
///
/// Returns [`AssembleError::SignatureTypeInvalid`] when a parameter or the
/// return type is not a launch type.
pub fn signature_descriptor(record: &ProblemRecord) -> Result<Value, AssembleError> {
    counterspell_grimoire::record::signature_descriptor(&record.signature)
        .map_err(AssembleError::SignatureTypeInvalid)
}

/// Assembles the static import model for a pack that passed validation.
///
/// The content hash comes from the manifest; `meta` supplies samples and the
/// generator settings; `record` supplies the checked signature, schema and
/// limits. Reference code and known ghosts are read private-first.
///
/// # Errors
///
/// Returns an [`AssembleError`] with the stable refusal code for each failure.
pub fn assemble_import(
    record: &ProblemRecord,
    meta: &Meta,
    files: &PackFiles,
    manifest: &Manifest,
) -> Result<StaticImport, AssembleError> {
    let hints_text = files
        .text_any_utf8("hints.toml")?
        .ok_or(AssembleError::HintsInvalid)?;
    let ladder = AuthoredLadder::parse(hints_text).map_err(|_| AssembleError::HintsInvalid)?;
    let hints = (1..=4)
        .map(|rung| {
            ladder
                .body(rung)
                .map(|body| (i32::from(rung), body.to_string()))
        })
        .collect::<Option<Vec<_>>>()
        .ok_or(AssembleError::HintsInvalid)?;

    let reference_code = files
        .text_any_utf8("reference/solution.py")?
        .ok_or(AssembleError::ReferenceMissing)?
        .to_string();

    let known_ghosts = files
        .text_any_utf8("known_ghosts.toml")?
        .map(ghost_rows)
        .unwrap_or_default();

    let param_names: Vec<&str> = record
        .signature
        .params
        .iter()
        .map(|param| param.name.as_str())
        .collect();
    let starter_signature = StarterSignature {
        function_name: &record.signature.function,
        params: &param_names,
    };
    let starters = [(Lang::Python, "python"), (Lang::JavaScript, "javascript")]
        .iter()
        .map(|(lang, name)| {
            starter(*lang, &starter_signature)
                .map(|code| ((*name).to_string(), code))
                .map_err(|_| AssembleError::StarterInvalid(name))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let signature = signature_descriptor(record)?;
    let limits = json!({
        "time_ms": record.limits.time_ms,
        "mem_mb": record.limits.mem_mb,
    });
    let bench = record
        .bench
        .as_ref()
        .map(|bench| json!({"size_param": bench.size_param, "sizes": bench.sizes}));

    Ok(StaticImport {
        slug: record.slug.clone(),
        version: i32::try_from(record.version).map_err(|_| AssembleError::JsonFailed)?,
        content_sha256: *manifest.content_sha256(),
        manifest_public: manifest.public_map(),
        title: record.title.clone(),
        track: record.track.clone(),
        difficulty: match record.difficulty {
            counterspell_grimoire::record::Difficulty::Easy => "easy",
            counterspell_grimoire::record::Difficulty::Medium => "medium",
            counterspell_grimoire::record::Difficulty::Hard => "hard",
        }
        .to_string(),
        statement_markdown: record.statement_md.clone(),
        constraints: constraints(&record.statement_md),
        signature,
        input_schema: serde_json::to_value(&record.input_schema)
            .map_err(|_| AssembleError::JsonFailed)?,
        comparator: match record.comparator {
            Comparator::Unordered => "unordered",
            Comparator::Exact => "exact",
        }
        .to_string(),
        limits,
        reference_complexity: record.reference_complexity.clone(),
        bench,
        complexity_note: record.complexity_note.clone(),
        engine_version: ENGINE_VERSION.to_string(),
        generator_version: GENERATOR_VERSION.to_string(),
        hidden_count: i32::try_from(meta.generator.hidden_count)
            .map_err(|_| AssembleError::JsonFailed)?,
        concepts: record.concepts.clone(),
        examples: meta
            .samples
            .iter()
            .map(|sample| (sample.input.clone(), sample.expected.clone()))
            .collect(),
        starters,
        hints,
        reference_code,
        known_ghosts,
    })
}

/// Maps a known-ghosts file to import rows.
fn ghost_rows(text: &str) -> Vec<KnownGhost> {
    sandbox_ghosts(text)
        .into_iter()
        .map(|ghost| KnownGhost {
            ghost_id: ghost.known.id,
            kind: kind_name(ghost.known.kind),
            input: ghost.known.input,
            got: ghost.known.got,
            exception_class: ghost.known.exception_class,
            explanation: ghost.known.explanation,
            concepts: ghost.known.concept_tags,
        })
        .collect()
}

/// The catalogue `known_ghosts.kind` name for an engine failure kind.
fn kind_name(kind: FailureKind) -> String {
    match kind {
        FailureKind::WrongAnswer => "wrong",
        FailureKind::RuntimeError => "crash",
        FailureKind::Timeout => "timeout",
    }
    .to_string()
}

/// One `[[case]]` entry in a private `hidden.toml`.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HiddenCase {
    /// The authored input (the argument tuple).
    input: Value,
    /// The authored expected value, verified by the sandbox fill before storage.
    expected: Value,
    /// The author's note; parsed and never reported.
    #[allow(dead_code)]
    why: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct HiddenFile {
    /// The curated hidden cases.
    #[serde(default, rename = "case")]
    cases: Vec<HiddenCase>,
}

/// The private `generator.toml`; mirrors the public `[generator]` meta table's
/// fields at top level so either layout parses.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct GeneratorFile {
    /// The candidate-stream seed.
    seed: i64,
    /// The hidden-test count override; defaults to the meta value.
    hidden_count: Option<i64>,
}

/// The owned data a `validate-sandbox` run needs. Borrows nothing so it can be
/// assembled on disk order and passed across the async gate and write calls.
pub struct SandboxPackData {
    /// The recomputed combined content hash.
    pub content_sha256: [u8; 32],
    /// The pack version, used by the retired-rotation check.
    pub pack_version: i32,
    /// The pack comparator.
    pub comparator: Comparator,
    /// The launch return-type name.
    pub returns: String,
    /// The per-call time limit.
    pub limit: Duration,
    /// The public examples `(input, expected)`.
    pub examples: Vec<(Value, Value)>,
    /// The curated hidden cases `(input, authored expected)` from `hidden.toml`.
    pub curated: Vec<(Value, Value)>,
    /// The candidate-stream seed.
    pub seed: u64,
    /// How many generated hidden cases to produce.
    pub hidden_count: usize,
    /// The engine input schema.
    pub schema: Schema,
    /// The sandbox ghosts (`incorrect/` program + known record).
    pub ghosts: Vec<SandboxGhost>,
    /// The generator version recorded into `hidden_generation`.
    pub generator_version: String,
    /// Private-tree file hashes `(private/<path>, sha256)`.
    pub sources: Vec<(String, [u8; 32])>,
}

impl SandboxPackData {
    /// Borrows this data as the `SandboxPack` `validate_sandbox` consumes.
    #[must_use]
    pub fn sandbox_pack<'a>(
        &'a self,
        slug: &'a str,
        retired: &'a RetiredFile,
        image_digest: &'a str,
    ) -> SandboxPack<'a> {
        SandboxPack {
            content_sha256: self.content_sha256,
            check: SandboxCheck {
                slug,
                pack_version: self.pack_version,
                comparator: self.comparator,
                returns: &self.returns,
                limit: self.limit,
                examples: &self.examples,
                curated: &self.curated,
                seed: self.seed,
                hidden_count: self.hidden_count,
                schema: self.schema.clone(),
                ghosts: &self.ghosts,
                retired,
                image_digest,
            },
            generator_version: self.generator_version.clone(),
            sources: self.sources.clone(),
        }
    }
}

/// Assembles the sandbox inputs for one paired pack. Reads `hidden.toml`,
/// `generator.toml` and `known_ghosts.toml` private-first, mirroring
/// [`assemble_import`]'s combined-tree convention.
///
/// # Errors
///
/// Returns an [`AssembleError`] with the stable refusal code for each failure.
pub fn assemble_sandbox(
    record: &ProblemRecord,
    meta: &Meta,
    files: &PackFiles,
    manifest: &Manifest,
) -> Result<SandboxPackData, AssembleError> {
    let curated = files
        .text_any_utf8("hidden.toml")?
        .map(|text| {
            toml::from_str::<HiddenFile>(text)
                .map(|file| {
                    file.cases
                        .into_iter()
                        .map(|case| (case.input, case.expected))
                        .collect::<Vec<_>>()
                })
                .map_err(|_| AssembleError::HiddenInvalid)
        })
        .transpose()?
        .unwrap_or_default();

    let (seed, hidden_count) = match files.text_any_utf8("generator.toml")? {
        Some(text) => {
            let generator = toml::from_str::<GeneratorFile>(text)
                .map_err(|_| AssembleError::GeneratorInvalid)?;
            (
                u64::from_ne_bytes(generator.seed.to_ne_bytes()),
                generator
                    .hidden_count
                    .unwrap_or(meta.generator.hidden_count),
            )
        }
        None => (
            u64::from_ne_bytes(meta.generator.seed.to_ne_bytes()),
            meta.generator.hidden_count,
        ),
    };

    let ghosts = files
        .text_any_utf8("known_ghosts.toml")?
        .map(sandbox_ghosts)
        .unwrap_or_default();

    let limit = u64::try_from(record.limits.time_ms)
        .map(Duration::from_millis)
        .map_err(|_| AssembleError::JsonFailed)?;

    Ok(SandboxPackData {
        content_sha256: *manifest.content_sha256(),
        pack_version: i32::try_from(record.version).map_err(|_| AssembleError::JsonFailed)?,
        comparator: record.comparator,
        returns: record.signature.returns.clone(),
        limit,
        examples: meta
            .samples
            .iter()
            .map(|sample| (sample.input.clone(), sample.expected.clone()))
            .collect(),
        curated,
        seed,
        hidden_count: usize::try_from(hidden_count).map_err(|_| AssembleError::GeneratorInvalid)?,
        schema: record.input_schema.clone(),
        ghosts,
        generator_version: GENERATOR_VERSION.to_string(),
        sources: files.private_sources(),
    })
}
