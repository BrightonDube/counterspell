# problem-catalogue blockers and approvals

## C00 approval — 15 Nov 2026 10:26 (Brighton)

Brighton approved this spec for implementation. Details are also in `.kiro/steering/brighton-decisions.md`.

Decided now:

- The pack importer uses its own limited database role, not the database owner (security default). This overrules D4 / open question 5 (owner login, no new role).
- Open question 22, on the role only: the approved v2 hardening brief's `counterspell_importer` role wins. Table names stay as in this spec (`hidden_tests`, `known_ghosts`) unless a later task must reconcile them.

Everything else listed under C00 that Brighton did not name keeps the assumed choice in `design.md`. Stop only for a serious security, licensing-risk, cost or production-data decision that is not already settled.

## C00a — still waiting on Brighton

C00a stays unchecked. Brighton has not yet created `BrightonDube/counterspell-packs-private` or its read-only credential. No agent may create the repo, a key or a token. Tasks that need that repo should be left unchecked, recorded here, and skipped until he has done C00a.

## Run of 16 Nov 2026 (implementation pass)

Done this pass (ticked in tasks.md): C01, C02, C03, C04, C05, C06, C07, C08, C08a, C09, C10, C11, C11a, C12, C13.
C03 repair: C03's submission foreign key broke 75 existing database tests. The repair is in `be2ca27`.

Not started (remaining work, in tasks.md order):
- C14 to C23: the importer writes, dry run, sandbox fill, validate-sandbox, CLI, taxonomy sync, purity guard. C16 depends on `PackRunner` (exists in `counterspell-grimoire`, `runner.rs`). The pure parts can start now. Sandbox runs need the rootless daemon and must stay open until T37.
- C19b to C22: the CLI and seeding tasks. C19b needs the approval-gate fixtures from C11a and C19.
- C24 to C32: the runtime read path and API. They depend on judge-sandbox T24 (`ProblemTests` seam), which is open.
- C26: BLOCKED on judge-sandbox T24 (open).
- C33: BLOCKED on D16 (the owner of the `ghosts` table DDL is unresolved).
- C34 to C37: runbook, cross-spec test helpers (X2, X13), full pass and repo-wide gates. Depends on the tasks above.
- C38, C39, C40: HUMAN. The sandbox import of the approved starter packs, the approval gate before publication, and rotation of hidden values in public git history. Left for Brighton.
- C00a: HUMAN, still unchecked. Brighton has not created `BrightonDube/counterspell-packs-private`. Nothing in this pass reads that repo.

Design gap noticed in C02: the publish gate cannot see an unknown track, because `problems.track_slug` is a foreign key. An unknown track is refused by the key (23503), not by the gate. The test `pg_catalogue_publish_gate_unknown_track_refused_by_key` records that. Brighton may want the gate's wording changed in the spec.

Left untracked on purpose: `crates/server/tests/pg_auth_tokens.proptest-regressions`, a seed file written by a proptest run that failed on the stale migration embed (`VersionMissing`). It is not part of this work.

## Run of 15 Nov 2026 (attempt 2)

Done this run: C14 (static import transaction, commit f23934f) and C15 (read-only dry run, commit 115bfd0).
- C14 tests: `pg_catalogue_import_idempotent`, `pg_catalogue_versions_history`, `pg_catalogue_refusals_write_nothing`, `pg_catalogue_rollback_on_failure`.
- C15 tests: `pg_catalogue_dry_run_no_writes`, `pg_catalogue_dry_run_reports_generator_change`, plus unit tests for `diff_files`.
- The pack builder moved into `catalogue/testkit.rs` as `static_import`, so the test files share it. No test assertion changed.

Not started, in tasks.md order: C16 (sandbox checks through `PackRunner`), C17, C18, C19, C19b, C20 to C23, C24 onward. Nothing is blocked on Brighton for C16 to C23 except sandbox runs, which stay open until T37 on a real rootless daemon. C00a and the HUMAN tasks C38 to C40 are unchanged.

Open design note from C14/C15: the `VersionNotBumped` changed-file list covers public files only, because `problem_versions.manifest` holds public entries only (CAT-PT9). A private-only change still refuses, but its path is not named. Brighton should decide whether the refusal should name private paths by hash prefix.
