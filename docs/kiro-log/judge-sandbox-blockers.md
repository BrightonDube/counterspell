# judge-sandbox blockers

Items here stay open until the named person acts. Kiro left each task unchecked and carried on with work that does not depend on it.

## T02 host spike (HUMAN, SUPERVISED)
- **Needs:** someone on the sandbox VPS to run the steps in `docs/kiro-log/judge-sandbox-human-steps.md` and record the results in `deploy/sandbox-verification.md`.
- **Unblocks:** T03, T21 (file delivery choice), T22 (preflight fixtures), T19 (daemon info fixtures).
- **Status:** not started. Kiro cannot reach the VPS.

## T03 Python digest pin (HUMAN)
- **Needs:** pulling `python:3.13-slim` on the sandbox daemon and writing the real digest into `languages/languages.toml`.
- **Current state:** `languages/languages.toml` holds an all-zero placeholder digest for every language. It is syntactically valid and is never pulled. The sandbox fails closed on a missing image.
- **Unblocks:** running any real container.

## Python driver file (owned by language-drivers)
- **Issue:** `languages/languages.toml` enables Python, and the loader refuses an enabled language whose `drivers/python/driver.py` is missing. So the real catalogue cannot be loaded until language-drivers adds that file. The test suite uses a stand-in driver in a temporary directory.
- **Effect:** the judge, once it loads the catalogue (T24), stops at start-up with `MissingDriver`. This is the intended fail-closed behaviour.
- **Unblocks:** T24 wiring on a real host.

## Design open item
- The design says stderr over 64 KiB kills the run (`docs/system-design.md` section 7 and VAL-5). Amendment A2 changed this to truncation. Brighton must decide whether to update the brief documents. Recorded in `design.md` follow-ups, not edited here.

## Follow-up for run-lifecycle T22 (note, not a blocker)
- `crates/judge/tests/data/source_vectors.json` is the shared validation vector file (judge-sandbox T12). Run-lifecycle T22 tests should load the same file. This was recorded here, and the run-lifecycle spec was not edited.

## Run stop point (14 Nov 2026)
- **Done:** T01, T04 to T13 (with the wave 1 review fixes).
- **Not started in this run:** T14 to T18 (backend traits, fake backend, session lifecycle, call runner, run orchestration, fault properties). They do not need a Docker daemon, but the backend trait shape is a design decision that the next run should review. T15 and T16 are SUPERVISED.
- **Blocked:** T19 to T31 depend on T02 (HUMAN, VPS host spike). T23 and T24 also depend on run-lifecycle T16 and T19. T25 to T31 need a real sandbox daemon and are `sbx_` scenario tests.
- **Human-only:** T02 and T03 (`docs/kiro-log/judge-sandbox-human-steps.md`).

## Run stop point (attempt 2, 14 Nov 2026, after commit 293e4b5)
- **Done this run:** T14 (backend traits, fake), T15 (session lifecycle, cleanup guard), T16 (call runner), T17 (run orchestration), T18 (fault-injection properties), and the wave 2 review fixes. Each has its own commit (T14 `7ecee7c`, T15 `9b85496`, T16 `0a5094b`, T17 `98b34db`, T18 `c67110d`, fixes `293e4b5`).
- **Blocked, unchanged:** T19 to T31 depend on T02 (HUMAN, VPS spike). T19 needs the recorded daemon fixtures from T02. T20 and T21 need T19. T23 and T24 need T20 and run-lifecycle T16 and T19. T25 to T31 need a sandbox daemon.
- **Human-only, not run:** T02 and T03. Steps are in `docs/kiro-log/judge-sandbox-human-steps.md`.
- **New dependency to watch:** the sweeper (T23) is what makes a failed removal safe. Until T23 lands, a removal failure leaves a container behind. See the wave 2 entry in `judge-sandbox-review-queue.md`.
- **Contracts the bollard backend must meet (written in `backend.rs`):** every daemon call times out at `SANDBOX_API_TIMEOUT`; `Exited` only after a successful inspect; exec output capped at 64 KiB with `overflowed` set.
- **Residual risk, not fixed:** a program can answer a later call before that call's input is written, if its answer matches. See the review queue.

