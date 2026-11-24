//! The language catalogue: one table per language in `languages/languages.toml`.
//!
//! The loader rejects anything the sandbox could not use safely. An image must be
//! pinned by digest, so a tag can never reach the Docker daemon. File names and
//! driver paths are checked, so they cannot escape the sandbox directory. Disabled
//! languages are parsed and returned, but they are marked disabled so the executor
//! refuses them.

use std::fs;
use std::path::{Path, PathBuf};

use serde::Deserialize;
use thiserror::Error;

/// The only driver protocol version this judge understands.
const SUPPORTED_PROTOCOL: u32 = 1;
/// Length of a SHA-256 digest written in hexadecimal.
const DIGEST_HEX_LEN: usize = 64;
/// The separator that introduces a digest in an image reference.
const DIGEST_MARKER: &str = "@sha256:";
/// The longest allowed file name.
const FILE_NAME_MAX: usize = 64;
/// The longest allowed language name.
const LANGUAGE_NAME_MAX: usize = 32;

/// Why a catalogue cannot be used.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum LanguageError {
    /// The catalogue file could not be read.
    #[error("the language catalogue could not be read")]
    Read(#[source] ReadFailure),
    /// The file is not valid TOML, has an unknown field, or repeats a table.
    #[error("the language catalogue is not valid: {0}")]
    Syntax(String),
    /// A language name is not lowercase letters, digits and underscores.
    #[error("language name {0:?} is not allowed")]
    BadName(String),
    /// The image is not pinned by a sha256 digest. Tags are never accepted.
    #[error("{language}: the image must be pinned by digest (name@sha256:<64 hex>)")]
    ImageNotPinned {
        /// The language whose image is wrong.
        language: String,
    },
    /// A file name or driver path is empty, too long, or leaves its directory.
    #[error("{language}: {field} is not a valid file name")]
    BadFileName {
        /// The language with the bad name.
        language: String,
        /// The field that holds the bad name.
        field: &'static str,
    },
    /// A limit is zero, so nothing could run.
    #[error("{language}: {field} must be a positive number")]
    ZeroLimit {
        /// The language with the bad limit.
        language: String,
        /// The limit that is zero.
        field: &'static str,
    },
    /// A single test may not be allowed longer than the largest allowed limit.
    #[error("{language}: time_ms_per_test cannot exceed max_time_ms_per_test")]
    TimeAboveMaximum {
        /// The language with the bad limits.
        language: String,
    },
    /// The run command must name a program to start.
    #[error("{language}: the run command is empty")]
    EmptyRun {
        /// The language with the empty command.
        language: String,
    },
    /// The driver protocol version is not one this judge speaks.
    #[error("{language}: protocol_version {found} is not supported")]
    UnsupportedProtocol {
        /// The language with the unsupported version.
        language: String,
        /// The version the catalogue declares.
        found: u32,
    },
    /// An enabled language has no driver file to run.
    #[error("{language}: the driver file is missing")]
    MissingDriver {
        /// The enabled language without its driver.
        language: String,
    },
}

/// The I/O error behind a failed catalogue read, kept comparable for tests.
#[derive(Debug, PartialEq, Eq)]
pub struct ReadFailure {
    kind: std::io::ErrorKind,
}

impl std::fmt::Display for ReadFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "{:?}", self.kind)
    }
}

impl std::error::Error for ReadFailure {}

/// An image reference that is pinned by a sha256 digest.
///
/// The field is private, so the only way to get a value is [`PinnedImage::parse`],
/// which refuses tags. A tag therefore cannot reach the Docker daemon.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinnedImage {
    reference: String,
}

impl PinnedImage {
    /// Accepts `name@sha256:<64 lowercase hex digits>` and refuses everything else.
    ///
    /// Returns `None` for a tag, a short or uppercase digest, or a missing name.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let (name, digest) = text.split_once(DIGEST_MARKER)?;
        let name_ok = !name.is_empty()
            && name
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '_' | '/' | ':' | '-'));
        let digest_ok = digest.len() == DIGEST_HEX_LEN
            && digest
                .chars()
                .all(|c| c.is_ascii_digit() || ('a'..='f').contains(&c));
        (name_ok && digest_ok).then(|| Self {
            reference: text.to_owned(),
        })
    }

    /// The full reference, for example `python:3.13-slim@sha256:…`.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.reference
    }
}

