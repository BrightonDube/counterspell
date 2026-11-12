# Requirements: judge-sandbox

Status: **draft for Brighton's review of the trusted/untrusted boundary. Nothing in this spec is implemented, and no task may start until task T00 is signed off.**

## Purpose

Give `counterspell-judge` a Python-only execution service that runs hostile submitted code and produces verdicts that user code cannot forge. The trusted judge worker owns the tests, the expected outputs, the comparator, the deadlines and the final verdict. The submitted program only reads one input at a time and prints one raw answer. Everything runs on a separate rootless Docker daemon that is used for nothing else.

This spec also records the pre-sandbox submission validation contract (VAL-1 to VAL-5). The API server enforces it before a run is queued; this spec adds the judge-side re-check and the shared test vectors. The sandbox, not any validation rule, is the security boundary.

## How this document was prepared (Analyze Requirements pass)

Kiro CLI runs this spec non-interactively, so the "Analyze Requirements" button in the IDE could not be clicked. The analysis was done by hand instead: every sentence of the brief and of `docs/system-design.md` §2, §5, §6, §7, §7.1 and `docs/requirements.md` (Sandbox criteria, VAL-1 to VAL-5) was cross-checked against the existing specs `run-lifecycle`, `language-drivers`, `ghost-engine`, `counterspell-mcp-power` and `repo-skeleton`. **Please also click Analyze Requirements on this file in the IDE before approving.**

Findings, with the resolution this spec adopts. Items marked ⚠ touch isolation and are repeated in `design.md` under "Open questions" as questions to answer **before** implementation.

| # | Finding | Resolution adopted |
|---|---|---|
| A1 | The brief says wall time must be "≤ limit + 1 s". The system design says the run is killed at `tests × time_ms + compile + 2 s`, hard cap 20 s. Both are needed. | Two bounds. Call level: a test never takes longer than its limit plus `KILL_SLACK` (1 s). Run level: the verdict is decided within the run cap plus `KILL_SLACK`. See R5. |
| A2 | ⚠ The system design lists `nproc` among the ulimits. Docker's rootless documentation says `nproc` counts per namespaced UID. All concurrent containers use the same UID, so one fork bomb could starve the other slot's fork calls. | The PID cgroup limit is the control. `nproc` is **not** set. A concurrency test proves a fork bomb in one slot does not hurt the other slot (R11). Alternative offered: one UID per slot. |
| A3 | Both `docs/system-design.md` §6 and the `language-drivers` spec give Python `compile = []`. A syntax error would then surface inside the driver, and the program could pretend to be a "compile error". | Python gets a worker-run compile step (`py_compile`), executed by the worker with a fixed command. The verdict then comes from an exit code the worker observed, never from anything the program printed. ⚠ Changes `languages.toml` relative to the other specs. |
| A4 | The brief requires digest-pinned images. `language-drivers` says tags are "local/development defaults". | Strict: the loader rejects a tag for any enabled language and there is no override. ⚠ |
| A5 | §7 says `/box` is `exec` only "where compiled binaries live". Python has none. | Python's scratch space is `noexec,nosuid,nodev`. A per-language flag turns exec on for compiled languages later. |
| A6 | §7 says "reads at most 1 MB of stdout and 64 KB of stderr". It does not say per what. | Stdout: 1 MiB per call. Stderr: 64 KiB per container session. Either breach kills the container with `output_limit`. |
| A7 | "Reaper within 60 s" depends on the sweep interval, not only on the age threshold. | Containers expire by themselves (PID 1 is `sleep <cap + 5 s>`), and the sweeper removes anything older than 40 s every 10 s, so no container outlives about 50 s from creation. Stated and tested as such (R10). |
| A8 | "Never Accepted for the hostile corpus" is wrong for a program that probes the network, fails, and then returns the right answer. That program deserves Accepted. | Every hostile corpus program is written so that it **cannot** produce a correct answer. Containment is checked out of band (host listener, daemon-reported filesystem changes, kernel counters), never from what the program reports. |
| A9 | Run events and `test_results` (run-lifecycle) have no "not run" per-test verdict. | Tests that were never executed are omitted from events and `test_results`. The final verdict and the `passed`/`total` counts explain why. Open question 10. |
| A10 | `ghost-engine` defines `Oracle::evaluate` as a **synchronous** trait that runs inside `spawn_blocking`. | `SandboxOracle` bridges to async with a Tokio runtime handle (R15). |
| A11 | run-lifecycle already builds server-side VAL-1 to VAL-4 (`validate.rs`, `ratelimit.rs`). Writing them again here would create two implementations. | This spec states the contract, adds a judge-side re-check, and ships shared test vectors. The server code stays in run-lifecycle. Open question 11. |
| A12 | Docker's rootless documentation says the cgroup flags are **silently ignored** when cgroup v2 and the systemd driver are missing. Only `memory` and `pids` are delegated to users by default, so `--cpus` needs extra systemd configuration. | Preflight must check observed behaviour (kernel files inside a probe container), not configuration. The judge fails closed when preflight fails (R4). |
| A13 | `docker cp` into a read-only container may be refused, and files copied before start would be hidden by the `tmpfs` mount. | Files go in over the stdin of a fixed `tar` command. An early spike confirms this on the target host and records the fallback (T02). |
| A14 | `OOMKilled` from the Docker API is unreliable for a process that is not PID 1. | The worker reads the kernel's own counter (`memory.events`) from inside the container after an abnormal end, and also records the Docker flag (R9). |
| A15 | The brief asks to verify the limits on the target host. The development machine is WSL2 and will not have a delegated rootless daemon. | All logic is tested without Docker (model properties). Real-sandbox tests are marked `sbx_` and need `SANDBOX_DOCKER_SOCKET`. The authoritative evidence comes from the VPS (R16). |

