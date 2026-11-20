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

## Run of 16 Nov 2026 (attempt 3)

Done this run (ticked): C20 (9d232b5), C24 (ffa2596), C25 (824fe60), C27 (commit below).

- **C16 blocked on a design decision (not started).** C16 needs `counterspell_grimoire::PackRunner` in the judge crate, which means judge depends on grimoire. The 15 Nov routine decision in `brighton-decisions.md` keeps the judge crate free of authoring code. Brighton should choose one: (a) allow `counterspell-judge` to depend on `counterspell-grimoire` for the trait only, or (b) move `PackRunner` and `Target` into `counterspell-common`. Until then C16, C17, C18, C19, C19b, C21, C22 and C23 stay unchecked, because each depends on C16 or the CLI.
- **C21 and C22** also depend on C18 (publish needs validated versions; seeding needs the CLI).
- **C26 (BLOCKED on judge-sandbox T24).** The `ProblemTests` trait is not in the judge code yet. C24 and C25 use free functions in `problem_store.rs`. The trait wiring waits for T24.
- **C28 to C31 (not started, in order).** C28 needs the server's `PgProblemLookup`. C29 needs run-lifecycle T24. C30 and C31 need auth T11. These were not reached in this run. C27 is done.
- **C32 (BLOCKED on language accessor X10).** Not specified yet.
- **C33 (BLOCKED on D16).** Unchanged.
- **C34 to C37** depend on the C16 to C23 chain and are not reached.
- **C00a, C38, C39, C40** stay HUMAN and unchecked. No private repo or credential was read.

Test-integrity notes for review:
- C25: I appended the loader code before running the red test. The tests were written first and then run, but the red step was skipped. The tests passed after two fixture fixes: the testkit content hash now depends on the version (the unique key on `(problem_slug, content_sha256)` broke a second version), and `problem_concepts` inserts use `ON CONFLICT DO NOTHING`. These change fixture bytes only, not any assertion.
- C27: the seam test changed from sync `#[test]` to `#[tokio::test]` because `published` is now async. Its assertions are unchanged. One test was added (`in_memory_catalogue_reports_the_pinned_version`). Server test count: 392 before, 393 after.

Open design note: C30's `status` field and C31's no-leak tests depend on the auth resolver; not checked.

## Run of 15 Nov 2026 (attempt 5)

Done this run: C28 (commit 455e8db; review-queue entry added).

**C16 is still blocked (design decision, not reached).** C16 needs `PackRunner` from `counterspell-grimoire` in the judge crate. The 15 Nov routine decision in `brighton-decisions.md` keeps the judge free of authoring code, and the security note says do not invent a second crate layout. Brighton should choose (a) a judge dependency on grimoire for the trait only, or (b) move `PackRunner` and `Target` into `counterspell-common`. C17, C18, C19, C19b, C21, C22 and C23 depend on C16, so they stay unchecked.

**C29 is blocked on a server test fixture decision (not started, no code changed).** C29 makes `enqueue` insert the submission with `INSERT … SELECT … WHERE publication_state = 'published'`. Then every server test that queues a run for `two-sum` needs a real published problem. The server's lib-level test kit (`crates/server/src/testkit.rs`, `ensure_problem`) only inserts an unpublished row, because publishing needs the full child set (versions, two examples, four hints, sandbox stamp, hidden test). The only complete fixture is `counterspell_judge::catalogue::testkit::insert_published_fixture`, which the server can call only from test files (its dev-dependency). Options for Brighton:
- (a) call the judge fixture from each affected server test file (about 8 files; no dependency change);
- (b) make the server test kit depend on the judge test kit (adds an optional dependency edge, which the rules say to avoid without approval);
- (c) copy the fixture SQL into the server test kit (about 200 lines of duplicated test SQL).
This is the same work as C35. Until one is chosen, C29 and C35 stay unchecked. C30 and C31 go ahead because their tests can call the judge fixture directly from test files.

