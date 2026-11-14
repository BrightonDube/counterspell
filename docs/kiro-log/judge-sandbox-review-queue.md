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