/// A file name inside a sandbox directory. It holds no path separators.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileName {
    name: String,
}

impl FileName {
    /// Accepts 1 to 64 characters from `[A-Za-z0-9_.-]`, other than `.` and `..`.
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let allowed = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-');
        let ok = (1..=FILE_NAME_MAX).contains(&text.len())
            && text.chars().all(allowed)
            && text != "."
            && text != "..";
        ok.then(|| Self {
            name: text.to_owned(),
        })
    }

    /// The file name as written in the catalogue.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

/// A driver path relative to the catalogue directory, for example `drivers/python/driver.py`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DriverPath {
    path: String,
}

impl DriverPath {
    /// Accepts a relative path whose every part is a valid [`FileName`].
    #[must_use]
    pub fn parse(text: &str) -> Option<Self> {
        let ok =
            !text.starts_with('/') && text.split('/').all(|part| FileName::parse(part).is_some());
        ok.then(|| Self {
            path: text.to_owned(),
        })
    }

    /// The path as written in the catalogue.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.path
    }
}

/// One language, validated. Fields are private; use the accessors.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageDefinition {
    enabled: bool,
    image: PinnedImage,
    file: FileName,
    driver: DriverPath,
    compile: Vec<String>,
    run: Vec<String>,
    bench_args: Vec<String>,
    mem_mb: u64,
    time_ms_per_test: u64,
    max_time_ms_per_test: u64,
    compile_budget_ms: u64,
    max_source_bytes: u64,
    exec_scratch: bool,
}

impl LanguageDefinition {
    /// Whether the executor may run this language.
    #[must_use]
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// The digest-pinned image.
    #[must_use]
    pub fn image(&self) -> &PinnedImage {
        &self.image
    }

    /// The name of the solution file inside the sandbox.
    #[must_use]
    pub fn file(&self) -> &FileName {
        &self.file
    }

    /// The driver file, relative to the catalogue directory.
    #[must_use]
    pub fn driver(&self) -> &DriverPath {
        &self.driver
    }

    /// The compile command, used as an argument list and never joined into a shell string.
    #[must_use]
    pub fn compile(&self) -> &[String] {
        &self.compile
    }

    /// The run command, used as an argument list.
    #[must_use]
    pub fn run(&self) -> &[String] {
        &self.run
    }

    /// Benchmark arguments. They are parsed and kept but not used yet.
    #[must_use]
    pub fn bench_args(&self) -> &[String] {
        &self.bench_args
    }

    /// Memory limit in MiB.
    #[must_use]
    pub fn mem_mb(&self) -> u64 {
        self.mem_mb
    }

    /// Default time limit for one test, in milliseconds.
    #[must_use]
    pub fn time_ms_per_test(&self) -> u64 {
        self.time_ms_per_test
    }

    /// The largest time limit one test may ask for, in milliseconds.
    #[must_use]
    pub fn max_time_ms_per_test(&self) -> u64 {
        self.max_time_ms_per_test
    }

    /// Time allowed for the compile step, in milliseconds.
    #[must_use]
    pub fn compile_budget_ms(&self) -> u64 {
        self.compile_budget_ms
    }

    /// Largest accepted source file, in bytes.
    #[must_use]
    pub fn max_source_bytes(&self) -> u64 {
        self.max_source_bytes
    }

    /// Whether the scratch directory may be executable. Only compiled languages need it.
    #[must_use]
    pub fn exec_scratch(&self) -> bool {
        self.exec_scratch
    }
}

/// The on-disk shape of one language table. Unknown fields are refused.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawLanguage {
    enabled: bool,
    image: String,
    file: String,
    driver: String,
    compile: Vec<String>,
    run: Vec<String>,
    bench_args: Vec<String>,
    mem_mb: u64,
    time_ms_per_test: u64,
    max_time_ms_per_test: u64,
    compile_budget_ms: u64,
    max_source_bytes: u64,
    exec_scratch: bool,
    protocol_version: u32,
}

/// Every language in the catalogue, in name order.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LanguageCatalog {
    languages: std::collections::BTreeMap<String, LanguageDefinition>,
}

