# language-drivers blockers

Run: attempt 1, 15 Nov 2026. Non-interactive run approved by Brighton (12 Nov 2026).

## Summary

- **Done and ticked:** T01 (`b8f8c7e`): `languages/protocol.md` plus `crates/judge/tests/model_protocol_doc.rs`.
- **Partly done, not ticked:** T05 (`cb52f4c`): `ExecSpec::bench` appends `bench_args` to the catalogue run argv. The judge can build the bench command. Starting it in a sandbox is still unverified, so T05 stays open.
- **Blocked:** T02 to T04, T06 to T12. Every remaining task needs either a real rootless sandbox daemon to run code, or a human-pinned image digest.
- **Stretch, not started:** T13 to T16 (Java and C++). T12 must pass first.

## The main blocker: no verified sandbox to run driver code

The spec says all code execution for tests goes through the judge's sandbox runner on the rootless sandbox daemon. Nothing runs on the host. Three things block that:

1. **judge-sandbox T02 (HUMAN, VPS spike)** is open. The only local run used rootful WSL Docker, which the identity guard refuses. Rootless behaviour is still unverified.
2. **judge-sandbox T20 `#[ignore]` guard.** The PreToolUse guard refuses `#[ignore]` on `sbx_*` tests. Every real-daemon test in the judge-sandbox chain depends on that decision. Brighton has not yet answered (see `judge-sandbox-blockers.md`).
3. **judge-sandbox executor.** `crates/judge/src/executor.rs` still uses `NoExecutor`. There is no working path that starts a driver in a container and returns answers.

Effect: T03 (Python driver), T04 (Node driver), T06 (parity), T07 to T12 cannot be verified as the tasks require. Kiro did not write driver code without a way to run it, because that would break the red-green rule (TDD rule 2). Kiro also did not run driver tests on the host.

**Unblock:** Brighton decides on judge-sandbox `#[ignore]` for `sbx_*` tests. A rootless sandbox daemon then runs the `sbx_` tests. With those, the next run can start T03 as RED: write the protocol fixture tests, run them through the sandbox, then write `driver.py`.

## T02: registry

- **Python driver file missing.** `languages/languages.toml` enables Python. The loader refuses an enabled language whose driver is missing, so the real catalogue fails to load. Fixing this needs `languages/drivers/python/driver.py` (T03, blocked above).
- **JavaScript digest is a placeholder.** The JavaScript entry is disabled and holds the all-zero digest. The spec needs `node:22-slim` enabled. A real digest must be pinned first (HUMAN step, the same as judge-sandbox T03 for Python). Renovate then refreshes it weekly.
- **JavaScript driver file missing.** `drivers/javascript/driver.js` is needed before enabling the entry (T04, blocked above).

**Unblock:** pin the Node digest (human), write the driver (T04), then enable JavaScript.

## T05: JavaScript registry entry and packaging

- The bench argv part is done (`cb52f4c`).
- The verification "a sandbox starts the configured command with a known-correct fixture" is blocked (see the main blocker).
- Needs the Node digest pin.

## T07: driver integration with the sandbox runner

- The judge has `ExecSpec::driver` and `ExecSpec::bench` for argv. The driver is copied with `build_tar`.
- **Signature descriptor not built.** `signature.json` must hold only public signature metadata. The shape is in `languages/protocol.md`. The only signature type in the repo is `counterspell_grimoire::record::Signature` (`crates/grimoire/src/signature.rs`). The judge does not depend on grimoire, and the task names no such dependency. Kiro did not add one.
  - **Decision needed from Brighton:** either (a) add a `counterspell-grimoire` dependency to the judge, or (b) move a shared public signature type into `counterspell-common`. This choice changes the crate graph, so Kiro did not make it.
- The sandbox start and comparison verification is blocked (main blocker).

## T08 to T12

