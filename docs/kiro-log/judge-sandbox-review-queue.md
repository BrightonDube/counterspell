# judge-sandbox review queue (SUPERVISED tasks)

Each entry is a supervised change that Brighton should check before the work builds on it. Kiro wrote every change test-first. The checks below say what to look at.

## T05 container policy and hardening checker (commit 5715c7a)
- **Files:** `crates/judge/src/sandbox/policy.rs`, `crates/judge/tests/model_policy.rs`.
- **Check:**
  - `ContainerPolicy::for_language` matches the control table in `design.md` section 7. In particular: no `nproc` ulimit, `seccomp` is `Default` (no entry), `log_driver` is `none`, the idle PID 1 is `sleep <run cap + 5>`.
  - `hardening_violations` reports every control named in the table. A missing check here is the main risk. The 42 mutation rows in `tests/model_policy.rs` show one check each.
  - The checker is pure. It does not know the Docker API, so the read-back step (T20) must run it again on the daemon's answer.
- **Open decision:** the policy stores the scratch `noexec` choice as `ScratchExec`. Python keeps `Forbidden`.
