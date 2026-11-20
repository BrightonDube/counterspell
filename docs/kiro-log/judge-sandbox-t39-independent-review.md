# Independent security review: judge-sandbox boundary (T39)

Reviewer: separate session, read-only. Scope: `crates/judge`, the `crates/mcp` import surface, the boundary tests, and the hostile corpus. Date: 19 Nov 2026. I did not rely on `verification-report.md` or `judge-sandbox-done-pending-tick.md`.

Evidence run: `cargo test -p counterspell-judge --offline` gave 434 passed, 0 failed, 226 ignored. The ignored tests are the `sbx_*` and `pg_*` tests. The six `boundary.rs` tests and the `model_*` tests pass. No real-daemon behaviour was observed, so every claim about Docker's runtime answers below is from reading code, not from running it.

**Verdict**: NEEDS_CHANGES. No Critical finding: I found no path where program output can produce `Pass` or `Accepted`, and every control in `docs/system-design.md` §7 is set in the request builder. There are two Important findings (one consumer-side integrity gap and one likely misclassification of a hostile program as infrastructure failure) and several Minor ones. T39 should stay open until the Important items are fixed or Brighton explicitly accepts them.

## Summary of findings

| # | Severity | Confidence | Where | One line |
|---|---|---|---|---|
| 1 | Important | confirmed | `sandbox/sandbox_executor.rs:580`, `sandbox/problem.rs:108-112`, `server/src/dashboard/query.rs:45`, `server/src/catalogue/read.rs:280` | A sample-only Run, or a custom ghost run on one chosen input, stores `verdict = accepted` and counts as "solved". |
| 2 | Important | likely | `sandbox/bollard_backend.rs:919-925` | A program that closes fd 1 and fd 2 and keeps running turns into an infrastructure error, not a user verdict. |
| 3 | Minor | confirmed | `sandbox/daemon.rs:42-53`, `mcp/src/runner.rs:52` | The main-daemon socket refusal is an exact string match. |
| 4 | Minor | confirmed | `sandbox/sandbox_executor.rs:403` | The preflight gate passes vacuously when the state list is empty. |
| 5 | Minor | confirmed | `sandbox/bench.rs:177`, `languages/languages.toml:23` | Bench answers are never checked, and user code can see it is in bench mode. |
| 6 | Minor | confirmed | `tests/boundary.rs:212`, `tests/boundary.rs:~300`, `docker-compose.yml:63-87` | Boundary tests cover less than their names say; the judge service has no container hardening. |
| 7 | Minor | confirmed | `sandbox/policy.rs:251,263`, `languages.rs:406-412` | Unchecked arithmetic on config values with no upper bound on `mem_mb`. |
| 8 | Minor | confirmed | `tests/hostile/` | Corpus gaps (see section 5). |

## 1. Docker request builder (checklist items 1, 3, 6, 7, 9)

`ContainerPolicy::for_language` (`policy.rs:248-330`) is the only constructor, and `SessionSpec::new` (`spec.rs:134-160`) adds only the run UUID, a validated worker id (`[A-Za-z0-9_-]{1,64}`, `spec.rs:163-168`), and an expiry label. Nothing in the request depends on the submitted source. The exec argv comes from the catalogue as a list, and tar entry names are validated `FileName`s (`spec.rs:51-77`).

Each §7 control is set and checked by `hardening_violations` (`policy.rs:330-470`), and `to_create_body` (`bollard_backend.rs:73-132`) copies every field into the request:

- Network `none`, read-only root, user `65534:65534`, `cap_drop ["ALL"]`, empty `cap_add`, `no-new-privileges:true`.
- Seccomp: no entry is written for `Default`. `Custom` and `Unconfined` are refused by the checker (`policy.rs:355-357`).
- `privileged: false`, no binds/devices/links/extra hosts/volumes-from, private pid/ipc/cgroup namespaces.
- `NanoCpus` 1e9, memory equal to memory-plus-swap, pids 64, ulimits `nofile`/`fsize`/`core`, and `nproc` forbidden (design deviation D1).
- tmpfs `/box` 64 MiB (`nosuid,nodev,uid/gid 65534,mode 0700`, plus `noexec` unless `exec_scratch`), `/tmp` 16 MiB `noexec,nosuid,nodev`, shm 16 MiB.
- Log driver `none`, restart `no`, `auto_remove` false, fixed environment, `counterspell.managed` label.
- Exec requests set `privileged: false`, a fixed user, and `tty: false` (`bollard_backend.rs:1023-1042`).

