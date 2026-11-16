//! Entry point for the `counterspell-mcp` stdio server.
//!
//! Protocol messages go to stdout. Logs go to stderr only. The server opens no socket.
//! A missing or refused sandbox endpoint leaves the pure tools running and makes the
//! code-running tools fail closed.

use counterspell_mcp::config::SANDBOX_SOCKET_VAR;
use counterspell_mcp::judge_adapter::python_image_digest;
use counterspell_mcp::runner::{production_slot, RunnerSlot};
use counterspell_mcp::server::CounterspellServer;
use rmcp::{transport::stdio, ServiceExt};
use std::path::PathBuf;
use std::process::ExitCode;

/// The environment variable that overrides the workspace root. Default: the current directory.
const WORKSPACE_VAR: &str = "COUNTERSPELL_WORKSPACE";
/// The pinned language catalogue, relative to the workspace root.
const LANGUAGES_FILE: &str = "languages/languages.toml";

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            eprintln!("counterspell-mcp stopped: {error}");
            ExitCode::FAILURE
        }
    }
}

/// Starts the service and waits until the client closes the connection.
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    let root = match std::env::var(WORKSPACE_VAR) {
        Ok(text) => PathBuf::from(text),
        Err(_) => std::env::current_dir()?,
    };
    let socket = std::env::var(SANDBOX_SOCKET_VAR).ok();
    let slot = production_slot(socket.as_deref()).unwrap_or_else(|error| {
        eprintln!("sandbox runner unavailable: {error}");
        RunnerSlot::unconfigured()
    });
    let digest = match python_image_digest(&root.join(LANGUAGES_FILE), &root) {
        Ok(digest) => Some(digest),
        Err(error) => {
            eprintln!("pinned image unavailable: {}", error.message);
            None
        }
    };
    let service = CounterspellServer::new(root, digest, slot)
        .serve(stdio())
        .await?;
    service.waiting().await?;
    Ok(())
}
