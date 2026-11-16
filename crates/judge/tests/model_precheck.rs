//! Model tests for the judge-side source re-check (requirements R14.1, R14.3, R14.6 to
//! R14.8). No Docker.
//!
//! Every shared vector in `tests/data/source_vectors.json` is run here. run-lifecycle
//! loads the same file so the server and the judge judge the same bytes the same way.

use std::fs;
use std::path::Path;

use counterspell_judge::sandbox::precheck::{precheck, SourceRejection};
use serde::Deserialize;

const VECTORS: &str = include_str!("data/source_vectors.json");

#[derive(Deserialize)]
struct VectorFile {
    vectors: Vec<Vector>,
}

#[derive(Deserialize)]
struct Vector {
    name: String,
    text: Option<String>,
    hex: Option<String>,
    repeat: Option<Repeat>,
    max_bytes: u64,
    expect: String,
}

#[derive(Deserialize)]
struct Repeat {
    text: String,
    count: usize,
}

impl Vector {
    fn bytes(&self) -> Vec<u8> {
        if let Some(text) = &self.text {
            return text.as_bytes().to_vec();
        }
        if let Some(hex) = &self.hex {
            return decode_hex(hex);
        }
        let repeat = self
            .repeat
            .as_ref()
            .expect("a vector needs text, hex or repeat");
        repeat.text.repeat(repeat.count).into_bytes()
    }
}

fn decode_hex(hex: &str) -> Vec<u8> {
    (0..hex.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex[i..i + 2], 16).unwrap())
        .collect()
}

#[test]
fn every_shared_vector_gives_its_expected_result() {
    let file: VectorFile = serde_json::from_str(VECTORS).unwrap();
    assert!(
        file.vectors.len() >= 16,
        "the shared vector file lost cases"
    );
    for vector in &file.vectors {
        let bytes = vector.bytes();
        let result = precheck(&bytes, vector.max_bytes);
        let outcome = match &result {
            Ok(_) => "ok",
            Err(SourceRejection::TooLarge { .. }) => "too_large",
            Err(SourceRejection::NotUtf8) => "not_utf8",
            Err(SourceRejection::NulByte) => "nul",
            Err(SourceRejection::ControlCharacter) => "control",
        };
        assert_eq!(outcome, vector.expect, "vector `{}`", vector.name);
    }
}

#[test]
fn a_source_one_byte_over_the_limit_reports_both_numbers() {
    let error = precheck(&[b'a'; 11], 10).unwrap_err();
    assert_eq!(
        error,
        SourceRejection::TooLarge {
            limit: 10,
            actual: 11
        }
    );
    let message = error.to_string();
    assert!(
        message.contains("11") && message.contains("10"),
        "{message}"
    );
}

#[test]
fn an_accepted_source_is_returned_as_the_same_text() {
    let source = precheck("x = 'é'\n".as_bytes(), 100).unwrap();
    assert_eq!(source, "x = 'é'\n");
}

#[test]
fn the_judge_crate_has_no_keyword_blocklist() {
    // R14.6: the size, encoding and control-character rules are the only filters. A
    // blocklist of module names would be easy to bypass and must not appear in the crate.
    let mut files = Vec::new();
    collect_rust_files(
        Path::new(env!("CARGO_MANIFEST_DIR")).join("src").as_path(),
        &mut files,
    );
    for file in files {
        let text = fs::read_to_string(&file).unwrap().to_lowercase();
        assert!(
            !text.contains("blocklist"),
            "{} mentions a blocklist",
            file.display()
        );
        assert!(
            !text.contains("denylist"),
            "{} mentions a denylist",
            file.display()
        );
    }
}

fn collect_rust_files(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(dir).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            collect_rust_files(&path, out);
        } else if path.extension().is_some_and(|ext| ext == "rs") {
            out.push(path);
        }
    }
}