The checker runs before create (`create_checked`, `bollard_backend.rs:208`) and again on the daemon's `inspect` answer (`verify_readback`, `:374-396`). The read-back fails closed: a missing section, an unknown runtime, or a malformed value is an error (`from_inspect`, `:139-196`), absent namespace modes count as shared (`:481`), and any image-declared volume appears in `Mounts` and is refused (`:520-530`). The image is digest-pinned and never pulled (`check_image`, `:304-326`). Preflight probes use the same `ContainerPolicy`, and there is no skip flag: `SandboxConfig::from_lookup` reads no bypass variable (`config.rs:69-150`). `NoExecutor` fails every run (`executor.rs:248-260`) in a build without the `sandbox` feature. `main.rs` calls `verify` before the worker starts (`main.rs:144`).

Residual points (all Minor, no change needed to ship):

- Read-back compares only the modelled fields. `inspect` does not report Docker's default seccomp profile, so a daemon-level `seccomp-profile` override would not be seen. Deploy hardening should pin it. *possible*.
- `hardening_violations` cannot check the image pin or that `memory_bytes` matches the language. Those are covered only by `check_image` and by construction.

## 2. Where `Pass` and `Accepted` are built (checklist items 4, 5)

`TestVerdict::Pass` in the sandbox crate is built in exactly one place, `judge_answer` (`compare.rs:88`), behind the private-field proof type `Verified`. The compile-fail doctest passes. `FinalVerdict::Accepted` is built in exactly one place, `decide` (`verdict.rs:127`), and only when no test failed, nothing was cut by the cap, `planned > 0`, and the executed ordinals are unique and equal in count to the plan. Driver exit codes and stderr only reach `RuntimeError`, `MemoryLimit`, `TimeLimit`, `OutputLimit`, or `WrongAnswer` (`run.rs:297-312`). The answer line is accepted only if it is one line, nothing else is buffered (`call.rs:150-160`), keys are exactly `t` and `out`, there are no duplicate keys, and `t` equals the call id (`driver_protocol.rs`). Program output cannot reach `Pass` without the value being the program's own answer, and the comparator needs the expected value, which never enters the container. The Python driver contains no expected values.

### Finding 1 (Important, confirmed): `Accepted` is minted outside `decide` for runs that do not prove a solution

`execute_ghost` (`sandbox_executor.rs:531-620`) converts the engine's `Classification::Pass` straight into `Verdict::Accepted` and `TestVerdict::Pass` (`:580`). The queue's `TestVerdict` in `counterspell-common` is a plain enum, so the `Verified` guarantee does not cover it. This breaks checklist item 4 ("an accepted run in one function") in letter. The effect is real because of what consumes it:

- A custom ghost run (`ghost_input` set, `mode = "run"`, `server/src/runs/custom.rs:231`) judges one user-chosen input against the reference. A wrong solution that agrees with the reference on a trivial input such as `[]` gets `accepted`.
- A plain `Run` is limited to sample tests (`problem.rs:108-112`). A solution that passes only the samples also gets `accepted`.
- The dashboard's `solved` set is `owned_completed ... WHERE verdict = 'accepted'` with no filter on `mode` or `ghost_input` (`dashboard/query.rs:45`). The catalogue list does the same (`catalogue/read.rs:280`). A user can mark any problem solved, and the "weak concepts" and `<> 'accepted'` logic inherits the same flaw.

This does not break the sandbox, and it needs no hostile code. It does mean "Accepted" does not reliably mean "passed the hidden tests", which `docs/requirements.md` W1 step 8 relies on (solved only after 12/12).

Suggested fix: either filter consumers on `mode = 'submit' AND ghost_input IS NULL AND verdict = 'accepted'`, or have the judge store a distinct verdict for sample-only and custom runs. Preferably do both, and add a test in `crates/server` for each case. Also give the ghost path a `Verified`-style proof so the single-constructor property holds in code.

### Other verdict observations

- `decide` checks the count and uniqueness of ordinals but not that they match the planned set. A store that returned duplicates would give `Incomplete` (an infrastructure error), which fails safe. *confirmed, informational*.
- Every pass/fail bit for hidden tests is visible to the submitter. That is the normal oracle leak of any judge. The rate limits are the mitigation. Not a finding against this design.

## 3. Import boundaries (checklist item 2 and the user's item 3)

