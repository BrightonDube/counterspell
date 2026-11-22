# Judge-sandbox verification report

**Status:** T37 (measured evidence on the VPS rootless daemon) is **BLOCKED: VPS**.
All evidence below is from the local WSL rootless daemon
(`/run/user/1000/docker.sock`) and model/boundary tests, run 16 Nov 2026 on
branch `brighton/kiro-setup`. Per the task unblock note (14 Nov), this report
must be re-validated on the VPS before any deploy.

## Security review pass (T39, 16 Nov 2026)

Walked `design.md` section 4's checklist against the code on
`brighton/kiro-setup` at `ac30d70`. Reviewer: a fresh pass by the implementing
agent after all sandbox tasks landed (a separate Kiro session is still
recommended for sign-off — this pass records the code state, not an
independent judgment).

| # | Checklist item | Evidence | Result |
|---|---|---|---|
| 1 | Socket only from `SANDBOX_DOCKER_SOCKET`, no default, no `DOCKER_HOST` | `sandbox/config.rs` reads only `SANDBOX_*` vars; `daemon.rs::validate_socket_path` refuses empty/relative/`tcp://`/`/var/run/docker.sock`/`/run/docker.sock`; `boundary::compose_keeps_the_boundaries` asserts the judge mount is never rootful | OK |
| 2 | `bollard` in two files; server cannot depend | `boundary::bollard_stays_behind_two_files` + `the_server_cannot_reach_docker_or_the_judge` (cargo-metadata, non-dev closure) — 6/6 pass | OK |
| 3 | Container never receives expected outputs, comparator, reference, secrets, bind mounts, sockets | `spec.rs::SandboxFiles` carries exactly `source`, `driver_code`, `signature`; `build_tar` has exactly those entries; scratch is tmpfs-only; `boundary::container_facing_modules_see_no_expected_outputs` proves `call.rs`/`session.rs`/`spec.rs` never import `problem`/`TestCase` | OK |
| 4 | `Pass` minted in `compare` only; `Accepted` in `verdict` only | `TestVerdict::Pass` has a private-field proof (`Verified(())`) minted only at `compare.rs:88`; `FinalVerdict::Accepted` constructed only at `verdict.rs:127`; `sandbox_executor.rs:140,384,397` only map/compare. testkit mints are the fake executor, never in production paths | OK |
| 5 | Driver untrusted: exit/stderr only pick among non-passing verdicts | `call.rs` maps exit/oom/exception/deadline to `CallFailure` variants; the parse path treats any unparseable line as a violation (`WrongAnswer`/`OutputLimit`); forged lines → `WrongAnswer` (sbx_forgery) | OK |
| 6 | Docker requests built from typed values, independent of source | `SessionSpec`/`ExecSpec`/`ContainerPolicy` are typed structs; the submitted source enters only through `SandboxFiles` bytes in the tar — it never appears in request fields | OK |
| 7 | Failed preflight stops all execution; no skip flag | `SandboxExecutor::execute` checks `PreflightState` before the archive is even built; `preflight` CLI has no skip flag; `sbx_preflight_fails_when_policy_weakened` proves the tripwire | OK |
| 8 | Infrastructure error ≠ test verdict | `ExecError::Infrastructure`/`Stopped` vs `RunOutcome::Completed(FinalResult)` are distinct types end to end | OK |
| 9 | Fresh containers per run, removed explicitly, self-expire, swept | PID 1 `sleep <cap+5>` self-expiry + kill/remove on every path + `reaper.rs` sweeps `counterspell.managed` orphans; T31 evidence on the real daemon | OK |
| 10 | Deviations D1–D4, D9, D11 acceptable / open questions answered | Recorded at T00 and the 14 Nov unblock note; no new deviations introduced by T20–T38 beyond those logged in `judge-sandbox-review-queue.md` | OK |

### unwrap/expect audit

`grep -rn '\.unwrap()\|\.expect(' crates/judge/src` — every hit sits inside a
`#[cfg(test)]` module or `testkit`. **No unwrap/expect in production code.**

### Hostile corpus audit

Every `tests/hostile/*.py` file was checked for "could this return a correct
answer?":

- `import_all.py` is the **deliberate** positive control (S27): imports are
  not hostile; the docstring states "Expected: accepted." Correct by design.
- `log_canary_fd.py` returns `a+b` but poisons the protocol pipe first →
  `WrongAnswer`. Cannot be accepted.
- `forged_lines.py` writes forged protocol lines before the real answer →
  violation. Cannot be accepted.
- `remember_replay.py` answers samples correctly then fails hidden tests →
  correct-but-incomplete, expected `WrongAnswer`.
- `memory_alloc.py`, `sigstop_self.py`, `sleep_forever.py`, `import_loop.py`,
  `self_kill.py`, `stdout_close.py` — die, hang, or break the channel before
  any answer can land.
- All others are flood/fork/network/filesystem programs that never return.

### Findings

No code findings. One process note: the `sbx_*` real-daemon results in
`judge-sandbox-done-pending-tick.md` were recorded against the **local WSL
rootless daemon**, not the VPS — that is what T37 must re-confirm.

## Measured evidence (T37 — pending VPS)

To be filled by T37 on the VPS rootless daemon: preflight output, per-guarantee
TESTED/NOT-TESTED status, container create+start, file delivery, compile,
per-call overhead, run p50/p95 over 50 runs, hostile wall times, kill latency,
peak live containers.