## Scope

### Built here

- Language catalogue loader for `languages/languages.toml` (Python enabled; JavaScript, Java and C++ present but disabled).
- Container policy, session lifecycle, call runner, run orchestration, trusted comparator, verdict function, output sanitiser, reaper, preflight and probe.
- `SandboxExecutor` (implements the `RunExecutor` trait from run-lifecycle) and `SandboxOracle` (implements the `Oracle` trait from ghost-engine).
- The `ProblemTests` seam that supplies tests, expected outputs and limits to the executor.
- Judge-side source re-check, the shared validation test vectors, and the output contract for VAL-5.
- The hostile-program corpus and the real-sandbox tests.
- Deployment notes for the sandbox daemon (delegation, socket permissions, aggregate limits).

### Not built here

- JavaScript, Java and C++ execution, and bench mode (`language-drivers`, `efficiency-familiar`).
- Queue, leases, SSE and the terminal write (`run-lifecycle`). This spec plugs in behind its `RunExecutor` trait.
- Ghost-hunt orchestration, shrink events and the `ghosts` table (`ghost-engine` and `run-lifecycle`). This spec only supplies the Oracle.
- Server-side validation code and rate limits (`run-lifecycle`, T22 and T23).
- The catalogue tables that hold problems and tests (a later server spec).
- gVisor as a supported option. It stays selectable by configuration and is **not claimed** (R12).

## Glossary

- **Judge**: the trusted worker, package `counterspell-judge`.
- **Sandbox daemon**: the separate rootless Docker daemon, run by the unprivileged `sandbox` user, used only for sandboxes.
- **Session**: one container and everything that happens in it. A run may use several sessions (a fresh one after every abnormal end).
- **Driver**: the per-language program that loads the submitted function and answers one call at a time. It runs inside the container, so the judge treats it as untrusted.
- **Call**: one input line sent to the driver and one answer line expected back.
- **Facts**: things the worker observed itself (exit codes, elapsed time, byte counts, kernel counters). Verdicts are computed from facts and the expected outputs only.
- **Hostile corpus**: the set of programs in `crates/judge/tests/hostile/` that attack the sandbox.
- **Out-of-band evidence**: evidence the hostile program cannot influence, such as a listener on the host or the daemon's own report of filesystem changes.

## Requirements (EARS)

Each criterion names the property (`P…`, model-based, no Docker) or scenario (`S…`, real sandbox) that verifies it. The properties and scenarios are defined in `design.md`.

### R1. Daemon separation and trust boundary

