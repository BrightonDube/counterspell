//! One textual mutation per static diagnostic code (GRM-51).
//!
//! Each case copies the starter pack `packs/count-pairs` into a temporary
//! directory, applies one edit, and checks the exact list of error codes the
//! validator reports. The expected lists are written by hand. A companion code
//! appears only when the edit also breaks a later check, and the case says why.
//! Symlink cases need Unix, so this file is compiled only there.

#![cfg(unix)]

use counterspell_grimoire::codes;
use counterspell_grimoire::validate_static;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// The starter pack every mutation starts from.
const BASE_PACK: &str = "../../packs/count-pairs";

/// Codes with no textual trigger. GEN-02 guards the engine's own candidate stream, so
/// no pack text can reach it. The engine's property tests cover it instead.
const NOT_TEXT_REACHABLE: [&str; 1] = ["GEN-02"];

/// One change to the copied pack.
enum Edit {
    /// Replaces the first occurrence of `from` in `file` with `to`.
    Replace {
        file: &'static str,
        from: &'static str,
        to: String,
    },
    /// Deletes a file from the pack.
    Remove { file: &'static str },
    /// Writes a whole file, creating its folder if needed.
    Write { file: &'static str, text: String },
    /// Makes `path` a symbolic link to `target`, both relative to the pack.
    Symlink {
        path: &'static str,
        target: &'static str,
    },
}

/// One mutation, the code it aims at, and the exact error codes the validator must report.
struct Case {
    code: &'static str,
    name: &'static str,
    edits: Vec<Edit>,
    expected: Vec<&'static str>,
}

fn replace(file: &'static str, from: &'static str, to: &str) -> Edit {
    Edit::Replace {
        file,
        from,
        to: to.to_owned(),
    }
}

/// Every mutation, one or more per static code. Each expected list is hand-written.
#[allow(
    clippy::too_many_lines,
    reason = "one flat table of cases is easier to review than many small functions"
)]
fn cases() -> Vec<Case> {
    vec![
        Case {
            code: "LAY-01",
            name: "required problem.md is missing",
            edits: vec![Edit::Remove {
                file: "problem.md",
            }],
            expected: vec!["LAY-01"],
        },
        Case {
            code: "LAY-02",
            name: "a file outside the allow-list",
            edits: vec![Edit::Write {
                file: "notes.txt",
                text: "scratch".to_owned(),
            }],
            expected: vec!["LAY-02"],
        },
        Case {
            code: "LAY-03",
            name: "a symbolic link inside the pack",
            edits: vec![Edit::Symlink {
                path: "incorrect/link.py",
                target: "reference/solution.py",
            }],
            expected: vec!["LAY-03"],
        },
        Case {
            code: "LAY-04",
            name: "reference larger than its 64 KB cap",
            edits: vec![Edit::Write {
                file: "reference/solution.py",
                text: format!(
                    "def count_pairs(values, target):\n{}",
                    "# padding line\n".repeat(5000)
                ),
            }],
            // A rejected file counts as missing, so the reference check also fires (REF-01).
            expected: vec!["LAY-04", "REF-01"],
        },
        Case {
            code: "LAY-05",
            name: "a NUL byte in problem.md",
            edits: vec![replace(
                "problem.md",
                "The public examples",
                "The public\u{0} examples",
            )],
            // A rejected file is skipped, so the statement checks do not run.
            expected: vec!["LAY-05"],
        },
        Case {
            code: "LAY-06",
            name: "slug differs from the directory name",
            edits: vec![replace("meta.toml", "slug = \"count-pairs\"", "slug = \"count-pair\"")],
            expected: vec!["LAY-06"],
        },
        Case {
            code: "MET-01",
            name: "meta.toml has a TOML syntax error",
            edits: vec![replace("meta.toml", "title = \"Count the Pairs\"", "title = \"Count the Pairs")],
            expected: vec!["MET-01"],
        },
        Case {
            code: "MET-02",
            name: "a required field is missing",
            edits: vec![replace("meta.toml", "pack_version = 1\n", "")],
            expected: vec!["MET-02"],
        },
        Case {
            code: "MET-03",
            name: "licence is not MIT",
            edits: vec![replace("meta.toml", "licence = \"MIT\"", "licence = \"GPL\"")],
            expected: vec!["MET-03"],
        },
        Case {
            code: "MET-04",
            name: "parameter type outside the launch set",
            edits: vec![replace("meta.toml", "type = \"list[int]\"", "type = \"list[float]\"")],
            expected: vec!["MET-04"],
        },
        Case {
            code: "MET-05",
            name: "parameter kind does not match the schema",
            edits: vec![replace("meta.toml", "name = \"target\"\ntype = \"int\"", "name = \"target\"\ntype = \"str\"")],
            expected: vec!["MET-05"],
        },
        Case {
            code: "MET-06",
            name: "schema with minimum above maximum",
            edits: vec![replace(
                "meta.toml",
                "min = -50, max = 50 }",
                "min = 50, max = -50 }",
            )],
            // The broken range no longer holds the samples or the ghost inputs. Each of the
            // three samples reports its own SMP-02, and the first ghost reports KNG-02.
            expected: vec!["KNG-02", "MET-06", "SMP-02", "SMP-02", "SMP-02"],
        },
        Case {
            code: "MET-07",
            name: "unordered comparator on an int return",
            edits: vec![replace("meta.toml", "comparator = \"exact\"", "comparator = \"unordered\"")],
            expected: vec!["MET-07"],
        },
        Case {
            code: "MET-08",
            name: "time limit above the launch cap",
            edits: vec![replace("meta.toml", "time_ms = 2000", "time_ms = 5000")],
            expected: vec!["MET-08"],
        },
        Case {
            code: "MET-09",
            name: "reference_complexity without a bench table",
            edits: vec![replace(
                "meta.toml",
                "pack_version = 1\n",
                "pack_version = 1\nreference_complexity = \"n^2\"\n",
            )],
            expected: vec!["MET-09"],
        },
        Case {
            code: "STM-01",
            name: "statement title differs from meta.toml",
            edits: vec![replace("problem.md", "# Count the Pairs\n", "# Count Pairs\n")],
            expected: vec!["STM-01"],
        },
        Case {
            code: "STM-02",
            name: "raw HTML in the statement",
            edits: vec![replace(
                "problem.md",
                "The public examples are listed in the pack's samples.",
                "The public examples are listed in the pack's samples. <b>Note</b>",
            )],
            expected: vec!["STM-02"],
        },
        Case {
            code: "SMP-01",
            name: "only one sample",
            edits: vec![
                replace(
                    "meta.toml",
                    "[[samples]]\ninput = [[5, 5], 10]\nexpected = 1\n",
                    "",
                ),
                replace(
                    "meta.toml",
                    "[[samples]]\ninput = [[-3, 3, 0], 0]\nexpected = 1\n",
                    "",
                ),
            ],
            expected: vec!["SMP-01"],
        },
        Case {
            code: "SMP-02",
            name: "sample expected value is a float, not an int",
            edits: vec![replace(
                "meta.toml",
                "input = [[5, 5], 10]\nexpected = 1\n",
                "input = [[5, 5], 10]\nexpected = 1.0\n",
            )],
            expected: vec!["SMP-02"],
        },
        Case {
            code: "SMP-03",
            name: "two samples with the same input",
            edits: vec![replace("meta.toml", "input = [[-3, 3, 0], 0]", "input = [[5, 5], 10]")],
            expected: vec!["SMP-03"],
        },
        Case {
            code: "GEN-01",
            name: "hidden count below 5",
            edits: vec![replace("meta.toml", "hidden_count = 12", "hidden_count = 4")],
            expected: vec!["GEN-01"],
        },
        Case {
            code: "HNT-01",
            name: "only three hints",
            edits: vec![replace(
                "hints.toml",
                "\n[[hint]]\nrung = 4\ntext = \"Your inner loop may start at the same index as the outer loop. Change where it starts.\"\n",
                "",
            )],
            expected: vec!["HNT-01"],
        },
        Case {
            code: "HNT-02",
            name: "a hint longer than 600 characters",
            edits: vec![replace(
                "hints.toml",
                "Ask what a pair of positions needs before you count it.",
                &"a".repeat(601),
            )],
            expected: vec!["HNT-02"],
        },
        Case {
            code: "KNG-01",
            name: "two known ghosts with the same id",
            edits: vec![replace("known_ghosts.toml", "id = \"empty-min\"", "id = \"same-index\"")],
            expected: vec!["KNG-01"],
        },
        Case {
            code: "KNG-02",
            name: "known ghost input does not fit the schema",
            edits: vec![replace("known_ghosts.toml", "input = [[0], 0]", "input = [[0], 999]")],
            expected: vec!["KNG-02"],
        },
        Case {
            code: "KNG-03",
            name: "crash ghost that also has a got value",
            edits: vec![replace(
                "known_ghosts.toml",
                "exception_class = \"ValueError\"",
                "exception_class = \"ValueError\"\ngot = 1",
            )],
            expected: vec!["KNG-03"],
        },
        Case {
            code: "KNG-04",
            name: "explanation claims minimality",
            edits: vec![replace(
                "known_ghosts.toml",
                "The early exit calls min()",
                "The smallest early exit calls min()",
            )],
            expected: vec!["KNG-04"],
        },
        Case {
            code: "KNG-05",
            name: "ghost concept not in the pack",
            edits: vec![replace("known_ghosts.toml", "concepts = [\"arrays\"]", "concepts = [\"sorting\"]")],
            expected: vec!["KNG-05"],
        },
        Case {
            code: "KNG-06",
            name: "dogfood flag in a catalogue pack",
            edits: vec![replace(
                "known_ghosts.toml",
                "concepts = [\"arrays\"]",
                "concepts = [\"arrays\"]\ndogfood = true",
            )],
            expected: vec!["KNG-06"],
        },
        Case {
            code: "REF-01",
            name: "reference defines a different function",
            edits: vec![replace(
                "reference/solution.py",
                "def count_pairs(values, target):",
                "def count_pair(values, target):",
            )],
            expected: vec!["REF-01"],
        },
    ]
}

