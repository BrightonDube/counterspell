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

## T32 — concurrency scenarios S18, S19 (done, pending Kiro tick)

- Commit: this commit (`judge-sandbox T32: concurrency scenarios`).
- Files: `tests/sbx_concurrency.rs`, `tests/sbx/mod.rs` (`shared_executor`, `execute_on`, `sample_managed_peak`/`ManagedPeak`).
- Verification run 16 Nov 2026 on `/run/user/1000/docker.sock`, `--test-threads=1`:
  - `sbx_concurrency -- --ignored`: **PASS 2/2 in 55 s** —
    - `sbx_s18`: fork bomb + `add_correct` concurrently through one executor → good run `Accepted` in 782 ms, bomb contained (2.98 s, never accepted); `peak_running=2`.
    - `sbx_s19`: six runs through one executor (`max_containers=4`) with a 200 ms daemon sampler → `peak_running=4` = limit, `peak_listed=4`; batch 8.75 s, per-run walls recorded for the report.
  - `cargo fmt --all -- --check` ok; `cargo clippy --workspace --all-targets --all-features -- -D warnings` ok; `cargo test -p counterspell-judge --all-features` ok (76 binaries).
- Interpretation recorded: the semaphore bounds concurrent runs; a killed container can linger listed/running on the daemon past the kill call, so the sampler reports `listed` and `running` peaks separately and the bound is asserted on `running` after a clean-slate `drain_dead`.

## T33 — infrastructure and runtime scenarios S21–S23, S25 (done, pending Kiro tick)

- Commit: this commit (`judge-sandbox T33: infrastructure and runtime scenarios`).
- Files: `tests/sbx_infra.rs`, `.kiro/specs/judge-sandbox/probe-report.json` (written by S25). S20 stays in `sbx_preflight.rs` (T22) — the infra file header cross-references it.
- Verification run 16 Nov 2026 on `/run/user/1000/docker.sock`, `--test-threads=1`:
  - `sbx_infra -- --ignored`: **PASS 4/4** —
    - `sbx_s21`: missing socket → `connect` refuses to build a client (DaemonError); deaf unix listener (accepts, never answers) → probe fails closed inside the API timeout and `execute` returns `ExecError::Infrastructure`, never a verdict.
    - `sbx_s22`: bogus `sha256:000…` pinned digest → `create` fails with `InfraError` in < 1 s; `docker images` identical before and after — nothing pulled.
    - `sbx_s23`: recorded **NOT TESTED** — `SANDBOX_RUNTIME` is not runsc; never assumed.
    - `sbx_s25`: probe JSON with expected/observed per check written to `.kiro/specs/judge-sandbox/probe-report.json`.
  - `cargo fmt --all -- --check` ok; `cargo clippy --workspace --all-targets --all-features -- -D warnings` ok.

## T34 — SandboxOracle (done, pending Kiro tick)

- Commit: this commit (`judge-sandbox T34: SandboxOracle`).
- Files: `src/sandbox/oracle.rs`, `src/sandbox/mod.rs` (export), `tests/model_oracle.rs`, `tests/sbx_oracle.rs`.
- `SandboxOracle` implements `counterspell_engine::Oracle` over two independent `Backend` instances (user + reference): archives contain driver + one side's source only, hidden inputs never enter the API; both sides evaluate concurrently via `tokio::join!` under `handle.block_on`. Outcome map: valid answer → `Returned`, parse rejection → `Garbled`, nonzero exit/OOM/exception → `Crashed{exception_class,message}`, deadline → `TimedOut`, session/exec failure → `OracleError`; diagnostics bounded by truncate(). Reopened sessions reset driver call ids.
- Verification run 16 Nov 2026 on `/run/user/1000/docker.sock`:
  - `model_oracle`: **PASS 8/8** — returned / crashed(exception) / crashed(OOM) / garbled / timed-out-within-limit+slack / infra→OracleError / compile-failure-side-specific / async-context-rejected.
  - `sbx_oracle -- --ignored`: **PASS 2/2** — good pair `Returned(5)` then `Returned(6)` across reused sessions; hostile users → `TimedOut` (3.53 s), `Crashed` (driver exception), `Crashed` (Exit137), `Garbled` (output limit), `Crashed` (OutOfMemory) — never `OracleError`, all within limit+slack.
  - `cargo fmt --all -- --check` ok; `cargo clippy --workspace --all-targets --all-features -- -D warnings` ok; `cargo test -p counterspell-judge --all-features` ok (79 binaries).

## T35 — log hygiene test (done, pending Kiro tick)

