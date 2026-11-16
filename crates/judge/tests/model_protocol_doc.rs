//! Model test for the driver protocol document (language-drivers T01, requirements 9 to 20).
//!
//! Every JSON fenced example in `languages/protocol.md` must parse. Answer examples must pass
//! the judge's own answer parser, so the document cannot drift from the code. No code runs.

use std::fs;
use std::path::PathBuf;

use counterspell_judge::sandbox::driver_protocol::parse_answer;
use serde_json::Value;

const PROTOCOL_DOC: &str = "../../languages/protocol.md";
const JSON_FENCE: &str = "```json";
const FENCE_END: &str = "```";

/// Every fenced JSON block in the document, in order.
fn json_examples(text: &str) -> Vec<String> {
    let mut examples = Vec::new();
    let mut rest = text;
    while let Some(start) = rest.find(JSON_FENCE) {
        let after = &rest[start + JSON_FENCE.len()..];
        let end = after
            .find(FENCE_END)
            .expect("every ```json block must be closed");
        examples.push(after[..end].trim().to_owned());
        rest = &after[end + FENCE_END.len()..];
    }
    examples
}

fn read_doc() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(PROTOCOL_DOC);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("protocol document missing at {}: {error}", path.display()))
}

#[test]
fn the_document_has_json_examples_and_each_one_parses() {
    let examples = json_examples(&read_doc());
    assert!(
        examples.len() >= 4,
        "expected call, answer, signature and error examples, found {}",
        examples.len()
    );
    for example in &examples {
        serde_json::from_str::<Value>(example)
            .unwrap_or_else(|error| panic!("example does not parse: {error}\n{example}"));
    }
}

#[test]
fn every_answer_example_passes_the_judge_answer_parser() {
    let examples = json_examples(&read_doc());
    let mut answers = 0;
    for example in &examples {
        let value: Value = serde_json::from_str(example).expect("checked above");
        let is_answer = value.get("out").is_some();
        if !is_answer {
            continue;
        }
        answers += 1;
        let id = value["t"].as_u64().expect("answer example has a numeric t");
        parse_answer(example.as_bytes(), id)
            .unwrap_or_else(|violation| panic!("answer example refused: {violation:?}"));
    }
    assert!(
        answers >= 2,
        "expected at least two answer examples, found {answers}"
    );
}
