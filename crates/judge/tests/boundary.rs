// SPDX-License-Identifier: MIT
//! Boundary tests (judge-sandbox T36, R1.4–R1.8, R2.6, R15.7, R7): static checks
//! that the sandbox stays the only way code is reached — the server never sees
//! Docker, Bollard stays behind two files, nothing spawns a host process, the
//! container-facing modules never see expected outputs, the compose file never
//! widens the judge, and the MCP crate touches only the sandbox-facing judge API.
//!
//! All checks are textual/graph scans of checked-in files, so they hold in CI
//! without a daemon.

use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

/// The repository root (this crate is `<root>/crates/judge`).
fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("crates/judge has a repo root")
        .to_owned()
}

/// Every `.rs` file under `dir`, recursively, with repo-relative paths.
fn rust_files(dir: &Path) -> Vec<(PathBuf, String)> {
    let mut files = Vec::new();
    let mut stack = vec![dir.to_owned()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("a readable source directory") {
            let entry = entry.expect("a readable entry");
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.extension().is_some_and(|e| e == "rs") {
                let text = std::fs::read_to_string(&path).expect("a readable source file");
                files.push((path, text));
            }
        }
    }
    files
}

fn relative<'a>(root: &Path, path: &'a Path) -> &'a Path {
    path.strip_prefix(root).unwrap_or(path)
}

/// 1. `counterspell-server`'s production dependency closure must not reach
///    Bollard or the judge. `Cargo.lock` merges dep kinds, so the graph comes
///    from `cargo metadata`; `kind: "dev"` edges are test-only and excluded —
///    the server's tests deliberately use the judge test kit (see the server's
///    Cargo.toml comment), which never ships in the artifact.
#[test]
#[allow(clippy::disallowed_methods)] // a test may shell out to `cargo metadata`; production code may not
fn the_server_cannot_reach_docker_or_the_judge() {
    let output = std::process::Command::new("cargo")
        .args(["metadata", "--format-version", "1"])
        .current_dir(root())
        .output()
        .expect("cargo metadata runs");
    assert!(
        output.status.success(),
        "cargo metadata failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let metadata: serde_json::Value =
        serde_json::from_slice(&output.stdout).expect("metadata is JSON");

    // package id -> package name, and id -> non-dev dependency ids.
    let mut names: BTreeMap<String, String> = BTreeMap::new();
    for package in metadata["packages"].as_array().expect("packages") {
        names.insert(
            package["id"].as_str().expect("id").to_owned(),
            package["name"].as_str().expect("name").to_owned(),
        );
    }
    let mut graph: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for node in metadata["resolve"]["nodes"]
        .as_array()
        .expect("resolve nodes")
    {
        let id = node["id"].as_str().expect("node id").to_owned();
        let deps = node["deps"]
            .as_array()
            .expect("node deps")
            .iter()
            .filter(|dep| {
                dep["dep_kinds"]
                    .as_array()
                    .is_some_and(|kinds| kinds.iter().any(|k| k["kind"].as_str() != Some("dev")))
            })
            .map(|dep| dep["pkg"].as_str().expect("dep pkg").to_owned())
            .collect();
        graph.insert(id, deps);
    }

    let server_id = names
        .iter()
        .find(|(_, name)| name.as_str() == "counterspell-server")
        .map(|(id, _)| id.clone())
        .expect("counterspell-server in the metadata");
    let mut seen = BTreeSet::new();
    let mut stack = vec![server_id];
    while let Some(id) = stack.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        if let Some(deps) = graph.get(&id) {
            stack.extend(deps.iter().cloned());
        }
    }
    let reached: BTreeSet<&str> = seen
        .iter()
        .filter_map(|id| names.get(id).map(String::as_str))
        .collect();

    for forbidden in ["bollard", "bollard-stubs", "counterspell-judge"] {
        assert!(
            !reached.contains(forbidden),
            "counterspell-server's production closure reaches {forbidden}"
        );
    }
}

