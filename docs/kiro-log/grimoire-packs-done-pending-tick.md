# grimoire-packs: done, pending tick

Each entry: task id, commit(s), tests that prove it, the exact verification commands and their result. A task is done only after Kiro re-verifies and ticks it.

## T20 judge implements PackRunner and wires `--sandbox`

- Commit: `41f3b96`. Open question 7 was already resolved in favour of the
  judge depending on grimoire (problem-catalogue design.md dependency rules,
  approved by Brighton); the judge-sandbox spec it depends on is implemented.
- New `crates/judge/src/sandbox/pack_runner.rs`:
  - `PackSandboxRunner<B: Backend + Clone + Send>` implements
    `counterspell_grimoire::runner::PackRunner` by reusing the oracle's
    `Side`/`evaluate_side` machinery (`oracle.rs` internals promoted to
    `pub(crate)`). `Target::Reference` maps to a `Role::Reference` side,
    `Target::Program(path)` to `Role::User` so an incorrect program's compile
    failure is its own `ProgramResult`, never infrastructure.
  - `run` drives `evaluate_side` via `Handle::block_on` on a scoped plain
    thread, because callers are async (`block_on` panics on any runtime
    worker thread) and `spawn_blocking` callers keep working too.
  - `bench` times one reference call per supplied `(size, input)` on the
    worker clock and returns `(size, ns)` pairs.
  - `PackRunnerFactory::from_env()` reads `SandboxConfig::from_env()` +
    `SANDBOX_DOCKER_SOCKET`, connects via `daemon::connect`, and requires an
    enabled `python` entry; any gap yields `None`, preserving `SBX-07`/exit 2.
    Its `Drop` uses `shutdown_background` because `Runtime::drop` panics when
    dropped inside an async context.
- `crates/judge/src/bin/counterspell-catalogue.rs`: `pack_runner()` stub
  replaced by `pack_runner_factory()`; `validate-sandbox` and the sandbox
  branch of `import`/`seed-dev`/`seed-demo` build a per-pack runner from the
  pack's public `reference/` + `incorrect/` sources only (private files never
  enter a runner archive). Shared tail extracted into `sandbox_stamp`.
- Proving test: `crates/judge/tests/sbx_pack_runner.rs` (5 tests, real-daemon
  gated). Verification 16 Nov 2026:
  - `cargo test -p counterspell-judge --features sandbox --test sbx_pack_runner -- --ignored --test-threads=1` -> 5 passed, 0 failed (82s), daemon left clean.
  - Live smoke on migrated Postgres (`counterspell_t20`) + real daemon:
    `import --slug add-one --packs /tmp/t20pub --private-root ~/t20priv --json`
    -> `{"outcome":"created","version":1}`, exit 0; DB shows
    `problem_versions.sandbox_validated_at` stamped, 6 `hidden_tests`
    (1 curated + 5 generated), 1 `hidden_generation`, `import` +
    `sandbox_validate` events. Re-run `validate-sandbox add-one` ->
    `refused:already_validated`, exit 1, no container opened (gate precedes
    fill). Without `SANDBOX_DOCKER_SOCKET` -> `SBX-07`, exit 2.
  - `cargo test -p counterspell-judge` -> 448 passed, 0 failed;
    `cargo clippy -p counterspell-judge --all-targets --features sandbox -- -D warnings` clean; `cargo fmt --all -- --check` clean; pg catalogue suites 32/32 green.
- Not covered: `counterspell-grimoire validate --sandbox` still reports
  "sandbox runner not configured" — grimoire cannot depend on the judge, so
  the real runner is only reachable through the judge's own binaries.

## T20 verification follow-up (commit 0ffc9c7)

Running the three published starter packs through the real runner exposed a
driver protocol defect: both drivers appended "driver: <message>" *after* the
solution traceback, so the judge read the class "driver" from the last stderr
line instead of the real exception class. count-pairs ghost empty-min
expects ValueError and failed SBX-03. Fixed by emitting the driver diagnostic
before the traceback/stack (same exit codes) and adding
starter_packs_pass_sandbox_checks_with_the_real_runner, which runs
grimoire validate_sandbox on all three packs with the production runner.

Real-daemon evidence (SANDBOX_DOCKER_SOCKET=/run/user/1000/docker.sock):
- sbx_pack_runner: 6/6 pass (113.9 s) — all starter packs validate
- sbx_python_driver 16/16, sbx_javascript_driver 20/20,
  sbx_driver_parity 5/5, sbx_driver_hostile 2/2
- cargo fmt --check clean; cargo clippy -p counterspell-judge
  --all-targets --all-features -D warnings clean
