//! Validates the Agent Plugins Power manifest against the 1.0.0 schema rules.
//!
//! The rules come from agent-plugins.org (plugin.schema.json and mcp.schema.json), as
//! recorded in docs/kiro-log/counterspell-mcp-power-decisions.md. The checks are
//! hand-written because the workspace has no JSON Schema validator. They read files only.

use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};

/// The keys the closed plugin.json schema allows at the top level.
const PLUGIN_KEYS: [&str; 10] = [
    "$schema",
    "name",
    "version",
    "description",
    "author",
    "homepage",
    "repository",
    "license",
    "keywords",
    "extensions",
];

/// The keys one stdio MCP server may have in mcp.json.
const STDIO_KEYS: [&str; 5] = ["type", "command", "args", "env", "cwd"];

/// The keywords the Power must declare.
const REQUIRED_KEYWORDS: [&str; 4] = ["ghost input", "grimoire", "problem pack", "shrink"];

/// The Power directory, two levels above this crate.
fn power_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../kiro-power")
}

fn read_json(name: &str) -> Value {
    let text = fs::read_to_string(power_dir().join(name)).expect("the manifest file exists");
    serde_json::from_str(&text).expect("the manifest is valid JSON")
}

/// The name rule from plugin.schema.json: lowercase letters, digits, dots and hyphens,
/// starting and ending with a letter or digit, with no `--` or `..`.
fn is_valid_name(name: &str) -> bool {
    let bytes = name.as_bytes();
    let edges_ok = bytes
        .first()
        .zip(bytes.last())
        .is_some_and(|(first, last)| is_alnum(*first) && is_alnum(*last));
    edges_ok
        && name.len() <= 64
        && name
            .bytes()
            .all(|byte| is_alnum(byte) || byte == b'-' || byte == b'.')
        && !name.contains("--")
        && !name.contains("..")
}

fn is_alnum(byte: u8) -> bool {
    byte.is_ascii_lowercase() || byte.is_ascii_digit()
}

#[test]
fn plugin_manifest_uses_only_the_closed_schema_keys() {
    let plugin = read_json("plugin.json");
    let object = plugin.as_object().expect("plugin.json is an object");
    for key in object.keys() {
        assert!(PLUGIN_KEYS.contains(&key.as_str()), "unexpected key {key}");
    }
}

#[test]
fn plugin_manifest_declares_the_schema_and_a_valid_name() {
    let plugin = read_json("plugin.json");
    assert_eq!(
        plugin["$schema"],
        "https://agent-plugins.org/schemas/1.0.0/plugin.schema.json"
    );
    let name = plugin["name"].as_str().expect("name is a string");
    assert!(is_valid_name(name), "name {name} breaks the schema pattern");
    assert_eq!(name, "counterspell-mcp");
}

#[test]
fn plugin_manifest_declares_the_required_keywords() {
    let plugin = read_json("plugin.json");
    let keywords: Vec<&str> = plugin["keywords"]
        .as_array()
        .expect("keywords is an array")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    for required in REQUIRED_KEYWORDS {
        assert!(keywords.contains(&required), "missing keyword {required}");
    }
}

#[test]
fn mcp_config_declares_one_stdio_server_with_a_plugin_relative_cwd() {
    let mcp = read_json("mcp.json");
    assert_eq!(
        mcp["$schema"],
        "https://agent-plugins.org/schemas/1.0.0/mcp.schema.json"
    );
    let servers = mcp["mcpServers"]
        .as_object()
        .expect("mcpServers is an object");
    assert_eq!(servers.len(), 1, "exactly one server is bundled");
    let server = &servers["counterspell-mcp"];
    assert_eq!(server["type"], "stdio");
    for key in server.as_object().expect("server is an object").keys() {
        assert!(
            STDIO_KEYS.contains(&key.as_str()),
            "unexpected server key {key}"
        );
    }
    let cwd = server["cwd"].as_str().expect("cwd is a string");
    assert!(
        cwd.starts_with("${PLUGIN_ROOT}"),
        "cwd must be plugin-rooted"
    );
}

#[test]
fn mcp_config_runs_the_counterspell_mcp_package_from_the_repository() {
    let mcp = read_json("mcp.json");
    let args: Vec<&str> = mcp["mcpServers"]["counterspell-mcp"]["args"]
        .as_array()
        .expect("args is an array")
        .iter()
        .filter_map(Value::as_str)
        .collect();
    assert!(args.contains(&"counterspell-mcp"));
    assert!(args.contains(&"../Cargo.toml"));
}

#[test]
fn power_files_hold_no_absolute_developer_path_and_no_approval_list() {
    for name in ["plugin.json", "mcp.json"] {
        let text = fs::read_to_string(power_dir().join(name)).expect("file exists");
        assert!(!text.contains("/mnt/"), "{name} holds a machine path");
        assert!(!text.contains("/home/"), "{name} holds a machine path");
        assert!(!text.contains("C:\\"), "{name} holds a machine path");
        assert!(
            !text.contains("autoApprove"),
            "{name} must not carry an approval list"
        );
    }
}

/// The six canonical tool names. The skills must use these names and no others.
const CANONICAL_TOOLS: [&str; 6] = [
    "validate_pack",
    "explain_verdict",
    "fit_complexity",
    "run_reference",
    "find_ghost_input",
    "shrink_input",
];

#[test]
fn the_two_skills_together_name_all_six_canonical_tools() {
    let mut combined = String::new();
    for skill in ["author-problem", "hunt-ghost"] {
        let path = power_dir().join("skills").join(skill).join("SKILL.md");
        let text = fs::read_to_string(&path).expect("skill file exists");
        assert!(text.starts_with("---\n"), "{skill} has frontmatter");
        assert!(
            text.contains(&format!("name: {skill}")),
            "{skill} names itself"
        );
        combined.push_str(&text);
    }
    for tool in CANONICAL_TOOLS {
        assert!(combined.contains(tool), "no skill names {tool}");
    }
}

#[test]
fn the_skills_never_tell_kiro_to_auto_approve_a_code_running_tool() {
    for skill in ["author-problem", "hunt-ghost"] {
        let path = power_dir().join("skills").join(skill).join("SKILL.md");
        let text = fs::read_to_string(path).expect("skill file exists");
        assert!(!text.contains("auto-approve find_ghost_input"));
        assert!(!text.contains("auto-approve run_reference"));
        assert!(!text.contains("auto-approve shrink_input"));
    }
}
