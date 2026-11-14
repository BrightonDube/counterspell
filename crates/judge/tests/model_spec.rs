//! Model tests for session and exec specifications and the tar builder (requirements
//! R2.3, R2.4, R3.13, R3.14). Property P4 is at the end. No Docker is involved.

use std::io::Read;
use std::time::Duration;

use counterspell_judge::languages::FileName;
use counterspell_judge::sandbox::config::OciRuntime;
use counterspell_judge::sandbox::spec::{
    build_tar, driver_file_name, ExecSpec, SandboxFiles, SessionSpec, SpecError,
};
use proptest::prelude::*;
use uuid::Uuid;

mod support;
use support::shipped_catalogue;

const RUN: &str = "00000000-0000-0000-0000-000000000001";

fn run_id() -> Uuid {
    Uuid::parse_str(RUN).unwrap()
}

fn files(source: Vec<u8>) -> SandboxFiles {
    SandboxFiles::new(
        FileName::parse("solution.py").unwrap(),
        FileName::parse("driver.py").unwrap(),
        source,
        b"print('driver')\n".to_vec(),
        b"{}".to_vec(),
    )
    .unwrap()
}

/// Reads every entry of a tar archive back as (name, bytes).
fn read_entries(archive: &[u8]) -> Vec<(String, Vec<u8>)> {
    let mut reader = tar::Archive::new(archive);
    reader
        .entries()
        .unwrap()
        .map(|entry| {
            let mut entry = entry.unwrap();
            let name = entry.path().unwrap().to_string_lossy().into_owned();
            let mut bytes = Vec::new();
            entry.read_to_end(&mut bytes).unwrap();
            (name, bytes)
        })
        .collect()
}

#[test]
fn the_archive_has_exactly_the_three_fixed_entries_in_order() {
    let archive = build_tar(&files(b"print(1)\n".to_vec())).unwrap();
    let names: Vec<String> = read_entries(&archive).into_iter().map(|(n, _)| n).collect();
    assert_eq!(names, ["solution.py", "driver.py", "signature.json"]);
}

#[test]
fn each_entry_holds_exactly_the_bytes_it_was_given() {
    let archive = build_tar(&files(b"print(1)\n".to_vec())).unwrap();
    let entries = read_entries(&archive);
    assert_eq!(entries[0].1, b"print(1)\n");
    assert_eq!(entries[1].1, b"print('driver')\n");
    assert_eq!(entries[2].1, b"{}");
}

