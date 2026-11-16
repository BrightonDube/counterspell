//! Tests for `catalogue::load` — the pack-file reader and the import
//! assemblers (problem-catalogue C19). Synthetic packs live in a tempdir;
//! nothing here is sandbox evidence.

use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use counterspell_grimoire::meta::Meta;
use counterspell_grimoire::record::{Difficulty, ProblemRecord, Signature};
use counterspell_judge::catalogue::load::{
    assemble_import, assemble_sandbox, read_pack, AssembleError,
};
use serde_json::json;

const META: &str = r#"
slug = "two-sum"
title = "Two Sum"
status = "published"
difficulty = "easy"
track = "arrays"
concepts = ["arrays"]
origin = "original"
author = "test"
licence = "MIT"
pack_version = 3
function = "two_sum"
returns = "list[int]"
comparator = "unordered"

[[params]]
name = "nums"
type = "list[int]"

[[params]]
name = "target"
type = "int"

[limits]
time_ms = 2000
mem_mb = 256

[input_schema]
type = "tuple"
items = [
  { type = "int_array", min_len = 0, max_len = 10, min = -10, max = 10 },
  { type = "int", min = -10, max = 10 },
]

[generator]
seed = 1729
hidden_count = 6

[[samples]]
input = [[2, 7], 9]
expected = [0, 1]

[[samples]]
input = [[3, 3], 6]
expected = [0, 1]
"#;

const PROBLEM: &str = "# Two Sum\n\n\
## Task\nFind the pair.\n\n\
## Input\nA list and a target.\n\n\
## Output\nThe pair indices.\n\n\
## Examples\nSee samples.\n\n\
## Constraints\nAt most 10 values.\n";

const HINTS: &str = r#"
[[hint]]
rung = 1
text = "one"

[[hint]]
rung = 2
text = "two"

[[hint]]
rung = 3
text = "three"

[[hint]]
rung = 4
text = "four"
"#;

const GHOSTS: &str = r#"
[[ghost]]
id = "adds-pair"
program = "incorrect/adds-pair.py"
kind = "wrong"
input = [[1], 1]
got = [0, 0]
concepts = ["arrays"]
explanation = "It returns the wrong pair."
"#;

const HIDDEN: &str = r#"
[[case]]
input = [[1, 1], 2]
expected = [0, 1]
why = "smallest pair"
"#;

const GENERATOR: &str = "seed = 42\nhidden_count = 4\n";

const APPROVAL: &str = r#"
slug = "two-sum"
pack_version = 3
content_sha256 = "0000000000000000000000000000000000000000000000000000000000000000"
public_commit = "0000000000000000000000000000000000000000"
approved_by = "t"
approved_at = "t"
"#;

/// Writes a synthetic pack into `(public, private)` dirs under `root`.
fn write_pack(root: &Path) -> (PathBuf, PathBuf) {
    let public = root.join("two-sum");
    let private = root.join("private").join("two-sum");
    fs::create_dir_all(public.join("reference")).unwrap();
    fs::create_dir_all(public.join("incorrect")).unwrap();
    fs::create_dir_all(&private).unwrap();
    fs::write(public.join("meta.toml"), META).unwrap();
    fs::write(public.join("problem.md"), PROBLEM).unwrap();
    fs::write(public.join("hints.toml"), HINTS).unwrap();
    fs::write(public.join("known_ghosts.toml"), GHOSTS).unwrap();
    fs::write(
        public.join("reference/solution.py"),
        "def two_sum(nums, target):\n    return [0, 1]\n",
    )
    .unwrap();
    fs::write(
        public.join("incorrect/adds-pair.py"),
        "def two_sum(nums, target):\n    return [0, 0]\n",
    )
    .unwrap();
    fs::write(private.join("hidden.toml"), HIDDEN).unwrap();
    fs::write(private.join("generator.toml"), GENERATOR).unwrap();
    fs::write(private.join("approval.toml"), APPROVAL).unwrap();
    fs::write(private.join(".gitkeep"), "").unwrap();
    (public, private)
}

fn record_and_meta(dir: &Path) -> (ProblemRecord, Meta) {
    let record = counterspell_grimoire::record::build(dir).expect("record");
    let meta: Meta = toml::from_str(META).expect("meta");
    (record, meta)
}

/// A record built from `meta` for tests that bypass `record::build`.
fn synthetic_record(meta: &Meta) -> ProblemRecord {
    ProblemRecord {
        slug: meta.slug.clone(),
        title: meta.title.clone(),
        track: meta.track.clone(),
        difficulty: Difficulty::parse(&meta.difficulty).unwrap(),
        statement_md: PROBLEM.to_string(),
        signature: Signature {
            function: meta.function.clone(),
            params: meta.params.clone(),
            returns: meta.returns.clone(),
        },
        input_schema: meta.input_schema.clone(),
        comparator: meta.engine_comparator(),
        limits: meta.limits,
        reference_complexity: meta.reference_complexity.clone(),
        bench: meta.bench.clone(),
        complexity_note: meta.complexity_note.clone(),
        version: u32::try_from(meta.pack_version).unwrap(),
        concepts: meta.concepts.clone(),
    }
}

