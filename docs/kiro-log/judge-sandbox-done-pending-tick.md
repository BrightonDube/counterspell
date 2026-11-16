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

## T26 — functional + forgery scenarios (done, pending Kiro tick)

- Commit: this commit (`judge-sandbox T26: functional and forgery scenarios`).
- Files: `tests/sbx_functional.rs` (S01–S04, S27, S28), `tests/sbx_forgery.rs` (S14, S15a–d), `tests/hostile/{fake_verdict,forged_lines,early_exit,self_kill,stdout_close,import_all,remember_replay}.py`, `tests/sbx/mod.rs` (+`cleared_within`).
- Verification run 16 Nov 2026 on `/run/user/1000/docker.sock`:
  - `cargo test -p counterspell-judge --features sandbox --test sbx_functional --test sbx_forgery -- --ignored`: **PASS** — 7/7 functional, 5/5 forgery.
  - `cargo fmt --all -- --check` ok; `cargo clippy --workspace --all-targets --all-features -- -D warnings` ok; `cargo test -p counterspell-judge --all-features` ok.
- Noted deviations: (1) `fake_verdict.py` proves the forged verdict twice — `print` is swallowed by the driver's `sys.stdout` sink, `os.write(1,…)` hits the protocol pipe and violates the one-answer contract; the verdict stays `WrongAnswer` because only `verdict::decide` produces `Pass`. (2) `remember_replay.py` answers ordinals 1–3 correctly so the run genuinely reaches the hidden phase; the no-fail-fast scheduler then fails it on hidden 4–5. (3) `cleared_within` polls for up to 15 s because abnormal-end removal can lag the verdict by a moment.

## T27 — time, memory and process scenarios S05–S08 (done, pending Kiro tick)

- Commit: this commit (`judge-sandbox T27: time, memory, process scenarios`).
- Files: `tests/sbx_limits.rs`, `tests/hostile/{infinite_loop,import_loop,sleep_forever,sigstop_self,memory_list,memory_alloc,fork_bomb,thread_bomb}.py`, `tests/sbx/mod.rs` (+`run_wall`, +`drain_dead`).
- Verification run 16 Nov 2026 on `/run/user/1000/docker.sock`, `--test-threads=1`:
  - `cargo test -p counterspell-judge --features sandbox --test sbx_limits -- --ignored`: **PASS 9/9** — S05/S06 time limits inside bounds; S07 memory contained (`MemoryLimit`/`RuntimeError`, never accepted) + `oom_kill` channel probe; S08 PID-limit containment.
  - `cargo fmt --all -- --check` ok; `cargo clippy --workspace --all-targets --all-features -- -D warnings` ok; `cargo test -p counterspell-judge --all-features` ok.
- Daemon observations for review: (1) removal of a deadline-killed container is asynchronous and lagged past 15 s under load — corpses are `Exited`, still contained; tests assert "not running within 20 s" then reap like the sweeper. (2) Under a fork bomb a daemon call can exceed the API timeout, so the run may end `ExecError::Infrastructure` instead of a completed non-accepted verdict — both accepted for S08a; never `accepted`, always bounded.

## T28 — output scenarios S09–S11 + attach-stream kill fix (done, pending Kiro tick)

- Commit: this commit (`judge-sandbox T28: output scenarios, attach-stream kill fix`).
- Files: `tests/sbx_output.rs`, `tests/hostile/{stdout_flood,stdout_flood_raw,stderr_flood,long_line}.py`, `src/sandbox/session.rs` (`Opened.driver` → `Option`), `src/sandbox/call.rs` (driver dropped before every abort).
- Real bug found by the scenarios: `abnormal()` called `session.abort()` while the driver's attach stream was still open; a program flooding stdout/stderr made the daemon's removal wait on stream teardown (>10 s on the dev daemon, 0.5 s detached), so `abort` timed out, `KillFailed` propagated, and the run ended `Infrastructure` instead of the recorded fact. Both abort paths now drop the driver first.
- Verification run 16 Nov 2026 on `/run/user/1000/docker.sock`, `--test-threads=1`:
  - `sbx_output -- --ignored`: **PASS 5/5** — stdout floods (text + raw `os.write`) and the giant line are `output_limit`; the stderr flood is `time_limit` (stderr never trips the cap) with worker RSS bounded; flood-then-exception is `runtime_error`.
  - Regression: `sbx_functional` 7/7, `sbx_forgery` 5/5, `sbx_limits` 9/9 (incl. s08a clean) — 26/26 on the daemon.
  - `cargo fmt --all -- --check` ok; `cargo clippy --workspace --all-targets --all-features -- -D warnings` ok; `cargo test -p counterspell-judge --all-features` ok.