#[test]
fn a_solution_named_like_another_entry_is_refused() {
    let error = SandboxFiles::new(
        FileName::parse("signature.json").unwrap(),
        FileName::parse("driver.py").unwrap(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
    )
    .unwrap_err();
    assert_eq!(error, SpecError::DuplicateName);
}

#[test]
fn a_name_with_a_path_separator_cannot_become_a_file_name() {
    // The only way to name an entry is `FileName`, and that type refuses separators.
    assert!(FileName::parse("../escape.py").is_none());
    assert!(FileName::parse("drivers/driver.py").is_none());
}

#[test]
fn the_driver_is_named_by_the_last_part_of_its_catalogue_path() {
    let (_dir, catalog) = shipped_catalogue(false);
    let python = catalog.get("python").unwrap();
    assert_eq!(driver_file_name(python).unwrap().as_str(), "driver.py");
}

#[test]
fn the_session_is_named_from_the_run_and_its_ordinal() {
    let (_dir, catalog) = shipped_catalogue(false);
    let python = catalog.get("python").unwrap();
    let session = SessionSpec::new(
        python,
        Duration::from_secs(20),
        OciRuntime::Runc,
        run_id(),
        "worker-1",
        3,
        1_800_000_000,
    )
    .unwrap();
    assert_eq!(session.name, format!("cs-sbx-{RUN}-3"));
}

#[test]
fn the_session_carries_the_run_worker_and_expiry_labels() {
    let (_dir, catalog) = shipped_catalogue(false);
    let python = catalog.get("python").unwrap();
    let session = SessionSpec::new(
        python,
        Duration::from_secs(20),
        OciRuntime::Runc,
        run_id(),
        "worker-1",
        3,
        1_800_000_000,
    )
    .unwrap();
    let labels = &session.policy.labels;
    assert_eq!(
        labels.get("counterspell.managed").map(String::as_str),
        Some("true")
    );
    assert_eq!(
        labels.get("counterspell.run").map(String::as_str),
        Some(RUN)
    );
    assert_eq!(
        labels.get("counterspell.worker").map(String::as_str),
        Some("worker-1")
    );
    assert_eq!(
        labels.get("counterspell.expires").map(String::as_str),
        Some("1800000000")
    );
}

#[test]
fn a_worker_id_with_separators_or_blanks_is_refused() {
    let (_dir, catalog) = shipped_catalogue(false);
    let python = catalog.get("python").unwrap();
    for bad in ["", "../w", "w 1", "w/1"] {
        let error = SessionSpec::new(
            python,
            Duration::from_secs(20),
            OciRuntime::Runc,
            run_id(),
            bad,
            1,
            1,
        )
        .unwrap_err();
        assert_eq!(error, SpecError::BadWorkerId, "worker id {bad:?}");
    }
}

#[test]
fn the_compile_exec_is_the_catalogue_argv_run_as_the_sandbox_user() {
    let (_dir, catalog) = shipped_catalogue(false);
    let python = catalog.get("python").unwrap();
    let compile = ExecSpec::compile(python).unwrap();
    assert_eq!(compile.argv, python.compile().to_vec());
    assert_eq!(compile.user, "65534:65534");
    assert_eq!(compile.working_dir, "/box");
    assert_eq!(
        compile.env,
        vec![
            ("PATH".to_owned(), "/usr/local/bin:/usr/bin:/bin".to_owned()),
            ("LANG".to_owned(), "C.UTF-8".to_owned()),
            ("HOME".to_owned(), "/box".to_owned()),
        ]
    );
}

#[test]
fn the_driver_exec_is_the_catalogue_run_argv() {
    let (_dir, catalog) = shipped_catalogue(false);
    let python = catalog.get("python").unwrap();
    let driver = ExecSpec::driver(python);
    assert_eq!(driver.argv, ["python3", "-I", "-S", "driver.py"]);
}

#[test]
fn the_bench_exec_appends_the_bench_arguments_and_keeps_the_run_argv() {
    let (_dir, catalog) = shipped_catalogue(false);
    let python = catalog.get("python").unwrap();
    let bench = ExecSpec::bench(python);
    assert_eq!(
        bench.argv,
        ["python3", "-I", "-S", "driver.py", "--mode", "bench"]
    );
    assert_eq!(bench.user, "65534:65534");
    assert_eq!(bench.working_dir, "/box");
    assert_eq!(
        ExecSpec::driver(python).argv,
        ["python3", "-I", "-S", "driver.py"],
        "the normal driver command must not gain the bench arguments"
    );
}

proptest! {
    /// P4: sources and inputs differ, but the session, the entry names and the archive
    /// shape stay the same. Only the content bytes change.
    #[test]
    fn different_sources_change_only_the_content(
        first in prop::collection::vec(any::<u8>(), 0..512),
        second in prop::collection::vec(any::<u8>(), 0..512),
    ) {
        let (_dir, catalog) = shipped_catalogue(false);
        let python = catalog.get("python").unwrap();
        let session_a = SessionSpec::new(python, Duration::from_secs(20), OciRuntime::Runc, run_id(), "w", 1, 1).unwrap();
        let session_b = SessionSpec::new(python, Duration::from_secs(20), OciRuntime::Runc, run_id(), "w", 1, 1).unwrap();
        prop_assert_eq!(session_a, session_b);

        let archive_a = build_tar(&files(first.clone())).unwrap();
        let archive_b = build_tar(&files(second.clone())).unwrap();
        let entries_a = read_entries(&archive_a);
        let entries_b = read_entries(&archive_b);
        let names_a: Vec<&str> = entries_a.iter().map(|(n, _)| n.as_str()).collect();
        let names_b: Vec<&str> = entries_b.iter().map(|(n, _)| n.as_str()).collect();
        prop_assert_eq!(names_a, names_b);
        prop_assert_eq!(&entries_a[0].1, &first);
        prop_assert_eq!(&entries_b[0].1, &second);
        prop_assert_eq!(&entries_a[1].1, &entries_b[1].1);
    }
}
