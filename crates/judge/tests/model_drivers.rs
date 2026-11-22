//! Model checks for the shipped language drivers.
//!
//! These tests do not execute user code. They pin the stderr invariants that keep the
//! sandbox bounded: successful calls may return a bounded copy, while every driver still
//! mirrors bytes to process stderr so deadlines and abrupt exits keep their diagnostics.

use std::fs;
use std::path::PathBuf;

fn driver(name: &str) -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../languages/drivers")
        .join(name);
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()))
}

// test-change: CP-5.3 (stderr may be retained only inside a bounded driver buffer)
#[test]
fn no_driver_spends_the_sandbox_tmpfs_on_stderr() {
    for name in [
        "python/driver.py",
        "javascript/driver.js",
        "java/Driver.java",
        "cpp/driver.cpp",
    ] {
        let source = driver(name);
        for forbidden in ["tempfile", "TemporaryFile", "tmpfile"] {
            assert!(
                !source.contains(forbidden),
                "{name} stores stderr in a sandbox temp file ({forbidden})"
            );
        }
    }
}

// test-change: CP-5.3 (deadline diagnostics need live process stderr, while sample
// detail uses the bounded copy embedded in the answer line)
#[test]
fn every_driver_marks_call_boundaries_and_mirrors_stderr_live() {
    let python = driver("python/driver.py");
    assert!(python.contains("CALL_STDERR_MARKER"));
    assert!(python.contains("os.pipe()"));
    assert!(python.contains("threading.Thread"));
    assert!(python.contains("os.write(self._saved_fd"));

    let javascript = driver("javascript/driver.js");
    assert!(javascript.contains("CALL_STDERR_MARKER"));
    assert!(javascript.contains("let activeCapture = null"));
    assert!(javascript.contains("process.stderr.write ="));
    assert!(javascript.contains("fs.writeSync ="));
    assert!(javascript.contains("fs.write ="));
    assert!(!javascript.contains("error.capturedStderr"));

    let java = driver("java/Driver.java");
    assert!(java.contains("CALL_STDERR_MARKER"));
    assert!(java.contains("BoundedErr"));
    assert!(java.contains("activeErr"));
    assert!(java.contains("REAL_ERR.write"));

    let cpp = driver("cpp/driver.cpp");
    assert!(cpp.contains("CALL_STDERR_MARKER"));
    assert!(cpp.contains("CapturedErr"));
    assert!(cpp.contains("std::thread"));
    assert!(cpp.contains("saved_, buffer"));

    let catalogue = fs::read_to_string(
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../languages/languages.toml"),
    )
    .expect("cannot read languages/languages.toml");
    assert!(catalogue.contains("\"-pthread\""));
}