- Commit: `judge-sandbox T35: log hygiene canary test`.
- Files: `tests/model_logs.rs`, `tests/sbx_logs.rs`, `tests/support/mod.rs` (`LogCapture`), `tests/hostile/log_canary_fd.py`, `tests/hostile/log_canary_exc.py`, `Cargo.toml` (`tracing-subscriber` dev-dep).
- Design note: `sbx/logs.rs` from the task text lands as top-level `tests/sbx_logs.rs` per the T25 convention (all `sbx_` scenario files sit beside `tests/sbx/mod.rs`).
- Verification run 16 Nov 2026 on `/run/user/1000/docker.sock`:
  - `model_logs`: **PASS 1/1** — canary through answer-parse, raw protocol violation, stderr and exception paths; sentinel proves the capture works; canary absent from every log line.
  - `sbx_logs -- --ignored`: **PASS 1/1** (5.3 s) — `log_canary_fd` → `WrongAnswer` (fd1 write is a protocol violation), `log_canary_exc` → crashed, `CANARY-FD-8c3e`/`CANARY-EXC-4d91` never in captured `tracing` output at TRACE.
  - `cargo fmt --all -- --check` ok; `cargo clippy --workspace --all-targets --all-features -- -D warnings` ok.

## T36 — boundary tests (done, pending Kiro tick)

- Commit: `judge-sandbox T36: boundary tests`.
- Files: `tests/boundary.rs`, `Cargo.toml` (`serde_yml` dev-dep for the compose check).
- Six static checks, no daemon needed: (1) `cargo metadata` graph — server's *production* closure reaches neither `bollard` nor `counterspell-judge` (`kind: "dev"` edges excluded); (2) `bollard` imports confined to `bollard_backend.rs` + `daemon.rs`; (3) no `process::`/`Command::new`/`tokio::process` in `judge/src` or `mcp/src` (`ExitCode` allowed — a type, not a spawn); (4) `call.rs`/`session.rs`/`spec.rs` never touch `problem`/`TestCase`; (5) compose parse — server has no docker.sock mount, judge has none either (T38 adds the rootless mount), never `/var/run/docker.sock`, no published ports, no `VALKEY_URL`; (6) `mcp` imports only `counterspell_judge::sandbox::` and `::languages::`.
- Interpretation recorded: check 1 uses `cargo metadata` not `Cargo.lock` because the lock file merges dep kinds; the server's dev-dep on the judge test kit is deliberate (comment in its Cargo.toml) and never ships in the artifact.
- Verification run 16 Nov 2026: **PASS 6/6**; the first run caught the raw `Cargo.lock` reading (server reaches judge via dev-dep) and prose mentions of `bollard` — both resolved by the narrower, honest checks.
- `cargo fmt --all -- --check` ok; `cargo clippy --workspace --all-targets --all-features -- -D warnings` ok.

## T38 — deployment notes and CI (done, pending Kiro tick)

