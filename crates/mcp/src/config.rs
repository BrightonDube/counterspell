//! Non-secret configuration for the MCP process, including the sandbox socket guard.
//!
//! The sandbox socket is the only path to code execution. It must name the separate
//! rootless daemon. The main Docker daemon's socket is refused by the judge's guard,
//! which this module reuses.

use counterspell_judge::sandbox::daemon::{validate_socket_path, SocketPathError};
use std::fmt;
use std::path::{Path, PathBuf};

/// The environment variable that names the sandbox-only daemon socket. The judge reads the same name.
pub const SANDBOX_SOCKET_VAR: &str = "SANDBOX_DOCKER_SOCKET";

/// Why the sandbox endpoint is not usable.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ConfigError {
    /// No sandbox socket was configured. Pure tools still work.
    NotConfigured,
    /// A socket was configured but the guard refused it.
    Rejected(SocketPathError),
}

impl fmt::Display for ConfigError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotConfigured => formatter.write_str("no sandbox endpoint is configured"),
            Self::Rejected(error) => write!(formatter, "the sandbox endpoint was refused: {error}"),
        }
    }
}

impl std::error::Error for ConfigError {}

/// Checks the configured sandbox socket and returns it.
///
/// `raw` is the value of [`SANDBOX_SOCKET_VAR`], passed in so tests never read the environment.
///
/// # Errors
///
/// Returns [`ConfigError::NotConfigured`] when nothing is set, and
/// [`ConfigError::Rejected`] when the path is not the sandbox-only rootless socket.
pub fn sandbox_socket(raw: Option<&str>) -> Result<PathBuf, ConfigError> {
    match raw {
        None => Err(ConfigError::NotConfigured),
        Some(text) => validate_socket_path(text).map_err(ConfigError::Rejected),
    }
}

/// Finds the workspace root: the nearest directory at or above `start` whose `Cargo.toml`
/// declares a `[workspace]`. A Power launched from `kiro-power/` therefore still finds the repository.
#[must_use]
pub fn workspace_root(start: &Path) -> Option<PathBuf> {
    start
        .ancestors()
        .find(|dir| {
            std::fs::read_to_string(dir.join("Cargo.toml"))
                .is_ok_and(|text| text.contains("[workspace]"))
        })
        .map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_workspace_root_is_found_from_a_nested_directory() {
        let nested = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
        let root = workspace_root(&nested).expect("the repository is a workspace");
        assert!(root.join("crates").is_dir());
        assert!(root.join("Cargo.toml").is_file());
    }

    #[test]
    fn a_directory_outside_any_workspace_has_no_root() {
        assert_eq!(workspace_root(Path::new("/")), None);
    }

    #[test]
    fn a_missing_sandbox_socket_is_not_configured() {
        assert_eq!(sandbox_socket(None), Err(ConfigError::NotConfigured));
    }

    #[test]
    fn the_main_docker_socket_is_refused() {
        assert_eq!(
            sandbox_socket(Some("/var/run/docker.sock")),
            Err(ConfigError::Rejected(SocketPathError::MainDaemonSocket))
        );
    }

    #[test]
    fn a_tcp_endpoint_is_refused() {
        assert_eq!(
            sandbox_socket(Some("tcp://127.0.0.1:2375")),
            Err(ConfigError::Rejected(SocketPathError::NotPlainPath))
        );
    }

    #[test]
    fn a_rootless_socket_path_is_accepted() {
        assert_eq!(
            sandbox_socket(Some("/run/user/1000/docker.sock")),
            Ok(PathBuf::from("/run/user/1000/docker.sock"))
        );
    }
}
