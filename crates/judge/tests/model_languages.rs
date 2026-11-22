//! Model tests for the language catalogue (requirements R2.1, R2.5, R12.1 to R12.3).
//! Property P13 is at the end. No Docker is involved.
//!
//! The fixture image is syntactically valid but is never pulled.

use std::fs;
use std::path::PathBuf;

use counterspell_judge::languages::{LanguageCatalog, LanguageError, PinnedImage};
use proptest::prelude::*;
use tempfile::TempDir;

const VALID_IMAGE: &str =
    "python:3.13-slim@sha256:0000000000000000000000000000000000000000000000000000000000000000";

/// One `[name]` table in the catalogue format of design.md section 6.
fn language_table(name: &str, enabled: bool, image: &str, driver: &str) -> String {
    format!(
        r#"
[{name}]
enabled = {enabled}
image = "{image}"
file = "solution.py"
driver = "{driver}"
compile = ["python3", "-I", "-S", "-m", "py_compile", "/box/solution.py"]
run = ["python3", "-I", "-S", "driver.py"]
bench_args = ["--mode", "bench"]
mem_mb = 256
time_ms_per_test = 2000
max_time_ms_per_test = 10000
compile_budget_ms = 5000
max_source_bytes = 65536
exec_scratch = false
protocol_version = 1
"#
    )
}

fn python_table() -> String {
    language_table("python", true, VALID_IMAGE, "drivers/python/driver.py")
}

/// Writes `languages.toml` with `text` and the driver file the fixture expects.
/// Returns the directory (kept alive by the caller) and the catalogue path.
fn fixture(text: &str) -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let driver_dir = dir.path().join("drivers").join("python");
    fs::create_dir_all(&driver_dir).unwrap();
    fs::write(driver_dir.join("driver.py"), "print('driver')\n").unwrap();
    let path = dir.path().join("languages.toml");
    fs::write(&path, text).unwrap();
    (dir, path)
}

fn load_text(text: &str) -> Result<LanguageCatalog, LanguageError> {
    let (_dir, path) = fixture(text);
    LanguageCatalog::load(&path)
}

fn strings(items: &[&str]) -> Vec<String> {
    items.iter().map(|item| (*item).to_owned()).collect()
}

#[test]
fn a_valid_python_entry_loads_with_every_field() {
    let catalog = load_text(&python_table()).unwrap();
    let python = catalog.get("python").unwrap();

    assert!(python.is_enabled());
    assert_eq!(python.image().as_str(), VALID_IMAGE);
    assert_eq!(python.file().as_str(), "solution.py");
    assert_eq!(python.driver().as_str(), "drivers/python/driver.py");
    assert_eq!(
        python.compile().to_vec(),
        strings(&[
            "python3",
            "-I",
            "-S",
            "-m",
            "py_compile",
            "/box/solution.py"
        ])
    );
    assert_eq!(
        python.run().to_vec(),
        strings(&["python3", "-I", "-S", "driver.py"])
    );
    assert_eq!(python.mem_mb(), 256);
    assert_eq!(python.time_ms_per_test(), 2000);
    assert_eq!(python.max_time_ms_per_test(), 10_000);
    assert_eq!(python.compile_budget_ms(), 5000);
    assert!(!python.exec_scratch());
    assert_eq!(catalog.max_source_bytes("python"), Some(65_536));
    assert_eq!(catalog.max_source_bytes("cobol"), None);
}

#[test]
fn the_shipped_catalogue_enables_all_four_languages() {
    // test-change: IDE-1 (language-drivers T16 enabled Java and C++ after their
    // stretch suites passed on the rootless daemon)
    // The real file in the repository. Its drivers are supplied by language-drivers,
    // so this test provides stand-in driver files in a temporary directory.
    let text = include_str!("../../../languages/languages.toml");
    let (dir, _path) = fixture(text);
    for (lang, file) in [
        ("javascript", "driver.js"),
        ("java", "Driver.java"),
        ("cpp", "driver.cpp"),
    ] {
        let lang_dir = dir.path().join("drivers").join(lang);
        fs::create_dir_all(&lang_dir).unwrap();
        fs::write(lang_dir.join(file), "// stand-in\n").unwrap();
    }
    let catalog = LanguageCatalog::parse(text, dir.path()).unwrap();

    for name in ["python", "javascript", "java", "cpp"] {
        assert!(catalog.get(name).unwrap().is_enabled(), "{name} is enabled");
    }
}

#[test]
fn a_disabled_language_is_parsed_and_marked_disabled() {
    let text = format!(
        "{}{}",
        python_table(),
        language_table(
            "javascript",
            false,
            VALID_IMAGE,
            "drivers/javascript/driver.js"
        )
    );
    let catalog = load_text(&text).unwrap();

    let javascript = catalog.get("javascript").unwrap();
    assert!(!javascript.is_enabled());
    assert_eq!(
        catalog.names().collect::<Vec<_>>(),
        ["javascript", "python"]
    );
}

#[test]
fn a_tag_without_a_digest_is_refused() {
    let text = language_table(
        "python",
        true,
        "python:3.13-slim",
        "drivers/python/driver.py",
    );
    assert_eq!(
        load_text(&text).unwrap_err(),
        LanguageError::ImageNotPinned {
            language: "python".to_owned()
        }
    );
}

