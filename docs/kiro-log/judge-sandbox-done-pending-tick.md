# judge-sandbox: done, pending tick

Each entry: task id, commit(s), tests that prove it, the exact verification commands and their result. A task is done only after Kiro re-verifies and ticks it.

## T20 bollard backend: create, read-back, image check (SUPERVISED)

- Commits: `d74705c` (backend part 1 + model tests), `149a08f` (sbx test), `a16d2ad` (read-back tolerates image-declared env).
- Proving tests: `tests/model_bollard.rs` (10 tests incl. 26-row mutation table), `tests/sbx_bollard_create.rs::sbx_create_readback_matches` on the rootless daemon.
- Verification run 16 Nov 2026:
  - `cargo test -p counterspell-judge --features sandbox --test sbx_bollard_create -- --ignored --nocapture` with `SANDBOX_DOCKER_SOCKET` from `~/.config/counterspell/dev-test.env`: **PASS** (1 passed, 0.22 s). Identity check passed (cpuset/io delegated), shipped catalogue loaded now that `languages/drivers/python/driver.py` and `languages/drivers/javascript/driver.js` exist, create + read-back matched the policy, container removed.
  - Workspace gate: `cargo fmt --all -- --check` ok; `cargo clippy --workspace --all-targets -- -D warnings` ok; `cargo test --workspace` 1369 passed, 0 failed.

## T22 probe, preflight state and the `preflight` command (SUPERVISED)

- Commit: this commit (`judge-sandbox T22: probe, preflight state and preflight command`).
- Proving tests: `tests/model_preflight.rs` (15 tests incl. probe fold, `PreflightState`, refresh loop, cleanup-on-failure), `tests/sbx_preflight.rs` (2 `sbx_` tests).
- Verification run 16 Nov 2026:
  - `SANDBOX_DOCKER_SOCKET=/run/user/1000/docker.sock cargo test -p counterspell-judge --features sandbox --test sbx_preflight -- --ignored`: **PASS** (2 passed, 0.70 s). `sbx_preflight_passes` probed the production policy; `sbx_preflight_fails_when_policy_weakened` created a container with a writable root via the testkit-only `create_started_unchecked` and the report failed `root_read_only` and `read_only_write_refused`.
  - `SANDBOX_DOCKER_SOCKET=/run/user/1000/docker.sock cargo run -p counterspell-judge -- preflight`: prints the JSON report; 12 checks × {javascript, python} all `ok`, exit 0.
  - `cargo fmt --all -- --check` ok; `cargo clippy -p counterspell-judge --all-targets -- -D warnings` ok; `cargo test -p counterspell-judge` all 64 binaries green.
- Recorded deviation: the write probe uses `/var/tmp/cs-probe` (mode 1777) instead of the design's `/usr/cs-probe` (mode 755) — for the `nobody` exec user, `touch /usr` exits non-zero on a writable root too (EACCES), so the design path could never discriminate. Queued for Brighton in the review-queue entry.

## T23 sweeper: list, reap, repeat until stopped (SUPERVISED)

- Commit: this commit (`judge-sandbox T23: sweeper + Backend::list_managed`).
- Proving tests: `tests/model_reaper.rs` (6 tests: old removed, young kept, unlabelled untouched, removal error counted and loop continues, list error returned, `run_sweeper` runs until stopped), `tests/sbx_sweeper.rs` (`sbx_sweeper_removes_orphan_and_ignores_unlabelled`).
- Verification run 16 Nov 2026:
  - `cargo test -p counterspell-judge --test model_reaper`: **PASS** (6 passed).
  - `SANDBOX_DOCKER_SOCKET=/run/user/1000/docker.sock cargo test -p counterspell-judge --features sandbox --test sbx_sweeper -- --ignored`: **PASS** (1 passed, 1.98 s). The managed orphan was force-removed; the same-aged unlabelled container survived.
  - `cargo fmt --all -- --check` ok; `cargo clippy --workspace --all-targets -- -D warnings` ok; `cargo test --workspace` all binaries green.
- Recorded deviation: `sweep_once(backend, orphan_age, now)` injects `now` instead of taking `cfg` — model tests script container ages without a full `SandboxConfig`; `run_sweeper` supplies `now_unix()`. See the review-queue entry.

## T24 ProblemTests, SandboxExecutor and judge wiring (SUPERVISED)

- Commit: this commit (`judge-sandbox T24: ProblemTests, SandboxExecutor, judge wiring`).
- Proving tests: `tests/model_sandbox_executor.rs` (10 tests) + `tests/fixtures/problems/two-sum.json`.
- Verification run 16 Nov 2026:
  - `cargo test -p counterspell-judge --test model_sandbox_executor`: **PASS** (10 passed).
  - `cargo fmt --all -- --check` ok; `cargo clippy --workspace --all-targets -- -D warnings` ok; `cargo test --workspace` all binaries green.
- Recorded deviations queued in the review entry: `stop_watch()` added to `RunContext`; `Vec<Arc<PreflightState>>` instead of a single state; per-test events emitted after `execute_run` returns rather than streamed; `expires` label = `now + orphan_age`.

## T25 sandbox test harness, fixtures and corpus layout (SUPERVISED)

- Commit: this commit (`judge-sandbox T25: sbx harness, fixtures, corpus layout`).
- Proving tests: `tests/model_corpus.rs::hostile_files_are_data` (non-ignored, passes); harness verified end-to-end by a temporary smoke test on the rootless daemon (`Accepted`, 5/5 on `add`; temp file removed after the run).
- Verification run 16 Nov 2026:
  - `cargo test -p counterspell-judge --test model_corpus`: **PASS** (1 passed).
  - `SANDBOX_DOCKER_SOCKET=/run/user/1000/docker.sock` smoke run of `run_source`: **PASS** — `add_correct.py` accepted, events `[compiling, test×5]`.
  - `cargo fmt --all -- --check` ok; `cargo clippy --workspace --all-targets -- -D warnings` ok.
- Files: `tests/sbx/mod.rs` (harness), `tests/fixtures/problems/{add,sort_list}.json`, `tests/fixtures/solutions/{add_correct,add_wrong,sort_correct,sort_wrong}.py`, `tests/hostile/README.md`. Scenario files in T26+ will live at `tests/sbx_*.rs` top level with `mod sbx;`, matching the existing `sbx_support` convention.
