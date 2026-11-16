//! PA10 (PIP-21, PIP-47, PIP-46): the reviewer checklist covers every category, has unique IDs,
//! and every `RV-*` ID that a stage prompt cites exists in it.
//!
//! The parser is plain string handling: lines, backtick pairs and `split_once`. It reads the
//! checklist as text and never runs anything.

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// The seven categories of PIP-21, by the key used in each ID (`RV-<key>-<n>`).
const CATEGORY_KEYS: [&str; 7] = [
    "correctness",
    "clarity",
    "originality",
    "difficulty",
    "hints",
    "ghost",
    "access",
];

/// The repository root, found from this crate's manifest directory.
fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("..").join("..")
}

/// Returns the IDs of the checklist items in `text`, in file order.
///
/// An item line is a dash, a space, then an RV ID in backticks. The ID is the text
/// between the first pair of backticks on that line.
fn checklist_ids(text: &str) -> Vec<String> {
    text.lines()
        .filter(|line| line.trim_start().starts_with("- `RV-"))
        .filter_map(|line| line.split('`').nth(1))
        .map(str::to_owned)
        .collect()
}

/// Splits `RV-<key>-<n>` into its key and number. Returns `None` for any other shape.
fn split_id(id: &str) -> Option<(&str, &str)> {
    let rest = id.strip_prefix("RV-")?;
    let (key, number) = rest.rsplit_once('-')?;
    if number.is_empty() || !number.bytes().all(|byte| byte.is_ascii_digit()) {
        return None;
    }
    Some((key, number))
}

/// Every `RV-*` ID cited inside backticks in `text`.
fn cited_ids(text: &str) -> BTreeSet<String> {
    text.split('`')
        .skip(1)
        .step_by(2)
        .filter(|segment| segment.starts_with("RV-"))
        .map(str::to_owned)
        .collect()
}

/// Why a checklist is refused.
#[derive(Debug, PartialEq, Eq)]
enum ChecklistError {
    MissingCategory(String),
    DuplicateId(String),
    MalformedId(String),
}

/// Checks that every category has an item, that every ID is well formed, and that no ID repeats.
fn check_checklist(text: &str) -> Result<Vec<String>, ChecklistError> {
    let ids = checklist_ids(text);
    let mut seen = BTreeSet::new();
    for id in &ids {
        let Some((key, _)) = split_id(id) else {
            return Err(ChecklistError::MalformedId(id.clone()));
        };
        if !seen.insert(id.clone()) {
            return Err(ChecklistError::DuplicateId(id.clone()));
        }
        if !CATEGORY_KEYS.contains(&key) {
            return Err(ChecklistError::MalformedId(id.clone()));
        }
    }
    for key in CATEGORY_KEYS {
        if !ids
            .iter()
            .any(|id| split_id(id).is_some_and(|(found, _)| found == key))
        {
            return Err(ChecklistError::MissingCategory(key.to_owned()));
        }
    }
    Ok(ids)
}

fn real_checklist() -> String {
    fs::read_to_string(repo_root().join(".kiro/pipeline/checklists/reviewer.md"))
        .expect("the reviewer checklist exists at .kiro/pipeline/checklists/reviewer.md")
}

fn fixture_checklist(name: &str) -> String {
    fs::read_to_string(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/fixtures/pipeline")
            .join(name)
            .join("reviewer.md"),
    )
    .expect("fixture is readable")
}

#[test]
fn the_reviewer_checklist_has_an_item_in_each_of_the_seven_categories() {
    let ids = check_checklist(&real_checklist()).expect("the real checklist is complete");

    assert!(
        ids.len() >= 7,
        "expected at least one item per category, got {}",
        ids.len()
    );
}