impl LanguageCatalog {
    /// Reads and validates the catalogue at `path`. Driver paths are checked against
    /// the directory that holds the file.
    ///
    /// # Errors
    ///
    /// Returns [`LanguageError`] when the file cannot be read or any language is invalid.
    pub fn load(path: &Path) -> Result<Self, LanguageError> {
        let text = fs::read_to_string(path)
            .map_err(|error| LanguageError::Read(ReadFailure { kind: error.kind() }))?;
        let base: PathBuf = path
            .parent()
            .map_or_else(|| PathBuf::from("."), Path::to_path_buf);
        Self::parse(&text, &base)
    }

    /// Validates catalogue text. `base_dir` is the directory that driver paths are relative to.
    ///
    /// # Errors
    ///
    /// Returns [`LanguageError`] for the first rule the text breaks.
    pub fn parse(text: &str, base_dir: &Path) -> Result<Self, LanguageError> {
        let table: toml::Table = text
            .parse()
            .map_err(|error: toml::de::Error| LanguageError::Syntax(error.to_string()))?;

        let mut languages = std::collections::BTreeMap::new();
        for (name, value) in table {
            if !valid_language_name(&name) {
                return Err(LanguageError::BadName(name));
            }
            let raw: RawLanguage = value
                .try_into()
                .map_err(|error: toml::de::Error| LanguageError::Syntax(error.to_string()))?;
            let definition = validate(&name, raw, base_dir)?;
            languages.insert(name, definition);
        }
        Ok(Self { languages })
    }

    /// The language called `name`, enabled or not.
    #[must_use]
    pub fn get(&self, name: &str) -> Option<&LanguageDefinition> {
        self.languages.get(name)
    }

    /// The largest accepted source size for `name`, or `None` when the language is unknown.
    #[must_use]
    pub fn max_source_bytes(&self, name: &str) -> Option<u64> {
        self.get(name).map(LanguageDefinition::max_source_bytes)
    }

    /// Every language name, in order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.languages.keys().map(String::as_str)
    }
}

fn valid_language_name(name: &str) -> bool {
    let mut chars = name.chars();
    let first_ok = chars.next().is_some_and(|c| c.is_ascii_lowercase());
    first_ok
        && name.len() <= LANGUAGE_NAME_MAX
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
}

fn validate(
    name: &str,
    raw: RawLanguage,
    base_dir: &Path,
) -> Result<LanguageDefinition, LanguageError> {
    let language = name.to_owned();

    let image = PinnedImage::parse(&raw.image).ok_or_else(|| LanguageError::ImageNotPinned {
        language: language.clone(),
    })?;
    let file = FileName::parse(&raw.file).ok_or_else(|| LanguageError::BadFileName {
        language: language.clone(),
        field: "file",
    })?;
    let driver = DriverPath::parse(&raw.driver).ok_or_else(|| LanguageError::BadFileName {
        language: language.clone(),
        field: "driver",
    })?;

    if raw.protocol_version != SUPPORTED_PROTOCOL {
        return Err(LanguageError::UnsupportedProtocol {
            language,
            found: raw.protocol_version,
        });
    }

    let limits = [
        ("mem_mb", raw.mem_mb),
        ("time_ms_per_test", raw.time_ms_per_test),
        ("max_time_ms_per_test", raw.max_time_ms_per_test),
        ("compile_budget_ms", raw.compile_budget_ms),
        ("max_source_bytes", raw.max_source_bytes),
    ];
    for (field, value) in limits {
        if value == 0 {
            return Err(LanguageError::ZeroLimit { language, field });
        }
    }
    if raw.time_ms_per_test > raw.max_time_ms_per_test {
        return Err(LanguageError::TimeAboveMaximum { language });
    }

    if raw.run.first().is_none_or(String::is_empty) {
        return Err(LanguageError::EmptyRun { language });
    }

    if raw.enabled && !base_dir.join(driver.as_str()).is_file() {
        return Err(LanguageError::MissingDriver { language });
    }

    Ok(LanguageDefinition {
        enabled: raw.enabled,
        image,
        file,
        driver,
        compile: raw.compile,
        run: raw.run,
        bench_args: raw.bench_args,
        mem_mb: raw.mem_mb,
        time_ms_per_test: raw.time_ms_per_test,
        max_time_ms_per_test: raw.max_time_ms_per_test,
        compile_budget_ms: raw.compile_budget_ms,
        max_source_bytes: raw.max_source_bytes,
        exec_scratch: raw.exec_scratch,
    })
}
