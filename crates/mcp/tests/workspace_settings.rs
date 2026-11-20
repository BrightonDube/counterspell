//! Checks `.kiro/settings/mcp.json` against the Power's `kiro-power/mcp.json`
//! (counterspell-mcp-power T14, MCP-1). Files are read only; nothing is executed.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// The only tools that may be auto-approved: they never run code.
const PURE_TOOLS: [&str; 3] = ["validate_pack", "explain_verdict", "fit_complexity"];

/// The tools that run pack code in the sandbox. Each needs explicit approval.
const CODE_RUNNING_TOOLS: [&str; 3] = ["run_reference", "find_ghost_input", "shrink_input"];

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../..")
}

fn read_json(relative: &str) -> Value {
    let text = fs::read_to_string(repo_root().join(relative)).expect("the config file exists");
    serde_json::from_str(&text).expect("the config is valid JSON")
}

fn workspace_server() -> Value {
    read_json(".kiro/settings/mcp.json")["mcpServers"]["counterspell-mcp"].clone()
}

fn power_server() -> Value {
    read_json("kiro-power/mcp.json")["mcpServers"]["counterspell-mcp"].clone()
}

#[test]
fn workspace_server_runs_the_same_command_as_the_power() {
    let workspace = workspace_server();
    let power = power_server();
    assert_eq!(workspace["command"], power["command"]);
    assert_eq!(workspace["args"], power["args"]);
}

#[test]
fn workspace_server_uses_a_workspace_relative_cwd_that_reaches_the_root_manifest() {
    let cwd = workspace_server()["cwd"]
        .as_str()
        .expect("cwd is a string")
        .to_owned();
    assert_eq!(cwd, "${workspaceFolder}/kiro-power");
    let relative = cwd.trim_start_matches("${workspaceFolder}/");
    let manifest = repo_root().join(relative).join("../Cargo.toml");
    assert!(
        manifest.is_file(),
        "../Cargo.toml from the cwd must be the workspace manifest"
    );
}

#[test]
fn only_the_three_pure_tools_are_auto_approved() {
    let approved: Vec<String> = workspace_server()["autoApprove"]
        .as_array()
        .expect("autoApprove is an array")
        .iter()
        .map(|value| value.as_str().expect("tool names are strings").to_owned())
        .collect();
    let mut sorted = approved.clone();
    sorted.sort();
    let mut expected: Vec<String> = PURE_TOOLS.iter().map(|name| (*name).to_owned()).collect();
    expected.sort();
    assert_eq!(sorted, expected);
    for tool in CODE_RUNNING_TOOLS {
        assert!(
            !approved.iter().any(|name| name == tool),
            "{tool} runs code and must not be auto-approved"
        );
    }
}

#[test]
fn workspace_settings_carry_no_environment_or_secrets() {
    let workspace = workspace_server();
    assert!(
        workspace.get("env").is_none(),
        "the sandbox socket comes from the developer's own environment, never this file"
    );
    let text = fs::read_to_string(repo_root().join(".kiro/settings/mcp.json"))
        .expect("the config file exists");
    for needle in ["docker.sock", "DOCKER_HOST", "password", "token", "key"] {
        assert!(
            !text.to_lowercase().contains(&needle.to_lowercase()),
            "the settings file must not mention {needle}"
        );
    }
}

#[test]
fn the_power_manifest_itself_auto_approves_nothing() {
    assert!(power_server().get("autoApprove").is_none());
}
