//! The code-running MCP tools on the real rootless sandbox daemon
//! (counterspell-mcp-power T10, MCP-1).
//!
//! The tools go through `production_slot`, which runs the judge's daemon identity check and builds
//! the judge's own sandbox runner. These tests prove four things: the code really runs in a
//! container and not on the host, a hostile program that never ends is a time-out result and not an
//! error, a real incorrect program yields a minimized counterexample, and no container is left
//! behind. Ignored by default.
//!
//! `SANDBOX_DOCKER_SOCKET=/run/user/1000/docker.sock cargo test -p counterspell-mcp --test sbx_tools -- --ignored --test-threads=1`

use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

use counterspell_judge::sandbox::backend::Backend;
use counterspell_judge::sandbox::bollard_backend::BollardBackend;
use counterspell_judge::sandbox::daemon::connect;
use counterspell_mcp::config::SANDBOX_SOCKET_VAR;
use counterspell_mcp::judge_adapter::python_image_digest;
use counterspell_mcp::runner::{production_slot, RunnerSlot};
use counterspell_mcp::schemas::{
    FindGhostArgs, RunReferenceArgs, ShrinkInputArgs, ShrinkTargetArg,
};
use counterspell_mcp::tools::{find_ghost_input, run_reference, shrink_input};
use serde_json::{json, Value};

const META: &str = "slug = \"job-demo\"\ntitle = \"Count the Pairs\"\nstatus = \"draft\"\ndifficulty = \"easy\"\ntrack = \"arrays\"\nconcepts = [\"arrays\", \"pairs\"]\norigin = \"original\"\nauthor = \"Brighton\"\nlicence = \"MIT\"\npack_version = 1\nfunction = \"count_pairs\"\nreturns = \"int\"\ncomparator = \"exact\"\n\n[[params]]\nname = \"values\"\ntype = \"list[int]\"\n\n[[params]]\nname = \"target\"\ntype = \"int\"\n\n[limits]\ntime_ms = 2000\nmem_mb = 256\n\n[input_schema]\ntype = \"tuple\"\nitems = [\n  { type = \"int_array\", min_len = 0, max_len = 30, min = -50, max = 50 },\n  { type = \"int\", min = -100, max = 100 },\n]\n\n[generator]\nseed = 1729\nhidden_count = 12\n\n[[samples]]\ninput = [[1, 2, 3, 2], 4]\nexpected = 2\n\n[[samples]]\ninput = [[5, 5], 10]\nexpected = 1\n";
const HINTS: &str = "[[hint]]\nrung = 1\ntext = \"One.\"\n\n[[hint]]\nrung = 2\ntext = \"Two.\"\n\n[[hint]]\nrung = 3\ntext = \"Three.\"\n\n[[hint]]\nrung = 4\ntext = \"Four.\"\n";
const PROBLEM: &str = "# Count the Pairs\n\n## Task\nCount.\n\n## Input\nA.\n\n## Output\nB.\n\n## Examples\nC.\n\n## Constraints\nD.\n";
const GHOSTS: &str = "[[ghost]]\nid = \"same-index\"\nprogram = \"incorrect/same-index.py\"\nkind = \"wrong\"\ninput = [[0], 0]\ngot = 1\nconcepts = [\"pairs\"]\nexplanation = \"A value cannot pair with itself, so the single item must not be counted.\"\n";
/// The correct reference. It reports whether it runs inside a container, so a test can tell the
/// sandbox from the host.
const REFERENCE: &str = "import os\n\ndef count_pairs(values, target):\n    if target == 99:\n        return 1 if os.path.exists('/.dockerenv') else 0\n    if target == 98:\n        while True:\n            pass\n    total = 0\n    for i in range(len(values)):\n        for j in range(i + 1, len(values)):\n            if values[i] + values[j] == target:\n                total += 1\n    return total\n";
/// Counts a value paired with itself: the authored same-index bug.
const SAME_INDEX: &str = "def count_pairs(values, target):\n    total = 0\n    for i in range(len(values)):\n        for j in range(i, len(values)):\n            if values[i] + values[j] == target:\n                total += 1\n    return total\n";

/// A throwaway workspace: the repository's `languages/` plus one public pack named `job-demo`.
struct Workspace(PathBuf);

fn copy_dir(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).expect("create directory");
    for entry in std::fs::read_dir(from).expect("read directory") {
        let entry = entry.expect("entry");
        let target = to.join(entry.file_name());
        if entry.file_type().expect("type").is_dir() {
            copy_dir(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).expect("copy file");
        }
    }
}

