# Tasks: judge-sandbox

No task in this document has been executed. This spec-generation run only wrote `requirements.md`, `design.md` and `tasks.md`.

**Do not start T01 until T00 is signed off.** T00 is Brighton's review of the trusted/untrusted boundary and his answers to the isolation questions in `design.md` ("Open questions" 1 to 7 and 16).

Conventions:

- `[P]` can run in parallel with other tasks of its wave once its dependencies are done. `[S]` is sequential.
- `[SUPERVISED]` means Brighton reviews each change before it is accepted (security-critical code).
- `[HUMAN]` means a person must do it on the VPS or another host Kiro cannot reach. Kiro prepares the commands and records the results.
- Each task is sized for 30 minutes or less. Each Rust task ends with a short **Rust lesson** for Brighton.
- Real-sandbox tests are `#[ignore]` and named `sbx_…`. They need `SANDBOX_DOCKER_SOCKET` and refuse to run on anything that is not a rootless sandbox daemon. **No hostile program is ever run on the host, in tests or anywhere else.**
- Dependencies on other specs: T24 needs run-lifecycle T16 (`RunExecutor`) and T19 (judge wiring). T12 produces vectors that run-lifecycle T22 should reuse. T04 creates `languages.rs`, which language-drivers extends.

## Wave 0: gate and early facts

- [ ] **T00 [S] [GATE] Boundary review (Brighton)**
  - Read `design.md` sections 1 to 4. Tick the checklist in section 4. Answer open questions 1 to 7 and 16 (each lists the assumed choice).
  - Also click **Analyze Requirements** on `requirements.md` in the IDE (this run could not).
  - Output: a short note appended to `design.md` ("Review decisions") listing the answers. If any answer weakens or changes an isolation control, update `requirements.md` before T01.
  - Depends on: none.
  - Verify: no implementation task is started before this note exists.

- [ ] **T01 [S] (25 min) Judge dependencies, features, lint bans and configuration struct**
  - Files: `crates/judge/Cargo.toml`, `crates/judge/clippy.toml`, `crates/judge/src/sandbox/{mod,config}.rs`, `THIRD-PARTY-NOTICES.md`.
  - Add the dependencies listed in `design.md` section 5 (`bollard` only behind feature `sandbox`; `testkit` feature; both off for libraries that do not need them). Add `clippy.toml` with `disallowed-methods` for `std::process::Command::new` and `tokio::process::Command::new` (also add the same file for `crates/mcp`). Implement `SandboxConfig::from_env()` with every variable of `design.md` section 6, range checks, and the rule `orphan_age > run_cap + 15 s`. No `.env` file is read.
  - Tests: unit tests for defaults, out-of-range values, missing `SANDBOX_DOCKER_SOCKET`, orphan-age rule.
  - Requirements: R2.6, R10.4, R17.
  - Depends on: T00.
  - Verify: `cargo check -p counterspell-judge` with and without `--features sandbox`; `cargo deny check licenses`.
  - Rust lesson: Cargo features and why `bollard` hides behind one; `Result` with `thiserror` for configuration errors.

- [ ] **T02 [S] [HUMAN] [SUPERVISED] Host spike on the target VPS**
  - Files: `deploy/sandbox-verification.md` (new, results only), plus the exact commands Kiro prepared.
  - On the VPS, as the `sandbox` user, record the observed facts: `docker info` (rootless, `Cgroup Version: 2`, driver `systemd`, data root), the controllers delegated to the user (`cgroup.controllers`), whether the `Delegate=cpu cpuset io memory pids` drop-in is needed and applied, and the output of a trusted probe (`cat /sys/fs/cgroup/memory.max`, `pids.max`, `cpu.max`, `/proc/self/status`) in a container started with the intended flags. Then answer these open API questions by experiment, each with the command and output: (1) does a `tar -x` exec with stdin close/EOF work on the pinned Python image; (2) does the archive upload API work into a running read-only container's `tmpfs`; (3) does the kernel `oom_kill` counter in `memory.events` increase when a non-PID-1 process is OOM-killed, and what does `docker inspect` report for `OOMKilled`; (4) do `sleep`, `tar`, `cat`, `touch` exist in the image; (5) do the `tmpfs` options `uid=`, `gid=`, `mode=` apply in rootless mode; (6) the bollard version and the exact request/exec/attach types available (a short compile-only snippet is fine).
  - Decide and record: tar-over-exec or archive upload for file delivery. If neither works, stop and ask Brighton.
  - Depends on: T00.
  - Verify: the file lists each answer with the command, output, host and date, and marks anything not checked as NOT TESTED.