#[test]
fn read_pack_collects_both_trees() {
    let root = tempfile::tempdir().unwrap();
    let (public, private) = write_pack(root.path());
    // A public dotfile is skipped by the reader (grimoire flags it separately).
    fs::write(public.join(".hidden"), "").unwrap();
    let files = read_pack(&public, Some(&private)).unwrap();
    assert!(files.public.contains_key("meta.toml"));
    assert!(files.public.contains_key("reference/solution.py"));
    assert!(files.public.contains_key("incorrect/adds-pair.py"));
    assert!(files.private.contains_key("hidden.toml"));
    assert!(!files.public.contains_key(".hidden"), "dotfiles skipped");
    assert!(!files.private.contains_key(".gitkeep"), "dotfiles skipped");
}

#[test]
fn manifest_prefixes_trees_and_excludes_approval() {
    let root = tempfile::tempdir().unwrap();
    let (public, private) = write_pack(root.path());
    let files = read_pack(&public, Some(&private)).unwrap();
    let manifest = files.manifest().unwrap();
    let paths: Vec<&str> = manifest
        .entries()
        .iter()
        .map(|entry| entry.path.as_str())
        .collect();
    assert!(paths.contains(&"public/meta.toml"));
    assert!(paths.contains(&"private/hidden.toml"));
    assert!(!paths.iter().any(|p| p.contains("approval.toml")));
    assert!(manifest.public_map().contains_key("public/meta.toml"));
    assert!(!manifest.public_map().contains_key("private/hidden.toml"));
}

#[test]
fn approval_text_prefers_private() {
    let root = tempfile::tempdir().unwrap();
    let (public, private) = write_pack(root.path());
    fs::write(public.join("approval.toml"), "slug = \"other\"\n").unwrap();
    let files = read_pack(&public, Some(&private)).unwrap();
    assert!(
        files.approval_text().unwrap().contains("two-sum"),
        "private copy wins"
    );
}

#[test]
fn private_sources_prefix_and_hash() {
    let root = tempfile::tempdir().unwrap();
    let (public, private) = write_pack(root.path());
    let files = read_pack(&public, Some(&private)).unwrap();
    let sources = files.private_sources();
    let paths: Vec<&str> = sources.iter().map(|(path, _)| path.as_str()).collect();
    assert!(paths.contains(&"private/hidden.toml"));
    assert!(paths.contains(&"private/generator.toml"));
    assert!(!paths.iter().any(|p| p.contains("approval")));
    assert!(sources.iter().all(|(_, hash)| hash.len() == 32));
}

#[test]
fn assemble_import_maps_everything() {
    let root = tempfile::tempdir().unwrap();
    let (public, private) = write_pack(root.path());
    let files = read_pack(&public, Some(&private)).unwrap();
    let manifest = files.manifest().unwrap();
    let (record, meta) = record_and_meta(&public);
    let import = assemble_import(&record, &meta, &files, &manifest).unwrap();
    assert_eq!(import.slug, "two-sum");
    assert_eq!(import.version, 3);
    assert_eq!(import.title, "Two Sum");
    assert_eq!(import.difficulty, "easy");
    assert_eq!(import.track, "arrays");
    assert_eq!(import.comparator, "unordered");
    assert_eq!(import.hidden_count, 6);
    assert_eq!(import.examples.len(), 2);
    assert_eq!(import.hints.len(), 4);
    assert_eq!(import.hints[0], (1, "one".to_string()));
    assert_eq!(import.starters.len(), 2);
    assert!(import
        .starters
        .iter()
        .any(|(lang, code)| lang == "python" && code.contains("def two_sum")));
    assert!(import.starters.iter().any(|(lang, _)| lang == "javascript"));
    assert!(import.reference_code.contains("def two_sum"));
    assert_eq!(import.known_ghosts.len(), 1);
    assert_eq!(import.known_ghosts[0].ghost_id, "adds-pair");
    assert_eq!(import.known_ghosts[0].kind, "wrong");
    // The stored signature is the driver-protocol descriptor: the judge writes it
    // verbatim to `/box/signature.json` at run launch.
    assert_eq!(import.signature["protocol_version"], 1);
    assert_eq!(import.signature["function"], "two_sum");
    assert_eq!(import.signature["parameters"][0]["name"], "nums");
    assert_eq!(
        import.signature["parameters"][0]["type"],
        json!({"kind": "list", "items": {"kind": "int"}})
    );
    assert_eq!(
        import.signature["returns"],
        json!({"kind": "list", "items": {"kind": "int"}})
    );
    assert_eq!(import.limits["time_ms"], 2000);
    assert_eq!(*manifest.content_sha256(), import.content_sha256);
    assert!(!import.manifest_public.is_empty());
}