- `bollard` appears by path only in `bollard_backend.rs` and `daemon.rs`. `catalogue-cli` lists `bollard` only under `[dev-dependencies]` (tests). The `boundary.rs` scan covers all `src` trees and passes.
- The server's production dependency closure reaches neither `bollard` nor `counterspell-judge` (checked by `cargo metadata` in `boundary.rs`, passing). The server's only judge dependency is a dev-dependency with `default-features = false` and `testkit`.
- `counterspell-judge` has no `counterspell-grimoire` dependency (`crates/judge/Cargo.toml`). Only comments mention grimoire.
- `call.rs`, `session.rs`, and `spec.rs` import no expected-output type. I also grepped `oracle.rs`, `hunt.rs`, `bench.rs`, `stream.rs`, `backend.rs`, `driver_protocol.rs`, `preflight.rs`, `reaper.rs`, and `daemon.rs`. Only `hunt.rs` imports `ProblemSpec` (judge-side ghost orchestration, which is correct), and none imports `compare::Expected`.
- `mcp` reaches the judge only through `sandbox::daemon` and `languages::LanguageCatalog`. Its code-running slot is `unconfigured()` (`runner.rs:52-54`), so every code-running MCP tool fails closed today. Nothing in `crates/mcp` or `crates/judge/src` spawns a host process (`boundary.rs` passes; `clippy.toml` also bans `Command::new`).

### Finding 3 (Minor, confirmed): socket refusal is a string compare

`validate_socket_path` (`daemon.rs:42-53`) rejects only the exact strings `/var/run/docker.sock` and `/run/docker.sock`. `/var/run//docker.sock`, `/run/./docker.sock`, `/proc/self/root/run/docker.sock`, or a symlink to the main socket pass the path check. The real control is `verify` (`daemon.rs:217-230`), which requires `name=rootless`, a non-default data root, cgroup v2, and no "No ... support" warnings, and the judge calls it before starting (`main.rs:63`, `:144`). So the judge itself is safe. The MCP path (`mcp/src/runner.rs:52`, `config.rs:42-47`) calls only the path check. When the MCP sandbox runner is wired (Option 2, T05/T10), it must call `verify` too, and that should be a test. Optionally canonicalise the path before comparing.

### Finding 4 (Minor, confirmed): empty preflight list passes

`self.preflights.iter().all(|state| state.passed())` (`sandbox_executor.rs:403`) is true for an empty vector, and `SandboxExecutor::new` accepts one. `main.rs` builds one state per enabled language, so production is fine, but the "no skip" property rests on the caller. Make `new` reject an empty list, or check that a state exists for the run's language.

### Finding 6 (Minor, confirmed): the boundary tests are narrower than they read

- `container_facing_modules_see_no_expected_outputs` (`boundary.rs:211-229`) checks three files, and only for the strings `problem` and `TestCase`. It would not catch `use super::compare::Expected` in `call.rs`, nor a `ProblemSpec` import. Add `Expected` and `compare` to the forbidden tokens, and add `oracle.rs`, `bench.rs`, `stream.rs`, and `driver_protocol.rs` to the list.
- `compose_keeps_the_boundaries` forbids only the two literal main-socket paths. A mount of `/run` or `/var/run` as a directory would pass.
- The `judge` service in `docker-compose.yml:63-87` sets no `read_only`, `cap_drop`, `no-new-privileges`, or non-root `user`. It holds the sandbox socket, which is code execution as the sandbox user, so it deserves the same hardening as the sandboxes. This is a deploy-ops item. The default host path `/run/user/1000/docker.sock` (`:81`) is a convenience default and relies on the identity guard.

## 4. `unwrap` / `expect` in production code

I scanned every `.rs` file under `crates/judge/src` and `crates/mcp/src`, cutting each file at its first `#[cfg(test)]` and skipping `testkit` files, for `.unwrap()`, `.expect(`, `panic!`, `unreachable!`, `todo!`, `unimplemented!`, `unwrap_unchecked`, and `unsafe`. **There are none.** The workspace forbids `unsafe_code`. The 72 `unwrap`/`expect` hits in these crates are all inside test modules or `testkit`. `mcp/src/main.rs:37,40` use `unwrap_or` and `unwrap_or_else`, which do not panic.

Arithmetic and indexing worth a look, none reachable by user input:

- `policy.rs:251` computes `lang.mem_mb() * MIB` and `policy.rs:263` computes `run_cap.as_secs() + IDLE_EXTRA_SECS`. `run_cap` is bounded to 5-60 s (`config.rs`), but `mem_mb` is only checked for non-zero (`languages.rs:406-412`), so a very large value in `languages.toml` overflows (wraps in release). The file is trusted operator config, so this is Minor (Finding 7). Add an upper bound in the loader.
- `stream.rs:116-119` uses `STDERR_STORE_CAP - self.stored.len()` and a slice. The stored length never exceeds the cap, so this cannot underflow.
- `policy.rs` indexes `self.command[0]` only after a length check short-circuits.

## 5. Hostile corpus (checklist items 5 and the user's item 5)

I read the forging programs (`fake_verdict.py`, `forged_lines.py`, `remember_replay.py`, `early_exit.py`, `self_kill.py`, `stdout_close.py`, `sigstop_self.py`, `read_host.py`) and `sbx_forgery.rs`. They all assert a non-passing verdict, and `forged_lines.py` is the strongest test because it returns the correct sum while injecting extra lines, and the test requires `WrongAnswer`. No program in the corpus can obtain `Accepted` against a hidden test it cannot solve.

