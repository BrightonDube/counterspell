//! `counterspell-judge healthcheck` report (deploy-ops T07, R10.2, R4.4).
//!
//! The binary's `healthcheck` arm only prints `health::report` and maps it to an exit code. These
//! tests call the same function in-process. The judge's clippy policy forbids spawning processes
//! in judge code, so the tests do not run the binary. The report itself never touches Postgres.

use std::{
    path::PathBuf,
    time::{Duration, SystemTime},
};

use counterspell_judge::health::{report, MAX_AGE};
use tempfile::TempDir;

/// Writes a heartbeat file in a fresh temporary directory and returns its path with the directory.
fn heartbeat_file() -> (TempDir, PathBuf, SystemTime) {
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("judge.heartbeat");
    std::fs::write(&path, "beat\n").expect("write heartbeat");
    let written_at = std::fs::metadata(&path)
        .and_then(|metadata| metadata.modified())
        .expect("heartbeat mtime");
    (dir, path, written_at)
}

#[test]
fn fresh_heartbeat_reports_healthy() {
    let (_dir, path, written_at) = heartbeat_file();

    let line =
        report(&path, written_at + Duration::from_secs(5)).expect("a 5 s old heartbeat is healthy");

    assert!(line.starts_with("judge healthy"), "{line}");
}

#[test]
fn heartbeat_older_than_sixty_seconds_reports_stale() {
    let (_dir, path, written_at) = heartbeat_file();

    let line = report(&path, written_at + MAX_AGE + Duration::from_secs(60))
        .expect_err("a 120 s old heartbeat is unhealthy");

    assert_eq!(line, "judge unhealthy: heartbeat stale");
}

#[test]
fn missing_heartbeat_reports_missing_without_the_path() {
    let dir = TempDir::new().expect("temp dir");
    let path = dir.path().join("absent.heartbeat");

    let line = report(&path, SystemTime::now()).expect_err("no heartbeat is unhealthy");

    assert_eq!(line, "judge unhealthy: heartbeat missing");
    assert!(!line.contains(&path.display().to_string()));
}