## Run stop point (attempt 3, 14 Nov 2026)
- **Done this run:** T19 code (`crates/judge/src/sandbox/daemon.rs`, `crates/judge/tests/model_daemon.rs`). Path guard (P14 cases, without the `DOCKER_HOST` test), the pure `check_facts`, `From<&SystemInfo>` and `connect`/`verify`. 15 tests pass, clippy and fmt clean.
- **T19 is NOT ticked.** Its verification asks for parsing against recorded daemon fixtures from T02. The tests here use hand-written synthetic facts, which are labelled as such. T19 can be ticked once T02 records real `docker info` output and those fixtures are added.
- **Blocked, unchanged:** T20 (depends on T19), T21, T22 (T02 facts), T23 and T24 (depend on T20 and run-lifecycle), T25 to T31 (need a real sandbox daemon on the VPS). T02 and T03 are HUMAN steps in `docs/kiro-log/judge-sandbox-human-steps.md`.
- **Open for Brighton:** the VPS spike (T02) and the Python digest pin (T03). Nothing after T19 can be verified without them.


## Run stop point (14 Nov 2026, run 4): T19 ticked, T20 code done but not ticked

**Decision needed from Brighton: `#[ignore]` on `sbx_` tests.**
- The PreToolUse guard (tdd-and-quality, Test integrity) refused to write `crates/judge/tests/sbx_bollard_create.rs`. It allows `#[ignore]` only on `pg_*` and `valkey_*` tests.
- The judge-sandbox spec (tasks.md conventions, T20 to T37) requires real-daemon tests named `sbx_` and marked `#[ignore]`, so they never run in `cargo test --workspace` without `SANDBOX_DOCKER_SOCKET`.
- Kiro did not add a workaround (no feature gate, no runtime skip, no shell write). The file was not created.
- **Needs:** Brighton to choose one: (a) allow `#[ignore]` on `sbx_` tests in the guard, or (b) another gating method he approves in writing.
- **Unblocks:** T20's last verification item (`sbx_create_readback_matches`), and then T21, T22, T23, T25 to T33, T34 and T35, which each have an `sbx_` test. T36 has no `sbx_` test but depends on T24.
- **Note:** the T20 code is complete and its model tests pass. T20 stays `[ ]` until the real-daemon test exists and passes on the rootless sandbox daemon (T37).

**Done this run:**
- T19 ticked (`cd7b8d5`): the recorded rootful WSL `docker info` is a real rejected fixture. The accepted rootless fixture is still synthetic. A source scan replaces the `DOCKER_HOST` environment check (P14).
- T20 code (not ticked): `crates/judge/src/sandbox/bollard_backend.rs` and `tests/model_bollard.rs` (10 tests, including a 26-row mutation table). Workspace tests: 860 passed, 0 failed. Clippy and fmt clean.

**Not started:** T21 onward, because they depend on T20 and on the `sbx_` decision above.

**Minor findings (T20):**
- The read-back assumes the daemon echoes the tmpfs option string unchanged. The ignored real-daemon test must confirm it (T37).
- `host_resources` reads inspect `Mounts`, so an image-declared `VOLUME` is reported as a host resource. That is a fail-closed choice, but it is not in the design text. Brighton should confirm it.

## Run stop point (attempt 3 of the continuation, 15 Nov 2026): T20 still blocked by the `#[ignore]` guard

- **Task:** T20 (`sbx_create_readback_matches`) stays `[ ]`. Its code and model tests are already in commit `d74705c`.
- **What happened:** I retried the write of `crates/judge/tests/sbx_bollard_create.rs`, the real-daemon test that needs `SANDBOX_DOCKER_SOCKET`, with `#[ignore]`. The PreToolUse guard refused it: "Rust #[ignore] is allowed only on tests named pg_* or valkey_*". No workaround was used. There was no shell write, no feature gate and no runtime skip.
- **Why it matters:** every later task depends on T20 or on T24. T21 depends on T20, T22 on T21, T23 on T20, and T24 on T22 and T23. Each of them has an `sbx_` test, or a `#[ignore]` sweep, so none can be ticked. The `T25`–`T33` scenario tests also need `#[ignore]` on `sbx_` tests.
- **Decision needed from Brighton (one of these):**
  1. Extend the guard to allow `#[ignore]` on `sbx_*` tests, as the `pg_`/`valkey_` exception already does. This is the spec's own convention (tasks.md, conventions). It is my preferred option.
  2. Another gating method you approve in writing. A runtime skip on a missing socket is not acceptable, because it would pass silently.
- **Unblocks:** T20 to T24 and T25 to T33 (`sbx_` tests), then T34 to T39.
- **Not done in this run:** no code changed. T21 onward were not started, because their dependencies are open.
- **Note:** `run-lifecycle` T16 and T19 are already ticked, so the T24 dependency on them is met. The block is only the `sbx_` guard.