- Commit: `judge-sandbox T38: deploy notes, CI job, compose socket, renovate`.
- Files: `deploy/rootless-sandbox-daemon.md` (rewritten skeleton → full runbook), `.github/workflows/ci.yml` (`sandbox-integration` job), `docker-compose.yml` (judge-only `SANDBOX_DOCKER_SOCKET` + socket mount), `renovate.json` (new).
- The runbook now covers: `Delegate=cpu cpuset io memory pids` drop-in + verification commands (T02's check), socket group ownership so only the judge reaches it, aggregate `CPUQuota`/`MemoryMax`/`TasksMax` on the sandbox user's slice, pull-by-digest procedure, `preflight` on every deploy, secrets mode `0600` owned by a different user.
- CI: `sandbox-integration` is `continue-on-error: true` and outside `ci-ok`'s `needs` — advisory per open question 14. It builds with `--features sandbox`, runs the non-ignored suite, and asserts `preflight` refuses the runner's rootful `/var/run/docker.sock`.
- Compose: `SANDBOX_DOCKER_SOCKET=/sandbox/docker.sock` env + `${SANDBOX_DOCKER_SOCKET_HOST:-/run/user/1000/docker.sock}` mount on `judge` only; `/dev/null` is documented as the safe no-sandbox escape hatch.
- Renovate: regex manager matches `enabled = true\nimage = "dep:tag@sha256:…"` so only *enabled* images are refreshed; a package rule disables every update type except `digest` and groups into one weekly PR, no auto-merge.
- Verification 16 Nov 2026: `renovate.json`/`ci.yml`/`docker-compose.yml` parse; `docker compose config --quiet` ok; `boundary` **6/6** still pass with the new mount; fmt+clippy clean.

## T39 — security review pass (done, pending Kiro tick; T37 VPS evidence still owed)

- Commit: `judge-sandbox T39: security review pass`.
- Files: `.kiro/specs/judge-sandbox/verification-report.md` (new — review table + pending-T37 section).
- Walked `design.md` §4's 10-item checklist against the code at `ac30d70`: every item verified by the boundary suite, mint-site greps (`Pass` only in `compare.rs:88` via the private-field `Verified` proof; `Accepted` only in `verdict.rs:127`), the `SandboxFiles` archive shape (source + driver + signature only), and preflight enforcement in `SandboxExecutor::execute`. No skip flag exists anywhere.
- unwrap/expect audit: all hits are `#[cfg(test)]` or `testkit` — zero in production.
- Hostile corpus audit: `import_all.py` is the deliberate S27 positive control (docstring: "Expected: accepted"); every other file dies, hangs, breaks the channel, or poisons the protocol before any answer can land.
- **No code findings.** Post-review reruns: `boundary` 6/6, `model_logs` 1/1, `model_oracle` 8/8, `model_sandbox_executor` 12/12, `model_run` 10/10.
- Caveat recorded in the report: this was a fresh-eyes pass by the implementing agent, not an independent reviewer — a separate Kiro session sign-off is still recommended; T37's measured VPS evidence remains the hard deploy gate.

## Verified (spec-driver `verify all`, 16 Nov 2026)

Verified in a clean detached worktree at `36d6064` (the uncommitted work in the main tree was not touched). Rootless daemon `/run/user/1000/docker.sock`, no cgroup warnings.

Fresh gate at `36d6064`:
- `cargo fmt --all -- --check`: ok.
- `cargo clippy --workspace --all-targets -- -D warnings` and with `--all-features`: ok.
- `cargo test --workspace`: 1411 passed, 0 failed, 450 ignored.
- Every `sbx_` test, `--features sandbox --ignored --test-threads=1`: all pass (bollard_create 1, bollard_session 5, concurrency 2, driver_parity 2, filesystem 4, forgery 5, functional 7, infra 4, javascript_driver 20, javascript_run 5, limits 9, network 1, oracle 2, output 5, preflight 2, python_driver 16, recovery 3, sweeper 1). `sbx_logs` has no `sbx_` name prefix, so a name filter skips it; it was run by file and passes (`user_output_never_reaches_the_logs`).
- `counterspell-judge preflight` on the rootless daemon: every check `ok`.
- Model tests: boundary 6, model_bollard 14, model_logs 1, model_oracle 8, model_preflight 15, model_reaper 6, model_sandbox_executor 10.

| Task | Result |
|---|---|
| T20 | ticked. `sbx_create_readback_matches` passes (this also clears the failure recorded in `auth-blockers.md` for auth T37). |
| T22 | ticked. `sbx_preflight` 2/2, `preflight` command exit 0. |
| T23 | ticked. `model_reaper` 6/6, `sbx_sweeper` 1/1. |
| T24 | ticked. `model_sandbox_executor` 10/10. |
| T25 | ticked. `model_corpus` passes; harness used by every `sbx_` file. |
| T26 | ticked. functional 7/7, forgery 5/5. |
| T27 | ticked. limits 9/9. |
| T28 | ticked. output 5/5. |
| T29 | ticked. network 1/1, control conclusive (per the T29 note). |
| T30 | ticked. filesystem 4/4. |
| T31 | ticked. recovery 3/3. |
| T32 | ticked. concurrency 2/2. |
| T33 | ticked. infra 4/4. S23 is recorded NOT TESTED (not runsc), as the task allows. |
| T34 | ticked. `model_oracle` 8/8, `sbx_oracle` 2/2. |
| T35 | ticked. `model_logs` 1/1, `sbx_logs` 1/1. |
| T36 | ticked. `boundary` 6/6. |
| T38 | NOT ticked. Depends on T02, which stays open (VPS re-verify). Files exist and `docker compose config` passes. |
| T39 | NOT ticked. Depends on T37 (VPS). The T39 record says the review was by the implementing agent, not an independent reviewer, which the task requires. |
| T37 | NOT ticked. Blocked: VPS. |

Corrections to the T39 record: it says `model_sandbox_executor` 12/12 and `model_run` 10/10. The real counts are 10 and 12. Nothing was removed (`model_sandbox_executor.rs` has one commit and 10 tests).