#[test]
fn assemble_import_refuses_missing_reference() {
    let root = tempfile::tempdir().unwrap();
    let (public, private) = write_pack(root.path());
    fs::remove_file(public.join("reference/solution.py")).unwrap();
    let files = read_pack(&public, Some(&private)).unwrap();
    let manifest = files.manifest().unwrap();
    let meta: Meta = toml::from_str(META).unwrap();
    // Static validation refuses the missing required file first.
    assert!(counterspell_grimoire::record::build(&public).is_err());
    // A private-only reference is found through the private tree.
    fs::create_dir_all(private.join("reference")).unwrap();
    fs::write(
        private.join("reference/solution.py"),
        "def two_sum(nums, target):\n    return [0, 1]\n",
    )
    .unwrap();
    let files = read_pack(&public, Some(&private)).unwrap();
    let record = synthetic_record(&meta);
    let import = assemble_import(&record, &meta, &files, &manifest).unwrap();
    assert!(import.reference_code.contains("def two_sum"));
}

#[test]
fn assemble_import_refuses_when_reference_absent_everywhere() {
    let root = tempfile::tempdir().unwrap();
    let (public, private) = write_pack(root.path());
    fs::remove_file(public.join("reference/solution.py")).unwrap();
    let files = read_pack(&public, Some(&private)).unwrap();
    let manifest = files.manifest().unwrap();
    let meta: Meta = toml::from_str(META).unwrap();
    let record = synthetic_record(&meta);
    let error = assemble_import(&record, &meta, &files, &manifest).unwrap_err();
    assert_eq!(error, AssembleError::ReferenceMissing);
    assert_eq!(error.code(), "reference_missing");
}

#[test]
fn assemble_import_refuses_bad_hints() {
    let root = tempfile::tempdir().unwrap();
    let (public, private) = write_pack(root.path());
    fs::write(
        private.join("hints.toml"),
        "[[hint]]\nrung = 1\ntext = \"only one\"\n",
    )
    .unwrap();
    let files = read_pack(&public, Some(&private)).unwrap();
    let manifest = files.manifest().unwrap();
    let (record, meta) = record_and_meta(&public);
    let error = assemble_import(&record, &meta, &files, &manifest).unwrap_err();
    assert_eq!(error, AssembleError::HintsInvalid);
    assert_eq!(error.code(), "hints_invalid");
}

#[test]
fn assemble_sandbox_parses_hidden_and_generator() {
    let root = tempfile::tempdir().unwrap();
    let (public, private) = write_pack(root.path());
    let files = read_pack(&public, Some(&private)).unwrap();
    let manifest = files.manifest().unwrap();
    let (record, meta) = record_and_meta(&public);
    let pack = assemble_sandbox(&record, &meta, &files, &manifest).unwrap();
    assert_eq!(pack.pack_version, 3);
    assert_eq!(pack.curated.len(), 1);
    assert_eq!(pack.curated[0].1, json!([0, 1]));
    assert_eq!(pack.seed, 42);
    assert_eq!(pack.hidden_count, 4);
    assert_eq!(pack.examples.len(), 2);
    assert_eq!(pack.ghosts.len(), 1);
    assert_eq!(pack.ghosts[0].program, "incorrect/adds-pair.py");
    assert_eq!(pack.returns, "list[int]");
    assert_eq!(pack.limit, Duration::from_millis(2000));
    assert!(pack
        .sources
        .iter()
        .any(|(path, _)| path == "private/hidden.toml"));
    assert_eq!(*manifest.content_sha256(), pack.content_sha256);
}

#[test]
fn assemble_sandbox_falls_back_to_meta_generator() {
    let root = tempfile::tempdir().unwrap();
    let (public, private) = write_pack(root.path());
    fs::remove_file(private.join("generator.toml")).unwrap();
    let files = read_pack(&public, Some(&private)).unwrap();
    let manifest = files.manifest().unwrap();
    let (record, meta) = record_and_meta(&public);
    let pack = assemble_sandbox(&record, &meta, &files, &manifest).unwrap();
    assert_eq!(pack.seed, 1729);
    assert_eq!(pack.hidden_count, 6);
}

#[test]
fn assemble_sandbox_refuses_bad_hidden_toml() {
    let root = tempfile::tempdir().unwrap();
    let (public, private) = write_pack(root.path());
    fs::write(private.join("hidden.toml"), "not toml [").unwrap();
    let files = read_pack(&public, Some(&private)).unwrap();
    let manifest = files.manifest().unwrap();
    let (record, meta) = record_and_meta(&public);
    let error = assemble_sandbox(&record, &meta, &files, &manifest)
        .err()
        .unwrap();
    assert_eq!(error, AssembleError::HiddenInvalid);
    assert_eq!(error.code(), "hidden_invalid");
}