## Assistant update (15 Nov 2026, 00:52 SAST)

Resolved without a VPS or a new Kiro session:

- Official image digests pinned in `languages/languages.toml` (verified with a registry digest read and `docker pull` on this machine; recorded in `deploy/sandbox-verification.md` run 2): Node, Temurin 21 and GCC 14. Python was already pinned on 14 Nov. JavaScript, Java and C++ stay disabled until their drivers exist.
- The test-integrity hook now allows `#[ignore]` on `sbx_*` tests, matching the approved judge-sandbox convention (`brighton-decisions`, 15 Nov).
- The 14 Nov local-verification decision is extended to later sandbox-verification blockers, including language-drivers. The identity guard still refuses a rootful daemon. T02 and T37 stay open.

Still blocked or still needing Brighton:

- A real rootless sandbox daemon (VPS T02/T37, or `sudo apt install uidmap` in WSL plus a rootless setup). Until then, `sbx_` tests can be written but cannot be ticked, and driver code still must not run on the host.
- How `signature.json` is typed for T07: add a `counterspell-grimoire` dependency to the judge, or move a shared type into `counterspell-common`.
- Pack review (grimoire-packs T28) before T10 fixtures for published packs.
- Installing the free Renovate GitHub app, if weekly digest PRs should actually run.


## Assistant update (15 Nov 2026, 01:03 SAST)

A local rootless Docker daemon is now running for user `brighton` (systemd user `docker.service`, socket `/run/user/1000/docker.sock`, CLI context `rootless`). The default Docker context is still Docker Desktop. `cs-dev-pg` and `cs-dev-valkey` were not disturbed. `SANDBOX_DOCKER_SOCKET` is in `~/.config/counterspell/dev-test.env`, and `scripts/kiro-implement-all.sh` sources that file.

Still blocked for ticking `sbx_` tests (T20, T25–T33):

- Cgroup v2 `cpuset` and `io` are not delegated (`cpu memory pids` only). `docker --context rootless info` warns "No cpuset support" and "No io.* support", so the judge identity check will refuse the daemon until Brighton applies the `Delegate=cpu cpuset io memory pids` drop-in (commands in `brighton-decisions.md`, 15 Nov 01:03).
- No `sbx_*` test files exist yet. They can be written (the hook now allows `#[ignore]` on `sbx_*`), but they cannot be ticked until they pass after the delegation drop-in.

T02 and T37 stay open for the real VPS. The 15 Nov 00:55 signature-type decision is already recorded. Pack review and the Renovate GitHub app still need Brighton.


## Run note (15 Nov 2026, continuation): T20 stays `[ ]`; T21 to T39 are blocked behind it

- **Done this run:** `crates/judge/tests/sbx_bollard_create.rs` (`sbx_create_readback_matches`, `#[ignore]`). It compiles. `cargo fmt --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass (model_bollard: 10 passed).
- **Real-daemon run:** `SANDBOX_DOCKER_SOCKET=/run/user/1000/docker.sock cargo test -p counterspell-judge --features sandbox --test sbx_bollard_create -- --ignored` fails at the identity check: `Check(LimitSupport("WARNING: No cpuset support"))`. The guard refused the daemon before any container was created. The daemon still lacks cpuset and io delegation.
- **Why T20 is not ticked:** the 15 Nov 00:52 decision says `sbx_` tests are not a reason to tick a task until they pass on a rootless sandbox daemon (T37). The T20 `sbx_` test has not passed.
- **Unblock:** Brighton applies the `Delegate=cpu cpuset io memory pids` drop-in (commands in `brighton-decisions.md`, 15 Nov 01:03). Then rerun the command above. If it passes, tick T20 and the same test becomes the T20 evidence.
- **Downstream:** T21 to T24 and T31 to T33 each contain `sbx_` tests and depend on T20 or on each other. T25 to T30, T34 to T36 depend on T24. T37 (VPS run) and T39 depend on everything. Nothing after T20 can be ticked honestly until the delegation drop-in is applied.
- **Also not done:** T23's fake-backend sweeper needs a list-containers method that the `Backend` trait does not have (`backend.rs` has only create and session methods). Adding it is a trait change to T14, so I did not make it without a decision.

## Run note (16 Nov 2026, judge-sandbox continuation)

- **T20 stays `[ ]`.** Cgroup delegation (cpuset, io) is now applied on the rootless daemon, so the identity check passes. The `sbx_create_readback_matches` test then fails at catalogue load: `languages/languages.toml` enables Python and JavaScript, but `languages/drivers/` does not exist. The loader refuses it (`MissingDriver`). The driver files belong to language-drivers T03 and T04, which are not done. Unblock: language-drivers T03 and T04, or Brighton's decision to disable JavaScript again (brighton-decisions 15 Nov says JavaScript stays disabled until its driver exists, which conflicts with commit 0d94d99).
- **T21: code done, not ticked.** The four `sbx_` tests (`tests/sbx_bollard_session.rs`) compile but have not run. The rootless daemon has no images, so `create` cannot find the pinned `python:3.13-slim`. Unblock: Brighton pulls the pinned digest onto the sandbox daemon (`docker -H unix:///run/user/1000/docker.sock pull python:3.13-slim@sha256:e48f59cdba69a3cbd9ed2cf0f348d90d45a8d350d98134cf1070bc106ee9ed90`), then runs `cargo test -p counterspell-judge --features sandbox --test sbx_bollard_session -- --ignored`. Kiro did not pull the image, because that is a setup step for Brighton.
- T22 onward depend on T21's real-daemon results or on `sbx_` runs. The judge-sandbox work continues with the fake-backend parts.

