# Tasks: language-drivers

All tasks below are implementation tasks for the design above. No task is being executed by this spec-generation run. Each task is intended to be reviewable in 30 minutes or less. `[P]` means it can run in parallel within its wave after dependencies are complete. `[S]` means sequential or gated. `[stretch]` means it must wait for the core freeze-safe gate.

## Wave 0: protocol and registry foundations

- [ ] **T01 [S] (25 min) Define the versioned driver protocol**
  - Files: `languages/protocol.md` and any narrowly scoped protocol fixture file.
  - Document the signature descriptor, JSON-lines request/answer envelopes, call IDs, supported JSON-safe types, stdout purity, stderr behavior, errors, source export conventions, and protocol version 1.
  - Depends on: repo-skeleton completion and the judge-sandbox interface being available.
  - Verify: the examples parse; the document explicitly states that drivers never emit verdicts, expected values, timings, or Familiar moods.
  - Preserve: the trusted worker owns comparison, deadlines, timing, and verdicts; no code execution on the host.

- [ ] **T02 [P] (25 min) Add and validate the language registry schema**
  - Files: `languages/languages.toml`; registry validation in the existing `crates/judge/` modules.
  - Add Python and enabled JavaScript entries with driver path, run command, bench arguments, limits, protocol version, and byte limit. Add disabled Java and C++ entries with their stretch images and commands.
  - Depends on: T01.
  - Verify: the loader rejects malformed entries and missing drivers; Python and JavaScript load as enabled; Java and C++ remain unavailable.
  - Preserve: the six-package workspace and the canonical `languages/` location; no runtime image pulls.

- [ ] **T03 [P] (25 min) Establish the Python baseline driver contract**
  - Files: `languages/drivers/python/driver.py` and Python driver tests/fixtures.
  - Bring the Python driver into the versioned protocol if needed. Add signature loading, one-call-at-a-time framing, JSON-safe return validation, stdout capture, clean stderr failures, repeated calls, and `--mode bench` parity.
  - Depends on: T01.
  - Verify: protocol fixtures pass for scalar, list, nested-list, boolean, string, null-where-allowed, exception, invalid-output, and repeated-call cases.
  - Preserve: the Python driver never loads the Python reference solution and never prints a verdict or timing.

## Wave 1: JavaScript launch driver

- [ ] **T04 [S] (30 min) Implement the Node.js JavaScript driver**
  - Files: `languages/drivers/javascript/driver.js` and JavaScript driver fixtures/tests.
  - Load `/box/signature.json`, resolve the documented CommonJS function export, parse request lines, invoke scalar and array signatures, await returned Promises, validate JSON-safe results, capture language-level stdout, and emit only `{t,out}` answer lines.
  - Depends on: T01 and T03.
  - Verify: the driver handles named exports and callable module exports, preserves call IDs, rejects undefined/non-finite/cyclic outputs, flushes each answer, and sends diagnostics only to stderr.
  - Preserve: Node runs only in the sandbox; user output cannot declare a verdict.

- [ ] **T05 [P] (20 min) Add the JavaScript registry entry and driver packaging**
  - Files: `languages/languages.toml`, language-driver packaging/configuration used by the judge.
  - Add `node:22-slim`, `solution.js`, `drivers/javascript/driver.js`, `node --max-old-space-size=192 driver.js`, `--mode bench`, memory/time/source limits, and protocol version 1.
  - Depends on: T02 and T04.
  - Verify: a sandbox can start the configured command with a known-correct fixture and the judge can append bench arguments without changing the normal command.
  - Preserve: no compile step for JavaScript and no per-language reference solution.

- [ ] **T06 [S] (30 min) Verify Python/JavaScript observable parity**
  - Files: driver test fixtures only, plus implementation fixes in the two driver templates if required.
  - Run identical protocol cases through Python and Node, including generated JSON-safe values, function errors, user prints, repeated calls, and bench-mode calls.
  - Depends on: T03, T04, and T05.
  - Verify: both drivers produce equivalent parsed answer envelopes and equivalent process/error categories; no driver stdout contains verdict/timing/expected fields.
  - Preserve: JSON value semantics and the worker-owned verdict boundary.