/// A copy of the starter pack in a temporary directory, removed when the case ends.
struct Workspace(PathBuf);

impl Workspace {
    /// Copies the starter pack under a temporary root, keeping the directory name the slug needs.
    fn copy_of_base(label: &str) -> Self {
        let root =
            std::env::temp_dir().join(format!("grimoire-mutation-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        let pack = root.join("count-pairs");
        copy_dir(
            &Path::new(env!("CARGO_MANIFEST_DIR")).join(BASE_PACK),
            &pack,
        );
        Self(pack)
    }

    fn pack(&self) -> &Path {
        &self.0
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        if let Some(root) = self.0.parent() {
            let _ = fs::remove_dir_all(root);
        }
    }
}

/// Copies a directory tree. Only the starter pack is copied, so no links are followed.
fn copy_dir(from: &Path, to: &Path) {
    fs::create_dir_all(to).expect("create copied directory");
    for entry in fs::read_dir(from).expect("read starter pack") {
        let entry = entry.expect("read directory entry");
        let target = to.join(entry.file_name());
        if entry.path().is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            fs::copy(entry.path(), target).expect("copy starter file");
        }
    }
}

/// Applies one edit to the copied pack.
fn apply(pack: &Path, edit: &Edit) {
    match edit {
        Edit::Replace { file, from, to } => {
            let path = pack.join(file);
            let text = fs::read_to_string(&path).expect("read file to edit");
            assert!(
                text.contains(from),
                "{file} does not contain the text to replace"
            );
            fs::write(path, text.replacen(from, to, 1)).expect("write edited file");
        }
        Edit::Remove { file } => fs::remove_file(pack.join(file)).expect("remove file"),
        Edit::Write { file, text } => {
            let path = pack.join(file);
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent).expect("create folder for file");
            }
            fs::write(path, text).expect("write file");
        }
        Edit::Symlink { path, target } => {
            let link = pack.join(path);
            if let Some(parent) = link.parent() {
                fs::create_dir_all(parent).expect("create folder for link");
            }
            std::os::unix::fs::symlink(pack.join(target), link).expect("create symlink");
        }
    }
}