1. THE judge SHALL connect only to a Unix socket whose path is given in `SANDBOX_DOCKER_SOCKET`. There SHALL be no default path. THE judge SHALL NOT read `DOCKER_HOST`, a Docker context, or any default socket location. [P14, S20]
2. IF the configured path is `/var/run/docker.sock` or `/run/docker.sock`, or the address is not a Unix socket, THEN THE judge SHALL refuse to start. [P14]
3. WHEN the judge starts, THE judge SHALL check that the daemon is rootless, that its data root is not `/var/lib/docker`, and that cgroup v2 with a non-`none` cgroup driver is in use. IF any check fails, THEN THE judge SHALL NOT run user code (R4). [S20]
4. THE `bollard` crate SHALL be imported only in `sandbox/bollard_backend.rs` and `sandbox/daemon.rs`. [boundary test T36]
5. THE `counterspell-server` package SHALL NOT depend, directly or transitively, on `bollard` or on `counterspell-judge`. [boundary test T36]
6. THE `counterspell-mcp` package SHALL reach Docker only through the judge library, with the same configuration, and SHALL NOT fall back to local execution when the sandbox is unavailable. [boundary test T36]
7. THE judge deployment SHALL mount only the sandbox socket and SHALL NOT mount the main daemon socket. [compose check T36]
8. THE judge SHALL NOT receive a Valkey URL, OAuth, email or LLM secrets. It holds only the limited Postgres role defined by run-lifecycle. [compose check T36]
9. THE sandbox container SHALL receive no bind mounts, named volumes, devices, host namespaces, Docker socket, secrets, or environment variable beyond the fixed list in `design.md`. [P3, P14, S20]

### R2. Image and command integrity

1. THE loader SHALL accept an enabled language only when its image reference ends in `@sha256:` followed by 64 hexadecimal characters. A tag SHALL be rejected. There SHALL be no override flag. [P13]
2. BEFORE creating a container, THE judge SHALL inspect the image on the sandbox daemon and compare its digest. IF the image is missing or the digest differs, THEN THE judge SHALL report an infrastructure error. THE judge SHALL NOT pull images at run time. [S22]
3. THE judge SHALL build every Docker request from typed values. The command arrays, environment, labels, names and paths of those requests SHALL be identical for any two submitted sources or inputs. [P4]
4. THE judge SHALL NOT build a command by joining strings, SHALL NOT use `sh -c` for any command it constructs, and SHALL pass submitted source and test inputs only as data on stdin. [P4]
5. THE loader SHALL reject a configured file name that is empty, longer than 64 bytes, contains a character outside `[A-Za-z0-9_.-]`, or equals `.` or `..`. [P13]
6. THE judge and `counterspell-mcp` production code SHALL NOT spawn host processes. THE workspace SHALL enforce this with Clippy's `disallowed-methods` and a source test. [T36]

### R3. Container isolation policy

THE judge SHALL create every sandbox container with all of the following. A single function builds the policy; no other code path creates a container.

1. No network: network mode `none` and networking disabled. [S12, S20]
2. Read-only root filesystem. [S13, S20]
3. Scratch space only on `tmpfs`: `/box` (64 MiB, `nosuid,nodev`, and `noexec` unless the language needs compiled binaries) and `/tmp` (16 MiB, `noexec,nosuid,nodev`). A 16 MiB `/dev/shm`. [S13, S20]
4. A non-root user and group that are the same for the container, the driver and every exec the worker starts (65534:65534 unless open question 2 changes it). [S20]
5. All capabilities dropped and `no-new-privileges` set. [S20]
6. Docker's default seccomp profile: no `seccomp=unconfined`, no custom profile. [P3, S20]
7. Privileged mode off, no added capabilities, no devices, private PID, IPC and cgroup namespaces. [P3]
8. A CPU quota of one CPU (`NanoCpus = 1_000_000_000`). It is a quota, not pinning, and the code and documentation SHALL say so. [S20]
9. Memory limit equal to `mem_mb` and memory-plus-swap equal to the same value (no swap). [S07, S20]
10. A PID limit of 64. [S08, S20]
11. File-descriptor limit 64, maximum file size 16 MiB, core dumps disabled. `nproc` SHALL NOT be set (finding A2). [P3]
12. Log driver `none`. [P3]
13. A fixed environment: `PATH=/usr/local/bin:/usr/bin:/bin`, `LANG=C.UTF-8`, `HOME=/box`. [P4]
14. Labels `counterspell.managed`, `counterspell.run`, `counterspell.worker`, `counterspell.expires`. [P4, S16]
15. AFTER creating a container, THE judge SHALL read its configuration back from the daemon and compare every item above. IF any item differs, THEN THE judge SHALL remove the container and report an infrastructure error. [P14]
16. THE policy checker SHALL reject a policy in which any one control is missing or weakened. A mutation test SHALL prove that each control is checked. [P3]

