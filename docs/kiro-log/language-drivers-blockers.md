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