- [ ] **T03 [S] [HUMAN] (15 min) Pin the Python image digest**
  - Files: `languages/languages.toml` (digest only), `deploy/sandbox-verification.md`.
  - On the sandbox daemon, pull `python:3.13-slim` (or the tag Brighton chooses), read its digest, and record `image = "python:3.13-slim@sha256:<digest>"` in `languages/languages.toml` (T04 creates the rest of the file; if T04 has not run, record the digest in the verification file and apply it in T04). Document the weekly refresh procedure in `deploy/rootless-sandbox-daemon.md` (T38).
  - Depends on: T02.
  - Verify: `docker image inspect` on the sandbox daemon shows the same digest; no tag-only image reference remains for an enabled language.

## Wave 1: pure core (no Docker, runs anywhere)

- [ ] **T04 [S] (30 min) Language catalogue loader and `languages.toml`**
  - Files: `crates/judge/src/languages.rs`, `languages/languages.toml`, `crates/judge/tests/model_languages.rs`.
  - Implement `PinnedImage` (private field; only builds from `…@sha256:` + 64 hex), `LanguageDefinition`, `LanguageCatalog::load(path)` and validation as in `design.md` section 6. Python enabled; JavaScript, Java and C++ present with `enabled = false`. File-name validation. Parse `bench_args` but do not use it. Expose `max_source_bytes(lang)` for the server's `LanguageCatalog`.
  - Tests: property P13 (generated invalid files rejected; valid file round-trips); a tag is rejected; duplicate names; unknown fields; zero limits; path-like file names; disabled language is returned but marked disabled.
  - Requirements: R2.1, R2.5, R12.1, R12.2, R12.3.
  - Depends on: T01. (T03 supplies the real digest; tests use a fixture with a syntactically valid fake digest that is never pulled.)
  - Rust lesson: newtypes with private fields; `serde` with `deny_unknown_fields`.

- [ ] **T05 [S] [SUPERVISED] (30 min) Container policy and the hardening checker**
  - Files: `crates/judge/src/sandbox/policy.rs`, `crates/judge/tests/model_policy.rs`.
  - Implement the pure `ContainerPolicy`, `Limits`, `PolicyViolation` and `ContainerPolicy::for_language(lang, run_cap, runtime)` with every control of `design.md` section 7 (including no `nproc`, log driver `none`, PID 1 `sleep`, labels, fixed environment). Implement `hardening_violations()`.
  - Tests: P3. Mutation test: for each control, weaken only that control and assert the checker reports exactly that violation. A test that the production constructor sets `seccomp` to nothing (default profile), never `unconfined`.
  - Requirements: R3.1 to R3.14, R3.16.
  - Depends on: T04.
  - Rust lesson: plain data structs vs library types; why the checker is a separate function that is itself tested.

- [ ] **T06 [P] (25 min) Session and exec specs and the tar builder**
  - Files: `crates/judge/src/sandbox/spec.rs`, `crates/judge/tests/model_spec.rs`.
  - Implement `SessionSpec`, `ExecSpec` (fixed argv, env, user, working dir), container names and labels, and `build_tar(files)` that writes exactly three entries with validated fixed names (source, driver, `signature.json`). There is no function that takes a command string.
  - Tests: P4 (two random sources and inputs produce equal specs, equal entry names, differing content only); the tar contains only the three names; a name that is not valid cannot be passed (type-level or checked at construction).
  - Requirements: R2.3, R2.4, R3.13, R3.14.
  - Depends on: T04, T05.
  - Rust lesson: building values from typed parts instead of formatting strings.

- [ ] **T07 [P] (25 min) Bounded line reader**
  - Files: `crates/judge/src/sandbox/stream.rs`, `crates/judge/tests/model_stream.rs`.
  - Pure state machine: `LineReader::new(cap)`, `push(chunk) -> Vec<Line> | Overflow`; `StderrBuffer` with a 64 KiB cap that keeps the first bytes and reports overflow.
  - Tests: P5 (any chunking gives the same lines and overflow point; held bytes never exceed cap + one chunk; a line exactly at the cap is accepted; one byte over overflows).
  - Requirements: R6.1 to R6.4.
  - Depends on: T01.
  - Rust lesson: ownership of byte buffers and why the reader never copies more than it must.

- [ ] **T08 [P] (25 min) Driver protocol framing**
  - Files: `crates/judge/src/sandbox/driver_protocol.rs`, `crates/judge/tests/model_protocol.rs`.
  - `encode_call(t, args) -> Vec<u8>` using `serde_json` and a trailing newline. `parse_answer(line, expected_t) -> Result<Value, ProtocolViolation>`: exactly the keys `t` and `out`, `t` equal to the expected id, no blank lines, no trailing data. Named to match language-drivers' `driver_protocol.rs` so it can extend it.
  - Tests: round-trip; wrong id; extra key; missing key; not an object; blank; huge numbers; non-UTF-8; a line containing `{"verdict":"pass"}`; property that `encode_call` output never contains an unescaped newline inside.
  - Requirements: R8.4.
  - Depends on: T01.
  - Rust lesson: `serde_json::Value` and why parsing is separate from judging.