### R4. Host verification (preflight and probe)

1. WHEN the judge starts, AND every 60 s afterwards, AND on `counterspell-judge preflight`, THE judge SHALL run a probe container built by the same policy function and read facts from the kernel inside it. [S20]
2. THE probe SHALL check at least: `NoNewPrivs: 1`; `Seccomp: 2`; all capability sets zero; non-root UID; only the loopback interface in `/proc/self/net/dev`; `memory.max`, `memory.swap.max`, `pids.max` and `cpu.max` equal to the policy values; root filesystem mounted read-only; the scratch mounts have the expected options and sizes; no other mounts. [S20]
3. THE probe SHALL also check that the daemon reports rootless, cgroup v2 and no "limit support" warnings, because Docker silently ignores cgroup flags when delegation is missing. [S20]
4. IF any probe check fails or the probe cannot run, THEN THE judge SHALL treat every run as an infrastructure error until a later probe passes. THERE SHALL be no switch to skip or downgrade the probe. [S20, S21]
5. THE preflight SHALL print a JSON report listing each check with the expected value, the observed value and the result, and SHALL redact nothing needed to audit the host. [S25]
6. A negative test SHALL prove that the probe fails against a deliberately weakened policy (no memory limit, network enabled, writable root). [S20]

### R5. Time enforcement from outside

1. THE worker SHALL measure every call with its own monotonic clock, starting before the input is written and ending when the complete answer line has been received. THE driver SHALL NOT report or influence time. [P6]
2. IF the complete answer has not been received when the per-call limit is reached, THEN THE worker SHALL kill the container from outside and record `time_limit` for that test. [P6, S05, S06]
3. THE per-call limit SHALL be the problem's `limits.time_ms` when present, otherwise the language's `time_ms_per_test`, clamped to the range 100 ms to the language's `max_time_ms_per_test`. [P6]
4. THE worker SHALL decide the run cap as `min(RUN_WALL_CAP, tests × per-call limit + compile budget + 2 s)`, where `RUN_WALL_CAP` is 20 s. [P7]
5. THE time between the instant a call times out and the instant the worker has killed the container and recorded the verdict SHALL be at most `KILL_SLACK` (1 s). [P6, S05, S17]
6. THE verdict for the whole run SHALL be decided no later than the run cap plus `KILL_SLACK`. [P7, S05, S17]
7. WHEN a call ends in a timeout, THE worker SHALL discard the session and use a fresh container for the remaining tests, while the run cap allows it. [P7, S17]
8. WHEN `SANDBOX_MAX_CONSECUTIVE_ABNORMAL` (default 3) tests in a row end abnormally (timeout, crash before answering, memory limit, output limit), THE worker SHALL stop running the remaining tests. [P7]
9. WHEN the stop signal from run-lifecycle fires, THE worker SHALL kill the session at once and return `Stopped`. [S24]

### R6. Bounded output

1. THE worker SHALL read the answer stream through a reader that never holds more than 1 MiB plus one chunk of stdout for a call. [P5]
2. IF a call's stdout exceeds 1 MiB before a newline or in total for that call, THEN THE worker SHALL kill the container and record `output_limit`. [P5, S09, S11]
3. IF a session's stderr exceeds 64 KiB, THEN THE worker SHALL kill the container and record `output_limit`. [P5, S10]
4. THE results SHALL be the same for any way the byte stream is split into chunks. [P5]
5. THE worker's memory use SHALL stay bounded while a program floods output. [S09]

### R7. Separate responsibilities

THE implementation SHALL keep these as separate modules with no shared mutable state, and SHALL document what each one may know.