## Wave 2: trusted judge and Efficiency Familiar bench handoff

- [ ] **T07 [S] (30 min) Integrate configured drivers with the existing sandbox runner**
  - Files: `crates/judge/src/` language integration modules and focused tests.
  - Copy the selected source, read-only driver template, and non-secret signature descriptor into the sandbox; build normal argv; start the configured image with the existing isolation, output caps, cleanup labels, and process lifecycle.
  - Depends on: T02, T03, T05, and the judge-sandbox implementation.
  - Verify: a passing and failing JavaScript run produce worker-owned comparison results; malformed answers, exceptions, and process failures never become Accepted.
  - Preserve: the server has no Docker access; the judge uses only the separate rootless sandbox daemon.

- [ ] **T08 [S] (30 min) Add generic bench-mode process handoff**
  - Files: `crates/judge/src/bench.rs` or the existing judge lifecycle module, plus focused tests.
  - After correctness passes, stop the normal driver and start a clean driver with `bench_args` in the same sandbox container. Send two untimed warm-ups, five timed calls per generated size, and enforce Run/Submit budgets and per-call deadlines using the worker clock.
  - Depends on: T07 and the problem bench interface.
  - Verify: a JavaScript bench run records raw samples, computes medians for points, stops within 1.5 seconds/three sizes for Run and 4 seconds/five-to-six sizes for Submit, and cleans up on timeout.
  - Preserve: the driver emits no timing; expected outputs and complexity classification remain outside the driver.

- [ ] **T09 [P] (25 min) Persist and publish bench measurements safely**
  - Files: the existing judge run-timing/event integration modules and tests; only the migrations already owned by the run-lifecycle design if required.
  - Persist `run_timings` rows and emit `bench_start`, `bench_point`, `complexity`, and `familiar` through the existing Postgres `run_events` path, with only public fields.
  - Depends on: T08 and run-lifecycle event persistence.
  - Verify: public events contain no hidden input, expected output, source, or raw answer; event sequence and terminal-write behavior remain compatible with SSE replay.
  - Preserve: Postgres is truth and event storage; Valkey is not used for queueing or events.

- [ ] **T10 [P] (30 min) Add generated-input driver properties for every published problem**
  - Files: `packs/<slug>/fixtures/<lang>/known_correct.*`, property/integration harness files in the existing judge or Grimoire test locations, and no production reference rows.
  - Add a known-correct Python and JavaScript fixture for every published problem. Run generated schema-valid inputs through each fixture, assert exact canonical JSON round trips, compare each language fixture to the Python reference with the pack comparator, and include deterministic seeds in failures.
  - Depends on: T03, T04, T07, and the available pack generator/reference interfaces.
  - Verify: the property suite covers every published pack, repeated calls are deterministic, and all fixture/reference execution occurs through the trusted sandbox path.
  - Preserve: Python is the only authoritative reference; fixtures are test-only and do not become user-visible or production expected-output sources.

- [ ] **T11 [S] (25 min) Verify hostile protocol and cleanup behavior**
  - Files: focused judge/driver integration tests and only implementation fixes required by those tests.
  - Exercise infinite loops, huge output, exceptions, malformed JSON, mismatched call IDs, invalid JSON values, fake verdict-shaped prints, and direct protocol corruption. Confirm external deadlines, output limits, non-Accepted outcomes, and container cleanup.
  - Depends on: T07, T08, and T10.
  - Verify: no hostile fixture can declare a verdict; infrastructure failures are not reported as Wrong Answer; stale containers are reaped by the existing boundary.
  - Preserve: no keyword blocklist is added as a security control.

- [ ] **T12 [S] (20 min) Run the core freeze-safe gate**
  - Files: no planned files; fixes only where a preceding core task fails verification.
  - Confirm Python and JavaScript launch judging, generated-input properties for every published problem, bench-mode timing/events, cleanup, and existing workspace checks are green on the target test environment.
  - Depends on: T06, T09, T10, and T11.
  - Verify: record the gate result and do not begin any `[stretch]` task unless the core fail/pass, sandbox, property, and bench checks are green.
  - Preserve: Java and C++ remain disabled until this gate passes; no stretch work is smuggled into the launch path.