#[test]
fn a_digest_with_one_hex_digit_too_few_is_refused() {
    let short = format!("python:3.13-slim@sha256:{}", "a".repeat(63));
    let text = language_table("python", true, &short, "drivers/python/driver.py");
    assert!(matches!(
        load_text(&text),
        Err(LanguageError::ImageNotPinned { .. })
    ));
}

#[test]
fn an_uppercase_digest_is_refused() {
    let upper = format!("python:3.13-slim@sha256:{}", "A".repeat(64));
    let text = language_table("python", true, &upper, "drivers/python/driver.py");
    assert!(matches!(
        load_text(&text),
        Err(LanguageError::ImageNotPinned { .. })
    ));
}

#[test]
fn pinned_image_only_builds_from_a_name_and_a_sha256_digest() {
    assert!(PinnedImage::parse(VALID_IMAGE).is_some());
    assert!(PinnedImage::parse("python:3.13-slim").is_none());
    assert!(PinnedImage::parse("@sha256:0000").is_none());
}

#[test]
fn a_repeated_language_table_is_refused() {
    let text = format!("{}{}", python_table(), python_table());
    assert!(matches!(load_text(&text), Err(LanguageError::Syntax(_))));
}

#[test]
fn an_unknown_field_is_refused() {
    let text = python_table().replace("bench_args", "colour = \"blue\"\nbench_args");
    assert!(matches!(load_text(&text), Err(LanguageError::Syntax(_))));
}

#[test]
fn a_zero_memory_limit_is_refused() {
    let text = python_table().replace("mem_mb = 256", "mem_mb = 0");
    assert_eq!(
        load_text(&text).unwrap_err(),
        LanguageError::ZeroLimit {
            language: "python".to_owned(),
            field: "mem_mb"
        }
    );
}

#[test]
fn a_per_test_limit_above_the_maximum_is_refused() {
    let text = python_table().replace("time_ms_per_test = 2000", "time_ms_per_test = 20000");
    assert_eq!(
        load_text(&text).unwrap_err(),
        LanguageError::TimeAboveMaximum {
            language: "python".to_owned()
        }
    );
}

#[test]
fn a_path_in_the_file_name_is_refused() {
    for bad in ["../solution.py", "sub/solution.py", "..", "."] {
        let text = python_table().replace("file = \"solution.py\"", &format!("file = \"{bad}\""));
        assert_eq!(
            load_text(&text).unwrap_err(),
            LanguageError::BadFileName {
                language: "python".to_owned(),
                field: "file"
            },
            "file name {bad:?} should be refused"
        );
    }
}

#[test]
fn a_missing_driver_file_is_refused_for_an_enabled_language() {
    let text = language_table("python", true, VALID_IMAGE, "drivers/python/missing.py");
    assert_eq!(
        load_text(&text).unwrap_err(),
        LanguageError::MissingDriver {
            language: "python".to_owned()
        }
    );
}

#[test]
fn an_unsupported_protocol_version_is_refused() {
    let text = python_table().replace("protocol_version = 1", "protocol_version = 2");
    assert_eq!(
        load_text(&text).unwrap_err(),
        LanguageError::UnsupportedProtocol {
            language: "python".to_owned(),
            found: 2
        }
    );
}

#[test]
fn an_empty_run_command_is_refused() {
    let text = python_table().replace(
        "run = [\"python3\", \"-I\", \"-S\", \"driver.py\"]",
        "run = []",
    );
    assert_eq!(
        load_text(&text).unwrap_err(),
        LanguageError::EmptyRun {
            language: "python".to_owned()
        }
    );
}

#[test]
fn a_language_name_outside_the_allowed_pattern_is_refused() {
    let text = python_table().replace("[python]", "[Python]");
    assert_eq!(
        load_text(&text).unwrap_err(),
        LanguageError::BadName("Python".to_owned())
    );
}

/// The rule the catalogue applies to a file name, written out independently.
fn file_name_is_valid(name: &str) -> bool {
    let allowed = |c: char| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-');
    (1..=64).contains(&name.len()) && name.chars().all(allowed) && name != "." && name != ".."
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(64))]

    /// P13: a file name is accepted exactly when it follows the rule.
    #[test]
    fn file_names_are_accepted_exactly_when_they_follow_the_rule(
        name in "[a-zA-Z0-9_./ -]{0,70}"
    ) {
        let text = python_table().replace("file = \"solution.py\"", &format!("file = \"{name}\""));
        let accepted = load_text(&text).is_ok();
        prop_assert_eq!(accepted, file_name_is_valid(&name));
    }

    /// P13: a valid limit set round-trips into the loaded definition unchanged.
    #[test]
    fn valid_limits_round_trip(mem in 1u64..=4096, time in 1u64..=9000, extra in 0u64..=1000) {
        let max = time + extra;
        let text = python_table()
            .replace("mem_mb = 256", &format!("mem_mb = {mem}"))
            .replace("time_ms_per_test = 2000", &format!("time_ms_per_test = {time}"))
            .replace("max_time_ms_per_test = 10000", &format!("max_time_ms_per_test = {max}"));
        let catalog = load_text(&text).unwrap();
        let python = catalog.get("python").unwrap();
        prop_assert_eq!(python.mem_mb(), mem);
        prop_assert_eq!(python.time_ms_per_test(), time);
        prop_assert_eq!(python.max_time_ms_per_test(), max);
    }
}
