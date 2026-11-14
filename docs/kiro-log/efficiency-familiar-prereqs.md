# efficiency-familiar: prerequisite audit (T01)

Recorded by the implementation run on 14 Nov 2026. No source was changed by this audit.

## Seam status

| Seam | Owner | Status | Evidence | Blocks |
|---|---|---|---|---|
| Migrations: run-lifecycle tables, guards, roles | run-lifecycle T05 to T07 | present | `migrations/20261113100000..200_run_lifecycle_*.sql` | T15, T16, T21 (DB side) |
| `FinalResult` (terminal write struct) | run-lifecycle T14 | present, no `measured` field | `crates/judge/src/queue/finish.rs:49` | T19 (add field) |
| `PublicEvent::Complexity`, `PublicEvent::Familiar` | run-lifecycle T04 | present, `BoundedJson` payloads | `crates/common/src/events.rs:232-234` | T04, T20 |
| `RunContext::emit` | run-lifecycle T12 | present | `crates/judge/src/queue/emit.rs` | T20 |
| Read model `GET /api/runs/{id}` | run-lifecycle T26 | present | `crates/server/src/runs/` | T23 |
| JSON error envelope | run-lifecycle T20 | present | `crates/server/src/error.rs` | T32, T38 |
| `Verdict`, `RunMode`, `RunStatus` | run-lifecycle T02 | present | `crates/common/src/status.rs` | T03 |
| `BoundedJson` | run-lifecycle T04 | present | `crates/common/src/events.rs:90` | T04 |
| Language-driver bench code, `run_timings` inserts | language-drivers | **missing**: no tasks done, no `languages/drivers/` | `.kiro/specs/language-drivers/tasks.md` (0 of 27 done) | T20 real collector, T22, T25, T40 |
| `hints::ai::generate(Kind::Familiar, …)`, `hints` table | hints | **missing**: no tasks done, no `Kind::Familiar` | `.kiro/specs/hints/tasks.md` (0 of 29 done) | T37, T38 |
| Judge sandbox runner, image digest | judge-sandbox | partial: 18 of 58 tasks done, runner trait `RunExecutor` exists, bollard backend not finished | `crates/judge/src/executor.rs`, tasks file | T22, T25, real bench in T40 to T41 |
| `PackRunner` with `bench` | grimoire-packs | partial: `PackRunner::run` exists; no `bench` method | `crates/grimoire/src/runner.rs:30` | T24, T25 |
| `web/static/familiar/` art | visuals phase | **missing**: `web/static` does not exist | filesystem | T26 (placeholder), T29 |
| `assets/pack-realistic/creatures/` art | visuals phase | **missing**: `assets/` does not exist | filesystem | visuals only (Open question 1) |
| `vitest` in `web/package.json` | run-lifecycle T32 | present (`vitest` 5.x, `npm test` = `vitest run`) | `web/package.json` | T27 to T34 |
| `gsap` in `web/package.json` | this spec | **missing**; must be added in T26 | `web/package.json` | T26, T30 |
| `web/src/lib/familiar/**` | ide-workspace T19/T20 | present: `Familiar.svelte`, `familiar-state.ts`, `types.ts` and tests | filesystem | T33 (extend), T39 |
| `auth` `AuthUser` extractor | auth | present | `crates/server/src/auth/extract.rs:16` | T32 |
| `users.familiar_prefs jsonb` | auth | present, `NOT NULL DEFAULT '{}'` | `migrations/20261113110000_auth_core.sql:20` | T32 |
| `fit_complexity` MCP tool | counterspell-mcp-power | **not started**; it will call `engine::complexity::fit` once T09 exists | `.kiro/specs/counterspell-mcp-power/` | none in this spec |
| Postgres for `pg_` tests | local | **not running** (`pg_isready`: no response on 5432) | shell | T15 verification, T16, T19 DB, T21, T23, T32, T37, T38 |

## Task table

| Task | Status in this run | Blocked by |
|---|---|---|
| T01 | done (this log) | none |
| T02 to T05 | implement | none beyond run-lifecycle T02 (present) |
| T06 to T14 | implement (engine and pure judge pieces) | none |
| T15 migration | written; DB verification blocked | Postgres not running |
| T16 `pg_` tests | blocked | T15 applied, Postgres |
| T17 `assess` | implement (pure) | T19 for the stored columns, not for `assess` itself |
| T18 load and gate | implement (pure core plus semaphore) | none |
| T19 terminal write | blocked on DB verification | T15, Postgres |
| T20 `measure_run` | partial: scripted collector only | real collector: language-drivers |
| T21 baseline store | blocked on DB | T15, Postgres |
| T22 `baseline` subcommand | blocked | language-drivers, judge-sandbox runner |
| T23 read model sections | blocked on DB | T19, Postgres |
| T24 grimoire SBX-05 and BEN-01 | implement with fake runner | grimoire-packs T19 (check) |
| T25 `PackRunner::bench` | blocked | language-drivers, judge-sandbox |
| T26 web prerequisites | implement with placeholder art | none |
| T27 to T31 mascot | implement pure parts and components | T26 |
| T32 prefs endpoints | blocked on DB for `pg_` cases; pure `prefs.rs` implement | Postgres |
| T33 to T34 browser state, explainer | implement | T23 for the read-model shape |
| T35 prompt builder | implement pure | hints `guard.rs` conventions (not present; follow design) |
| T36 claim guard | implement (pure) | T02 |
| T37 to T39 speech | blocked | hints (T37, T38), Postgres |
| T40 admission | blocked | language-drivers, judge-sandbox, Postgres, VPS |
| T41 real-stack proof | blocked | T40 and the VPS |
| T42 to T45 | blocked on the above (E2E and real stack) | as above |
| Stretch T46 to T48 | not started | `[STRETCH]`, skipped this pass |

## Notes

- The art is absent (design Open question 1). The placeholder path in T26 follows the contract and is labelled.
- `packs/` has `count-pairs`, `diagonal-total`, `longest-run-length` and `_test`. Their `[bench]` tables are not yet present, so no pack has bench enabled.
- No Postgres runs locally. `pg_` tests need a throwaway container, which this run does not start.
