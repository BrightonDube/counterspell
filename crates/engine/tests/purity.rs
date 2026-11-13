//! Purity guard: the engine must not run processes, touch the filesystem or
//! network, start threads, or use `unsafe`. This is a text scan with a simple
//! approximation (it skips `#[cfg(test)]` blocks and comment lines). It is a
//! backstop for code review, not a replacement for it.

use std::fs;
use std::path::{Path, PathBuf};

/// Tokens that production engine code must never contain.
const FORBIDDEN: [&str; 10] = [
    "std::process",
    "Command::new",
    "std::fs",
    "std::net",
    "tokio",
    "bollard",
    "std::thread",
    "unsafe",
    ".unwrap()",
    ".expect(",
];

/// Returns `(line number, token)` for every forbidden token in production code.
///
/// Lines inside a `#[cfg(test)]` item and comment lines are ignored.
fn scan_source(source: &str) -> Vec<(usize, &'static str)> {
    let mut findings = Vec::new();
    let mut pending_test_item = false;
    let mut skipping_depth: Option<i64> = None;

    for (index, line) in source.lines().enumerate() {
        let trimmed = line.trim_start();
        if let Some(depth) = skipping_depth.as_mut() {
            *depth += brace_balance(trimmed);
            if *depth <= 0 {
                skipping_depth = None;
            }
            continue;
        }
        if trimmed == "#[cfg(test)]" {
            pending_test_item = true;
            continue;
        }
        if pending_test_item {
            if trimmed.contains('{') {
                let depth = brace_balance(trimmed);
                pending_test_item = false;
                if depth > 0 {
                    skipping_depth = Some(depth);
                }
            } else if trimmed.ends_with(';') {
                pending_test_item = false;
            }
            continue;
        }
        if trimmed.starts_with("//") {
            continue;
        }
        for token in FORBIDDEN {
            if line.contains(token) {
                findings.push((index + 1, token));
            }
        }
    }
    findings
}

/// Counts `{` minus `}` on one line.
fn brace_balance(line: &str) -> i64 {
    let opened = line.matches('{').count();
    let closed = line.matches('}').count();
    i64::try_from(opened).unwrap_or(0) - i64::try_from(closed).unwrap_or(0)
}

#[test]
fn scanner_flags_a_forbidden_token_in_production_code() {
    let source = "fn run() {\n    let out = Command::new(\"ls\");\n}\n";
    assert_eq!(scan_source(source), vec![(2, "Command::new")]);
}

#[test]
fn scanner_ignores_forbidden_tokens_inside_test_modules_and_comments() {
    let source = "// unsafe is fine in a comment\n\
                  #[cfg(test)]\n\
                  mod tests {\n\
                  \x20   fn helper() { let value = std::fs::read(\"x\").unwrap(); }\n\
                  }\n\
                  fn production() -> u8 { 1 }\n";
    assert_eq!(scan_source(source), Vec::new());
}

#[test]
fn scanner_resumes_checking_after_a_test_module_closes() {
    let source = "#[cfg(test)]\nmod tests {\n    fn a() {}\n}\nfn after() { tokio::spawn(f); }\n";
    assert_eq!(scan_source(source), vec![(5, "tokio")]);
}

/// Returns the `.rs` files directly inside `dir`.
fn source_files(dir: &Path) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = fs::read_dir(dir)
        .expect("the engine has a src directory")
        .map(|entry| entry.expect("readable entry").path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "rs"))
        .collect();
    files.sort();
    files
}

/// Returns the dependency names listed under one manifest section.
fn dependency_names(manifest: &str, section: &str) -> Vec<String> {
    let header = format!("[{section}]");
    let mut names = Vec::new();
    let mut inside = false;
    for line in manifest.lines() {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            inside = trimmed == header;
            continue;
        }
        if inside && !trimmed.is_empty() && !trimmed.starts_with('#') {
            if let Some((name, _)) = trimmed.split_once('=') {
                names.push(name.trim().to_owned());
            }
        }
    }
    names.sort();
    names
}

#[test]
fn purity_production_sources_run_no_processes_and_use_no_unsafe_or_unwrap() {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut violations = Vec::new();
    for file in source_files(&src) {
        let source = fs::read_to_string(&file).expect("readable source file");
        for (line, token) in scan_source(&source) {
            violations.push(format!("{}:{line}: {token}", file.display()));
        }
    }
    assert!(
        violations.is_empty(),
        "forbidden tokens:\n{}",
        violations.join("\n")
    );
}

#[test]
fn purity_runtime_dependencies_are_serde_only() {
    let manifest = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("the engine has a manifest");
    assert_eq!(
        dependency_names(&manifest, "dependencies"),
        vec!["serde".to_owned(), "serde_json".to_owned()]
    );
}

#[test]
fn purity_dev_dependencies_are_proptest_only() {
    let manifest = fs::read_to_string(Path::new(env!("CARGO_MANIFEST_DIR")).join("Cargo.toml"))
        .expect("the engine has a manifest");
    assert_eq!(
        dependency_names(&manifest, "dev-dependencies"),
        vec!["proptest".to_owned()]
    );
}