- [ ] **T09 [P] (25 min) Display sanitiser**
  - Files: `crates/judge/src/sandbox/sanitize.rs`, `crates/judge/tests/model_sanitize.rs`.
  - `sanitize_display(bytes, cap) -> Sanitized { text, truncated }` as in `design.md` section 15. Caps: 64 KiB stored, 4 KiB per event detail field.
  - Tests: P11 (valid UTF-8, within cap, no forbidden controls, idempotent, truncation flag exact); multi-byte character at the cap boundary; ANSI escape bytes; NUL; invalid UTF-8.
  - Requirements: R13.3, R13.4, R14.5.
  - Depends on: T01.
  - Rust lesson: `String::from_utf8_lossy`, char boundaries, and `&str` slicing rules.

- [ ] **T10 [S] [SUPERVISED] (30 min) Trusted comparator and the only mint of `Pass`**
  - Files: `crates/judge/src/sandbox/compare.rs`, `crates/judge/tests/model_compare.rs`.
  - `Expected`, `Verified` (private constructor), `TestVerdict`, `judge_answer(expected, call_id, line, comparator)`. Use `counterspell_engine::Comparator` (exact and unordered). Only this module can build `TestVerdict::Pass`.
  - Tests: P2 (arbitrary bytes, verdict-looking JSON, wrong ids, extra lines, a value equal to expected but with the wrong id); exact vs unordered semantics; a compile-fail test (`trybuild` if licence allows, otherwise a doc-test with `compile_fail`) showing `Pass` cannot be constructed outside the module.
  - Requirements: R8.1, R8.2, R8.4.
  - Depends on: T08.
  - Rust lesson: privacy as a security tool; `pub(super)` and private tuple fields.

- [ ] **T11 [S] [SUPERVISED] (30 min) Final verdict function**
  - Files: `crates/judge/src/sandbox/verdict.rs`, `crates/judge/tests/model_verdict.rs`.
  - `ExecutedTest`, `RunFacts { compiled, planned, cut_by_cap, stopped }`, `decide(...) -> FinalVerdict` implementing the table and precedence of `design.md` section 9. `Accepted` is built in exactly one branch.
  - Tests: P1 and P10 (generated facts; accepted soundness; determinism; first-non-pass precedence; adding a failing test never produces accepted; cap-cut run is `time_limit`; compile failure wins). Exhaustive table test of every row.
  - Requirements: R8.3, R8.6, R9.1, R9.6.
  - Depends on: T10.
  - Rust lesson: enums with data and exhaustive `match`.

- [ ] **T12 [P] (30 min) Source re-check and shared validation vectors**
  - Files: `crates/judge/src/sandbox/precheck.rs`, `crates/judge/tests/data/source_vectors.json`, `crates/judge/tests/model_precheck.rs`.
  - `precheck(bytes, max_bytes) -> Result<&str, SourceRejection>`: size in bytes (not lines), UTF-8, no NUL or control characters except tab, LF, CR. Write the vectors file (oversize by one byte, exactly at the limit, multi-byte at the boundary, invalid UTF-8, NUL, BEL, ESC, tab/LF/CR allowed, empty, BOM, long single line, many lines, a source containing `import os, subprocess, socket, ctypes`). A test asserts that no keyword list exists in the judge crate.
  - Tests: every vector; the "forbidden words are accepted" vector passes the check.
  - Requirements: R14.1, R14.3, R14.6, R14.7, R14.8.
  - Depends on: T01.
  - Verify: add a note to run-lifecycle T22 that its tests should load the same vectors (recorded in the verification report, not by editing that spec).
  - Rust lesson: byte slices vs strings; `std::str::from_utf8`.

- [ ] **T13 [S] (30 min) Public records with redaction**
  - Files: `crates/judge/src/sandbox/records.rs`, `crates/judge/tests/model_records.rs`.
  - Build `TestRecord`, `PublicTest::Hidden` / `Sample` and the final `FinalResult` shape from executed tests. Hidden tests carry verdict and time only. Sample detail (`input`, `expected`, `got`) is sanitised and bounded to 4 KiB per field; `stderr_snip` for samples only.
  - Tests: P12 (canaries in hidden inputs, expected values and fake program output never appear in any serialised record or event for a hidden test); sample fields bounded; the run-lifecycle `test_results` check (hidden rows have null snippets) holds for every produced row.
  - Requirements: R13.1, R13.2, R13.6, R8.5.
  - Depends on: T09, T11.
  - Rust lesson: making impossible states unrepresentable with enum variants.

## Wave 2: session logic against a fake Docker