**Review queue:** C28 entry added to `problem-catalogue-review-queue.md`.

**C30 done (commit 7823ab3).** Two points differ from the design text, for Brighton to confirm:
- `attempted` counts any run the caller has (queued or running included), following CAT-A2's words "a submission exists". The design SQL joined only completed runs. `solved` still needs `status = 'completed' AND verdict = 'accepted'`.
- A well-formed track or concept slug that matches nothing returns an empty list, not `400`. Only malformed slugs and unknown difficulty values give `400 invalid_filter`.
- The 200-item cap is in the SQL but has no test (it would need 201 published fixtures). The track sort order is also untested. Both are noted for the review.

**Remaining tasks after C30 (not started):**
- C31 (`GET /api/problems/{slug}`) lists C17 as a dependency. C17 waits on C16, so it stays unchecked.
- C32 needs the language accessor X10, which is not specified yet.
- C33 waits on D16 (the `ghosts` table owner).
- C34 needs C21 and C22, which wait on C18 and C19 (C16 chain).
- C35 needs C29, which waits on the fixture decision above.
- C36 and C37 need everything before them.
- C38, C39 and C40 are HUMAN (operator import, Brighton's approval, rotation). C00a is still waiting on the private repo.
- Nothing here changes a fixed decision or touches the private repo.

## Run of 15 Nov 2026 (attempt 6)

Done this run (ticked): C31 (commit 209557e). Server unit tests, server `pg_` tests and judge `pg_catalogue_*` tests all pass.
- Test-change note: the judge test kit's stored signature (`insert_version_row` and `static_import`) now uses grimoire's full shape (`function`, `params`, `returns`), and the constraints expectation in `pg_detail_returns_the_pinned_public_fields` is `[]` to match the fixture. No assertion on the existing behaviour changed.
- Gap: the no-leak test does not yet plant a canary in a synthetic private fixture, because `crates/judge/tests/fixtures/private/` does not exist (it belongs to C19b). The canaries come from the judge test kit instead.

Still open, with the reason for each:
- C16 (sandbox checks through `PackRunner`) needs a decision from Brighton. Option (a): judge depends on grimoire for the trait only. Option (b): move `PackRunner` and `Target` into `counterspell-common`. The 15 Nov routine decision keeps the judge free of authoring code, so option (b) fits it better, but it changes grimoire's public API. Until he chooses, C16, C17, C18, C19, C19b, C21, C22 and C23 stay open.
- C26 is blocked on judge-sandbox T24 (not started).
- C29 (enqueue pins the version) is blocked on the fixture decision recorded in attempt 5: (a) call the judge fixture from each affected server test file, (b) add a dependency from the server test kit to the judge test kit, or (c) copy the fixture SQL. The implementation itself is not blocked. Option (a) needs no new dependency. Not started this run because it touches about eight test files and is best done with his choice.
- C32 needs the language accessor X10, which is not specified.
- C33 needs the `ghosts` table owner decision (D16).
- C34 to C37 depend on the open chain above.
- C00a, C38, C39 and C40 are HUMAN. Brighton has not created the private repo, so nothing reads it.

## Run of 15 Nov 2026 (attempt 7)

Done this run (ticked): C29 and C35 (commit 925ac4a), C23 (commit ebb3738). C29's option (a) needed no new dependency: the server already dev-depends on the judge test kit.

Counts: server suite 435 passed, 0 failed with `--include-ignored` (3 new `pg_catalogue_pin` tests). Judge `pg_` suite 110 passed, 0 failed.

Still open, with the reason for each:
- **C16 (sandbox checks through `PackRunner`) still needs Brighton's decision. This is a conflict between two approved documents, so I did not choose.**
  - C19 and the spec's X9 say the judge crate depends on `counterspell-grimoire`.
  - The 15 Nov 00:55 routine decision in `brighton-decisions.md` says the judge must not depend on grimoire.
  - The trait `PackRunner` is in grimoire and uses grimoire's `Target`. Its SBX checks (`grimoire/src/sandbox.rs`) reuse grimoire types, so option (b) would move more than the trait.
  - Options: (a) judge depends on grimoire, which matches the spec and breaks the 15 Nov decision. (b) move the trait, `Target` and `RunnerError` into `counterspell-common`, with `common` depending on `engine`, and grimoire re-exports them. The SBX check logic would still need to move, or be duplicated, for the judge to run them.
  - Recommendation: (a) for the trait and SBX logic together, with the 15 Nov decision amended, because option (b) duplicates or moves a large amount of logic. Brighton decides.
- C17, C18, C19, C19b, C21, C22 depend on C16, so they stay open. C34, C36 and C37 depend on those.
- C26 is BLOCKED on judge-sandbox T24 (not in the code).
- C32 needs the language accessor X10, which is still not specified. The `LanguageCatalog` has the byte limits and enabled flags, but no display name, and the spec does not say where display names come from.
- C33 is BLOCKED on D16 (the owner of the `ghosts` table).
- C00a, C38, C39 and C40 stay HUMAN and unchecked. Nothing read the private repo.

## Run of 15 Nov 2026 (attempt 8, resumed)

No task was ticked in this run. Every unchecked task is either blocked on a decision only Brighton can make, or depends on one.

Unchecked tasks and the reason for each:
- **C16 (first unchecked, non-stretch, non-HUMAN task). Waiting on Brighton's grimoire dependency decision.** Same conflict as attempts 3, 5 and 7. Choose one:
  - (a) `counterspell-judge` depends on `counterspell-grimoire` for `PackRunner`, `Target` and the SBX checks. Matches X9 and the C19 text. Amends the 15 Nov 00:55 routine decision in `brighton-decisions.md`.
  - (b) move `PackRunner`, `Target` and `RunnerError` into `counterspell-common`, and have grimoire re-export them. Keeps the 15 Nov decision. The SBX check logic would still need to move or be duplicated, which the crate structure steering treats as moving responsibilities.
  - Unblocks: C16, C17, C18, C19, C19b, C21, C22, C34, C36, C37.
- **C17, C18, C19, C19b, C21, C22, C34, C36, C37**: depend on C16 directly or through the chain above.
- **C32 (language display names)**: waits on the language accessor X10, which the spec does not define. `LanguageCatalog` has byte limits and enabled flags but no display name. Brighton or the language-drivers spec needs to name where display names come from.
- **C26**: BLOCKED on judge-sandbox T24 (not in the code).
- **C33**: BLOCKED on D16 (owner of the `ghosts` table DDL).
- **C38, C39, C40, C00a**: HUMAN. The private repo does not exist yet, so nothing reads it.

Working tree at the start of this run: only automation log files and `docs/KIRO_USAGE.md` were modified. None of them belong to this spec, so none were committed.

## 17 Nov 2026 verify all: follow-ups
- C19b: `pg_approval_gate_runs_the_sandbox_only_after_approval` (`crates/catalogue-cli/tests/pg_catalogue_approval_gate.rs`) fails after the pack-tree split; it uses the public-only `validate_sandbox` and gets `LAY-01`/`REF-01`. Use `validate_sandbox_pair` with the private root.
- Test helpers `catalogue_url` (catalogue_cli.rs, pg_catalogue_approval_gate.rs and the other pg catalogue tests) drop the password, so they only work against a trust-auth Postgres. Passing locally needs `PGPASSWORD`. Minor.
- `crates/grimoire/tests/mutations.rs` panics (`copy known_ghosts.toml`) instead of skipping when `COUNTERSPELL_PACKS_PRIVATE_ROOT` points at a scaffold with no `known_ghosts.toml`. The dev env points at the empty scaffold in `~/code/counterspell-packs-private`; the populated checkout is the sibling in `Personal Projects/`. Minor, but it turns a missing checkout into a red suite.


## 17 Nov 2026 - grimoire-packs T20 verify-line amendment

T20's verify line named `counterspell-grimoire validate packs/count-pairs --sandbox`. After the pack-tree split (0330f58/176cf9a, purged history), `reference/`, `incorrect/` and `known_ghosts.toml` live only in the private checkout, so the single-tree `validate` can no longer load a starter pack. The verify line now reads `counterspell-catalogue validate-sandbox <slug> --private-root <checkout>`, which runs `validate_sandbox_pair` over the merged pair - the same checks, on the production path. Evidence: `sbx_pack_runner::starter_packs_pass_sandbox_checks_with_the_real_runner` (114 s, real daemon) passes all three packs through `validate_sandbox_pair`.

## 19 Nov 2026 (Brighton): C00a, C39, C33 updates
- **C00a ticked:** `BrightonDube/counterspell-packs-private` and deploy key `~/.config/counterspell/packs-private-deploy-key` already verified (15 Nov 2026).
- **C39 ticked:** Brighton approved `count-pairs`, `diagonal-total`, `longest-run-length`; `packs/<slug>/approval.toml` written in `counterspell-packs-private` and verified via `counterspell-catalogue import --static-only --dry-run` (all 3 accepted: `6117438e8c95`, `075a3ea68459`, `a2188cd18387`).
- **C33 unblocked:** `GhostLookup` uses `submissions.ghost_source_id` / `submissions.ghost_input` (established by `ide-workspace` T02/T25).

## Unblock decisions, 20 Nov 2026 (Brighton, in chat)

- **PA08 (X1/X2).** Brighton approved the drafted amendment with edits; the approved
  text is recorded in `.kiro/specs/grimoire-packs/tasks.md` under "Amendment X1 + X2
  (approved by Brighton, 2026-11-20)". PA08 proceeds.
- **PA20 open questions.** Brighton confirmed the assumed answers: Q3 = private repo
  `BrightonDube/counterspell-packs-private` with a read-only deploy key kept outside the
  repo (already in place since 15 Nov); Q6 = the originality list lives in the private
  repo (drafted from `docs/licence-rules.md` in this session, Brighton to review);
  Q7 = authoring agents do not open third-party problem statements.
- **PA27.** Brighton approved the code-running tools for this session (PIP-39): the
  agent runs S10 and S11 for the three launch packs through the real rootless sandbox
  and records the image digests in each pack's state.
- **PA28.** The launch packs are `published`; the approval commit and `approval.toml`
  were confirmed by Brighton on 18 Nov and the importer accepts the recorded hashes
  (verified 20 Nov). The task's verify legs are met.
- **Grimoire T29.** Brighton's call on design open question 11: convert `packs/_test/`
  to the two-tree format now (ghost-engine has shipped).
- **Catalogue C40.** Brighton chose full rotation: every hidden value published in
  public git history at grimoire T21–T23 (known-ghost inputs/got values, the reference
  and the incorrect programs) is retired; all three packs get new hidden cases, a new
  generator seed and a `pack_version` bump. Open question 21: do NOT rewrite public
  git history — rotation invalidates the leaked values.
- **MCP T16/T17.** The agent runs the scripted find/fix/re-verify session through
  kiro-cli and files the transcript under `docs/kiro-log/chats/`; screenshot captures
  for T17 remain Brighton's leg in the Kiro IDE.
- **Deploy T23–T32.** The shared-host path superseded the dedicated-VPS order: the host
  (15.204.92.66, shared with Evorayn) answers SSH with publickey-only auth, Caddy is
  live, the four `production` environment secrets and `DEPLOY_ENABLED=true` are set, and
  Deploy runs queue behind the environment's required reviewers. Remote evidence is
  recorded in `deploy/vps/verification-report.md`; legs that need an on-host session
  (rootless facts, env file modes, Postgres roles, backup/restore, monitors, rollback
  rehearsal) stay marked as recorded there — nothing is claimed without evidence.

## C40 rotation mechanics — done by agent, records pending Brighton (2026-11-20)

The rotation's mechanical work is complete; the records and re-approval are
Brighton's writes per the spec (C40: "Brighton records each retired hidden
value's SHA-256" and "agents do not write the `[[rotated]]` record").

**Done in the trees**

- Public metas: `[generator]` table removed (X1: it is private-only and would
  now be LAY-05); `pack_version` bumped 4→5 (count-pairs), 5→6
  (diagonal-total), 6→7 (longest-run-length); `status` set back to `draft`
  because the recorded approvals cover the old versions — rotation needs
  Brighton's re-approval (grimoire T28 / PIP-35).
- Private repo (uncommitted, in `/home/brighton/code/counterspell-packs-private`):
  `generator.toml` with new seeds (4815 / 6072 / 8934 — old seeds 1729 / 2718 /
  3141 leaked at T21–T23) and fresh `hidden.toml` curated sets (4 / 4 / 5
  cases, hand-derived, all validate clean).
- `_demo/mcp-dogfood` migrated the same way: `[generator]` moved out of the
  public meta into the private `generator.toml`.

**Leaked-value audit**

- `hidden.toml` and `generator.toml` were NEVER committed publicly
  (`git log --diff-filter=A` confirms zero public history).
- Leaked: the three `[generator]` seeds (1729/2718/3141) and the six
  known-ghost input/got pairs, at grimoire T21–T23.
- The leaked seeds let anyone regenerate the old generated cases; the new
  seeds retire that whole class. The curated `[[case]]` sets are new.

**Proposed `retired-values.toml` content (Brighton writes)**

Each `[[retired]]` sha256 is `canonical_hash(input, expected)` from
`crates/judge/src/catalogue/retired.rs`. For every leaked ghost the input is
retired with its true answer (so the leaked input can never return as a
hidden case); the three wrong-answer ghosts additionally retire the literal
leaked (input, got) pair.

- count-pairs — `same-index` input `[[0], 0]` (correct 0, leaked got 1):
  `71b3f088054cd962bfac6c2f6475ad6580947168fa448f6b0411cb769945267c`,
  `733e1974103028bd96ea470260f9823c7005b0b53ec921c4e35cde1221ca139b`
- count-pairs — `empty-min` input `[[], 0]` (correct 0):
  `41ef8b5a8af27e052042d040904325971b62ccc8a020632269c718b0e7c27cec`
- longest-run-length — `forgot-last-run` input `["a"]` (correct 1, got 0):
  `c44d245ca0a07a179438471af81c0888e4c541dfbf9b23e741819f87494fbf54`,
  `efb49a0fef08a0f67fcca6755db13ae0bb3173c80c965704c81dbd4a095897b5`
- longest-run-length — `empty-first-char` input `[""]` (correct 0):
  `ea54ac556f1bf03c25620539b9fe37e11ede28ded5a9f23151c6b6c3cac11a50`
- diagonal-total — `abs-diagonal` input `[[[-1]]]` (correct -1, got 1):
  `58ee6bc87139911f47ce28ac246c1e806f6846f0cb8820821ee2b4a187d97485`,
  `8e382855831c8784fb9d58ec93e9ad21f70dca7b95ee04c6c35083a51bb89760`
- diagonal-total — `assumes-square` input `[[[]]]` (correct 0):
  `930d6f0b9b402763552c4ff58c9ca658baf3fc7b13042442a1b6e953f91339a6`

`[[rotated]]` records needed (Brighton writes):
`count-pairs` pack_version 5, `diagonal-total` pack_version 6,
`longest-run-length` pack_version 7, all `rotated_on = "2026-11-20"`.

**Still Brighton's legs for C40/PA28**

- Paste the `[[retired]]` entries (or authorize me to write them), write the
  three `[[rotated]]` records, and re-approve the rotated packs (new
  `approval.toml` content hashes + `status = "published"`).
- The stale `approval.toml` files (v4/v5/v6 hashes) intentionally remain —
  they record the approvals that did happen for those versions.