1. **Prepare** (`session`): creates and starts the container, delivers the files. It knows nothing about tests or verdicts.
2. **Compile** (`session` + language config): runs the fixed compile command and reports exit code and a bounded message. It knows nothing about tests.
3. **Execute** (`call`, `run`): feeds one input at a time and returns facts. It never sees an expected output.
4. **Compare** (`compare`): the only module that sees an expected output and the only module that can create a passing test verdict.
5. **Verdict** (`verdict`): turns per-test results and run facts into the final verdict.
6. **Cleanup** (`session` guard, `reaper`): removes containers. It makes no decision about verdicts.

### R8. Verdict authority

1. THE final verdict, every per-test verdict, every time and every limit decision SHALL come from facts the worker observed and from expected outputs the worker holds. [P1, P2]
2. A per-test `pass` SHALL be creatable only inside `compare`, and only when the parsed answer carries the expected call id and the comparator says its value equals the expected value. [P2]
3. THE final verdict `accepted` SHALL be creatable only when the compile step succeeded, every test of the run was executed, every test passed, no infrastructure error occurred and no stop was requested. [P1]
4. A line that looks like a verdict (for example `{"verdict":"pass"}`), an answer to another call id, a duplicate or unsolicited line, or any other text SHALL be treated as an answer to compare, or as a protocol violation. It SHALL NOT change a verdict in any other way. [P2, S14]
5. THE expected outputs, hidden inputs of tests not yet sent, the comparator and the limits SHALL NOT be written into the container, its environment, its arguments or its labels. [P4, P12]
6. THE driver's exit code, stderr text and exception class SHALL be used only to choose among the non-passing verdicts. [P1]

### R9. Verdict mapping and infrastructure separation

1. THE mapping from facts to per-test verdicts SHALL be the table in `design.md` ("Verdict decision table"), and the same facts SHALL always give the same verdict. [P10]
2. IF the compile command exits non-zero or exceeds its budget, THEN THE run SHALL end with `compile_error` and no test SHALL run. [S03]
3. IF the driver ends abnormally and the kernel's `oom_kill` counter in `memory.events` is greater than zero, THEN THE test SHALL be `memory_limit`. A kill signal without that evidence SHALL be `runtime_error`. [S07]
4. IF any Docker call fails, times out, or returns something unexpected, or the image is missing, or the probe fails, or the policy read-back differs, THEN THE run SHALL end with an infrastructure error (`ExecError::Infrastructure`) and SHALL NOT produce any test verdict from that failure. [P9, S21, S22]
5. WHEN a run ends with an infrastructure error after some tests have run, THE executor SHALL return the error and the partial results SHALL NOT be written as a final result. run-lifecycle retries the whole run. [P9]
6. THE final verdict SHALL be: `compile_error` if compilation failed; otherwise the verdict of the first non-passing test in ordinal order; otherwise `time_limit` if the run cap cut the run short; otherwise `accepted`. [P1, P10]

### R10. Cleanup and crash recovery

1. EVERY container created by the judge SHALL be removed before `execute` returns, within `SANDBOX_CLEANUP_BUDGET` (5 s), on success, failure, timeout, stop, cancel and panic. [P8, S17]
2. EVERY container's main process SHALL be `sleep <seconds>` with a duration equal to the remaining run cap plus 5 s, so the container's processes end by themselves if the worker dies. [S16]
3. THE judge SHALL run a sweeper at start-up and every `SANDBOX_REAPER_INTERVAL` (10 s) that force-removes every container with label `counterspell.managed=true` older than `SANDBOX_ORPHAN_AGE` (40 s), in any state. [S16]
4. THE configuration check SHALL reject values where the orphan age is not greater than the run cap plus 15 s, so the sweeper never removes a live run's container. [P13]
5. IF the judge process is killed during a run, THEN no labelled container SHALL remain 60 s after that container was created. [S16]
6. THE sweeper SHALL be shown to work with the self-expiry path disabled, so the sweeper is tested on its own. [S16]
7. THE judge SHALL NOT reuse a container for a different run, a different user or after an abnormal end. [P8, S17]

### R11. Concurrency and isolation between runs

