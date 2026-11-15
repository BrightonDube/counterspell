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