Reasoning on why a correct answer cannot be forged: a program shares a process with the driver, so it can write arbitrary lines to fd 1, read fd 0, or kill the driver. All of that only lets it choose the *value* of an answer line, which it could do by returning that value. The expected values live only in the worker. The program receives one input at a time and the next call is written after the previous answer. So the only way to a correct answer is to compute it, or to guess.

Gaps in the corpus (Finding 8, Minor, confirmed by listing the files):

1. No program closes both fd 1 and fd 2 (or `dup2`s them to `/dev/null`) and then keeps running. This is the trigger for Finding 2 below. `stdout_close.py` closes only stdout, so stderr keeps the exec stream open and the test passes without exercising the path.
2. No stale-injection case: a background thread or `atexit` hook that writes a line, or a partial line, *between* calls, aimed at the next call id. I believe the result is `ExtraOutput`, `WrongId`, or `NotJson` (all wrong answers), but nothing tests it.
3. No program that forks a child holding fd 1/2 and then exits. I expect `Deadline` (time limit) because the exec stream never ends, which is acceptable, but it is untested.
4. No program that detects bench mode (Finding 5).
5. The `sbx_*` scenario tests have not run against a rootless daemon (T37), so none of the corpus has been observed against real Docker.

### Finding 2 (Important, likely): closed fds become an infrastructure error

`BollardDriver::exited` (`bollard_backend.rs:919-929`) runs when the attach stream ends. It calls `inspect_exec` and turns a missing `exit_code` into `InfraError::Unexpected("driver finished without an exit code")` (`:924`). I believe the Docker API reports `ExitCode` as null while the exec is still running. If so, a program that closes fd 1 and fd 2 and then sleeps ends the stream while still running, and `exited` returns an error. That propagates out of `run_call` and `execute_run`, so the run ends as `internal_error` after the queue's retries (up to 3 attempts), and each attempt can spend the full call deadline in the container. A user-caused behaviour is reported as an infrastructure fault, which also pollutes alerting. It is not a verdict forgery. I could not confirm the daemon behaviour without running Docker, which I was told not to touch.

A related, less certain issue: there may be a short window after normal process exit where the stream has closed but the exit code is not yet recorded, giving the same error for an ordinary crash. *possible*.

Suggested fix: when `exit_code` is `None`, poll `inspect_exec` until `running == false` or the call deadline passes. If it is still running at the deadline, return a deadline or runtime-error fact, not an infrastructure error. Add the two-fd-close program to the corpus and an `sbx_` test for it.

### Finding 5 (Minor, confirmed): bench answers are unchecked and bench mode is visible

`timed_call` (`bench.rs:177`) calls `parse_answer(answer, id)` and discards the value. Timing is correctly the worker's clock, but a solution that passes every correctness test can read `sys.argv` (the catalogue passes `--mode bench`, `languages.toml:23`) and return a constant instantly at bench sizes. The Familiar would then show a good mood from "measured facts" that were not real. The effect is cosmetic, since the mood never changes a verdict, but it contradicts the product rule that mood reflects measured facts. Options: check bench answers against the reference for at least the largest size, or stop passing a visible mode flag and use a separate driver file.

## 6. Things I checked and found sound

- Infrastructure errors are a distinct type (`InfraError`) from `TestVerdict`. `run_call` kills and removes the container on any `Err` (`call.rs:97-111`).
- Per-test deadlines are on the worker's clock and are enforced by killing the container from outside (`call.rs`, `abnormal`). On an abnormal end the next test uses a fresh container (`run.rs`).
- Output is bounded at the stream (1 MiB stdout, a capped stderr buffer) and at `run_exec` (64 KiB for non-driver execs, with a kill on overflow).
- Compile output reaches the public record only as the fixed `COMPILE_MESSAGE` (`sandbox_executor.rs`, `report`), so source text and compiler output cannot leak.
- Once a hidden test has run, every later record is verdict-and-time only (`report`, `visible = timing.sample && !hidden_phase`). Samples run before hidden tests (`schedule`).
- The user and reference programs run in separate containers with separate backends (`oracle.rs`).

## Required before T39 can be ticked

1. Fix Finding 1 (consumer filter and/or distinct verdict) with a test in `crates/server`, or record Brighton's explicit acceptance.
2. Fix Finding 2 and add the corpus program and `sbx_` test, then run it on the rootless daemon (T37).
3. Findings 3 to 8 can be fixed in the same pass or logged as follow-ups. Findings 3 and 4 are one-line changes worth taking now.
