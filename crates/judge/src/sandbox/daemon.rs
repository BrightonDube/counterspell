//! Daemon identity: the judge may only talk to the rootless sandbox daemon, through an
//! explicit Unix socket path. This module never reads `DOCKER_HOST`, and it refuses
//! the main Docker daemon's sockets.
//!
//! Only this file and `bollard_backend.rs` import bollard (see `Cargo.toml`).

use std::path::PathBuf;

use thiserror::Error;

use crate::sandbox::config::OciRuntime;

/// Socket paths of the main Docker daemon. The judge must never use these.
const MAIN_DAEMON_SOCKETS: [&str; 2] = ["/var/run/docker.sock", "/run/docker.sock"];

/// The data directory of the main Docker daemon. A sandbox daemon must not use it.
const MAIN_DATA_ROOT: &str = "/var/lib/docker";

/// Why a socket path was refused.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum SocketPathError {
    /// The path is empty or only spaces.
    #[error("the sandbox daemon socket path is empty")]
    Empty,
    /// The path is relative. Only an absolute path names one socket.
    #[error("the sandbox daemon socket path must be absolute")]
    NotAbsolute,
    /// The text is a URL such as `tcp://` or `unix://`. Only a plain path is accepted.
    #[error("the sandbox daemon socket must be a plain path, not a URL")]
    NotPlainPath,
    /// The path is the main Docker daemon's socket.
    #[error("the main Docker daemon socket is not allowed for sandboxes")]
    MainDaemonSocket,
}

/// Checks a configured socket path and returns it as a path.
///
/// # Errors
///
/// Returns [`SocketPathError`] for an empty, relative, URL-style or main-daemon path.
pub fn validate_socket_path(raw: &str) -> Result<PathBuf, SocketPathError> {
    let text = raw.trim();
    if text.is_empty() {
        return Err(SocketPathError::Empty);
    }
    if text.contains("://") {
        return Err(SocketPathError::NotPlainPath);
    }
    let path = PathBuf::from(text);
    if !path.is_absolute() {
        return Err(SocketPathError::NotAbsolute);
    }
    if MAIN_DAEMON_SOCKETS.contains(&text) {
        return Err(SocketPathError::MainDaemonSocket);
    }
    Ok(path)
}

/// The facts about a daemon that the judge needs, copied out of the daemon's `info`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DaemonFacts {
    /// True when the daemon reports the rootless security option.
    pub rootless: bool,
    /// The daemon's data directory.
    pub data_root: String,
    /// True when the daemon uses cgroup v2.
    pub cgroup_v2: bool,
    /// The cgroup driver name, for example `systemd`. `none` means no limits.
    pub cgroup_driver: String,
    /// The OCI runtimes the daemon has registered.
    pub runtimes: Vec<String>,
    /// Warnings the daemon printed in its `info`.
    pub warnings: Vec<String>,
}

/// Why the daemon is not the sandbox daemon the judge expects.
#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DaemonCheckError {
    /// The daemon is not rootless, so it could control the host.
    #[error("the sandbox daemon is not rootless")]
    NotRootless,
    /// The daemon keeps its data in the main Docker directory.
    #[error("the sandbox daemon uses the main Docker data directory")]
    MainDataRoot,
    /// The daemon does not use cgroup v2, so the memory and PID limits may not apply.
    #[error("the sandbox daemon does not use cgroup v2")]
    NotCgroupV2,
    /// The daemon has no cgroup driver, so it cannot enforce limits.
    #[error("the sandbox daemon has no cgroup driver")]
    NoCgroupDriver,
    /// The runtime the configuration asks for is not registered with the daemon.
    #[error("the sandbox daemon does not register the runtime {0}")]
    RuntimeMissing(String),
    /// The daemon warned that it cannot apply a limit.
    #[error("the sandbox daemon cannot apply a limit: {0}")]
    LimitSupport(String),
}

/// The registered name of an OCI runtime.
fn runtime_name(runtime: OciRuntime) -> &'static str {
    match runtime {
        OciRuntime::Runc => "runc",
        OciRuntime::Runsc => "runsc",
    }
}

