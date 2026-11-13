//! Layout and file-safety checks (LAY-01 to LAY-05) on real temporary packs.
//!
//! Each case builds a small pack in the system temp directory and asserts the
//! exact list of codes, so a case proves its own rule and nothing else.

use counterspell_grimoire::pack;
use counterspell_grimoire::report::Report;
use std::fs;
use std::path::{Path, PathBuf};

/// A temporary pack directory that is removed when the test ends.
struct TempPack(PathBuf);

impl TempPack {
    fn new(label: &str) -> Self {
        let path =
            std::env::temp_dir().join(format!("grimoire-layout-{label}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&path);
        fs::create_dir_all(&path).expect("create temp pack directory");
        Self(path)
    }

    fn write(&self, relative: &str, bytes: &[u8]) {
        let path = self.0.join(relative);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).expect("create parent directory");
        }
        fs::write(path, bytes).expect("write fixture file");
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempPack {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

/// Writes the four required files with small, valid-looking content.
fn required_files(pack: &TempPack) {
    pack.write("problem.md", b"# Count\n");
    pack.write("meta.toml", b"slug = \"count\"\n");
    pack.write("hints.toml", b"# hints\n");
    pack.write("reference/solution.py", b"def count(x):\n    return 0\n");
}

/// Loads the pack and returns the codes it reported, in order.
fn codes_for(pack: &TempPack) -> Vec<String> {
    let mut report = Report::new("fixture");
    let _ = pack::load(pack.path(), &mut report).expect("temp root is readable");
    report
        .diagnostics()
        .iter()
        .map(|diagnostic| diagnostic.code.to_owned())
        .collect()
}

#[test]
fn clean_pack_reports_no_layout_codes() {
    let pack = TempPack::new("clean");
    required_files(&pack);

    assert_eq!(codes_for(&pack), Vec::<String>::new());
}

#[test]
fn loaded_pack_exposes_the_text_of_each_file() {
    let pack = TempPack::new("text");
    required_files(&pack);
    let mut report = Report::new("fixture");

    let loaded = pack::load(pack.path(), &mut report).expect("temp root is readable");

    assert_eq!(loaded.text("meta.toml"), Some("slug = \"count\"\n"));
    assert_eq!(
        loaded.text("reference/solution.py"),
        Some("def count(x):\n    return 0\n")
    );
}

#[test]
fn missing_required_file_is_lay_01() {
    let pack = TempPack::new("missing");
    required_files(&pack);
    fs::remove_file(pack.path().join("hints.toml")).expect("remove required file");

    assert_eq!(codes_for(&pack), ["LAY-01"]);
}

#[test]
fn dot_prefixed_file_is_lay_02() {
    let pack = TempPack::new("dot");
    required_files(&pack);
    pack.write(".hidden", b"secret\n");

    assert_eq!(codes_for(&pack), ["LAY-02"]);
}

#[test]
fn unknown_top_level_file_is_lay_02() {
    let pack = TempPack::new("notes");
    required_files(&pack);
    pack.write("notes.txt", b"scratch\n");

    assert_eq!(codes_for(&pack), ["LAY-02"]);
}

#[test]
fn compiled_python_file_is_lay_02() {
    let pack = TempPack::new("pyc");
    required_files(&pack);
    pack.write("reference/solution.pyc", b"\x00\x01compiled");

    assert_eq!(codes_for(&pack), ["LAY-02"]);
}

#[test]
fn incorrect_program_without_known_ghosts_is_lay_02() {
    let pack = TempPack::new("ghost-less");
    required_files(&pack);
    pack.write("incorrect/off-by-one.py", b"def count(x):\n    return 1\n");

    assert_eq!(codes_for(&pack), ["LAY-02"]);
}

#[test]
fn incorrect_program_with_known_ghosts_is_allowed() {
    let pack = TempPack::new("ghosts");
    required_files(&pack);
    pack.write("known_ghosts.toml", b"# ghosts\n");
    pack.write("incorrect/off-by-one.py", b"def count(x):\n    return 1\n");

    assert_eq!(codes_for(&pack), Vec::<String>::new());
}

#[test]
fn nested_directory_under_reference_is_lay_02() {
    let pack = TempPack::new("nested");
    required_files(&pack);
    pack.write("reference/extra/helper.py", b"x = 1\n");

    assert_eq!(codes_for(&pack), ["LAY-02"]);
}

#[test]
fn oversize_problem_statement_is_lay_04() {
    let pack = TempPack::new("big-problem");
    required_files(&pack);
    pack.write("problem.md", &vec![b'a'; 33 * 1024]);

    assert_eq!(codes_for(&pack), ["LAY-04"]);
}

#[test]
fn oversize_hints_file_is_lay_04() {
    let pack = TempPack::new("big-hints");
    required_files(&pack);
    pack.write("hints.toml", &vec![b'a'; 9 * 1024]);

    assert_eq!(codes_for(&pack), ["LAY-04"]);
}

#[test]
fn pack_over_512_kb_in_total_is_lay_04() {
    let pack = TempPack::new("big-pack");
    required_files(&pack);
    pack.write("known_ghosts.toml", b"# ghosts\n");
    for index in 0..9 {
        pack.write(
            &format!("incorrect/program-{index}.py"),
            &vec![b'#'; 60_000],
        );
    }

    assert_eq!(codes_for(&pack), ["LAY-04"]);
}

#[test]
fn nul_byte_in_text_is_lay_05() {
    let pack = TempPack::new("nul");
    required_files(&pack);
    pack.write("meta.toml", b"slug = \"count\"\0\n");

    assert_eq!(codes_for(&pack), ["LAY-05"]);
}

#[test]
fn non_utf8_text_is_lay_05() {
    let pack = TempPack::new("utf8");
    required_files(&pack);
    pack.write("meta.toml", b"title = \"\xff\"\n");

    assert_eq!(codes_for(&pack), ["LAY-05"]);
}

#[cfg(unix)]
#[test]
fn symbolic_link_is_lay_03() {
    let pack = TempPack::new("symlink");
    required_files(&pack);
    let outside = std::env::temp_dir().join(format!("grimoire-outside-{}.md", std::process::id()));
    fs::write(&outside, b"# outside\n").expect("write the link target outside the pack");
    let problem = pack.path().join("problem.md");
    fs::remove_file(&problem).expect("remove the real problem.md");
    std::os::unix::fs::symlink(&outside, &problem).expect("create symbolic link");

    let codes = codes_for(&pack);
    let _ = fs::remove_file(&outside);

    assert_eq!(codes, ["LAY-03"]);
}

#[cfg(unix)]
#[test]
fn non_utf8_file_name_is_lay_03() {
    use std::ffi::OsStr;
    use std::os::unix::ffi::OsStrExt;

    let pack = TempPack::new("name");
    required_files(&pack);
    let name = OsStr::from_bytes(b"meta\xff.toml");
    fs::write(pack.path().join(name), b"slug = \"x\"\n").expect("write non-UTF-8 name");

    assert_eq!(codes_for(&pack), ["LAY-03"]);
}
