//! Session and exec specifications, and the tar archive that delivers files into a
//! sandbox.
//!
//! Nothing here takes a command string. Every argv comes from the language catalogue
//! as a list, and every file name is a validated [`FileName`], so no name can carry a
//! path. Specs are pure data and do not depend on the submitted source.

use std::time::Duration;

use thiserror::Error;
use uuid::Uuid;

use super::config::OciRuntime;
use super::policy::{fixed_env, ContainerPolicy, SANDBOX_USER, WORKING_DIR};
use crate::languages::{FileName, LanguageDefinition};

/// The fixed name of the archive entry that carries the problem's signature.
const SIGNATURE_NAME: &str = "signature.json";
/// Prefix of every sandbox container name. It contains no user text.
const CONTAINER_PREFIX: &str = "cs-sbx";
/// The UID and GID that own every file in the archive (the sandbox user).
const SANDBOX_UID: u64 = 65534;
/// The longest allowed worker id.
const WORKER_ID_MAX: usize = 64;

/// Why a spec or archive cannot be built.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SpecError {
    /// The worker id is empty, too long, or has a character outside `[A-Za-z0-9_-]`.
    #[error("the worker id must be 1 to 64 letters, digits, '-' or '_'")]
    BadWorkerId,
    /// Two of the three archive entries would have the same name.
    #[error("two files in the sandbox would have the same name")]
    DuplicateName,
    /// The archive could not be written.
    #[error("the sandbox archive could not be written")]
    ArchiveFailed,
}

/// The three files delivered into a sandbox. The names are fixed by the language and
/// the protocol, so a caller cannot choose any other name.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SandboxFiles {
    solution: FileName,
    driver: FileName,
    source: Vec<u8>,
    driver_code: Vec<u8>,
    signature: Vec<u8>,
}

impl SandboxFiles {
    /// Groups the three files. The two validated names must differ from each other
    /// and from the signature entry.
    ///
    /// # Errors
    ///
    /// Returns [`SpecError::DuplicateName`] when two entries would share a name.
    pub fn new(
        solution: FileName,
        driver: FileName,
        source: Vec<u8>,
        driver_code: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<Self, SpecError> {
        let names = [solution.as_str(), driver.as_str(), SIGNATURE_NAME];
        let distinct = names[0] != names[1] && names[0] != names[2] && names[1] != names[2];
        if !distinct {
            return Err(SpecError::DuplicateName);
        }
        Ok(Self {
            solution,
            driver,
            source,
            driver_code,
            signature,
        })
    }

    /// Each entry name with its bytes, in archive order.
    fn entries(&self) -> [(&str, &[u8]); 3] {
        [
            (self.solution.as_str(), &self.source),
            (self.driver.as_str(), &self.driver_code),
            (SIGNATURE_NAME, &self.signature),
        ]
    }
}

/// Builds the tar archive that is streamed into the container. It has exactly the
/// three entries of [`SandboxFiles`], owned by the sandbox user.
///
/// # Errors
///
/// Returns [`SpecError::ArchiveFailed`] when an entry cannot be written.
pub fn build_tar(files: &SandboxFiles) -> Result<Vec<u8>, SpecError> {
    let mut builder = tar::Builder::new(Vec::new());
    for (name, bytes) in files.entries() {
        let mut header = tar::Header::new_gnu();
        header.set_size(u64::try_from(bytes.len()).map_err(|_| SpecError::ArchiveFailed)?);
        header.set_mode(0o600);
        header.set_uid(SANDBOX_UID);
        header.set_gid(SANDBOX_UID);
        header.set_mtime(0);
        builder
            .append_data(&mut header, name, bytes)
            .map_err(|_| SpecError::ArchiveFailed)?;
    }
    builder.into_inner().map_err(|_| SpecError::ArchiveFailed)
}

/// The file name the driver gets inside the sandbox: the last part of its catalogue path.
/// Returns `None` if that part is not a valid file name.
#[must_use]
pub fn driver_file_name(lang: &LanguageDefinition) -> Option<FileName> {
    let last = lang.driver().as_str().rsplit('/').next()?;
    FileName::parse(last)
}

/// Everything needed to create one container for one session.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionSpec {
    /// The container name, built from the run id and an ordinal. It contains no user text.
    pub name: String,
    /// The hardened policy, with the run, worker and expiry labels added.
    pub policy: ContainerPolicy,
}

impl SessionSpec {
    /// Builds the spec for one session of one run.
    ///
    /// # Errors
    ///
    /// Returns [`SpecError::BadWorkerId`] when the worker id is not a safe label value.
    pub fn new(
        lang: &LanguageDefinition,
        run_cap: Duration,
        runtime: OciRuntime,
        run: Uuid,
        worker: &str,
        ordinal: u32,
        expires_unix: i64,
    ) -> Result<Self, SpecError> {
        if !valid_worker_id(worker) {
            return Err(SpecError::BadWorkerId);
        }
        let mut policy = ContainerPolicy::for_language(lang, run_cap, runtime);
        policy
            .labels
            .insert("counterspell.run".to_owned(), run.to_string());
        policy
            .labels
            .insert("counterspell.worker".to_owned(), worker.to_owned());
        policy
            .labels
            .insert("counterspell.expires".to_owned(), expires_unix.to_string());
        Ok(Self {
            name: format!("{CONTAINER_PREFIX}-{run}-{ordinal}"),
            policy,
        })
    }
}

fn valid_worker_id(worker: &str) -> bool {
    (1..=WORKER_ID_MAX).contains(&worker.len())
        && worker
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
}

/// One command to run inside a container with `exec`. The argv comes from the language
/// catalogue as a list, and the environment is the fixed set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecSpec {
    /// The program and its arguments. Never joined into a shell string.
    pub argv: Vec<String>,
    /// The user the command runs as.
    pub user: String,
    /// The working directory.
    pub working_dir: String,
    /// The fixed environment variables.
    pub env: Vec<(String, String)>,
}

impl ExecSpec {
    /// The compile step, or `None` when the language has no compile command.
    #[must_use]
    pub fn compile(lang: &LanguageDefinition) -> Option<Self> {
        (!lang.compile().is_empty()).then(|| Self::with_argv(lang.compile()))
    }

    /// The command that unpacks the archive into the working directory. The argv is fixed.
    #[must_use]
    pub fn extract() -> Self {
        Self::with_argv(&[
            "tar".to_owned(),
            "-x".to_owned(),
            "-f".to_owned(),
            "-".to_owned(),
            "-C".to_owned(),
            WORKING_DIR.to_owned(),
            "--no-same-owner".to_owned(),
        ])
    }

    /// The driver command that answers test calls.
    #[must_use]
    pub fn driver(lang: &LanguageDefinition) -> Self {
        Self::with_argv(lang.run())
    }

    /// The driver command for bench mode: the run argv followed by the catalogue's
    /// `bench_args`. The normal driver command is not changed.
    #[must_use]
    pub fn bench(lang: &LanguageDefinition) -> Self {
        let mut argv = lang.run().to_vec();
        argv.extend_from_slice(lang.bench_args());
        Self::with_argv(&argv)
    }

    fn with_argv(argv: &[String]) -> Self {
        Self {
            argv: argv.to_vec(),
            user: SANDBOX_USER.to_owned(),
            working_dir: WORKING_DIR.to_owned(),
            env: fixed_env(),
        }
    }
}