- [ ] **T14 [S] (30 min) Backend traits and the fake backend**
  - Files: `crates/judge/src/sandbox/backend.rs`, `crates/judge/src/sandbox/testkit.rs`.
  - Define `Backend`, `BackendSession`, `DriverChannel`, `InfraError` (daemon unreachable, API timeout, image missing, digest mismatch, policy read-back mismatch, kill failed, exec setup failed, unexpected response) with generics, not `dyn`. Implement `FakeBackend` (feature `testkit`): ledger of created and removed containers, fault injection at the n-th call, and scripted program behaviours (answer after a delay, hang, flood, exit with code, OOM evidence) driven by Tokio's paused clock.
  - Tests: ledger counts; fault injection triggers exactly at the chosen call; scripted behaviours reproduce.
  - Requirements: R7, R9.4.
  - Depends on: T05, T06.
  - Rust lesson: traits with async methods and generics; `thiserror` error enums.

- [ ] **T15 [S] [SUPERVISED] (30 min) Session lifecycle and the cleanup guard**
  - Files: `crates/judge/src/sandbox/session.rs`.
  - Implement create → read-back check → start → load files → compile → start driver → close, with `SessionGuard` whose `Drop` spawns `kill + remove` on the runtime handle, and `close().await` that confirms removal. Compile runs the configured argv with `compile_budget_ms` and a 64 KiB output cap; a non-zero exit or a missed deadline is `CompileFailed`.
  - Tests (fake backend): ledger shows removal on success, on `?` early return, on task abort, and on panic; compile failure removes the container; a read-back mismatch removes the container.
  - Requirements: R3.15, R7.1, R7.2, R7.6, R10.1.
  - Depends on: T14.
  - Rust lesson: RAII with `Drop`, why `Drop` cannot be async, and spawning cleanup.

- [ ] **T16 [S] [SUPERVISED] (30 min) Call runner: deadline, kill, output limit**
  - Files: `crates/judge/src/sandbox/call.rs`, `crates/judge/tests/model_call.rs`.
  - Implement the algorithm in `design.md` section 9 with `tokio::select! { biased; … }`: deadline, stop signal, write+read within the bounded reader, stdout cap, stderr cap, stream end. On abnormal end read kernel facts if the container is alive (`memory.events` `oom_kill`, `pids.events`) through a fixed exec, then kill and confirm absence within `KILL_SLACK`. Produce `CallFacts` only; no comparison, no expected values.
  - Tests: P6 (virtual time: returns within limit + slack for any scripted timing; verdict is `time_limit` iff the answer is late; deadline wins ties); stdout flood; stderr flood; endless line; write that blocks because the program never reads; driver exit with and without OOM evidence; stop signal.
  - Requirements: R5.1, R5.2, R5.5, R5.9, R6.2, R6.3, R9.3.
  - Depends on: T07, T08, T14, T15.
  - Rust lesson: `select!`, cancellation safety, and why `biased` matters for fairness.

- [ ] **T17 [S] (30 min) Run orchestration**
  - Files: `crates/judge/src/sandbox/run.rs`, `crates/judge/tests/model_run.rs`.
  - Compile phase, then tests in ordinal order: per call limit, run cap, a fresh session after any abnormal end, fail-fast after `SANDBOX_MAX_CONSECUTIVE_ABNORMAL`, stop handling. Compare each answer through `compare::judge_answer`. Return `RunResult { executed, facts }` or `Err(InfraError)`. The run code never imports an expected-value type except through `compare`.
  - Tests: P7 (every call hangs; restarts only while the cap allows; fail-fast; cap cut gives `time_limit`; compile failure runs no test); restart after a timeout uses a new container; unexecuted tests are omitted.
  - Requirements: R5.3, R5.4, R5.6, R5.7, R5.8, R7.3, R9.2.
  - Depends on: T11, T16.
  - Rust lesson: state carried in a loop and early returns with `?`.

- [ ] **T18 [S] (30 min) Fault-injection properties**
  - Files: `crates/judge/tests/model_faults.rs`.
  - `proptest` over scripts, phases and fault points: P8 (ledger balanced after `execute`, cancel, abort and panic; no container shared between runs) and P9 (a fault at any Docker call gives `Err(Infrastructure)` and no verdict from the fault; partial results are never returned as a final result).
  - Requirements: R9.4, R9.5, R10.1, R10.7, R11.3.
  - Depends on: T17.
  - Rust lesson: property-based testing with scripted fakes.

## Wave 3: the real Docker backend [SUPERVISED]

- [ ] **T19 [S] [SUPERVISED] (30 min) Daemon connection, identity guard and daemon info**
  - Files: `crates/judge/src/sandbox/daemon.rs`, `crates/judge/tests/model_daemon.rs`.
  - Connect with the Unix-socket constructor and an explicit path only. Refuse: empty, relative, `tcp://`, `/var/run/docker.sock`, `/run/docker.sock`. Never read `DOCKER_HOST`. `info()` check: rootless, data root not `/var/lib/docker`, cgroup v2, driver not `none`, runtime registered, no limit-support warnings. All API calls wrapped in `SANDBOX_API_TIMEOUT`.
  - Tests: P14 (path guard cases, no environment dependence, including with `DOCKER_HOST` set); `info` parsing against recorded fixture JSON from T02 and mutated copies.
  - Requirements: R1.1, R1.2, R1.3, R4.3.
  - Depends on: T01, T02.
  - Rust lesson: validating inputs at a trust boundary and keeping the one `unsafe`-free network entry point small.