- Corpus note: `stdout_flood.py` writes `sys.__stdout__` (the real stream — the driver's `quiet()` only replaced `sys.stdout`); floods carry no newline, so the byte cap — not a parse failure — ends the call.

## T29 — network scenario S12 (done, pending Kiro tick)

- Commit: this commit (`judge-sandbox T29: network isolation scenario`).
- Files: `tests/sbx_network.rs`, `tests/hostile/{net_tcp,net_udp,net_dns,net_ipv6,net_raw,net_metadata}.py`, `tests/probes/net_control.py` (trusted control), `tests/sbx/mod.rs` (+`probes_dir`, +`host_ip`, listeners now bind `0.0.0.0`).
- Verification run 16 Nov 2026 on `/run/user/1000/docker.sock`:
  - `sbx_network -- --ignored`: **PASS, conclusive** — control reached the listeners via `host.docker.internal` and the host LAN IP (4 attempts recorded), then all six hostile programs added **zero** attempts and were never accepted.
  - `cargo fmt --all -- --check` ok; `cargo clippy --workspace --all-targets --all-features -- -D warnings` ok; `cargo test -p counterspell-judge --all-features` ok.
- Host-reachability findings (dev daemon): the bridge gateway (`172.17.0.1`) answers RST but does not forward to host listeners; the host is reachable only via its LAN IP — listeners must bind `0.0.0.0` and the test detects the IP with the UDP-connect route probe.

## T30 — filesystem and host-visibility scenarios S13 + S26 (done, pending Kiro tick)

- Commit: this commit (`judge-sandbox T30: filesystem and host-visibility scenarios`).
- Files: `tests/sbx_filesystem.rs`, `tests/hostile/{write_root,write_proc,write_sys,fill_box,big_file,read_host,syscalls_blocked}.py`.
- Verification run 16 Nov 2026 on `/run/user/1000/docker.sock`, `--test-threads=1`:
  - `sbx_filesystem -- --ignored`: **PASS 4/4** — all five write/fill programs never accepted; `container_changes` shows only docker-owned entries (`/etc/*` binds + tmpfs mountpoints) after all five ran inside one production-policy container; mount table matches the exact expected set; `read_host` finds only `/proc/self/mounts` and `/proc/1/environ` (image's public build metadata); mount/unshare/ptrace/keyctl all refused.
  - `cargo fmt --all -- --check` ok; `cargo clippy --workspace --all-targets --all-features -- -D warnings` ok; `cargo test -p counterspell-judge --all-features` ok (74 binaries).
- Dual-use corpus: `read_host.py` and `syscalls_blocked.py` print `REPORT:` lines when exec'd directly as probe scripts and answer wrongly when judged — one file serves both the hostile run and the out-of-band check.

## T31 — cleanup and crash scenarios S16, S17, S24 (done, pending Kiro tick)

- Commit: this commit (`judge-sandbox T31: cleanup and crash scenarios`).
- Files: `tests/sbx_recovery.rs`, `examples/sbx_victim.rs`, `Cargo.toml` ([[example]] required-features), `tests/sbx/mod.rs` (`spawn_source`, pub `repo_root`).
- Verification run 16 Nov 2026 on `/run/user/1000/docker.sock`, `--test-threads=1`:
  - `sbx_recovery -- --ignored`: **PASS 3/3 in 122 s** —
    - `sbx_s16`: victim (trusted `sbx_victim` example running `sleep_forever.py`) SIGKILLed mid-run; no `counterspell.managed` container of the run remained within 60 s of its `created_at`, twice: once with `sleep 25` (self-expiry + sweeper) and once with `sleep 125` (container still running at removal — sweeper alone).
    - `sbx_s17`: `sleep_forever` run → `TimeLimit`, daemon empty after; second run cancelled mid-call → `ExecError::Stopped`, daemon empty after.
    - `sbx_s24`: stop fired while the run's container was live on the daemon → `ExecError::Stopped`, container killed+removed inside cleanup budget + 5 s.
  - `cargo fmt --all -- --check` ok; `cargo clippy --workspace --all-targets --all-features -- -D warnings` ok; `cargo test -p counterspell-judge --all-features` ok (75 binaries, 0 failed).
- The victim is a real `cargo` example binary built by the test (`required-features = ["sandbox", "testkit"]`), spawned with `std::process::Command` — scoped `#[allow(clippy::disallowed_methods)]` since the repo lint bans host process spawn in judge code (the spec mandates `Child::kill` for S16).