1. THE judge SHALL honour `JUDGE_CONCURRENCY` (default 2) from run-lifecycle and SHALL additionally never exceed `SANDBOX_MAX_CONTAINERS` (default `2 × JUDGE_CONCURRENCY`) containers at once, counting ghost-hunt reference containers. [S19]
2. WHILE a hostile program exhausts processes, memory or CPU in one container, a correct solution running in the other slot SHALL still be judged `accepted` within its limits. [S18]
3. THE judge SHALL create a fresh container for each run and for each restart, with no shared writable volume. [P8]

### R12. Language configuration and gVisor

1. THE judge SHALL read `languages/languages.toml` at start-up, validate it completely, and refuse to start on any error. [P13]
2. THE field names SHALL be those of the `language-drivers` spec (`enabled`, `image`, `file`, `driver`, `compile`, `run`, `bench_args`, `mem_mb`, `time_ms_per_test`, `max_source_bytes`, `protocol_version`) plus the sandbox fields in `design.md`. [P13]
3. A language with `enabled = false` SHALL be refused by the executor with `language_unavailable` and SHALL never start a container. [S03]
4. THE judge SHALL support `SANDBOX_RUNTIME` (default `runc`). IF set to another runtime, THEN preflight SHALL require the runtime to be registered and the probe SHALL pass with it. [S23]
5. THE spec and every report SHALL describe gVisor (`runsc`) as **configurable but untested** until a recorded S23 run on the target host passes. [S23]

### R13. Public data, redaction and output text

1. FOR hidden tests, THE judge SHALL emit and store only the verdict and time. It SHALL NOT include input, expected value, answer, stdout or stderr. [P12]
2. FOR sample tests, THE judge MAY include input, expected value and the program's answer in the typed `SampleDetail`, and a stderr snippet in `test_results`. [S02]
3. THE judge SHALL pass all text that came from the program or the driver (answers, stderr, exception class, compile messages) through one sanitiser that returns valid UTF-8, removes NUL and control characters other than tab and newline, bounds the length, and marks truncation. [P11]
4. THE sanitiser SHALL cap stored snippets at 64 KiB and event detail fields at 4 KiB each. [P11]
5. THE judge's logs SHALL contain run id, session id, container id, phase, verdict kind and durations. They SHALL NOT contain source code, test data, program output or stderr. [log test T35]
6. THE judge's error messages that reach events SHALL come from a fixed table (run-lifecycle `ErrorPayload`). [P12]

### R14. Pre-sandbox validation contract (VAL-1 to VAL-5)

Browser checks are for feedback only. The API server repeats every check before a run is queued. The checks are a first filter and a blocklist is not used.