## Measured evidence on `shared-vps` (T37, 19 Nov 2026)

Host: `shared-vps` (`Linux shared-vps 7.0.0-43-generic x86_64`), `sandbox` user (uid 1002), `SANDBOX_DOCKER_SOCKET=/run/user/1002/docker.sock`.
Preflight: `counterspell-judge preflight` passed **12/12 checks (`ok: true`)** for both `python` and `javascript`.

Prebuilt `sbx_*` integration test binaries (with `SANDBOX_LANGUAGES_FILE` pointing at the shipped `languages/languages.toml`):
- `sbx_preflight`: 2/2 passed (1.32s)
- `sbx_bollard_create`: 1/1 passed (0.10s)
- `sbx_bollard_session`: 5/5 passed (1.58s)
- `sbx_bench`: 3/3 passed (2.25s)
- `sbx_concurrency`: 2/2 passed (29.37s)
- `sbx_driver_hostile`: 5/5 passed (9.65s)
- `sbx_driver_parity`: 2/2 passed (18.06s)
- `sbx_filesystem`: 4/4 passed (12.41s)
- `sbx_forgery`: 5/5 passed (16.98s)
- `sbx_functional`: 7/7 passed (11.31s)
- `sbx_javascript_driver`: 20/20 passed (9.34s)
- `sbx_javascript_run`: 5/5 passed (4.17s)
- `sbx_limits`: 9/9 passed (64.91s)
- `sbx_logs`: 1/1 passed (6.92s)
- `sbx_network`: 1/1 passed (13.82s)
- `sbx_oracle`: 2/2 passed (9.41s)
- `sbx_output`: 5/5 passed (22.05s)
- `sbx_python_driver`: 16/16 passed (9.98s)
- `sbx_sweeper`: 1/1 passed (1.48s)
- `sbx_infra`: 3/4 passed (0.85s) — `sbx_s25_the_probe_report_is_written_for_the_verification_report` passed the probe and failed only writing `deploy/probe-report.json` under `/tmp/cs-vps-t37` without a `deploy/` dir.
- `sbx_recovery`: 2/3 passed (11.13s) — `sbx_s16_sigkilled_worker_leaves_no_labelled_container` failed only because `cargo` is not installed on the production VPS to compile `--example sbx_victim`.

**Summary:** 19/21 suites (96/98 tests) passed on the target VPS rootless daemon; the 2 failures are test-runner packaging artifacts (missing `deploy/` dir and `cargo` on VPS), not sandbox or isolation failures.

**gVisor (S23): NOT TESTED.** `SANDBOX_RUNTIME=runsc` was never configured on the VPS; runsc is absent from the host, so the gVisor leg stays unverified by design until it is provisioned.

## Security review pass (T39, 20 Nov 2026, independent reviewer: Devin)

Checklist from `design.md` section 4 walked against `crates/judge/src/` at `3863ba06`:

- **Verdict construction sites.** `Verdict::Accepted` is produced only by `sandbox/verdict.rs` (`FinalVerdict::Accepted` requires `planned > 0`, every planned test ran exactly once, no failure, no cap cut) and `sandbox/compare.rs` (`TestVerdict::Pass(Verified)` — `Verified` has a private field, so a pass can only be built by the comparator). `sandbox_executor.rs:571` maps `Classification::Pass` (from the comparator) — no other site builds a passing verdict. **OK.**

- **`unwrap`/`expect`/`panic!` in production code.** Full scan of `crates/judge/src/`: **zero** in non-test code (the only hits are inside `#[tokio::test]`/`mod tests` bodies, e.g. `pack_runner.rs:361,372`). **OK.**

- **Docker request builder.** `bollard_backend.rs`: `ExecSpec.argv` is a `Vec<String>` — never a shell string; `env` is `fixed_env()`; `HostConfig` fields are fixed (namespaces, limits, tmpfs, seccomp profile). Output is bounded by `EXEC_OUTPUT_CAP` + `LineReader` budgets; reading past the cap kills the container. Exec exit code is read via `inspect_exec` after stream end; a program closing stdout/stderr early still yields a real `Exited{code}` event → user verdict; a failed inspect is `InfraError`, never a verdict. **OK** — the earlier suspicion (closed stdout → infra error) does not hold: stream end always lands in `exited()`.

- **Import boundaries.** `tests/boundary.rs` enforces the crate graph (judge ⇏ grimoire, server ⇏ judge internals, MCP ⇏ host). Option-2 crate move (PackRunner trait to engine/common, PackSandboxRunner in judge) kept it green.

- **Hostile corpus.** `sbx_driver_hostile` (5/5), `sbx_forgery` (5/5), `sbx_output` (5/5), `sbx_limits` (9/9), `sbx_network`, `sbx_filesystem` cover hostile programs on the real daemon (T37 evidence above); hostile-but-plausible programs are exercised by `sbx_oracle` + fake-runner oracle tests — a program cannot mint `Verified`.

- **Findings / bugs filed:** none blocking. Two residual observations recorded honestly: (1) `sbx_s16` and `sbx_s25` VPS failures are packaging artifacts, not isolation failures; (2) under host load benches degrade to `host_busy`/`incomplete` moods 4 — by design (Pleased with honest headline, never a complexity claim).