- [ ] **T20 [S] [SUPERVISED] (30 min) Bollard backend: create, read-back, image check**
  - Files: `crates/judge/src/sandbox/bollard_backend.rs` (part 1).
  - Convert `ContainerPolicy` to the bollard create request; before create, inspect the image and compare the digest (never pull); after create, `inspect` the container, convert to the pure policy type, and run `hardening_violations`. Mismatch removes the container.
  - Tests: unit tests of the conversion (fields present, no extra fields); a mutation test that corrupts each converted field and expects the read-back check to fail; `sbx_create_readback_matches` (ignored) on a real daemon.
  - Requirements: R2.2, R3.15, R1.9.
  - Depends on: T05, T14, T19.
  - Rust lesson: conversion traits (`From`/`TryFrom`) and keeping an external crate at the edge.

- [ ] **T21 [S] [SUPERVISED] (30 min) Bollard backend: start, exec, files, facts, kill, remove**
  - Files: `crates/judge/src/sandbox/bollard_backend.rs` (part 2).
  - Start; exec with attached stdin/stdout/stderr and an explicit user, working dir, fixed env and absolute-PATH command; deliver the tar stream over a `tar` exec (or the archive API, as decided in T02); run a bounded command to completion (compile, probe); read `memory.events` and `pids.events` through a fixed exec; kill; force remove; confirm absence. Map every daemon error to `InfraError`.
  - Tests: `sbx_exec_roundtrip`, `sbx_tar_delivery`, `sbx_kill_remove_confirms_absence`, `sbx_oom_counter_visible` (ignored, real daemon). Unit tests for error mapping.
  - Requirements: R5.2, R7.1, R7.2, R9.3, R9.4, R10.1.
  - Depends on: T20.
  - Rust lesson: async streams, `AsyncWrite` shutdown (half-close) and why EOF must be explicit.

- [ ] **T22 [S] [SUPERVISED] (30 min) Probe, preflight state and the `preflight` command**
  - Files: `crates/judge/src/sandbox/preflight.rs`, `crates/judge/src/main.rs` (subcommand only).
  - Parse `/proc/self/status`, `net/dev`, cgroup limit files and `/proc/mounts` into a `KernelReport`; compare with the policy; produce the JSON report. `PreflightState` keeps the latest result, refreshed at start-up and every 60 s. `counterspell-judge preflight` prints the report and exits non-zero on failure. There is no skip flag.
  - Tests: parsing against recorded outputs from T02 and mutated copies (each missing control is reported); `sbx_preflight_passes` and `sbx_preflight_fails_when_policy_weakened` (the weakened policy constructor is `testkit`-only).
  - Requirements: R4.1 to R4.6.
  - Depends on: T21.
  - Rust lesson: parsing text safely and returning structured reports instead of booleans.

- [ ] **T23 [P] [SUPERVISED] (25 min) Sweeper**
  - Files: `crates/judge/src/sandbox/reaper.rs`.
  - `sweep_once(backend, cfg)` lists containers with label `counterspell.managed=true` in any state and force-removes those older than `SANDBOX_ORPHAN_AGE`. `run_sweeper(...)` runs at start-up and every `SANDBOX_REAPER_INTERVAL` until shutdown. It never touches containers without the label.
  - Tests: fake-backend tests (old labelled removed, young kept, unlabelled untouched, errors do not stop the loop); `sbx_sweeper_removes_orphan` (ignored).
  - Requirements: R10.3, R10.4.
  - Depends on: T14, T20.
  - Rust lesson: periodic tasks with `tokio::time::interval` and shutdown signals.

- [ ] **T24 [S] [SUPERVISED] (30 min) `ProblemTests`, `SandboxExecutor` and judge wiring**
  - Files: `crates/judge/src/sandbox/{problem,sandbox_executor,mod}.rs`, `crates/judge/src/main.rs`.
  - Implement `ProblemTests`, `EmptyProblemTests`, and `SandboxExecutor` per `design.md` section 14: language lookup, problem load, preflight gate, container permit, `compiling` first, source re-check, run, per-test events, `decide`, cleanup on every path. Replace run-lifecycle's placeholder `NoExecutor` in `main.rs` with `SandboxExecutor` + `EmptyProblemTests`. `testkit`: `StaticProblemTests` and `DirProblemTests` reading `tests/fixtures/problems/`.
  - Tests (fake backend): `compiling` is always the first event; a returned `Completed` always follows it; stop signal ends in `Stopped` with cleanup; infra error returns `Infrastructure`; disabled language and unknown problem return the right `Failed` reasons; production wiring returns `problem_unavailable`.
  - Requirements: R15.1 to R15.3, R9.5, R12.3.
  - Depends on: T13, T17, T22, T23 and run-lifecycle T16, T19.
  - Rust lesson: implementing a trait for a generic struct and passing `Arc` to tasks.