1. **VAL-1 size.** THE server SHALL reject a source larger than the language's `max_source_bytes` (default 65 536 bytes), counted in bytes, with a message that states the limit and the actual size. THE server SHALL NOT use a line count. THE loader in this spec SHALL provide `max_source_bytes` to the server's `LanguageCatalog`. [vectors, server test in run-lifecycle]
2. **VAL-2 parse.** THE server SHALL parse the source with tree-sitter for the selected language and reject unparseable code before queuing. [run-lifecycle T22]
3. **VAL-3 plain text.** THE server SHALL accept one source field only, valid UTF-8, with no NUL and no control character other than tab, newline and carriage return, and no file upload. [vectors]
4. **VAL-4 limits.** THE server SHALL enforce per-user and per-IP rate limits in Valkey and a maximum number of queued jobs per user counted in Postgres, returning 429 with the wait time. [run-lifecycle T23]
5. **VAL-5 output.** THE judge SHALL deliver program output only as sanitised text (R13). THE UI SHALL render it as text nodes, never as HTML. [P11, ide-workspace]
6. WHEN the judge claims a run, THE judge SHALL re-check size, UTF-8 and control characters on the stored source. IF the check fails, THEN THE judge SHALL end the run with the verdict `compile_error` and a fixed public message, without starting a container. (run-lifecycle's `FailureReason` has no `invalid_source`; open question 13.) This is defence in depth: the judge does not trust the database content. [vectors, S03]
7. NO component SHALL use a list of forbidden words, modules or APIs as a security control. A program that imports `os`, `subprocess`, `socket` or `ctypes` SHALL pass validation and be contained by the sandbox. [S27]
8. THE judge crate SHALL publish the validation vectors as `crates/judge/tests/data/source_vectors.json` so the server's tests can run the same cases. [T12]

### R15. Integration contracts

1. `SandboxExecutor` SHALL implement the `RunExecutor` trait from run-lifecycle (`ClaimedRun`, `RunContext`, `RunOutcome`, `ExecError`) without changing the trait. It SHALL emit `compiling` before anything else, then one `test` event per executed test. [S01, S02]
2. `SandboxExecutor` SHALL load tests and limits through the `ProblemTests` trait defined here. THE production binary SHALL use an implementation that returns `problem_unavailable` until the catalogue spec provides one. [S01]
3. `SandboxExecutor` SHALL never write a verdict, a status or a job state. Only the worker's finish functions do. [run-lifecycle R10]
4. `SandboxOracle` SHALL implement `counterspell_engine::Oracle`. It SHALL run the user's program and the Python reference through the same session code and limits, SHALL return only `Returned`, `Crashed`, `Garbled` or `TimedOut`, SHALL return `OracleError` for infrastructure faults, and SHALL return within the call limit plus `KILL_SLACK`. [S05, S21]
5. THE hostile corpus run through `SandboxOracle` SHALL produce `TimedOut`, `Crashed` or `Garbled` and never `Returned` with a forged value. [S14]
6. `SandboxOracle` SHALL receive only public data (samples and generated inputs). Hidden test inputs SHALL NOT be passed to it. [P12]
7. `counterspell-mcp` code-running tools SHALL use the same executor/oracle types and configuration. [boundary test T36]

### R16. Evidence: hostile corpus, properties and honest reporting

1. THE spec SHALL include the model properties P1 to P14 and the real-sandbox scenarios S01 to S27 defined in `design.md`, each tied to a requirement above.
2. EVERY real-sandbox scenario SHALL run only against a daemon that passes the daemon-identity checks of R1, and SHALL refuse to run otherwise. No hostile code SHALL run on the host, including in tests.
3. EVERY program in the hostile corpus SHALL be unable to return a correct answer, except the explicit good-solution fixtures. For these programs the verdict SHALL never be `accepted`. [S05–S15]
4. FOR every scenario, THE test SHALL assert the documented limit (verdict set, wall time, container removal) and, where the attack has a side effect, SHALL check it out of band. [S12, S13]
5. THE implementation SHALL write `verification-report.md` listing, for each guarantee, one of: `TESTED on <host> <date>`, `TESTED in CI`, `MODEL ONLY`, `NOT TESTED`, with the evidence. A guarantee that was only reasoned about SHALL NOT be reported as tested. [T37]
6. THE definition of done SHALL be met only when all of these hold, as measured in the report:
   - a correct Python solution passes and a wrong one fails;
   - every hostile scenario is contained within its documented limits;
   - no program output can forge `accepted`;
   - no labelled container remains after recovery;
   - the probe passes on the VPS.

### R17. Operational limits and configuration

1. THE judge SHALL read these variables with the defaults shown, and reject out-of-range values at start-up: `SANDBOX_DOCKER_SOCKET` (required), `SANDBOX_RUNTIME` (`runc`), `SANDBOX_MAX_CONTAINERS` (`2 × JUDGE_CONCURRENCY`), `SANDBOX_REAPER_INTERVAL_SECS` (10), `SANDBOX_ORPHAN_AGE_SECS` (40), `SANDBOX_KILL_SLACK_MS` (1000), `RUN_WALL_CAP_SECS` (20), `SANDBOX_MAX_CONSECUTIVE_ABNORMAL` (3), `SANDBOX_CLEANUP_BUDGET_SECS` (5), `SANDBOX_API_TIMEOUT_SECS` (10), `LANGUAGES_FILE` (`languages/languages.toml`). [P13]
2. THE judge SHALL NOT read `.env`, key files or credentials. All values come from the process environment. [T01]

## Out of scope

- JavaScript, Java, C++ and bench mode.
- Queue, lease, SSE, cancel endpoint and terminal-write logic (run-lifecycle).
- Ghost-hunt orchestration and shrink events.
- Per-user rate limiting, authentication and the Valkey code.
- Making gVisor work. Only its selection by configuration is built.
- Container pooling or reuse across runs. This is deliberately not done (open question 5).
- Network access from sandboxes for any purpose, including package installation.
- Memory-peak measurement per test (`memory_kb` stays empty; open question 12).