#[test]
fn every_rv_id_cited_by_a_stage_prompt_exists_in_the_checklist() {
    let checklist_ids: BTreeSet<String> = checklist_ids(&real_checklist()).into_iter().collect();
    let prompts_dir = repo_root().join(".kiro/pipeline/prompts");
    let mut cited = BTreeSet::new();
    for entry in fs::read_dir(prompts_dir).expect("prompts folder is readable") {
        let path = entry.expect("directory entry is readable").path();
        let text = fs::read_to_string(&path).expect("prompt is UTF-8");
        cited.extend(cited_ids(&text));
    }

    let missing: Vec<&String> = cited
        .iter()
        .filter(|id| !checklist_ids.contains(*id))
        .collect();

    assert!(
        missing.is_empty(),
        "cited but not in the checklist: {missing:?}"
    );
    assert!(
        !cited.is_empty(),
        "the prompts should cite at least one checklist ID"
    );
}

#[test]
fn a_checklist_missing_a_category_is_refused() {
    let text = fixture_checklist("checklist-missing-category");

    assert_eq!(
        check_checklist(&text),
        Err(ChecklistError::MissingCategory("access".to_owned()))
    );
}

#[test]
fn a_repeated_id_is_refused() {
    let text = "- `RV-correctness-1` one\n- `RV-correctness-1` again\n";

    assert_eq!(
        check_checklist(text),
        Err(ChecklistError::DuplicateId("RV-correctness-1".to_owned()))
    );
}

#[test]
fn an_id_without_a_number_is_refused() {
    let text = "- `RV-clarity` no number\n";

    assert_eq!(
        check_checklist(text),
        Err(ChecklistError::MalformedId("RV-clarity".to_owned()))
    );
}

/// Every `AAA-00` style code named in `text` (three capitals, a hyphen, two digits).
fn named_codes(text: &str) -> BTreeSet<String> {
    text.split(|character: char| !(character.is_ascii_alphanumeric() || character == '-'))
        .filter(|token| {
            token.len() == 6
                && token.as_bytes()[..3].iter().all(u8::is_ascii_uppercase)
                && token.as_bytes()[3] == b'-'
                && token.as_bytes()[4..].iter().all(u8::is_ascii_digit)
        })
        .map(str::to_owned)
        .collect()
}

/// The code IDs the validator defines in `codes.rs`, read from `error("...")` and `warning("...")`.
fn grimoire_codes() -> BTreeSet<String> {
    let text = fs::read_to_string(repo_root().join("crates/grimoire/src/codes.rs"))
        .expect("codes.rs is readable");
    text.lines()
        .filter(|line| {
            line.contains("const ") && (line.contains("error(\"") || line.contains("warning(\""))
        })
        .filter_map(|line| line.split('"').nth(1))
        .map(str::to_owned)
        .collect()
}

#[test]
fn the_author_checklist_has_unique_items() {
    let text = fs::read_to_string(repo_root().join(".kiro/pipeline/checklists/author.md"))
        .expect("the author checklist exists at .kiro/pipeline/checklists/author.md");
    let ids: Vec<String> = text
        .lines()
        .filter(|line| line.trim_start().starts_with("- `AU-"))
        .filter_map(|line| line.split('`').nth(1))
        .map(str::to_owned)
        .collect();
    let unique: BTreeSet<&String> = ids.iter().collect();

    assert!(
        ids.len() >= 8,
        "expected at least eight author items, got {}",
        ids.len()
    );
    assert_eq!(
        unique.len(),
        ids.len(),
        "author checklist IDs must be unique"
    );
}

#[test]
fn every_grimoire_code_named_by_the_authoring_skill_exists() {
    let skill =
        fs::read_to_string(repo_root().join(".kiro/skills/grimoire-pack-authoring/SKILL.md"))
            .expect("the authoring skill exists");
    let defined = grimoire_codes();

    let unknown: Vec<String> = named_codes(&skill)
        .into_iter()
        .filter(|code| !defined.contains(code))
        .collect();

    assert!(
        unknown.is_empty(),
        "the skill names codes grimoire does not define: {unknown:?}"
    );
}
