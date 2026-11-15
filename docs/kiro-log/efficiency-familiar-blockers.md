# efficiency-familiar: blockers (run of 14 Nov 2026)

Tasks left unchecked, with the reason and what would unblock each.

## Blocked by missing prerequisites

| Task | Reason | Unblocked by |
|---|---|---|
| T15 migration | Not written. Postgres is not running locally (`pg_isready` got no response), so `sqlx migrate run` cannot be verified. | A throwaway Postgres container, and run-lifecycle migrations applied. |
| T16 `pg_` constraint, freeze and role tests | Needs T15 and a database. | T15 plus a database. |
| T19 terminal write with measured columns | Needs T15 for the columns and a database for the fenced transaction. | T15, Postgres. |
| T20 `measure_run` orchestration | Partly possible with the scripted collector, but the real collector comes from language-drivers, which has 0 of 27 tasks done. Not started this run. | language-drivers T08 and T09, and T17 and T19 first. |
| T21 baseline store | Needs T15 and Postgres. | T15, Postgres. |
| T22 `baseline` subcommand | Needs the judge-sandbox runner (18 of 58 tasks done) and language-drivers. | judge-sandbox runner, language-drivers T08. |
| T23 read-model sections and stream parity | Needs T19, Postgres and the server read model. | T19, Postgres. |
| T25 `PackRunner::bench` in the judge | Needs language-drivers bench and the judge runner. | language-drivers, judge-sandbox. |
| T32 prefs endpoints | Pure part not started. The `pg_` cases need Postgres. | Postgres for the database tests. |
| T37 speech orchestration | Needs hints `ai::generate` and the `hints` table (hints has 0 of 29 tasks done) and Valkey. | hints spec, Postgres, Valkey. |
| T38 `familiar-hint` route | Needs T37 and the hints table. | T37. |
| T40 admission on launch problems | Needs language-drivers, the judge runner, Postgres and the VPS. | Those, plus Brighton's VPS run. |
| T41 real-stack proof (30 runs each) | Needs the real sandbox and the VPS. | T40 and the VPS. |

## Not started this run (no hard blocker)

| Task | Note |
|---|---|
| T17 `assess` (pure) | Inputs exist (T03, T04, T05, T09, T14). Left for the next run to keep this pass within time. |
| T18 load guard and bench gate | Pure core is small. Not started. |
| T24 grimoire SBX-05 and BEN-01 | Needs grimoire-packs T19 checked first. |
| T26 to T31 web mascot | `web/static/familiar/` is absent, so a labelled placeholder is needed. GSAP is not in `web/package.json` and must be added (npm). Not started. |
| T27, T28, T34 (pure web modules) | Not started. |
| T33 browser state, T39 bubble | Depend on T23, T32, T38. |
| T35, T36 (speech prompt and claim guard) | Pure. Not started. |
| T42 to T45 | End-to-end and documentation depend on the above. E2E is deferred per the standing rule. |
| T46 to T48 | Stretch, skipped. |

## Decisions for Brighton

1. **Medium soundness.** The fit was calibrated (T12) by changing `se_k` from 2.5 to 1.0.
   Proptest found a rare Medium-two-classes-off case (about 4 per million linear series).
   Zero were found after the change, but a sample cannot prove zero. A stronger guarantee
   would need an algorithm change, for example requiring five or more points for `Medium`.
   The algorithm must not change without your decision.
2. **Jack-o'-lantern art** is absent (design Open question 1). Placeholder needed.
3. **Error envelope and `hints` table** are owned by other specs that are not yet implemented.

## Run 2 (14 Nov 2026, continuation): status and new blockers

### T15 migration: written, not verified, not committed
- File: `migrations/20261114120000_efficiency_familiar.sql` (untracked; not part of any commit).
- Blocker: no Postgres is available. `docker run postgres:17` was refused by the tool wrapper as a long-running daemon command, and only the Postgres client (no server) is installed locally, so no scratch cluster could start.
- Unblock: a throwaway Postgres 17 container started by Brighton (or an approved way to start one), then `psql -f` each migration in order, twice. Then tick T15 and commit the file.
- Also blocks T16, T19, T21, T23 and T32 (database tests and terminal write), because they need the T15 columns.

### T24 grimoire SBX-05 and BEN-01: partly done
- Done: `check_bench` runs `PackRunner::bench` on the declared sizes, fits with `engine::complexity::fit` in `Mode::Full`, and reports SBX-05 on a class mismatch or a `Low` fit. BEN-01 is raised for a size parameter that is not in the function, a negative size, or a size the schema cannot hold (including an impossible `unique` range). SBX-08 is no longer emitted. Runner faults still surface as SBX-06 (exit 2).
- Not done: the warning when bench sizes span less than 32× (R13.4). It needs a new warning code, but the codes test `only_three_codes_are_warnings` asserts the exact warning list, so it cannot be added without changing that test. Decision for Brighton: approve a new warning code (for example `BEN-02`) and the matching change to the codes test, which must cite a requirement ID that exists in `docs/requirements.md`.
- Test changes made in this run (please review):
  1. `crates/grimoire/tests/mutations.rs`: the static-mutation coverage filter now also skips `BEN-` codes, because BEN-01 is raised only by the sandbox stage, the same as `SBX-`. This is the narrowest fix, but it is a change to an assertion's scope.
  2. `crates/grimoire/tests/sandbox_fake.rs`: `declared_bench_warns_sbx_08_without_calling_the_bench_runner` is replaced by `declared_bench_calls_the_runner_and_a_fault_is_infrastructure`, marked `test-change: PCK-1`. The old test expected SBX-08 and no bench call, which R13.1 removes. PCK-1.3 in `docs/requirements.md` is the cited requirement.

## Run 3 (15 Nov 2026): T20 partly done

- T20 `measure_run` is implemented and tested with the scripted collector (`crates/judge/tests/familiar_measure.rs`, 8 tests: the failure matrix, event order, collector and stop errors). Not ticked, because three parts of T20 are still open:
  1. The canary test (hidden data absent from every payload) and the restart test are not written. The canary check belongs with the privacy tests (T43).
  2. The call from the real executor is not wired. It needs the language-drivers `bench.rs` collector (language-drivers T08 and T09).
  3. The `pg_` cases for T20 are not written, because they depend on T23 and the real collector.
