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