## Wave 4: hostile corpus and real-sandbox scenarios (run on the VPS)

- [ ] **T25 [S] [SUPERVISED] (30 min) Sandbox test harness, fixtures and corpus layout**
  - Files: `crates/judge/tests/sbx/mod.rs`, `crates/judge/tests/fixtures/problems/*`, `crates/judge/tests/hostile/README.md`.
  - Harness: reads `SANDBOX_DOCKER_SOCKET`, runs the daemon-identity checks and refuses otherwise; helpers to run a source against a fixture problem and return verdicts, events, timings; helper to list containers by label from the daemon; a host-side TCP/UDP listener helper for S12. Fixture problems (tiny, original): `add` (sum of two integers) and `sort_list`, each with samples, hidden tests, a correct and a wrong Python solution. A test (`hostile_files_are_data`) asserts no test source calls process-spawning APIs on corpus files.
  - Requirements: R16.2, R16.3.
  - Depends on: T24.
  - Rust lesson: integration test helpers and `#[ignore]` with environment guards.

- [ ] **T26 [P] [SUPERVISED] (30 min) Functional and forgery scenarios: S01 to S04, S14, S15, S27**
  - Files: `crates/judge/tests/sbx/{functional,forgery}.rs`, `crates/judge/tests/hostile/{fake_verdict,forged_lines,early_exit,self_kill,stdout_close,import_all}.py`.
  - Correct solution accepted with `compiling` first; wrong solution fails with sample detail and redacted hidden tests; syntax error and disabled language; exception class; fake verdict JSON printed with `print` and with raw `os.write`; forged and early answer lines; premature exit; harmless program that imports `os`, `subprocess`, `socket`, `ctypes` is accepted (S27).
  - Requirements: R8.4, R14.7, R15.1, R16.3.
  - Depends on: T25.

- [ ] **T27 [P] [SUPERVISED] (30 min) Time, memory and process scenarios: S05 to S08**
  - Files: `crates/judge/tests/sbx/limits.rs`, `crates/judge/tests/hostile/{infinite_loop,import_loop,sleep_forever,sigstop_self,memory_list,memory_alloc,fork_bomb,thread_bomb}.py`.
  - Assert verdict sets and the wall bound (limit + 1 s on the worker clock), container removal, `oom_kill` evidence for memory cases (read through the facts path and cross-checked with the daemon), and that the PID limit contains the bombs.
  - Requirements: R5, R9.3, R3.9, R3.10.
  - Depends on: T25.

- [ ] **T28 [P] [SUPERVISED] (25 min) Output scenarios: S09 to S11**
  - Files: `crates/judge/tests/sbx/output.rs`, `crates/judge/tests/hostile/{stdout_flood,stdout_flood_raw,stderr_flood,long_line}.py`.
  - `output_limit` for each; the worker's resident memory stays bounded during the flood (sample before and after); verdict not accepted.
  - Requirements: R6.
  - Depends on: T25.

- [ ] **T29 [P] [SUPERVISED] (30 min) Network scenario: S12**
  - Files: `crates/judge/tests/sbx/network.rs`, `crates/judge/tests/hostile/{net_tcp,net_udp,net_dns,net_ipv6,net_raw,net_metadata}.py`, a trusted control script `crates/judge/tests/probes/net_control.py`.
  - Attempt TCP, UDP, DNS, IPv6, raw sockets, link-local and metadata addresses, the bridge gateway and the host's loopback. The pass condition is zero connections at the host listeners. The control run (our own trusted script, networking deliberately enabled through a `testkit`-only policy) shows whether the listener can see a connection on this host; if it cannot, the test reports "control inconclusive" rather than passing silently.
  - Requirements: R3.1, R16.4.
  - Depends on: T25.

- [ ] **T30 [P] [SUPERVISED] (30 min) Filesystem and host-visibility scenarios: S13, S26**
  - Files: `crates/judge/tests/sbx/filesystem.rs`, `crates/judge/tests/hostile/{write_root,write_proc,write_sys,fill_box,big_file,read_host,syscalls_blocked}.py`.
  - After each run, ask the daemon for the container's filesystem changes: it must be empty. Fill tests stop at the tmpfs size and do not produce `accepted`. Read attempts for `/var/run/docker.sock`, `/proc/1/environ`, secrets paths find nothing useful. `mount`, `unshare`, `ptrace` fail. Mount list contains only the expected mounts.
  - Requirements: R3.2, R3.3, R3.5, R3.6, R1.9, R16.4.
  - Depends on: T25.