- Blocked on the sandbox runner (T07) and on the bench problem interface.
- **T09:** needs run-lifecycle event persistence for `bench_*` events. Not checked whether those event types exist yet.
- **T10:** needs known-correct fixtures for each published pack. The packs are still draft (grimoire-packs T28 is waiting for Brighton's review). Fixtures for unpublished packs would be guesswork.
- **T12:** the freeze-safe gate can only be recorded after T06, T09, T10 and T11 pass.

## Stretch (T13 to T16)

Not started. The spec's core gate (T12) has not passed, so Java and C++ stay out of this pass.

## Notes for the next run

- Next start: T02, once the JavaScript digest is pinned, or T03 once the sandbox tests can run.
- The next run's `driver.py` should start from `protocol.md` and the `ExecSpec` argv.

## Run stop point (attempt 2, 15 Nov 2026): nothing ticked, no code changed

- **Checked:** the judge-sandbox blockers are unchanged. `sbx_*` tests still cannot carry `#[ignore]` under the PreToolUse guard, so no real-daemon test can run. `languages.toml` still has the all-zero JavaScript digest, and `drivers/python/driver.py` and `drivers/javascript/driver.js` still do not exist.
- **T02:** `crates/judge/tests/model_languages.rs` already covers the malformed-entry and missing-driver rules (tag refused, short or uppercase digest refused, repeated table, unknown field, zero limit, bad file name, unsupported protocol, empty run, missing driver for an enabled language). The open criterion is "Python and JavaScript load as enabled". Python needs `driver.py` (T03). JavaScript needs a digest pinned by a human (`docker buildx imagetools inspect node:22-slim` on the sandbox host, then Brighton records it), plus `driver.js` (T04). Java and C++ stay disabled.
- **T03 to T12:** not started. Each one needs the driver run inside the sandbox runner, and the spec says nothing runs on the host. Running the Python or Node driver on the host to get a red test would break that rule, so I did not do it.
- **Stretch T13 to T16:** not started. T12 has not passed.
- **Decisions Brighton still needs, in order:**
  1. Allow `#[ignore]` on `sbx_*` tests (judge-sandbox T20 onward), or name another approved gate.
  2. Pin the `node:22-slim` digest (human step).
  3. Choose how `signature.json` is typed (add a `counterspell-grimoire` dependency to the judge, or move the signature type into `counterspell-common`). This is needed for T07.
- **Next start:** T03 once the sandbox can run driver tests, starting from the protocol fixtures in `languages/protocol.md`.


## Assistant update (15 Nov 2026, 00:52 SAST)

Resolved:

- The `node:22-slim` digest is now pinned (and Temurin 21 and GCC 14, still disabled). See `deploy/sandbox-verification.md` run 2 and `brighton-decisions`.
- `#[ignore]` on `sbx_*` tests is allowed by the hook, so the next judge-sandbox run can write the real-daemon tests. They still cannot be ticked until they pass on a rootless daemon.
- Sandbox-verification blockers follow the 14 Nov local-equivalent decision, extended on 15 Nov: work that does not need a real daemon may proceed; container starts on the rootless daemon stay open; do not run driver code on the host.

Still needs Brighton or the VPS:

- Rootless daemon (VPS, or local `uidmap` + rootless Docker). This still blocks ticking T03 to T12 verification that starts a sandbox.
- The T07 signature-type crate-graph choice (grimoire vs `counterspell-common`).
- Starter pack review before T10 published-pack fixtures.

## Assistant update (15 Nov 2026, 01:03 SAST)

A local rootless Docker daemon is now running (see `deploy/sandbox-verification.md` run 3 and `brighton-decisions.md`). Driver work that starts a container on that daemon still cannot be ticked until Brighton applies the cgroup `cpuset`/`io` delegation drop-in (the judge will refuse the daemon while those warnings remain) and the relevant `sbx_` tests exist and pass. T02 and T37 stay open for the VPS. Do not run driver or pack code on the host.

## Run note (attempt 1 continued, 15 Nov 2026 05:09 SAST)

- **T02 ticked** (`0d94d99`). `languages/languages.toml` now enables JavaScript (`node:22-slim`, digest already pinned). The shipped-catalogue test now expects Python and JavaScript enabled and Java and C++ disabled. It stubs both driver files. The assertion change is marked `test-change: IDE-1`, because launch languages are Python and JavaScript (IDE-1 in `docs/requirements.md`). Verified: `cargo fmt --check`, `cargo clippy --workspace --all-targets -D warnings` and `cargo test --workspace` all pass.
- **Open risk:** the real `languages/languages.toml` still fails `LanguageCatalog::load` until `drivers/python/driver.py` and `drivers/javascript/driver.js` exist. Nothing in production loads that file yet (only tests do). This is the same interim state the file header documents for Python. T03 and T04 remove it.
- **T03 to T12 still blocked.** `crates/judge/src/main.rs` still runs `NoExecutor`. There is no sandbox executor, so no driver can be started and no red-green cycle can run against a container. Running driver code on the host is forbidden by the spec.
- **Brighton decisions still open:** the T07 signature-type crate choice (grimoire dependency or `counterspell-common`), and the judge-sandbox `sbx_` ignore decision and cgroup delegation drop-in.
- **Stretch T13 to T16:** not started. T12 has not passed.

## Run stop point (attempt 3, 15 Nov 2026 later): nothing ticked, no driver code written

- **Re-checked the state.** `crates/judge/src/sandbox/` now has the sandbox runner library (`run.rs`, `session.rs`, `call.rs`, `driver_protocol.rs` parser for `{t,out}` answers). `crates/judge/src/executor.rs` still wires only `NoExecutor`, so no production path starts a driver in a container. `languages/drivers/` does not exist, so the real `languages.toml` still fails `LanguageCatalog::load` (Python and JavaScript are enabled).
- **Rootless daemon.** `/run/user/1000/docker.sock` answers (Docker 29.4.3, rootless, cgroup v2, systemd driver). It still prints "No cpuset support" and "No io.* support". The cgroup delegation drop-in in `brighton-decisions` is not yet applied, so the judge identity guard will refuse this daemon.
- **T03 (Python driver) not started.** Its red step is a protocol fixture run through a container. Running `driver.py` on the host is forbidden, and no sandbox executor path exists to run it through. Writing `driver.py` without a failing test would break the TDD rule.
- **T04 to T12 not started.** They depend on T03 or on the executor. T13 to T16 are stretch and wait for T12.
- **T07 signature-type choice is settled.** `brighton-decisions` (15 Nov 00:55) moves the shared signature type into `counterspell-common`. T07 no longer waits on that decision. It still waits on the executor and the cgroup drop-in.

**Unblock, in order:**
1. Apply the cgroup `cpuset`/`io` delegation drop-in (Brighton, sudo) so `docker --context rootless info` has no "No ... support" warnings.
2. Judge-sandbox: write the `sbx_` tests and a production `RunExecutor` that uses `sandbox::run::execute_run` with the rootless socket. Until then no driver can be run in a container.
3. Then T03 starts as RED: a Python protocol fixture (scalar, list, nested list, bool, string, null, exception, invalid output, repeated calls) run through that executor.

## Run note (attempt 1 resumed, 16 Nov 2026): T03 stays `[ ]`, nothing else can start

- **Checked the state before writing code.**
  - `languages/` has `languages.toml` and `protocol.md` only. `drivers/` does not exist.
  - `docker --context rootless info` shows no cpuset or io warnings, so the cgroup delegation drop-in appears applied.
  - `docker --context rootless images` lists no images. The pinned `python:3.13-slim` digest and `node:22-slim` digest are not on the sandbox daemon.
  - `crates/judge/src/executor.rs` still defines only `NoExecutor`, and `main.rs` still serves with it. `SandboxExecutor` is judge-sandbox T24, which is open.
- **T03 not started.** Its red step is a Python protocol fixture run through a container. Running `driver.py` on the host is forbidden. Writing `driver.py` without a failing test breaks the TDD rule. No driver file was written.
- **T04 to T12 not started.** Each depends on T03, on a container run, or on the executor. T07 and T08 need a real JavaScript run to verify. T09 depends on T08. T10 needs sandbox runs and published-pack fixtures. T11 and T12 need the sandbox. T13 to T16 are `[stretch]` and wait for T12.
- **Unblock, in order:**
  1. Judge-sandbox T21 and T24: a production `RunExecutor` on the rootless socket, plus the pinned image on that daemon. Brighton pulls the digests (`docker -H unix:///run/user/1000/docker.sock pull python:3.13-slim@sha256:e48f59cdba69a3cbd9ed2cf0f348d90d45a8d350d98134cf1070bc106ee9ed90` and the Node digest in `languages/languages.toml`). Kiro did not pull them, because the earlier decision treats image setup as Brighton's step.
  2. Then T03 starts as RED: a Python protocol fixture (scalar, list, nested list, bool, string, null, exception, invalid output, repeated calls) run through that executor.
  3. T04 follows the same pattern for `driver.js`, which also unblocks T05 and the enabled JavaScript entry in `languages/languages.toml` (the loader currently fails on the missing drivers).
- **Still needs Brighton:** starter pack review before T10 fixtures for published packs (grimoire-packs T28).

## 16 Nov 2026 03:15 SAST: images now on the sandbox daemon
The pinned Python and Node images are pulled onto the rootless sandbox daemon, and the Node digest is pinned (see brighton-decisions.md, 16 Nov 03:15). The image blockers above are cleared; rerun the sbx_ tests.

## Run (attempt 1 continued, 16 Nov 2026, later): T03 to T07 done; T08 onward blocked

**Done this run (all verified on the rootless sandbox daemon, `SANDBOX_DOCKER_SOCKET` from `dev-test.env`):**
- T03 Python driver, `eba247e`: 16 `sbx_python_*` tests pass.
- T04 Node driver, `374226d`: 20 `sbx_javascript_*` tests pass.
- T05 JavaScript registry contract tests, `8f8da09`: `model_spec` tests pass.
- T06 Python/Node parity, `8b50d8f`: 8 cases in normal and bench mode agree; `sbx_driver_stderr_carries_only_diagnostics` passes.
- T07 JavaScript run path through `execute_run`, `815f432`: 5 `sbx_javascript_run_*` tests pass.
- The Rust gate (`fmt --check`, `clippy -D warnings`, `cargo test --workspace`) is clean after each commit.

**T03 note:** the RED step was real. All 15 tests failed on the missing `driver.py`. The first GREEN attempt failed for two reasons. `contextlib.redirect_stdin` does not exist, and the backend emits stdout in chunks, not lines. Both were fixed in the driver and the test helper.

**T04 note:** the RED step was real. The tests failed on the missing `driver.js`. The first run passed all 19 tests.

**T06 note:** a verification task. The drivers were written before the parity table, so the parity test is an after-the-fact check. It passed on the first run.

### Judge-sandbox fix made in T07 (needs Brighton's review; the judge is the security boundary)

- **Defect found:** `BollardSession::read_policy` compared the container with the policy but did not remove variables the image declares. The create-time read-back (`verify_readback`) did remove them. The session path, which `open_session` uses for real runs, therefore failed for any image that declares its own variables. The Node image declares `NODE_VERSION` and `YARN_VERSION`, so the first real run failed with `PolicyReadbackMismatch(UnexpectedEnvironment)`. The Python tests never reached that path, because they call `create` and `open_driver` directly.
- **Fix:** `BollardSession` stores the image's declared environment at create time. `read_policy` applies the same `without_image_env` filter. `create` reads the image before it creates the container, so a failed image check creates nothing.
- **Verification:** `sbx_javascript_run_*` (five tests) and the regression `sbx_session_read_policy_tolerates_image_env` pass. I did not watch the regression test fail against the old code. The Node run test did fail on the old code with the error above.
- **Review question:** please confirm the policy still rejects injected or changed variables. `without_image_env` only removes a pair that matches one the image declares exactly.

### T08 (bench handoff): blocked

Two things are missing. Neither can be built inside T08 without changing an interface that another spec owns.

1. **No way to end the normal driver in the same container.** The spec says to stop the normal driver and start a clean bench driver in the same container. `DriverChannel` has only `write_line` and `next_event`. The bollard driver cannot close the driver's stdin, and there is no kill-the-process call. A second container would break "same container" and the one-compile-per-run rule. *Unblock:* add a `close_input` (or stop) primitive to `DriverChannel` and its bollard implementation. That is a judge-sandbox change, so judge-sandbox's owner should decide it.
2. **No bench inputs.** The bench needs the problem's generator to produce arguments for each size. Those come from `ProblemTests`, which is judge-sandbox T24 and is still open.

I did not build a separate-container bench, because it would not match the spec.

### T09 to T12, T13 to T16

- **T09** (persist and publish bench events) depends on T08 and on run-lifecycle `bench_*` persistence. Neither is ready.
- **T10** (known-correct fixtures for every published problem) needs published packs. The packs are still drafts and wait for Brighton's review (grimoire-packs T28). I did not write fixtures for draft packs.
- **T11** (hostile protocol and cleanup tests) depends on T08 and T10. See the open findings below, which T11 should cover.
- **T12** (the core freeze-safe gate) cannot be recorded until T06, T09, T10 and T11 pass. It is not recorded.
- **T13 to T16** are `[stretch]`, and T12 has not passed. They were not started.

### Open findings for the review (Minor and Important)

1. **Important (for T11): stray stdout between calls.** Both drivers capture user output only while a call runs. Output written later, from a timer, a callback or a background thread, still goes to the real stdout. The judge reads the next line as the answer to its next call. A program could therefore put a correct-looking answer line on the stream before the real answer, and the judge would compare it. The drivers cannot fully close this, because the judge only sees lines. Likely fix: the judge treats any stdout line that is not the answer to the outstanding call as a protocol failure, and T11 adds a test for a timer that prints a forged answer. Python's `os.write(1, ...)` and Node's `fs.writeSync(1, ...)` bypass the capture the same way. Node's stdout write is only overridden during a call.
2. **Minor: duplicate keys.** The Python driver rejects a call line with a repeated key. The Node driver cannot, because `JSON.parse` keeps the last copy. The judge writes every call line itself, so this is not exploitable. It is a parity gap and is documented in the Node driver's header.
3. **Minor: `process.exit` in user code.** The Node driver swaps `process.exit` only during a call, so a timer can exit the process with code 0 between calls. The judge treats an exit without an answer as a failure, so no run is Accepted. The exit code is simply less precise than the Python driver's.
4. **Minor: stdin.** Python hides `sys.stdin` during a call. The Node driver cannot, so user code that reads `process.stdin` could consume call lines. The external deadline still ends that run.
5. **Minor: early Python failures.** Two early failures were fixed: a driver bug (`contextlib.redirect_stdin` does not exist) and a harness race in the missing-function test, which now reads stderr instead of sending a call. Both are recorded in the T03 note above.

**Unblock, in order:** (a) Brighton decides the `DriverChannel` stop primitive, or names another route to end the normal driver in the same container. (b) judge-sandbox T24 (`ProblemTests`). (c) Brighton reviews the packs (grimoire-packs T28). (d) Brighton reviews the read-back fix above.

## Run stop point (attempt 2, 16 Nov 2026): nothing ticked, no code changed

- **Checked before writing anything.**
  - `DriverChannel` (`crates/judge/src/sandbox/backend.rs`) still has only `write_line` and `next_event`. There is no stop, close-input or kill primitive for the normal driver, so T08's "stop the normal driver, start a bench driver in the same container" cannot be built without a judge-sandbox interface change.
  - judge-sandbox T24 (`ProblemTests`, `SandboxExecutor`, judge wiring) is still `[ ]`. T08 also needs bench inputs from `ProblemTests`.
  - Every starter pack (`count-pairs`, `diagonal-total`, `longest-run-length`) is still `status = "draft"`. Brighton has not run grimoire-packs T28. T10 needs published packs, and fixtures for draft packs would be guesswork.
- **T08:** blocked on the two judge-sandbox items above. It is not marked `[BLOCKED]` in tasks.md, so this note records the reason.
- **T09, T11, T12:** depend on T08 (and T10 for T11 and T12). Not started.
- **T10:** blocked on grimoire-packs T28 (Brighton publishes the packs).
- **T13 to T16:** `[stretch]`, skipped this pass. T12 has not passed.
- **Next start, in order:**
  1. Brighton or the judge-sandbox owner decides the `DriverChannel` stop primitive, and judge-sandbox T24 lands (`ProblemTests`, `SandboxExecutor`).
  2. T08 then starts as RED: a bench handoff test that expects two warm-ups, five samples per size, and cleanup on timeout, run on the rootless daemon.
  3. Brighton publishes the starter packs (grimoire-packs T28). T10 then writes known-correct fixtures for published packs only.
- **Security note:** no stop primitive was added here. It changes the sandbox security boundary, and the judge-sandbox owner should review it.

## Run (16 Nov 2026, continued): T08 done, both blockers resolved without interface changes

- **Normal-driver stop:** no new `DriverChannel` primitive was needed. Dropping the channel (`opened.driver.take()` then drop) closes the exec attach, the driver process sees stdin end, and its serve loop exits on its own. A clean driver then opens with `ExecSpec::bench` in the same container, inside `SandboxBenchCollector`.
- **Bench inputs:** `counterspell_engine::complexity::{sized_input, bench_seed}` build seeded inputs from the signature schema directly; `ProblemTests` is not on this path. (judge-sandbox T24 has since landed anyway.)
- **Real bug found while wiring:** `BollardDriver::next_event` emits raw `Chunk`s, not `Line`s — `LineReader` (`sandbox/stream.rs`) does the splitting, exactly as `call.rs` does. The fake backend emits `Line`, which hid this in model tests. The collector now pushes both variants through `LineReader` with the `STDOUT_BUDGET` cap and treats any output that is not exactly one answer line as a protocol failure.
- **Real bug fixed:** call lines were encoded once per size and reused per repetition; ids are now sequential across all warm-ups, timed reps and sizes.
- **Budget semantics fixed:** a budget that runs out mid-call is `BenchOutcome::Incomplete` (finished points kept), not `Failed`.
- **Done:** T08 ticked in `tasks.md`. Evidence: `model_bench` 6/6, `sbx_bench` 2/2 on the rootless daemon (six Submit sizes, real medians, budget cut `Incomplete`, container reaped). Review entry in `language-drivers-review-queue.md`.
- **Still open:** T09 (bench persistence/events, needs run-lifecycle `bench_*` types and the keep-session handoff in `execute_run`), T10 (published-pack fixtures, still waiting on grimoire-packs T28), T11, T12. T13-T16 stretch, gate not passed.

## Run (16 Nov 2026, continued): T09 done; T10 onward still blocked

- **T09:** done and committed (`b16a9ae`). `run_timings` rows (one per timed call, fenced by `attempt`, recorded only on a normally ended bench loop) plus `bench_start`/`bench_point`/`complexity`/`familiar` through the fenced `run_events` path, `BoundedJson` payloads, and the `bench_start` contract carried through the `run_events.type` CHECK migration, `run_state`, and the web SSE parser/reducer. Measured columns reach the terminal write via `FinalResult.measured`. Evidence: `model_sandbox_executor` 14/14, `sbx_bench` 3/3 (30 timing rows on the real daemon), all sbx suites re-run green, workspace clippy clean, `svelte-check` 0 errors, web vitest 141/141.
- **Still blocked:** T10 needs published packs (grimoire-packs T28, Brighton); T11 and T12 chain behind T10; T13-T16 are `[stretch]` and gated on T12.
