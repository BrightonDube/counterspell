# judge-sandbox review queue (SUPERVISED tasks)

Each entry is a supervised change that Brighton should check before the work builds on it. Kiro wrote every change test-first. The checks below say what to look at.

## T05 container policy and hardening checker (commit 5715c7a)
- **Files:** `crates/judge/src/sandbox/policy.rs`, `crates/judge/tests/model_policy.rs`.
- **Check:**
  - `ContainerPolicy::for_language` matches the control table in `design.md` section 7. In particular: no `nproc` ulimit, `seccomp` is `Default` (no entry), `log_driver` is `none`, the idle PID 1 is `sleep <run cap + 5>`.
  - `hardening_violations` reports every control named in the table. A missing check here is the main risk. The 42 mutation rows in `tests/model_policy.rs` show one check each.
  - The checker is pure. It does not know the Docker API, so the read-back step (T20) must run it again on the daemon's answer.
- **Open decision:** the policy stores the scratch `noexec` choice as `ScratchExec`. Python keeps `Forbidden`.

## T10 trusted comparator and the only mint of `Pass` (commit 7d51aa0)
- **Files:** `crates/judge/src/sandbox/compare.rs`, `crates/judge/tests/model_compare.rs`.
- **Check:**
  - `Verified` has a private field, and `TestVerdict::Pass` can only be built from it. The `compile_fail` doc test in `compare.rs` proves it.
  - `Expected` prints as `Expected(<redacted>)`, so hidden expected values cannot reach a log.
  - `judge_answer` takes `call_id: u64`. The design said `u32`, but the protocol framing (T08) uses `u64`, so I kept one type.

## T11 final verdict function (commit bfb51df)
- **Files:** `crates/judge/src/sandbox/verdict.rs`, `crates/judge/tests/model_verdict.rs`.
- **Check:**
  - Precedence: stop, then compile failure, then the first failing test by ordinal, then the run cap, then `Accepted`. The design lists the first four without stop. I put stop first, because a stopped run never gets a verdict. Brighton should confirm.
  - `Incomplete` is a verdict the design did not list. It covers a run that ran fewer tests than planned with no known cause, and the run code should report it as `internal_error`, never as the user's answer. Brighton should confirm.
  - `Accepted` is built in one place, after every check passes.

## Wave 1 review (T04 to T13), findings and what was done

Reviewed by the semantic reviewer over `git diff 898c150..105208f -- crates/judge`. Fixed in the commit after 105208f:

- **Critical, duplicate ordinals in `decide`:** two passes with one ordinal could stand in for a missing test. `Accepted` now needs unique ordinals and a count equal to the plan. Test added.
- **Critical, hidden data in `Debug`:** `SampleDetail` and `TestRun` printed hidden inputs with `{:?}`. `SampleDetail` now prints a placeholder. Test added.
- **Critical, `noexec` on `/box` (partly open):** the checker trusts the `scratch` field that `for_language` sets. A policy whose `scratch` was flipped to `Allowed` with `noexec` removed passes the checker. The shipped Python entry sets `exec_scratch = false`, and a test now pins that Python's policy is `Forbidden`. **Residual risk for Brighton:** the checker cannot tell a compiled language from an interpreted one. The protection depends on the catalogue reviewer keeping `exec_scratch = false` for interpreted languages.
- **Important, per-call stdout budget:** the reader capped lines but not total bytes, so many short lines never overflowed. `LineReader::with_budget` adds a byte budget per call. Test added.
- **Important, duplicate mounts and ulimits:** the checker looked only at the first match. Each ulimit must now appear once, and a duplicate `/tmp` mount is a violation. Mutation rows added.
- **Important, duplicate JSON keys:** `serde_json::Value` kept the last `t`, so `{"t":2,"t":1,...}` answered call 1. The protocol now reports `DuplicateKey`. Test added.
- **Minor, not fixed:** (a) `ExtraKey(String)` holds an unbounded user key that goes into the reason value (`driver_protocol.rs`). (b) `sanitize_display` passes C1 controls U+0080 to U+009F through, and it turns CR into U+FFFD, which garbles CRLF output. The design lists only C0 and DEL, so I kept that behaviour. (c) The run argv is not checked against the driver file name in the loader. Java's `Driver` name would need a rule that does not yet exist.

## T15 session lifecycle and cleanup guard (commit 9b85496) [SUPERVISED]
- **Files:** `crates/judge/src/sandbox/session.rs`, `crates/judge/tests/model_session.rs`. The backend seam gained `read_policy` (`backend.rs`), `ExecSpec::extract()` (`spec.rs`), and fake-backend support for scripted exec exits and read-back overrides.
- **Check:**
  - `Drop` spawns kill then force-remove on the runtime. The normal path awaits `close()`. The fake tests cover success, `?` return, task abort and panic.
  - Kill errors are logged, not returned, because forced removal also stops a running container. Brighton should confirm this is acceptable.
  - A read-back that fails `hardening_violations` removes the container and returns `PolicyReadbackMismatch`.
  - The compile step has a budget and a 64 KiB output cap. **Not tested:** the timeout branch, because the fake's exec never hangs and the real budget is 5 s. T21 or T16 should add a test once a scripted hang exists.
- **Real daemon:** nothing here talks to Docker yet. The real read-back and kill behaviour is checked by T20, T21 and the `sbx_` tests.

## T16 call runner (commit 0a5094b) [SUPERVISED]
- **Files:** `crates/judge/src/sandbox/call.rs`, `crates/judge/tests/model_call.rs`. The seam gained `DriverEvent::Stderr`. `session.rs` gained `Session::abort(within)`, and `stream.rs` gained `LineReader::reset_budget`.
- **Check:**
  - Every wait is a biased `select!`: deadline, then stop, then I/O. A write the program never reads is bounded by the same deadline.
  - Stdout has a 1 MiB budget per call. Lines that arrive early are kept for the next call (a second line read together with the first answers the next call without reading again).
  - Stderr only fills a buffer. A stderr flood does not kill the container and does not change the answer.
  - `CallFacts` has no stderr-overflow variant (A2).