- [ ] **T31 [S] [SUPERVISED] (30 min) Cleanup and crash scenarios: S16, S17, S24**
  - Files: `crates/judge/tests/sbx/recovery.rs`, `crates/judge/examples/sbx_victim.rs` (a trusted helper binary that starts a hostile run and waits).
  - S16: spawn the trusted helper as a child process, SIGKILL it mid-run (`Child::kill`), then poll the daemon until no labelled container remains; assert gone within 60 s of the container's creation. Repeat with a test-only policy whose PID 1 outlives the orphan age so only the sweeper can remove it. S17: after timeout and cancel, the daemon lists no container for the run. S24: stop signal during a call kills and removes.
  - Requirements: R10, R5.9.
  - Depends on: T25, T23.

- [ ] **T32 [S] [SUPERVISED] (30 min) Concurrency scenarios: S18, S19**
  - Files: `crates/judge/tests/sbx/concurrency.rs`.
  - Two runs at once with `JUDGE_CONCURRENCY=2`: a fork bomb in one and a correct solution in the other; the good run is accepted within its limits. Submit more runs than slots and sample the daemon's container list: never more than `SANDBOX_MAX_CONTAINERS`. Record timing for the report.
  - Requirements: R11.
  - Depends on: T27, T31.

- [ ] **T33 [P] [SUPERVISED] (30 min) Infrastructure and runtime scenarios: S20 to S23, S25**
  - Files: `crates/judge/tests/sbx/infra.rs`.
  - S20 (probe passes; probe fails for a weakened policy), S21 (wrong socket, stopped daemon: `Infrastructure`, never a verdict), S22 (unknown digest: `Infrastructure`, no pull attempted), S23 (only when `SANDBOX_RUNTIME=runsc`: record TESTED or NOT TESTED), S25 (write the probe report to a file for the verification report).
  - Requirements: R4, R9.4, R12.4, R12.5.
  - Depends on: T25, T22.

- [ ] **T34 [S] [SUPERVISED] (30 min) `SandboxOracle`**
  - Files: `crates/judge/src/sandbox/oracle.rs`, `crates/judge/tests/sbx/oracle.rs`, `crates/judge/tests/model_oracle.rs`.
  - Implement `counterspell_engine::Oracle` with a Tokio `Handle`, a user session and a reference session in **separate containers**. Map outcomes to `Returned`, `Garbled`, `Crashed { exception_class, message }`, `TimedOut`, or `OracleError`. Reject being called from inside an async context.
  - Tests: fake-backend mapping tests; `sbx_oracle_hostile_user` (hostile corpus as the user side returns only `TimedOut`, `Crashed` or `Garbled`, within limit + slack); `sbx_oracle_good_pair`; hidden inputs cannot be passed (type-level: only a public input type is accepted).
  - Requirements: R15.4 to R15.6.
  - Depends on: T17, T24, T25.
  - Rust lesson: bridging sync and async with `Handle::block_on` and why it needs `spawn_blocking`.

- [ ] **T35 [P] (20 min) Log hygiene test**
  - Files: `crates/judge/tests/model_logs.rs`, `crates/judge/tests/sbx/logs.rs`.
  - Capture `tracing` output while the fake backend runs a program that prints a canary in stdout, stderr and the exception message; assert the canary never appears in logs. Repeat once against the real sandbox.
  - Requirements: R13.5.
  - Depends on: T24.

## Wave 5: boundary, documentation, evidence, review

- [ ] **T36 [P] (30 min) Boundary tests**
  - Files: `crates/judge/tests/boundary.rs`.
  - (1) Parse `Cargo.lock` and assert `counterspell-server`'s transitive dependencies contain neither `bollard` nor `counterspell-judge`. (2) Assert `bollard` is imported only in `sandbox/bollard_backend.rs` and `sandbox/daemon.rs`. (3) Assert no `Command::new` or `std::process` / `tokio::process` use in `crates/judge/src` and `crates/mcp/src`. (4) Assert the container-facing modules (`call.rs`, `session.rs`, `spec.rs`) do not import the expected-output type. (5) Parse `docker-compose.yml` (YAML dev-dependency) and assert: `server` has no Docker socket mount; `judge` mounts at most the sandbox socket path and never `/var/run/docker.sock`; `judge` has no published ports and no Valkey URL. (6) Assert `counterspell-mcp` depends on the judge library only through the sandbox types.
  - Requirements: R1.4 to R1.8, R2.6, R15.7, R7.
  - Depends on: T24.