## Wave 3: stretch languages, only after T12 passes

- [ ] **[stretch] T13 [S] (30 min) Implement the Java driver and compile path**
  - Files: `languages/drivers/java/Driver.java`, Java registry entry/configuration, and judge compile integration.
  - Implement the same protocol and signature descriptor contract, compile `Solution.java` and `Driver.java` with `javac -d /box/out`, and use `eclipse-temurin:21-jdk` with the initial `-Xss16m`, `-Xmx256m`, and SerialGC settings.
  - Depends on: T12 and the existing compile-capable sandbox runner.
  - Verify: a Java known-correct fixture passes normal and bench protocol tests without exposing expected outputs.
  - Preserve: this is `[stretch]`; do not enable Java in the launch API before its full property suite passes.

- [ ] **[stretch] T14 [P] (25 min) Select the smallest passing Java stack size**
  - Files: Java deep-recursion fixtures, Java configuration, and verification notes owned by the implementation.
  - Run designated deep-recursion problems at `-Xss` candidates from 8 MiB through 16 MiB, starting at 16 MiB, and choose the smallest setting that passes all designated fixtures.
  - Depends on: T13.
  - Verify: record the selected setting and confirm it does not weaken the memory or wall-time sandbox limits.
  - Preserve: use `eclipse-temurin`, never `openjdk`; Java remains a test fixture/launch option only after approval.

- [ ] **[stretch] T15 [S] (30 min) Add C++ driver and compile path**
  - Files: `languages/drivers/cpp/driver.cpp`, C++ registry entry/configuration, and judge compile integration.
  - Implement the same protocol/signature contract and compile with `gcc:14`, `g++ -O2 -std=c++20`, and `/box/out` output. Keep comparison, timing, and verdict authority in the worker.
  - Depends on: T12 and the existing compile-capable sandbox runner.
  - Verify: a C++ known-correct fixture passes normal and bench protocol tests in the sandbox and emits only answer envelopes.
  - Preserve: this is `[stretch]`; no host compilation and no C++ production reference.

- [ ] **[stretch] T16 [S] (25 min) Run stretch property suites and enable only approved languages**
  - Files: stretch fixtures, registry enablement, and focused verification documentation.
  - Run generated-input round-trip/reference-agreement properties for Java and C++, including compile errors, deep recursion, invalid values, timeouts, and bench mode. Enable only the language whose complete suite passes and whose freeze-safe review is approved.
  - Depends on: T14 and T15.
  - Verify: every enabled language has a known-correct fixture for every published problem, agrees with the Python reference under the trusted comparator, and passes the same security checks as launch languages.
  - Preserve: Python remains authoritative; partial stretch support stays disabled rather than being presented as complete.

## Completion checklist

- [ ] `languages/languages.toml` contains validated Python and JavaScript launch entries with Node run commands and bench arguments.
- [ ] Python and JavaScript drivers share protocol version 1 and emit answer envelopes only.
- [ ] JavaScript runs from `node:22-slim` with `solution.js` and `driver.js`.
- [ ] Bench mode uses the worker clock, two warm-ups, five repeats per size, raw `run_timings`, and safe public events.
- [ ] Every published problem has Python and JavaScript known-correct test fixtures.
- [ ] Generated-input round-trip and Python-reference agreement properties pass through the sandbox.
- [ ] Hostile output, fake verdict JSON, invalid protocol, timeouts, and cleanup behavior are verified.
- [ ] T12's core freeze-safe gate is recorded before any `[stretch]` task starts.
- [ ] Java uses `eclipse-temurin`, tests 8–16 MiB stack candidates, and retains the smallest passing setting before enablement.
- [ ] C++ uses `gcc`, follows the same protocol, and is enabled only after its full property suite passes.
- [ ] No per-language production references, alternate crate, second queue, or host execution path was added.