impl Workspace {
    fn new() -> Self {
        let repo = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let root =
            std::env::temp_dir().join(format!("counterspell-mcp-sbx-{}", std::process::id()));
        let pack = root.join("packs/job-demo");
        std::fs::create_dir_all(pack.join("reference")).expect("reference dir");
        std::fs::create_dir_all(pack.join("incorrect")).expect("incorrect dir");
        copy_dir(&repo.join("languages"), &root.join("languages"));
        std::fs::write(root.join("Cargo.toml"), "[workspace]\n").expect("marker");
        std::fs::write(pack.join("problem.md"), PROBLEM).expect("problem");
        std::fs::write(pack.join("meta.toml"), META).expect("meta");
        std::fs::write(pack.join("hints.toml"), HINTS).expect("hints");
        std::fs::write(pack.join("known_ghosts.toml"), GHOSTS).expect("ghosts");
        std::fs::write(pack.join("reference/solution.py"), REFERENCE).expect("reference");
        std::fs::write(pack.join("incorrect/same-index.py"), SAME_INDEX).expect("program");
        Self(root)
    }
}

impl Drop for Workspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// The production slot, built exactly as `main` builds it.
async fn slot(root: &Path) -> RunnerSlot {
    let socket = std::env::var(SANDBOX_SOCKET_VAR).ok();
    production_slot(root, socket.as_deref())
        .await
        .expect("the rootless sandbox daemon passes the identity check")
}

/// The ids of every sandbox container the daemon holds right now.
async fn container_ids() -> std::collections::HashSet<String> {
    let socket = std::env::var(SANDBOX_SOCKET_VAR).expect("SANDBOX_DOCKER_SOCKET is set");
    let timeout = Duration::from_secs(10);
    let docker = connect(Path::new(&socket), timeout).expect("connect");
    BollardBackend::new(docker, timeout)
        .list_managed()
        .await
        .expect("list containers")
        .into_iter()
        .map(|container| container.id)
        .collect()
}

/// Waits (briefly) for the daemon to finish removing every container created since `before`.
/// Containers that were already there are not this test's business.
async fn nothing_new_left(before: &std::collections::HashSet<String>) -> bool {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if container_ids().await.difference(before).count() == 0 {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(Duration::from_millis(400)).await;
    }
}

fn run(root: &Path, slot: &mut RunnerSlot, target: i64) -> Value {
    run_reference(
        root,
        slot,
        &RunReferenceArgs {
            pack_path: "packs/job-demo".to_owned(),
            input: json!([[1, 2, 3, 2], target]),
        },
    )
    .expect("the tool returns a result")
}

/// The reference runs in a container (`/.dockerenv` exists), not on the host (it does not), and
/// a correct answer comes back as a returned value.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "needs the rootless sandbox daemon (SANDBOX_DOCKER_SOCKET)"]
async fn sbx_run_reference_runs_inside_the_sandbox_and_nowhere_else() {
    assert!(
        !Path::new("/.dockerenv").exists(),
        "this host looks like a container, so the marker proves nothing"
    );
    let workspace = Workspace::new();
    let root = workspace.0.clone();
    let before = container_ids().await;
    let mut slot = slot(&root).await;

    let in_sandbox = tokio::task::spawn_blocking({
        let root = root.clone();
        move || run(&root, &mut slot, 99)
    })
    .await
    .expect("the tool call joins");

    assert_eq!(in_sandbox["outcome"], json!("returned"), "{in_sandbox}");
    assert_eq!(
        in_sandbox["value"],
        json!(1),
        "the marker file exists only in the sandbox"
    );
    assert!(
        nothing_new_left(&before).await,
        "a container was left behind"
    );
}

/// A program that never ends is killed from outside and reported as a time-out result, within
/// the limit plus slack. It is not an error and not a wrong answer.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "needs the rootless sandbox daemon (SANDBOX_DOCKER_SOCKET)"]
async fn sbx_a_program_that_never_ends_is_a_timeout_result() {
    let workspace = Workspace::new();
    let root = workspace.0.clone();
    let before = container_ids().await;
    let mut slot = slot(&root).await;

    let started = Instant::now();
    let result = tokio::task::spawn_blocking({
        let root = root.clone();
        move || run(&root, &mut slot, 98)
    })
    .await
    .expect("the tool call joins");

    assert_eq!(result["outcome"], json!("timed_out"), "{result}");
    assert!(
        started.elapsed() < Duration::from_secs(12),
        "the kill came too late: {:?}",
        started.elapsed()
    );
    assert!(
        nothing_new_left(&before).await,
        "a container was left behind"
    );
}