/// Checks daemon facts against what the judge needs. Missing facts count as failures.
///
/// # Errors
///
/// Returns the first [`DaemonCheckError`] that applies, in the order of the checks above.
pub fn check_facts(facts: &DaemonFacts, runtime: OciRuntime) -> Result<(), DaemonCheckError> {
    if !facts.rootless {
        return Err(DaemonCheckError::NotRootless);
    }
    if facts.data_root == MAIN_DATA_ROOT {
        return Err(DaemonCheckError::MainDataRoot);
    }
    if !facts.cgroup_v2 {
        return Err(DaemonCheckError::NotCgroupV2);
    }
    if facts.cgroup_driver.is_empty() || facts.cgroup_driver == "none" {
        return Err(DaemonCheckError::NoCgroupDriver);
    }
    let wanted = runtime_name(runtime);
    if !facts.runtimes.iter().any(|name| name == wanted) {
        return Err(DaemonCheckError::RuntimeMissing(wanted.to_owned()));
    }
    // A warning that a control is not supported means a limit may not apply.
    if let Some(warning) = facts.warnings.iter().find(|text| {
        let lower = text.to_ascii_lowercase();
        lower.contains("no ") && lower.contains("support")
    }) {
        return Err(DaemonCheckError::LimitSupport(warning.clone()));
    }
    Ok(())
}

#[cfg(feature = "sandbox")]
mod bollard_edge {
    use std::path::Path;
    use std::time::Duration;

    use bollard::models::{SystemInfo, SystemInfoCgroupVersionEnum};
    use bollard::{Docker, API_DEFAULT_VERSION};

    use super::{check_facts, DaemonCheckError, DaemonFacts, OciRuntime};
    use crate::sandbox::daemon::{validate_socket_path, SocketPathError};

    /// Why the judge could not confirm the sandbox daemon.
    #[derive(Debug, thiserror::Error)]
    pub enum DaemonError {
        /// The configured socket path was refused.
        #[error(transparent)]
        Path(#[from] SocketPathError),
        /// The socket could not be opened.
        #[error("could not open the sandbox daemon socket: {0}")]
        Connect(String),
        /// The daemon did not answer within the API timeout.
        #[error("the sandbox daemon did not answer in time")]
        ApiTimeout,
        /// The daemon answered with an error.
        #[error("the sandbox daemon returned an error: {0}")]
        Api(String),
        /// The daemon is not the expected rootless sandbox daemon.
        #[error(transparent)]
        Check(#[from] DaemonCheckError),
    }

    impl From<&SystemInfo> for DaemonFacts {
        fn from(info: &SystemInfo) -> Self {
            Self {
                rootless: info
                    .security_options
                    .as_ref()
                    .is_some_and(|options| options.iter().any(|o| o == "name=rootless")),
                data_root: info.docker_root_dir.clone().unwrap_or_default(),
                cgroup_v2: matches!(info.cgroup_version, Some(SystemInfoCgroupVersionEnum::_2)),
                cgroup_driver: info
                    .cgroup_driver
                    .map(|driver| driver.to_string())
                    .unwrap_or_default(),
                runtimes: info
                    .runtimes
                    .as_ref()
                    .map(|map| map.keys().cloned().collect())
                    .unwrap_or_default(),
                warnings: info.warnings.clone().unwrap_or_default(),
            }
        }
    }

    /// Opens a client on the sandbox socket at `socket`. Only this path is used.
    ///
    /// # Errors
    ///
    /// Returns [`DaemonError`] when the path is refused or the socket cannot be opened.
    pub fn connect(socket: &Path, api_timeout: Duration) -> Result<Docker, DaemonError> {
        let text = socket
            .to_str()
            .ok_or_else(|| DaemonError::Connect("the socket path is not UTF-8".to_owned()))?;
        let checked = validate_socket_path(text)?;
        let timeout_secs = api_timeout.as_secs().max(1);
        Docker::connect_with_unix(
            checked.to_string_lossy().as_ref(),
            timeout_secs,
            API_DEFAULT_VERSION,
        )
        .map_err(|error| DaemonError::Connect(error.to_string()))
    }

    /// Reads the daemon's `info` within the API timeout and checks it.
    ///
    /// # Errors
    ///
    /// Returns [`DaemonError`] on timeout, an API error or a failed identity check.
    pub async fn verify(
        docker: &Docker,
        runtime: OciRuntime,
        api_timeout: Duration,
    ) -> Result<DaemonFacts, DaemonError> {
        let info = tokio::time::timeout(api_timeout, docker.info())
            .await
            .map_err(|_| DaemonError::ApiTimeout)?
            .map_err(|error| DaemonError::Api(error.to_string()))?;
        let facts = DaemonFacts::from(&info);
        check_facts(&facts, runtime)?;
        Ok(facts)
    }
}

#[cfg(feature = "sandbox")]
pub use bollard_edge::{connect, verify, DaemonError};