/// Runs every edit of `case` on a fresh copy and returns the sorted error codes.
fn mutated_codes(index: usize, case: &Case) -> Vec<&'static str> {
    let workspace = Workspace::copy_of_base(&index.to_string());
    for edit in &case.edits {
        apply(workspace.pack(), edit);
    }
    let report = validate_static(workspace.pack()).expect("mutated pack is readable");
    let mut found: Vec<&'static str> = report
        .diagnostics()
        .iter()
        .filter(|diagnostic| diagnostic.severity == counterspell_grimoire::report::Severity::Error)
        .map(|diagnostic| diagnostic.code)
        .collect();
    found.sort_unstable();
    found
}

#[test]
fn each_mutation_reports_exactly_its_expected_codes() {
    let mut mismatches = Vec::new();
    for (index, case) in cases().iter().enumerate() {
        let mut expected = case.expected.clone();
        expected.sort_unstable();
        assert!(
            expected.contains(&case.code),
            "{}: the expected list must name the target code {}",
            case.name,
            case.code
        );

        let found = mutated_codes(index, case);
        if found != expected {
            mismatches.push(format!(
                "{} ({}): expected {:?}, found {:?}",
                case.name, case.code, expected, found
            ));
        }
    }
    assert!(mismatches.is_empty(), "{}", mismatches.join("\n"));
}

#[test]
fn every_static_code_has_a_mutation_except_the_listed_exemption() {
    let targeted: BTreeSet<&str> = cases().iter().map(|case| case.code).collect();
    let required: BTreeSet<&str> = codes::ALL
        .iter()
        .map(|code| code.id)
        .filter(|id| !id.starts_with("SBX-") && !NOT_TEXT_REACHABLE.contains(id))
        .collect();

    assert_eq!(targeted, required);
}
