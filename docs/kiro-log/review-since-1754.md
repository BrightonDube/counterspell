# Review of commits since b16a9ae^ (16 Nov 2026 17:54)

Reviewer: spec-driver, in a clean detached worktree at `fc73f3c` (HEAD when the review started; no newer commits appeared during it). Main working tree untouched.

Commits reviewed: `b16a9ae`, `6d7fcc1` (language-drivers T09); `54b1d2b`, `4d377ff` (C26); `34e3374`, `63b4de0` (C16); `aba3b7d`, `a7bed77` (C17); `1b70b6f`, `48806e9` (C18); `9cfd94d`, `3a619ae` (C19); `fc73f3c` (grimoire-packs T28).

## Gates (fresh, in the worktree)

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` (also with `counterspell-judge/sandbox`) | clean |
| `cargo test --workspace` | 1445 passed, 0 failed, 461 ignored |
| `cargo test --workspace -- --ignored pg_` (throwaway local Postgres) | 348 passed, 0 failed |
| `catalogue_sandbox_fill` (C16) | 10/10 |
| `pg_catalogue_sandbox_import` (C17) | 4/4 |
| `pg_catalogue_validate_sandbox` (C18) | 4/4 |
| `counterspell-catalogue` bin unit tests (C19) | 11/11 |
| `model_sandbox_executor` 14, `model_bench` 6, `familiar_measure` 10, `model_logs` 1 (T09) | all pass |
| `sbx_bench` on the rootless daemon (T09) | 3/3 |
| `validate --all packs --deny-warnings` (T28) | 0 errors, 0 warnings for all three packs |

Environment note: `/tmp` is a 3.9 GB tmpfs. A workspace build there fills it, so the review used `CARGO_TARGET_DIR=/home/brighton/cs-review-target`. A scratch database (`cs_review_scratch`) was created and dropped on the dev Postgres for the C19 dry-run check.

## Process findings (all commits)

1. **Ticks by the implementer.** `b16a9ae` (T09) and `54b1d2b` (C26) ticked their own tasks in the implementing commit. `63b4de0`, `a7bed77`, `48806e9` and `3a619ae` are "record completion evidence" commits whose only effect is to tick C16 to C19. All of this breaks "Who ticks". Each was re-verified here; the outcome is in the table.
2. **Commit-format drift.** C16 to C19 use `feat(judge): problem-catalogue C<nn> ...` plus a separate `docs(problem-catalogue): record C<nn> completion evidence`. The repo convention is `<spec> C<nn>: <summary>`. T09, C26 and T28 follow it. The C26 commit also carries a `Generated with [Devin]` trailer. `6d7fcc1` is a docs commit with a non-conforming `language-drivers:` prefix (no task id).
3. **Review-queue notes** exist for every task (C16 to C19, C26, T09). They are accurate except where noted below. The C19 note says the dry-run was verified live and prints `refused:draft_pack`. That is now stale, because T28 published the three packs.

## language-drivers T09: persist and publish bench measurements (b16a9ae) -> UNTICKED

Passes: code, clippy, 14/14 model tests, `sbx_bench` 3/3 on the rootless daemon (30 timing rows, container reaped). Payload types (`BenchStartPayload`, `BenchPointPayload`, complexity, familiar) use `deny_unknown_fields` and carry only sizes, `n` and `median_ns`, so by inspection nothing hidden can be added. Events go through the fenced `ctx.emit` path. The migration `20261116130000_run_events_bench_start.sql` only widens the `run_events.type` CHECK and is needed for `bench_start`. The design lists `bench_start` (system design 8b), but the `API conventions` steering list of event names does not. Please add it there.

Why it is not ticked. The task's verify clause is "public events contain no hidden input, expected output, source, or raw answer; event sequence and terminal-write behavior remain compatible with SSE replay".
- No test asserts the payload contents of the bench events. The model and sbx tests check event names and order, plus timing row counts only. A leak-by-construction test (serialise every emitted payload and assert the keys, and that no test input, expected value, source or raw answer string appears) is missing.
- `insert_run_timings` and the new `run_events.type` CHECK have no `pg_` test. The model tests use an in-memory `RecordingContext`. Nothing proves that a `bench_start` row inserts into Postgres, that `run_timings` rows are written with the `attempt` fence, or that SSE replay returns them in sequence.
- `crates/judge/src/executor.rs::record_timings` is only exercised through the fake path.

To tick: add (a) a payload-field test over all four bench events, (b) a `pg_` test inserting `bench_start` and `run_timings` rows and reading them back in `seq` order. Then re-run the gate.

No `unwrap`, `expect` or `unreachable!` was added in the T09 production diff, and no test was removed.

## problem-catalogue C26: version parameter in ProblemTests (54b1d2b) -> TICK CONFIRMED

Passes the task's verify: every implementation (`EmptyProblemTests`, `StaticProblemTests`, `DirProblemTests`, the new `PgProblemTests`) takes `version`, assertions are unchanged, and the doubles return `None` on a version mismatch. The workspace is green. Counts before and after are recorded in the review queue (28 before; the same 28 plus 3 new `problem_store` unit tests after).

Deviations and gaps (Minor, not blocking the tick):
- The change reaches outside the task's file list: `queue/claim.rs`, `worker.rs` and `executor.rs` now carry `problem_version`. It is needed to supply the version, but the task did not list those files.
- `PgProblemTests::load` has no `pg_` test (judge role, hidden rows only in Submit, pinned version, absent version returns `None`). Only the pure mapping (`pinned_spec`) has unit tests.
- The `None -> 0` pin mapping in `worker.rs` and the `problem_version` column read in `claim.rs` have no test of their own. The `problem_unavailable` outcome for an unpinned run is claimed in the commit message but not asserted.
- Needed follow-up: a `pg_` test for `PgProblemTests` and a worker test for the unpinned run.

## problem-catalogue C16: sandbox checks through PackRunner (34e3374) -> TICK CONFIRMED

All six verify cases have tests: reference matches, reference disagrees (sample and curated), reference timeout refused, ghost not reproduced, `RunnerError` as `Err`, retired value refused with hash prefix only. The file header states "not sandbox evidence". The check order SBX-01, SBX-02, SBX-03, retired follows the design. `PackRunner` stays in one file (purity test).

Deviations (Minor):
- `SandboxFill.hidden` is `Vec<HiddenFill>` with a `source` tag rather than `Vec<(input, expected)>`. The tag is needed for the `hidden_tests.source` column, so this is reasonable.
- Two `unreachable!` calls in production code (`shaped()` guarantees `Returned`). They are safe by construction but can be removed by matching once.
- The judge crate now depends on `counterspell-grimoire`. The design sanctions it (X9, C19 file list), but it cuts against the 15 Nov decision to keep authoring code out of the judge's sandbox path. Brighton should confirm.
- No property test. The spec did not require one for C16.

## problem-catalogue C17: one-transaction sandbox import (aba3b7d) -> TICK CONFIRMED

`pg_catalogue_sandbox_import_writes_hidden` (hidden count equals `hidden_count`, stamp set, ordinals continue after samples), `..._is_one_time` (second import is `Unchanged`) and `..._refusal_writes_nothing` all pass. The transaction takes the problem row lock, then inserts `hidden_tests`, `hidden_generation` and `hidden_sources`, then stamps and writes an event with counts only. Minor: `CaseSource::Sample` maps to `"sample"` and relies on the table CHECK to refuse it, with no test proving the rollback.

## problem-catalogue C18: validate-sandbox (1b70b6f) -> TICK CONFIRMED

`pg_catalogue_validate_sandbox` 4/4: stamps and publishes, refuses a second run with no write, refuses `private_tree_changed`, and a refusal writes nothing. The refusal is a value. The sandbox run happens outside the lock and `write_sandbox` re-checks under the lock. Deviation: the orchestration (`validate_sandbox`) lives in `sandbox_fill.rs`, not `write.rs`, so the purity test keeps passing; the task's file list names `write.rs`. Minor.

Related gap: through the CLI, `validate-sandbox` cannot run, because no runner is wired (see C19).

## problem-catalogue C19: CLI binary and exit codes (9cfd94d) -> UNTICKED

Passes: argument parsing, exit mapping (2 for a missing `--private-root` with no connection attempted), preflight before `connect()`. The bin has 11 unit tests, all green.

Fails or missing:
1. **Dry run writes to the database (CAT-I13, design P5).** `run_import` calls `sync_taxonomy` before any `--dry-run` check. I ran `import --dry-run` against an empty scratch database (migrations applied). The counts went from 0/0 tracks/concepts to 3 tracks and 8 concepts. CAT-I13 says `--dry-run` writes nothing, in any table. Fix: skip the sync in dry-run, and plan against the taxonomy file instead. Add the `pg_catalogue_dry_run_no_writes` test (P5) for the CLI path.
2. **The task's verify command cannot show a report without a database.** `cargo run -q -p counterspell-judge --bin counterspell-catalogue -- import --dry-run --packs packs --private-root <path>` exits 2 ("CATALOGUE_DATABASE_URL is not set") when the variable is unset. With a scratch database it printed three `refused:private_part_missing` lines and exited 1, as the spec allows. Dry-run needing a database is by design, but the review note says it was "verified live" without saying against which database. Any such run will have written taxonomy rows there (finding 1).
3. **`.expect("tokio runtime")` in `main`** breaks "never `unwrap()`/`expect()` outside tests". Return `ExitCode` 2 with a message instead.
4. **`unimplemented!()` stub (`unimplemented_runner`) and a dead `validate_one` path.** `runner_available()` is hard-coded `false`, so the assembled validate path never runs and is untested. This matches the spec's SBX-07 exit for a missing runner, but it ships a panic stub. Make the runner an `Option`/`Result` instead, or remove the dead path until the runner task lands.
5. **`load.rs` (530 lines of production logic: `read_pack`, `assemble_import`, `assemble_sandbox`) has no test of its own.** The only coverage is the pg suites that call the lower-level modules. This breaks the red/green rule. The unit tests `missing_private_root_exits_two` and `missing_root_dir_exits_two` assert only `is_err()`, not exit code 2, and `unimplemented_verbs_parse` checks only that arguments parse.
6. Not checked because no hidden values exist yet: report lines carrying hash prefixes only (CAT-PT9). Reviewed by reading; no leak found in the report builders.

To tick: fix 1, 3 and 4; add tests for `load.rs` (a fixture pack tree, including a missing private part and a changed manifest) and a pg dry-run test. Then re-run the gate and the verify command with a scratch database.

## grimoire-packs T28: publish starter packs (fc73f3c) -> NOT TICKED (human step)

The commit changes `status = "published"` in three `meta.toml` files, and its message records "Reviewed and approved for publication by Brighton Dube, 16 Nov 2026" as the task's verify requires. All three packs validate with `--deny-warnings`. I cannot confirm the person: every commit on the branch has the same git identity. The task is marked human-only and `tasks.md` still shows `[ ]`, so Brighton should tick T28 himself.

Open risks Brighton should decide (not agent work):
- `packs/<slug>/reference/` and `packs/<slug>/incorrect/` are still in the public repo for all three published packs. The 14 Nov 23:06 decision puts reference and incorrect solutions in the private repo. The private repo now exists (15 Nov). Moving them, and rotating any hidden values that were in public history, is still pending.
- Publication here is the pack `status` only. The catalogue importer still refuses publishing until a sandbox runner exists (C19 SBX-07), so nothing is publishable through the catalogue yet.

## Summary

| Task | Verdict | Reason |
|---|---|---|
| language-drivers T09 | Unticked | No test of public-only payload fields, no `pg_` test for `run_timings`, the `bench_start` CHECK or SSE replay |
| problem-catalogue C16 | Tick confirmed | Six verify cases pass; gates green |
| problem-catalogue C17 | Tick confirmed | 4/4 pg tests; one transaction, counts-only event |
| problem-catalogue C18 | Tick confirmed | 4/4 pg tests; refusals are values, no write on refusal |
| problem-catalogue C19 | Unticked | Dry-run writes tracks and concepts (CAT-I13); `expect` and `unimplemented!` in production; `load.rs` untested |
| problem-catalogue C26 | Tick confirmed | Verify passes; extra files and untested `PgProblemTests` noted as Minor |
| grimoire-packs T28 | Left for Brighton | Human step; sign-off in message, packs validate; public reference/incorrect files remain |