- **T22 partly done (16 Nov 2026).** `crates/judge/src/sandbox/preflight.rs` holds the pure evaluation: 12 checks, `parse_size`, and `tests/model_preflight.rs` (5 tests, including a mutation test that weakens each control and expects exactly its own check to fail). Still to do before the task can be ticked: the probe runner that starts the probe container and runs the five commands, the `PreflightState` with its 60 s refresh, the `counterspell-judge preflight` subcommand, and the `sbx_preflight_*` tests. Those need a real probe run, so they wait on the pinned image on the sandbox daemon.

## 16 Nov 2026 03:15 SAST: images now on the sandbox daemon
The pinned Python and Node images are pulled onto the rootless sandbox daemon, and the Node digest is pinned (see brighton-decisions.md, 16 Nov 03:15). The image blockers above are cleared; rerun the sbx_ tests.

## 16 Nov 2026 (judge-sandbox run, attempt 1): T20 stays `[ ]`, T21 ticked

- **Fixed in this run:** the create read-back rejected every real container with `the environment must be the fixed set`. Docker merges the pinned image's own variables (`GPG_KEY`, `PYTHON_VERSION`, `PYTHON_SHA256`) into the container. `bollard_backend::without_image_env` now drops only variables that the image declares with the same value. Tests: four new cases in `tests/model_bollard.rs`. The T21 `sbx_` tests pass on the rootless daemon after this.
- **T20 not ticked:** `sbx_create_readback_matches` fails at catalogue load with `MissingDriver { language: "javascript" }`. Also `languages/drivers/` does not exist, so the Python driver is missing too.
  - Commit 0d94d99 (language-drivers T02) set `enabled = true` for JavaScript. The file header and brighton-decisions (15 and 16 Nov) say JavaScript stays disabled until its driver exists. I did not change `languages.toml`, because it belongs to language-drivers.
  - Unblock: language-drivers T03 and T04 (add `drivers/python/driver.py` and `drivers/javascript/driver.js`), or Brighton's decision to disable JavaScript again.
- **Not ticked, waiting on real runs:** T22 (probe runner, `PreflightState`, `preflight` command, `sbx_preflight_*`), T23 (sweeper needs a list-containers method on `Backend`; that is a T14 trait change, so it needs a decision), and T24 (needs run-lifecycle wiring and a driver).

## T36 (boundary tests) not started: needs a decision

- Checks 1 to 4 and 6 need no new crate. Check 1 can parse `Cargo.lock` as text. Check 2 must look for `use bollard` and `bollard::` in code, not doc comments (`backend.rs` mentions bollard only in docs; `mod.rs` declares `pub mod bollard_backend`).
- Check 5 (`docker-compose.yml`) needs a YAML parser. T36 names a "YAML dev-dependency" but no crate, and `Cargo.lock` has no YAML crate. Brighton or the next run must choose one (licence checked with `cargo deny`). `serde_yaml` is archived, so do not use it by default.
- Unblock: pick the YAML dev-dependency. Then T36 can be written and ticked.


## Run note (16 Nov 2026, attempt 2 continuation): no new ticks; T20 blocked on language-drivers

