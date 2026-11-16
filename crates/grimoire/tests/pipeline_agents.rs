//! PA12 (PIP-2, PIP-22, PIP-38): the reviewer agent can read and cannot write or run code.
//!
//! The test reads the agent definition as JSON. The file format was confirmed against the
//! Kiro configuration reference: `tools` lists tags such as `read`, and omitting `write` or
//! `shell` removes those tools.

use std::fs;
use std::path::Path;

use serde_json::Value;

/// Tool names and tags that let an agent change files, run a shell, or run pack code.
const WRITE_OR_CODE_TOOLS: [&str; 16] = [
    "write",
    "fs_write",
    "fs_append",
    "str_replace",
    "delete_file",
    "code",
    "shell",
    "execute_bash",
    "execute_pwsh",
    "run_reference",
    "find_ghost_input",
    "shrink_input",
    "@builtin",
    "@mcp",
    "*",
    "subagent",
];

fn reviewer_definition() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(".kiro/agents/pack-reviewer.json");
    let text = fs::read_to_string(path).expect("the reviewer agent definition exists");
    serde_json::from_str(&text).expect("the reviewer agent definition is valid JSON")
}

fn tools_of(definition: &Value) -> Vec<String> {
    definition["tools"]
        .as_array()
        .expect("tools is a list")
        .iter()
        .map(|tool| tool.as_str().expect("each tool is a string").to_owned())
        .collect()
}

#[test]
fn the_reviewer_can_read_and_nothing_else() {
    let tools = tools_of(&reviewer_definition());

    assert_eq!(tools, vec!["read".to_owned()]);
}

#[test]
fn the_reviewer_has_no_write_tool_and_no_code_running_tool() {
    let tools = tools_of(&reviewer_definition());
    let forbidden: Vec<&String> = tools
        .iter()
        .filter(|tool| WRITE_OR_CODE_TOOLS.contains(&tool.as_str()))
        .collect();

    assert!(
        forbidden.is_empty(),
        "the reviewer must not have {forbidden:?}"
    );
}

#[test]
fn the_reviewer_prompt_names_the_checklist_and_refuses_author_notes() {
    let definition = reviewer_definition();
    let prompt = definition["prompt"].as_str().expect("the prompt is text");

    assert!(prompt.contains(".kiro/pipeline/checklists/reviewer.md"));
    assert!(
        prompt.to_lowercase().contains("author"),
        "the prompt must say it has no author notes"
    );
}

fn author_definition() -> Value {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../..")
        .join(".kiro/agents/pack-author.json");
    let text = fs::read_to_string(path).expect("the author agent definition exists");
    serde_json::from_str(&text).expect("the author agent definition is valid JSON")
}

#[test]
fn the_author_can_write_its_pack_but_cannot_run_code() {
    let tools = tools_of(&author_definition());
    let code_tools: Vec<&String> = tools
        .iter()
        .filter(|tool| WRITE_OR_CODE_TOOLS.contains(&tool.as_str()) && *tool != "write")
        .collect();

    assert!(
        tools.contains(&"write".to_owned()),
        "the author must be able to write its pack"
    );
    assert!(
        code_tools.is_empty(),
        "the author must not have {code_tools:?}"
    );
}

#[test]
fn the_author_prompt_forbids_publishing_approval_and_review_writes() {
    let definition = author_definition();
    let prompt = definition["prompt"].as_str().expect("the prompt is text");

    assert!(
        prompt.contains("published"),
        "the prompt must forbid setting published"
    );
    assert!(
        prompt.contains("approval.toml"),
        "the prompt must forbid approval.toml"
    );
    assert!(
        prompt.contains("docs/pack-reviews"),
        "the prompt must forbid review writes"
    );
}