/// 2. `bollard` may be named only by the backend and the daemon guard.
#[test]
fn bollard_stays_behind_two_files() {
    let allowed = [
        PathBuf::from("crates/judge/src/sandbox/bollard_backend.rs"),
        PathBuf::from("crates/judge/src/sandbox/daemon.rs"),
    ];
    let mut offenders = Vec::new();
    for krate in [
        "catalogue-cli",
        "common",
        "engine",
        "grimoire",
        "judge",
        "mcp",
        "server",
    ] {
        for (path, text) in rust_files(&root().join("crates").join(krate).join("src")) {
            if allowed
                .iter()
                .any(|p| p.as_path() == relative(&root(), &path))
            {
                continue;
            }
            // An import shows up as `use bollard…` or a `bollard::` path; the
            // `bollard_backend` module path and doc-comment mentions do not.
            for (line_no, line) in text.lines().enumerate() {
                let tail = line.trim_start();
                let imported = tail.contains("bollard::")
                    || tail.starts_with("use bollard ")
                    || tail.starts_with("use bollard;");
                if imported {
                    offenders.push(format!(
                        "{}:{}: {}",
                        relative(&root(), &path).display(),
                        line_no + 1,
                        tail
                    ));
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "bollard imported outside its seam: {offenders:?}"
    );
}

/// 3. Nothing in the judge or MCP production code spawns a host process. The only
///    permitted `std::process` use is `ExitCode` (a type, not a spawn).
#[test]
fn no_host_process_spawning() {
    let mut offenders = Vec::new();
    for dir in ["crates/judge/src", "crates/mcp/src"] {
        for (path, text) in rust_files(&root().join(dir)) {
            for (line_no, line) in text.lines().enumerate() {
                for rest in line.match_indices("process::") {
                    let tail = &line[rest.0 + "process::".len()..];
                    if !tail.starts_with("ExitCode") {
                        offenders.push(format!(
                            "{}:{}: {}",
                            relative(&root(), &path).display(),
                            line_no + 1,
                            line.trim()
                        ));
                    }
                }
                if line.contains("tokio::process") || line.contains("Command::new") {
                    offenders.push(format!(
                        "{}:{}: {}",
                        relative(&root(), &path).display(),
                        line_no + 1,
                        line.trim()
                    ));
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "host process spawning inside the trust boundary:\n{}",
        offenders.join("\n")
    );
}

/// 4. The container-facing modules never see expected outputs: `TestCase` and the
///    `problem` module stay on the judge side of the session.
#[test]
fn container_facing_modules_see_no_expected_outputs() {
    let facing = ["call.rs", "session.rs", "spec.rs"];
    for name in facing {
        let path = root().join("crates/judge/src/sandbox").join(name);
        let text = std::fs::read_to_string(&path).expect("a container-facing module");
        for (line_no, line) in text.lines().enumerate() {
            let line = line.trim();
            if line.starts_with("use ") || line.contains("::problem") || line.contains("TestCase") {
                assert!(
                    !line.contains("problem") && !line.contains("TestCase"),
                    "{name}:{line_no} imports expected-output types: {line}"
                );
            }
        }
    }
}

/// 5. The compose file must not give the server Docker, must not give the judge a
///    published port, a rootful socket, or the Valkey URL.
#[test]
fn compose_keeps_the_boundaries() {
    let compose = std::fs::read_to_string(root().join("docker-compose.yml"))
        .expect("docker-compose.yml exists");
    let doc: serde_norway::Value = serde_norway::from_str(&compose).expect("compose parses");
    let services = &doc["services"];

    let volumes = |service: &str| -> Vec<String> {
        services[service]["volumes"]
            .as_sequence()
            .map(|v| {
                v.iter()
                    .filter_map(|entry| {
                        entry
                            .as_str()
                            .map(str::to_owned)
                            .or_else(|| entry["source"].as_str().map(str::to_owned))
                    })
                    .collect()
            })
            .unwrap_or_default()
    };

    let docker_mounts = |service: &str| -> Vec<String> {
        volumes(service)
            .into_iter()
            .filter(|v| v.contains("docker.sock"))
            .collect()
    };

    assert!(
        docker_mounts("server").is_empty(),
        "server must never mount a Docker socket: {:?}",
        docker_mounts("server")
    );

    for mount in docker_mounts("judge") {
        assert!(
            !mount.contains("/var/run/docker.sock") && !mount.contains("/run/docker.sock"),
            "judge may only mount the rootless sandbox socket: {mount}"
        );
    }

    assert!(
        services["judge"]["ports"].is_null(),
        "judge publishes no ports: {:?}",
        services["judge"]["ports"]
    );

    let env = services["judge"]["environment"]
        .as_mapping()
        .map(|m| {
            m.iter()
                .map(|(k, v)| {
                    format!(
                        "{}={}",
                        k.as_str().unwrap_or_default(),
                        v.as_str().unwrap_or_default()
                    )
                })
                .collect::<Vec<_>>()
                .join("\n")
        })
        .unwrap_or_default();
    assert!(
        !env.contains("VALKEY_URL"),
        "judge has no Valkey URL:\n{env}"
    );
}

/// 6. `counterspell-mcp` touches only the sandbox-facing judge API (sandbox types
///    and the language catalogue it needs to build sessions) — never the queue,
///    executor, comparator or verdict internals.
#[test]
fn mcp_uses_only_the_sandbox_facing_judge_api() {
    const ALLOWED_PREFIXES: [&str; 2] = ["sandbox::", "languages::"];
    let mut offenders = Vec::new();
    for (path, text) in rust_files(&root().join("crates/mcp/src")) {
        for (line_no, line) in text.lines().enumerate() {
            for rest in line.match_indices("counterspell_judge::") {
                let tail = &line[rest.0 + "counterspell_judge::".len()..];
                if !ALLOWED_PREFIXES.iter().any(|p| tail.starts_with(p)) {
                    offenders.push(format!(
                        "{}:{}: {}",
                        relative(&root(), &path).display(),
                        line_no + 1,
                        line.trim()
                    ));
                }
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "mcp reaches past the sandbox-facing judge API:\n{}",
        offenders.join("\n")
    );
}

/// The package names listed under `[dependencies]` in a manifest. Dev and build tables are
/// ignored, so a test-only edge is not a production edge.
fn production_dependencies(manifest: &str) -> Vec<String> {
    let mut in_dependencies = false;
    let mut names = Vec::new();
    for line in manifest.lines() {
        let line = line.trim();
        if line.starts_with('[') {
            in_dependencies = line == "[dependencies]";
        } else if in_dependencies {
            if let Some((name, _)) = line.split_once('=') {
                names.push(name.trim().to_owned());
            }
        }
    }
    names
}

/// 7. The crate graph keeps the sandbox runner shareable: the judge never depends on Grimoire or
///    the operator crate, and `counterspell-mcp` never depends on the operator crate (it reuses
///    the judge's runner instead). Brighton's 15 Nov and 19 Nov decisions.
#[test]
fn the_judge_and_mcp_stay_off_the_authoring_crates() {
    let read =
        |path: &str| std::fs::read_to_string(root().join(path)).expect("a readable crate manifest");
    let judge = production_dependencies(&read("crates/judge/Cargo.toml"));
    let mcp = production_dependencies(&read("crates/mcp/Cargo.toml"));
    for banned in ["counterspell-grimoire", "counterspell-catalogue-cli"] {
        assert!(
            !judge.iter().any(|name| name == banned),
            "the judge depends on {banned}: {judge:?}"
        );
    }
    assert!(
        !mcp.iter().any(|name| name == "counterspell-catalogue-cli"),
        "mcp depends on the operator crate: {mcp:?}"
    );
    assert!(
        mcp.iter().any(|name| name == "counterspell-judge"),
        "mcp reuses the judge's runner: {mcp:?}"
    );
}

/// Breaks if the manifest reader reads dev-only edges as production edges, which would let a
/// real violation hide or a test-only crate fail the rule above.
#[test]
fn the_manifest_reader_sees_only_production_dependencies() {
    let manifest = "[package]\nname = \"x\"\n\n[dependencies]\na = { path = \"../a\" }\nb = \"1\"\n\n[dev-dependencies]\nc = \"2\"\n\n[features]\nd = []\n";
    assert_eq!(production_dependencies(manifest), vec!["a", "b"]);
}
