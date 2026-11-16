//! Protocol smoke test: a real MCP handshake over an in-memory pipe.
//!
//! The server reads and writes JSON-RPC lines exactly as a stdio client would. The test
//! checks that `initialize` advertises tools, that `tools/list` returns exactly the six
//! tool names, and that closing the client ends the service cleanly.

use counterspell_mcp::runner::RunnerSlot;
use counterspell_mcp::server::CounterspellServer;
use rmcp::ServiceExt;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

/// The six tool names. The list is fixed by the spec and must not drift.
const TOOL_NAMES: [&str; 6] = [
    "explain_verdict",
    "find_ghost_input",
    "fit_complexity",
    "run_reference",
    "shrink_input",
    "validate_pack",
];

/// The repository root, two levels above this crate.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .canonicalize()
        .expect("repository root exists")
}

#[tokio::test]
async fn initialize_and_list_tools_return_exactly_the_six_tools() {
    let (server_io, client_io) = tokio::io::duplex(64 * 1024);
    let (client_read, mut client_write) = tokio::io::split(client_io);
    let mut lines = BufReader::new(client_read).lines();

    let server = CounterspellServer::new(repo_root(), None, RunnerSlot::unconfigured());
    let service = tokio::spawn(async move { server.serve(server_io).await });

    let initialize = json!({
        "jsonrpc": "2.0", "id": 1, "method": "initialize",
        "params": {
            "protocolVersion": "2025-06-18",
            "capabilities": {},
            "clientInfo": { "name": "smoke", "version": "0" }
        }
    });
    client_write
        .write_all(format!("{initialize}\n").as_bytes())
        .await
        .expect("send initialize");
    let reply: Value =
        serde_json::from_str(&lines.next_line().await.expect("read").expect("reply"))
            .expect("json reply");
    assert_eq!(reply["id"], 1);
    assert!(reply["result"]["capabilities"].get("tools").is_some());

    let initialized = json!({ "jsonrpc": "2.0", "method": "notifications/initialized" });
    client_write
        .write_all(format!("{initialized}\n").as_bytes())
        .await
        .expect("send initialized");

    let list = json!({ "jsonrpc": "2.0", "id": 2, "method": "tools/list" });
    client_write
        .write_all(format!("{list}\n").as_bytes())
        .await
        .expect("send tools/list");
    let listed: Value =
        serde_json::from_str(&lines.next_line().await.expect("read").expect("reply"))
            .expect("json reply");
    let mut names: Vec<String> = listed["result"]["tools"]
        .as_array()
        .expect("tools array")
        .iter()
        .map(|tool| tool["name"].as_str().unwrap_or_default().to_string())
        .collect();
    names.sort();
    assert_eq!(names, TOOL_NAMES.to_vec());

    drop(client_write);
    let finished = service.await.expect("service task");
    assert!(finished.is_ok(), "the service ends cleanly on EOF");
}