- **T00:** present and signed off in `design.md` ("Review decisions", 14 Nov). T01 onward may run.
- **Re-run this session:** `sbx_create_readback_matches` against the rootless sandbox daemon (`SANDBOX_DOCKER_SOCKET` from `~/.config/counterspell/dev-test.env`, value not logged). The identity check passes now (cgroup delegation is applied). The test fails at catalogue load: `MissingDriver { language: "javascript" }`.
- **Root cause, precisely:** `languages/languages.toml` enables Python (line 17) and JavaScript (line 33). `languages/drivers/` does not exist, so the loader refuses the whole shipped catalogue. Disabling JavaScript alone does not unblock T20, because the Python driver is also missing.
- **What unblocks T20 (in order):** (1) language-drivers T03 (`languages/drivers/python/driver.py` with its tests), then (2) either language-drivers T04 for JavaScript or Brighton's decision to set `enabled = false` for JavaScript again, per the 15 Nov standing rule ("JavaScript stays disabled until its driver exists"). Kiro did not edit `languages.toml`, because it belongs to language-drivers and the standing rule needs Brighton's confirmation.
- **Not done, because they depend on T20 or on an `sbx_` run:** T22 (probe runner, `PreflightState`, `preflight` command, `sbx_preflight_*`), T23 (sweeper; also needs a list-containers method on `Backend`, a T14 trait change), T24 and everything after. T35 depends on T24. T36 needs the YAML dev-dependency choice (see the T36 entry above). T37 is VPS-only. T39 depends on T37.
- **Decision still open for Brighton:** the T23 `Backend` list method. The design (section 12, step 4) requires the sweeper to list labelled containers, so the method is in scope, but the trait change to T14 has not been approved. Assumed choice if he agrees: add `list_managed(&self) -> Vec<ManagedContainer>` to `Backend`, with the fake and bollard implementations, and keep the label filter `counterspell.managed=true`.
- **No code changed this run, no task ticked.**

## T20 unblocked (16 Nov 2026, non-Kiro run)
- `sbx_create_readback_matches` passes on the rootless sandbox daemon. The earlier blockers (MissingDriver for python and javascript, missing cpuset/io delegation, missing images) are all cleared. T20 is recorded in `judge-sandbox-done-pending-tick.md`.
- Remaining open for T22: probe runner, `PreflightState` with the 60 s refresh, `counterspell-judge preflight` subcommand, `sbx_preflight_*` tests. All now runnable on the local rootless daemon.
- T23 open decision now resolved (non-Kiro run, 16 Nov 2026): the sweeper needs a container-listing call. Smallest backward-compatible extension: add `Backend::list_managed(&self) -> Vec<ManagedContainer>` listing containers with the `counterspell.managed=true` label, implemented by the fake and the bollard backend. Recorded in `brighton-decisions.md`.

## T39 independent review (19 Nov 2026): T39 stays `[ ]`
Reviewer: separate semantic-reviewer session. Full report: `docs/kiro-log/judge-sandbox-t39-independent-review.md`. Verdict NEEDS_CHANGES, no Critical findings.
- **Important 1 (fixed in this run):** a sample-only Run (`mode = 'run'`) or a custom ghost run that came out `accepted` counted as "solved" on the dashboard and catalogue. Both queries now require `mode = 'submit' AND ghost_input IS NULL`. Tests: `pg_an_accepted_sample_run_is_not_a_solved_problem`, `pg_accepted_sample_run_is_attempted_not_solved`. Requirement text (dashboard-fsrs DASH-4, CAT-A2) says "accepted submission"; I read that as an accepted Submit. Brighton may overrule.
- **Important 2 (open):** a program that closes fd 1 and fd 2 and keeps running may leave `inspect_exec` without an exit code (`bollard_backend.rs:919-925`), reported as `InfraError` rather than a user verdict. Unconfirmed. Next step: add a corpus program that closes both fds and an `sbx_` test on the rootless daemon; fix the mapping if it shows an infrastructure error.
- **Minor (open):** MCP runner must call the daemon identity `verify`, not only the string check; empty preflight state list passes the gate (`sandbox_executor.rs:403`); bench answers unchecked and bench mode visible to user code (Familiar mood is only a display, but note it); judge compose service lacks container hardening; unchecked `mem_mb * MIB`; four corpus gaps listed in the review.

## T39 review Important 2 resolved (spec-driver, drive queue)
- Added corpus program `hostile/close_both_fds.py` (closes fd 1 and 2, keeps running) and `sbx_s15e_closing_both_fds_and_staying_alive_is_a_time_limit`.
- Run on the rootless daemon: passes with no code change. The run ends as `time_limit` (killed from outside), not an infrastructure error. The review's concern is not reproduced; the test is now a regression guard.
- T39 stays `[ ]`: it still depends on T37 (VPS). Minor review items remain open.