- [ ] **T37 [S] [HUMAN] (30 min) Run everything on the VPS and write the verification report**
  - Files: `.kiro/specs/judge-sandbox/verification-report.md`, `deploy/sandbox-verification.md` (append).
  - On the VPS: `counterspell-judge preflight`, then `cargo test -p counterspell-judge --features sandbox -- --ignored sbx_`. Record per guarantee one of `TESTED on VPS <date>`, `TESTED in CI`, `MODEL ONLY`, `NOT TESTED`, with evidence. Measure and report separately: container create+start, file delivery, compile, per-call overhead, Run end-to-end p50/p95 over 50 runs, hostile wall times against limits, kill latency, peak live containers. State plainly what could not be verified (for example gVisor, or S12's control if inconclusive).
  - Requirements: R16.5, R16.6.
  - Depends on: T26 to T35.
  - Verify: the report's definition-of-done table is filled with measured results only.

- [ ] **T38 [P] (30 min) Deployment notes and CI**
  - Files: `deploy/rootless-sandbox-daemon.md`, `.github/workflows/ci.yml`, `docker-compose.yml` (judge environment only).
  - Notes: the `Delegate=cpu cpuset io memory pids` drop-in and the check from T02; socket ownership and group so the unprivileged judge container can use it (and no other container can); aggregate CPU, memory and task limits on the `sandbox` user's systemd slice; the image pull-by-digest procedure and weekly refresh; `preflight` on every deploy; the `sandbox` user's home holds nothing sensitive and application secrets are mode 0600 for another user. CI: add the model tests (already run by `cargo test --workspace`) and a non-blocking `sandbox-integration` job (open question 14). Compose: add `SANDBOX_DOCKER_SOCKET` and the socket mount to the judge only.
  - Requirements: R1.7, R4, R16.
  - Depends on: T02, T36.

- [ ] **T39 [S] [SUPERVISED] (30 min) Security review pass**
  - Files: findings in `verification-report.md`; fixes in the relevant source files.
  - A reviewer other than the author (a separate Kiro session with a reviewer prompt, or the planned sandbox-auditor sub-agent if it exists) walks `design.md` section 4's checklist against the code: the Docker request builder, the only places `Pass` and `Accepted` are built, the import boundaries, every `unwrap`/`expect` (none in production code), and the hostile corpus for programs that could return a correct answer. Fix findings, rerun T36 and the model tests.
  - Depends on: T37.

## Dependency summary and parallel waves

- **Wave 0:** T00 first. T01 and T02 after it (T02 is human work on the VPS, in parallel with T01). T03 after T02.
- **Wave 1:** after T01 and T04: T05, then T06 in parallel with T07, T08, T09, T12. T10 after T08, T11 after T10, T13 after T09 and T11.
- **Wave 2:** T14 after T05 and T06; T15 after T14; T16 after T07, T08, T15; T17 after T11 and T16; T18 after T17.
- **Wave 3:** T19 after T01 and T02; T20 after T05, T14, T19; T21 after T20; T22 after T21; T23 after T14 and T20 (parallel with T21 and T22); T24 after T13, T17, T22, T23 and run-lifecycle T16/T19.
- **Wave 4:** T25 after T24; T26 to T30 in parallel after T25; T31 after T25 and T23; T32 after T27 and T31; T33 after T25 and T22; T34 after T17, T24, T25; T35 after T24.
- **Wave 5:** T36 after T24 (parallel with Wave 4); T37 after T26 to T35; T38 after T02 and T36; T39 last.

## Completion checklist

Nothing here may be ticked from reasoning alone. Each item is ticked from the evidence in `verification-report.md`.

- [ ] T00 signed off, open questions answered.
- [ ] A correct Python solution is accepted and a wrong one fails (S01, S02).
- [ ] Every hostile scenario (S05 to S15, S26) is contained within its documented limits, measured on the VPS.
- [ ] Wall time per call is at most the limit plus 1 s; the run is decided within the run cap plus 1 s (P6, P7, S05, S17).
- [ ] No program output can forge `accepted` (P1, P2, S14).
- [ ] No labelled container remains after worker recovery, including with the sweeper alone (S16, S17).
- [ ] No network connection reaches any host listener (S12), and the control result is stated.
- [ ] Preflight passes on the VPS and fails for a weakened policy (S20).
- [ ] Infrastructure failures are never reported as a user's wrong answer (P9, S21, S22).
- [ ] Concurrent runs do not harm each other (S18) and the container cap holds (S19).
- [ ] `counterspell-server` has no Docker dependency and `bollard` is confined to two files (T36).
- [ ] No host process spawning in `judge` or `mcp` production code (T36).
- [ ] Validation vectors exist and the judge re-check passes them (T12).
- [ ] gVisor is reported as untested unless S23 passed on the VPS.
- [ ] `cargo fmt`, `cargo clippy -D warnings`, `cargo test --workspace` and `cargo deny check licenses` pass.
- [ ] `verification-report.md` states what was tested, what was model-only and what was not tested.