/// A sandbox that cannot start (here: the pinned image is not on the daemon, and containers never
/// pull at run time) is an infrastructure error. It is never a program result, so it can never be
/// mistaken for a wrong answer.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "needs the rootless sandbox daemon (SANDBOX_DOCKER_SOCKET)"]
async fn sbx_a_sandbox_that_cannot_start_is_infrastructure_not_a_wrong_answer() {
    let workspace = Workspace::new();
    let root = workspace.0.clone();
    let languages = root.join("languages/languages.toml");
    let text = std::fs::read_to_string(&languages).expect("catalogue");
    let missing = format!("sha256:{}", "0".repeat(64));
    let text = text.replacen(
        "sha256:e48f59cdba69a3cbd9ed2cf0f348d90d45a8d350d98134cf1070bc106ee9ed90",
        &missing,
        1,
    );
    assert!(text.contains(&missing), "the python pin was replaced");
    std::fs::write(&languages, text).expect("write catalogue");
    let before = container_ids().await;
    let mut slot = slot(&root).await;

    let error = tokio::task::spawn_blocking({
        let root = root.clone();
        move || {
            run_reference(
                &root,
                &mut slot,
                &RunReferenceArgs {
                    pack_path: "packs/job-demo".to_owned(),
                    input: json!([[1, 2, 3, 2], 4]),
                },
            )
        }
    })
    .await
    .expect("the tool call joins")
    .expect_err("no container can start from a missing image");

    assert_eq!(
        error.category,
        counterspell_mcp::errors::ErrorCategory::SandboxInfrastructure,
        "{error:?}"
    );
    assert!(
        nothing_new_left(&before).await,
        "a container was left behind"
    );
}

/// A real incorrect program produces a minimized counterexample through the sandbox, and a shrink
/// of a larger failing input keeps the failure.
#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "needs the rootless sandbox daemon (SANDBOX_DOCKER_SOCKET)"]
async fn sbx_find_and_shrink_use_the_sandbox_and_leave_nothing_behind() {
    let workspace = Workspace::new();
    let root = workspace.0.clone();
    let digest = python_image_digest(&root.join("languages/languages.toml"), &root)
        .expect("the pinned image digest");
    let before = container_ids().await;
    let mut slot = slot(&root).await;

    let (found, shrunk) = tokio::task::spawn_blocking({
        let root = root.clone();
        move || {
            let found = find_ghost_input(
                &root,
                &mut slot,
                &digest,
                &FindGhostArgs {
                    pack_path: "packs/job-demo".to_owned(),
                    program: "incorrect/same-index.py".to_owned(),
                    seed_override: Some(1),
                    budget: None,
                },
            )
            .expect("a hunt result");
            let shrunk = shrink_input(
                &root,
                &mut slot,
                &ShrinkInputArgs {
                    pack_path: "packs/job-demo".to_owned(),
                    program: "incorrect/same-index.py".to_owned(),
                    input: json!([[2, 2, 5, 1, 4], 4]),
                    target: ShrinkTargetArg {
                        kind: "wrong".to_owned(),
                    },
                    max_evals: 100,
                    wall_ms: 20_000,
                },
            )
            .expect("a shrink result");
            (found, shrunk)
        }
    })
    .await
    .expect("the tool calls join");

    // The program counts a value paired with itself, so the real sandbox run disagrees with the
    // reference. The authored known ghost `[[0], 0]` is found first: got 1, expected 0.
    let hit = &found["outcome"]["Found"];
    assert!(hit.is_object(), "the hunt found a counterexample: {found}");
    assert_eq!(hit["input"], json!([[0], 0]), "{found}");
    assert_eq!(hit["expected"], json!(0), "{found}");
    assert_eq!(hit["got"], json!({"Returned": 1}), "{found}");
    let value = shrunk["value"].clone();
    assert!(value.is_array(), "the shrink returned an input: {shrunk}");
    assert!(
        value.to_string().len() < json!([[2, 2, 5, 1, 4], 4]).to_string().len(),
        "the shrink made the input smaller: {value}"
    );
    assert!(
        nothing_new_left(&before).await,
        "a container was left behind"
    );
}