- **Deviations Brighton should confirm:**
  1. **Kernel facts not read.** The design asks for `memory.events` and `pids.events` through a fixed exec on abnormal ends. This is not written. The OOM evidence for `DriverExited` comes from the `Exited` event that the backend sends. T21 must fill that from `memory.events`. Deadline and output limit verdicts do not depend on OOM evidence.
  2. **Infrastructure errors do not abort.** A daemon error during a call returns `Err` and leaves the container to the caller. The caller must close the session (`Opened` is dropped, so `Drop` cleans up).
  3. **Kill-timeout branch untested.** `Session::abort` returns `KillFailed` when removal misses the kill slack. The fake cannot hang a removal, so only the success path is covered.
  4. **Timing is real time.** The design asks for virtual time (P6). Tokio's `test-util` feature is not enabled and this run did not add it. Tests use 50 ms limits and bounds with 500 ms slack. This is a little slow on a loaded machine.

## Wave 2 review (T14 to T18): findings and what was done

Reviewed by the semantic reviewer over `git diff ee648ae..c67110d -- crates/judge`. Fixed in the commit after c67110d:

- **Critical, answers buffered across calls (fixed).** A line that arrived after an answer was kept and used as the next call's answer, with no input written for that call and no deadline. The call runner now treats any second line, or data after the answer, as `ExtraOutput`. That is a protocol violation (R8.4), it ends the container, and it is a `WrongAnswer`. **This changes the design text:** design section 9 said extra bytes are kept for the next call. Brighton should confirm the stricter rule. The T16 test that asserted the old behaviour now carries `test-change: R8.4`.
- **Residual, answers before their input (not fixed).** A line that arrives in a later read, before the next call writes its input, is still accepted if its `t` matches. The runner cannot tell that apart without a non-blocking check. The program would have to predict the hidden input, so the answer reveals nothing. The per-call time limit is still applied to the call that reads it.
- **Important, infrastructure errors (fixed).** A daemon error during a call now ends the container before the error returns. The run's open is bounded by the run cap (`timeout_at`), and a timeout is `InfraError::ApiTimeout`.
- **Important, compile output cap (partly fixed).** `ExecOutput` has an `overflowed` flag, and the compile step treats it as `OutputTooLarge`. The contract is written on `ExecOutput`: the backend must stop reading and kill at 64 KiB. **Not done:** the bollard backend (T21) must implement the cap while reading. Until then the fake only tests the flag.
- **Important, stalled daemon calls outside the call (contract, not code).** Each daemon call during setup (create, read-back, start, load, open) is not individually timed out. The contract is written in `backend.rs`: every call returns within `SANDBOX_API_TIMEOUT` or fails with `ApiTimeout`. The bollard backend must enforce it (T19 to T21).
- **Important, driver exit becoming a verdict (contract, not code).** `DriverEvent::Exited` must come only after a successful inspect. A failed inspect or OOM read must be an error, never an exit. This is written in `backend.rs` and must be enforced in T21.
- **Important, sweeper not yet built (dependency).** Cleanup after a failed removal depends on the sweeper (T23). Until T23 lands, a failed removal leaves the container for nothing to remove. The run still reports the verdict, because the tests passed.
- **Minor, daemon stall looks like a time limit (not fixed).** A slow daemon during a write or read is reported as `Deadline`, so the user sees TLE. The bollard backend should report daemon timeouts as errors (see the contract above).
- **Minor, dropped stop sender (fixed).** `run.rs` now treats a dropped sender as a stop, the same as the call runner. Test added.
- **Minor, `max_consecutive_abnormal = 0` (fixed).** Refused with `InfraError::Unexpected`. Test added.
- **Minor, verdict order (documented, not changed).** Runs go samples first, but the verdict picks the lowest ordinal among failures (design section 9). This is intended.
- **Minor, wrong error label (fixed).** A pre-create policy violation is now `InfraError::PolicyRejected`, not `PolicyReadbackMismatch`.
- **Minor, weak tests (partly addressed).** The fault-on-remove property now asserts the fault that left a container behind. The stderr-flood test still cannot show the flood was capped, because that is internal to the reader; `stream.rs` has its own cap tests (`model_stream.rs`). `model_backend.rs` mostly checks the fake's ledger. It is kept because it pins the fake that the other tests rely on.


## T20 bollard create, read-back and image check (SUPERVISED, code only; not ticked)
- **Files:** `crates/judge/src/sandbox/bollard_backend.rs`, `crates/judge/src/sandbox/mod.rs` (module behind feature `sandbox`), `crates/judge/tests/model_bollard.rs`.
- **Check:**
  - `create_checked` order: the hardening checker runs first, then the image digest is inspected (never pulled), then create, then read-back. A failed read-back removes the container.
  - `from_inspect` fails closed. An absent or unknown value (runtime, env entry, namespace mode, tmpfs size) is an error or a non-private value, never a default that passes.
  - The read-back must equal the policy it created, and must pass the checker. The mutation table shows each weakened daemon field gives exactly one violation.
  - `host_resources` includes inspect `Mounts`. That covers volumes an image declares. Brighton should confirm this is wanted.
  - Restart policy: only `no` passes. Any other text maps to an empty name that the checker rejects.
- **Not verified yet:** the real daemon round trip (`sbx_create_readback_matches`, blocked, see the blockers file). Tmpfs option echo and `pid_mode` `""` are the two daemon behaviours the real test must confirm.
