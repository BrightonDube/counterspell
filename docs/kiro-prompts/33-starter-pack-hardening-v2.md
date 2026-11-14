DRAFT - awaiting Brighton's approval

# 33 v2 - Starter packs: hardening, curated hidden tests, pack import, AI authoring

Replaces the ON HOLD brief `docs/kiro-prompts/33-starter-pack-hardening.md` (kept unchanged as Brighton's original). Written 14 Nov 2026 from the repo state on `brighton/kiro-setup`.

## How to run this brief

- Read `.kiro/steering/brighton-decisions.md` first. Decide reasonable design choices yourself, record them in the task notes, and continue. Stop for Brighton only on security, licensing, cost or production-data decisions. Write any such question to `docs/kiro-log/<spec>-blockers.md` and carry on with work that does not depend on it.
- Strict TDD with the workspace and global skills (`.kiro/steering/tdd-and-quality.md`). Red, green, refactor. Expected values are hand-derived literals. Test integrity rules apply. Changing an existing test needs a `test-change: <REQ-ID>` marker with an ID from `docs/requirements.md` (for example `GHO-6` or `PCK-2`).
- Extend existing code. Do not build a second validator, generator, hint ladder, sandbox path or schema. The parts that already exist are listed under "What already exists" below.
- No browser or E2E tests. Use the Rust test harness and the `pg_` ignored-test convention against a throwaway local Postgres container.
- Packs stay `status = "draft"`. Agents never set `published`.
- Commit locally in small TDD commits. Push only after a spec is finished with all its tests passing. Never push to `main`. Never deploy, never touch servers, never run the importer against any database except a throwaway local one, never spend money.
- Do not edit `scripts/kiro-implement-all.sh`, `packs/_test/` or `.kiro/specs/ghost-engine/`.

## Settled decisions (14 Nov 2026)

- Hidden tests and reference solutions stay in this public repo for now (accepted by Brighton on 14 Nov 2026). He plans to make the repo private later. No private store is needed now. The database remains the serving source; the importer still copies published packs into Postgres.

## Why v2 (what changed from the original)

Learning comes first and grading supports it. The original brief asked for good things, but much of sections 3 to 5 already exists, curated edge cases had no home in the pack format, and nothing loads packs into Postgres. Brighton also does not want to write problems himself. v2 therefore has four parts:

1. Harden the three starter packs (wording, worked examples, general hints, a realistic diagonal ghost).
2. Add curated hidden test cases to the pack format, with validator and importer support.
3. A new `pack-import` spec that loads validated, published packs into Postgres, so problems can change at any time without rebuilding or restarting the app.
4. An AI pack-authoring pipeline: an agent drafts, the validator checks, Brighton approves or rejects. Target: 10 polished problems by Sun 29 Nov.

## What already exists (read it, do not rebuild it)

- `crates/grimoire/` (`counterspell-grimoire`): `validate_static`, `validate_sandbox(root, &mut dyn PackRunner)`, the CLI `validate [--json] [--deny-warnings] [--sandbox] [--all]`, codes in `codes.rs`, `Meta` with `#[serde(deny_unknown_fields)]` in `meta.rs`, `ProblemRecord` and `record::build` in `record.rs`, the `PackRunner` trait in `runner.rs`, sandbox checks SBX-01 to SBX-08 in `sandbox.rs` (hidden inputs come from the engine `CandidateStream` there), and tests `starter_packs.rs`, `mutations.rs`, `sandbox_fake.rs` (model runner), `record.rs`, `layout.rs`, `purity.rs`, `cli.rs`, `pipeline.rs`. This covers original-brief sections 3 and 4.
- Hint ladder: four rungs enforced by `hints.rs` (GRM-26, GRM-27). The hints spec (`.kiro/specs/hints/`) defines the `problem_hints` table ("written only by the pack importer; the server role gets SELECT only") and the reveal-one-rung-at-a-time gate. This covers original-brief section 5.
- Hidden-test safety in the judge: samples run before any hidden test, and from the first hidden test onward only verdict and time are public (judge-sandbox amendment A1, `crates/judge/src/sandbox/records.rs`, `run.rs`). `test_results` has a CHECK that hidden rows carry no detail (`migrations/20261113100000_run_lifecycle_tables.sql`).
- Seams waiting for a catalogue: `ProblemLookup` / `EmptyCatalogue` in `crates/server/src/seams.rs` (synchronous `fn published(&self, slug) -> Option<ProblemRef>`), and `ProblemTests` / `ProblemSpec` in the judge-sandbox design (async `load(slug, mode)`, `reference_python` used only by `SandboxOracle`). `ProblemTests` is designed but not implemented yet.
- An `ide-workspace` task already has a client loader for `GET /api/problems/{slug}` (`web/src/lib/api/problems.ts`) with strict response validation. No server route exists yet.
- `meta.toml` already has optional `reference_complexity`, `[bench]` and `complexity_note` (240 characters max, GRM-18). `reference_complexity` requires `[bench]` (GRM-16, MET-09).

What does **not** exist: any `packs`, `problems`, `concepts`, `problem_concepts`, `test_cases` or `reference_solutions` table (none in `migrations/`); a catalogue spec (run-lifecycle, judge-sandbox, hints and dashboard-fsrs all point to one); storage for authored known ghosts at run time; an importer; a curated hidden-test format; fields for estimated time, learning objectives, prerequisites or an editorial.

---

## Part 1 - Harden the three starter packs

Packs: `packs/count-pairs`, `packs/diagonal-total`, `packs/longest-run-length`. Keep slugs, function names, signatures, schemas, seeds, `hidden_count = 12`, limits and the existing 3 samples each (GRM-21 allows 2 to 5). Bump `pack_version` only if the import spec is already merged; otherwise leave it at 1.

### All three statements (`problem.md`)

Keep the five required headings in order (GRM-19), with no raw HTML (GRM-20). For each:
- Say "integer" where negatives are allowed. Use "integer" for every numeric output too, for consistency.
- `## Task` opens with the exact call, for example `count_pairs(values, target)`, names each argument and its type, and says what to return.
- `## Examples` replaces the placeholder "The public examples are listed in the pack's samples." with at least one worked example: the call, the result, and two to four lines explaining why. Use sample 1, so the statement and samples agree.
- State the edge cases in plain words (empty input and so on) and what to return for them.
- `## Constraints` must match `input_schema` exactly.
- Short sentences for beginners. Original wording only (GRM-49).

Pack-specific content:
- **count-pairs:** a pair is two different positions i < j, not two different values. Equal values at different positions count (`[5, 5]` with target 10 gives 1). (i, j) and (j, i) are the same pair and count once. An empty list or a single value gives 0.
- **diagonal-total:** the main diagonal is the cells `grid[i][i]`. The grid can be rectangular, so the diagonal stops at the shorter side (`min(rows, columns)` cells). Show a wide example such as `[[-2, 9, 1]]` → -2. No rows, or rows with no columns, gives 0.
- **longest-run-length:** a run is consecutive equal letters. Runs can be anywhere, including the end of the text. Equal-length runs give that length. Empty text gives 0.

### Hints (`hints.toml`)

Four rungs, general, never assuming the learner's specific bug. The authored ladder is the same for every learner. Personalised help is the AI hint's job in the hints spec. Rung meanings: 1 conceptual nudge, 2 the state or indexing idea, 3 the algorithm in words or pseudo-code (code fence of 3 lines at most), 4 a near-solution step without full code.
- count-pairs rung 4 currently says "Your inner loop may start at the same index as the outer loop." That assumes a bug. Replace it with a general near-solution step, for example: for each position, look only at the positions after it, and add one when the two values reach the target.
- diagonal-total rung 4 currently says "keep each value's sign". That was written for the `abs-diagonal` ghost being removed. Rewrite it in general terms (how many diagonal cells there are, and adding exactly those).
- longest-run-length: keep the idea of rung 4 but phrase it as a general step (update the best run after every letter) rather than a correction.
- Update the rung labels in grimoire-packs `requirements.md` GRM-26 from "R4 targeted fix" to "R4 near-solution step". This is a docs-only change, so no test changes.

### Known ghosts

- **diagonal-total:** remove `abs-diagonal` (nobody writes `abs()` there by accident). Replace it with a realistic wrong-answer bug, `both-diagonals`: the learner also adds the anti-diagonal, `total += grid[i][i] + grid[i][size - 1 - i]`. Hand trace: on `[[1]]` the program returns 2 and the reference returns 1, so use `input = [[[1]]]`, `got = 2` (verify by hand and record the trace in the task notes). Concepts: `["matrices", "indexing"]`. The explanation describes the mistake without words from the engine's forbidden list (GRM-31) and without claiming the input is the smallest. Note: swapping rows and columns does **not** change the main diagonal (`grid[i][i]` is the same cell either way), so it is not usable as a wrong-answer ghost. Keep `assumes-square` (crash on `[[[]]]`).
- Replace `incorrect/abs-diagonal.py` with `incorrect/both-diagonals.py`. Update the model function and vectors in `crates/grimoire/tests/sandbox_fake.rs` (marker `test-change: GHO-6`), the starter table in grimoire-packs `design.md`, and the T23 note in `tasks.md`.
- count-pairs and longest-run-length ghosts stay. Re-read each explanation against its program and fix any wording that does not match what the program does.

### Complexity

Set `complexity_note` (240 characters max) in each `meta.toml`: Count the Pairs "O(n^2) time, O(1) extra space"; The Diagonal Total "O(min(rows, columns)) time, O(1) extra space"; The Longest Run "O(n) time, O(1) extra space", each with one short reason. Do **not** add `reference_complexity` yet. It needs a `[bench]` table (GRM-16), and the sandbox would then emit SBX-08 ("complexity not checked"), which fails CI under `--deny-warnings`. The efficiency-familiar spec adds `[bench]` and `reference_complexity` once the fit exists (grimoire-packs open question 8).

### Deferred (needs new fields or columns, so not in this brief)

Estimated completion time, learning objectives, prerequisite concepts, and a longer editorial explanation of the reference. List them in the changelog as later items.

---

## Part 2 - Curated hidden test cases

Today the hidden tests are only `[generator]` output from a seed. Random generation rarely hits the exact edge cases. Add a hand-picked hidden list.

### Format (new): optional `hidden.toml` in the pack

A separate file, not a table in `meta.toml`, so everything server-only sits in one file that the importer never sends to a public surface.

```toml
[[case]]
input = [[], 0]          # argument list in parameter order, same convention as samples
expected = 0             # hand-derived; the reference must agree (sandbox check)
why = "empty list"       # author note, 1-120 chars, never shown to learners
```

- Parsed with `#[serde(deny_unknown_fields)]` like `meta.toml`.
- Add `hidden.toml` to the layout allow-list in `pack.rs` (LAY-02) with a 32 KB cap (LAY-04). Update GRM-1, GRM-2 and GRM-4 wording and `packs/README.md`.
- `[[samples]]` and `[generator]` are unchanged. Hidden tests become the curated cases plus the generated ones. GRM-23 (`hidden_count` 5 to 50) still applies to the generated part only.
- Test order at import: samples, then curated hidden, then generated hidden (ordinals in that order). The judge already runs samples first (A1).

### New validator checks (codes go in `codes.rs`, each with a negative fixture and a `mutations.rs` case, GRM-51)

- **HID-01 (error):** `hidden.toml` has a TOML error or unknown key, has 0 cases, or has more than 30.
- **HID-02 (error):** a case input is rejected by the schema, `expected` does not match `returns`, or `why` is empty or too long.
- **HID-03 (error):** duplicate case inputs, or a case input equal to a sample input (an edge case must not also be public).
- **HID-04 (warning):** a non-`_` pack has no `hidden.toml` or fewer than 5 cases. Under `--deny-warnings` this makes curated edge cases mandatory for catalogue packs.
- **SBX-09 (error, sandbox):** the reference disagrees with a curated `expected`. Reuse the SBX-01 comparison code. Never replace the authored value.
- **STM-03 (error):** `## Examples` contains no call to the pack's `function(` (this catches the placeholder and forces a worked example).
- Diagnostics for hidden cases cite a pointer such as `hidden.toml /case/3`, never the input or the expected value (GRM-25). Add a test with a canary value in a hidden case that asserts the canary is absent from text and JSON output.

### Curated cases for the three packs (expected values hand-derived; at least these)

- **count-pairs:** empty list; single value; `[0]` target 0 (the same-index ghost input); all equal values (`[2, 2, 2, 2]` target 4 → 6); several valid pairs; negatives; zero target with opposite values; no matching pair; 30 values (max length); target at -100 and 100. Also `[[], 0]` as the empty-min ghost input.
- **diagonal-total:** `[]`; `[[]]` and `[[], []]` (rows with no columns); single cell; negative diagonal values; wide (1×6, 2×5); tall (6×1, 5×2); square 6×6 (max); `[[1]]` (the both-diagonals ghost input).
- **longest-run-length:** `""`; `"a"`; 40 identical letters (max); `"abababab"`; longest run at the start; longest run at the end (`"abccc"` → 3); two equal longest runs; a 40-letter mixed text.

Until the judge's `PackRunner` exists (grimoire-packs T20, blocked on judge-sandbox), check every curated case with the model runner in `sandbox_fake.rs`, following the T25 pattern. Label it in the file header as not evidence that the sandbox works.

Also fix the generator side if needed: confirm with a test that every candidate drawn for `diagonal-total` is rectangular and every generated input satisfies its schema. GRM-24 already checks 200 candidates with `Schema::contains`. Add a rectangular-matrix assertion only if `contains` does not already enforce it.

---

## Part 3 - New spec `pack-import` (the missing catalogue)

Make this a **new spec**, `.kiro/specs/pack-import/` (`requirements.md`, `design.md`, `tasks.md`), in the same style as the other specs. The grimoire-packs scope says that crate never touches a database or `migrations/` (GRM-54), so it does not belong there. This spec is the "catalogue spec" that run-lifecycle, judge-sandbox, hints and dashboard-fsrs refer to.

### Data flow (put this in the design, in plain English, for Brighton)

```
packs/<slug>/ in git (draft -> Brighton approves -> published)
  -> counterspell-grimoire: validate_static + validate_sandbox (judge PackRunner)
  -> importer: builds an import bundle, runs the reference in the sandbox for generated hidden expected values
  -> Postgres: packs, problems, concepts, problem_concepts, test_cases, reference_solutions,
     problem_hints, problem_known_ghosts
  -> server (SELECT, public data only): GET /api/problems, GET /api/problems/{slug}, ProblemLookup
  -> judge (SELECT, all test data): ProblemTests -> sandbox run -> test_results / run_events -> SSE to browser
```

### Requirements to cover

1. **Import bundle (grimoire, pure).** Extend `record.rs` with a bundle built only from a pack with zero static errors. It holds the existing `ProblemRecord`, plus status, samples, curated hidden cases, generated hidden inputs, hints, known ghosts, the reference source and a SHA-256 content hash of the pack files. Generated hidden inputs must come from the same function `sandbox.rs` uses, so validation and import see identical inputs. Extract that function rather than copying it. No sqlx, no I/O beyond the existing `pack.rs` reads (GRM-54, `purity.rs`).
2. **Only published packs.** `status = "draft"` packs and `_`-prefixed directories are never imported. A pack must pass static and sandbox validation in the same import run, or it is not written.
3. **Tables (new migration, guarded pattern like the auth and hints migrations):**
   - `packs`, `problems`, `concepts`, `problem_concepts`, `test_cases`, `reference_solutions` per `docs/system-design.md` §4. `problems` stores `reference_complexity`, `bench` and `complexity_note` from the record.
   - New `problems` columns: `published boolean` (what `ProblemLookup` checks), `track text`, and `content_sha256 bytea`.
   - New `problem_known_ghosts` (slug, version, id, input, kind, got, exception_class, explanation, concepts): server-only authored Bug Autopsy data that the judge turns into the engine's `KnownGhost`.
   - `problem_hints` belongs to the hints spec (H10). The importer writes to it but does not create it. If that migration is not present, mark the hint-writing task blocked instead of defining the table twice.
   - Add the foreign keys the other specs left for the catalogue (`submissions.problem_slug`, `problem_hints.problem_slug`) with the guarded `to_regclass` pattern.
4. **Privileges (security default, already decided by the existing specs):** new `counterspell_importer` NOLOGIN role (same pattern as `20261113100200_run_lifecycle_roles.sql`, no passwords in migrations) with write access to the catalogue tables only. `counterspell_server` gets SELECT on `packs`, `problems`, `concepts`, `problem_concepts`, `problem_hints`, and **sample test cases only** (for example a view over `test_cases WHERE is_sample`). It gets no access to hidden rows, `reference_solutions` or `problem_known_ghosts`. `counterspell_judge` gets SELECT on everything it needs to run and hunt. Add `pg_` tests proving that the server role cannot read a hidden row or a reference.
5. **Expected values.** Samples and curated hidden cases use their authored `expected` (already proven against the reference by SBX-01 and SBX-09). Generated hidden expected values come from running the reference through `PackRunner` at import time. Never on the host, never written to disk or logs.
6. **Idempotent and versioned.** Same slug, same `pack_version` and same content hash means no change. Same version with a different hash is an error ("bump pack_version"). A higher version replaces that problem's test cases, hints, reference and known ghosts in one transaction. A lower version is an error. A previously imported problem whose pack is now draft or missing becomes `published = false` and is never deleted, because submissions refer to it. Re-running the import twice gives identical rows (`pg_` test).
7. **No rebuild or restart needed.** The importer is a command run against the database (Kiro picks where it lives; the judge crate is the natural home because it already has sqlx and will implement `PackRunner`; record the choice). The server must see an imported or unpublished problem within 60 seconds without a restart. `ProblemLookup` is synchronous, so use a refreshed in-memory snapshot or change the seam; Kiro decides and records why.
8. **Production seams.** Implement a Postgres `ProblemLookup` (server) and a Postgres `ProblemTests` (judge, samples only for `Run`, everything for `Submit`, `reference_python` from `reference_solutions`). Keep `EmptyCatalogue` for tests and a missing database.
9. **Public API.** `GET /api/problems` (slug, title, difficulty, track, concepts) and `GET /api/problems/{slug}` (statement_md, signature, samples, limits), matching the response shape that `web/src/lib/api/problems.ts` validates. Never include hidden cases, generated tests, references, known ghosts, `why` notes or the content hash. Add a canary test: put canaries in a hidden case, the reference and a ghost explanation, import, then assert neither endpoint body contains any canary.
10. **Honest limit, document it.** The learner's program does receive each hidden input as its argument while it runs; that is how tests work. What it never gets is the expected output, and from the first hidden test onward nothing it prints is shown (A1). The browser never receives hidden inputs or expected outputs.

### pack-import tasks (outline; Kiro sizes them at 30 minutes or less)

- I01 Spec docs: requirements (IMP-n IDs), design (data flow above, tables, roles, versioning rules, decisions), tasks.
- I02 Import bundle in grimoire, plus the shared hidden-input function (unit tests, `purity.rs` still green).
- I03 Migration: tables, new columns, `problem_known_ghosts`, importer role, grants, guarded FKs (`pg_` tests, migration runs twice).
- I04 Importer core with a fake `PackRunner`: publish filter, versioning, idempotency, unpublish, one transaction per pack (`pg_` tests).
- I05 Import command and its report (counts and slugs only, no hidden data in output or logs; log canary test).
- I06 Postgres `ProblemTests` and known-ghost loading in the judge.
- I07 Postgres `ProblemLookup` with refresh, wired in `build_router` / `AppState`.
- I08 `GET /api/problems` and `/api/problems/{slug}` with leak canary tests.
- I09 [BLOCKED on judge-sandbox and grimoire-packs T20] Import the three hardened packs into a throwaway local Postgres through the real sandbox runner. Never a shared or production database.
- I10 `docs/pack-data-flow.md`: a one-page plain-English explanation for Brighton of how a problem gets from a pack to the screen, and how to change or add one.

---

## Part 4 - AI pack-authoring pipeline

Brighton does not design problems, solutions or hints. Agents do; Brighton approves or rejects.

```
agent picks an idea from docs/pack-backlog.md
  -> drafts packs/<slug>/ (status = "draft") following the authoring skill
  -> counterspell-grimoire validate packs/<slug> --deny-warnings   (static, always)
  -> model-runner vectors in sandbox_fake.rs until --sandbox exists; then validate --sandbox
  -> scripts/pack-review-digest.sh <slug> writes docs/pack-review/<slug>.md (one readable file)
  -> Brighton reads the digest: approve or reject with notes
  -> approve: status = "published" in a commit whose message contains "Approved by Brighton"
  -> the importer loads it (Part 3)
```

Build:
1. **Authoring skill** (new): `.kiro/skills/grimoire-pack-authoring/SKILL.md`, original text, listed in `.kiro/skills/SOURCES.md` as original and MIT. Point `.kiro/steering/grimoire-authoring.md` at it. Quality rules:
   - Original idea and wording; no LeetCode, HackerRank or other third-party text (PCK-3, GRM-49).
   - Only engine-supported types (`int`, `str`, `list[int]`, `list[list[int]]`) and comparators (`exact`, `unordered`).
   - Statement: exact call and argument types, a worked example from sample 1, edge cases in words, constraints identical to the schema, "integer" wording, beginner-friendly.
   - Samples (2 to 5) show typical cases. Edge cases go in `hidden.toml` (at least 8: empty, single, max size, boundary values, negatives, duplicates, each ghost input).
   - Ghosts: realistic beginner mistakes only (off-by-one, wrong loop bound, wrong starting value, forgetting the last element, empty input, mixing up index and value). Hand-trace each `got`. No contrived bugs like `abs()` on a total. Explanations describe the mistake, avoid the forbidden words and never claim minimality.
   - Hints: four general rungs, never assuming a specific bug; rung 4 near-solution without full code.
   - `complexity_note` with time and extra space; no `reference_complexity` until `[bench]` is supported.
   - A self-review checklist the agent fills in before handing over.
2. **Backlog** (new): `docs/pack-backlog.md` with 7 original problem ideas that balance the tracks (arrays, strings, matrices) and add a few medium problems. Kiro writes it, and Brighton can strike any idea during review.
3. **Review digest script** (new): `scripts/pack-review-digest.sh <slug>` concatenates a pack's files into one markdown with the latest validator report and the agent's checklist. Read-only; writes only under `docs/pack-review/`.
4. **Publish guard** (new): extend `scripts/kiro-hooks/pack-validate.sh` (or add a sibling hook) so a Kiro edit that sets `status = "published"` is blocked. Extend `scripts/pre-push.sh` so a push fails if any commit changes a pack to `published` without "Approved by Brighton" in its message. Test both with shell fixtures.
5. **Seven new packs**, one task each, using the skill. Each stops at `draft` with a digest written. Run the hardened starter packs through the same digest so Brighton reviews all 10 the same way.

Target: 10 packs with digests ready for Brighton's review by Thu 26 Nov, leaving time for fixes before Sun 29 Nov. The importer can load only the ones he approves.

---

## Where the work goes

- Parts 1, 2 and 4: new tasks appended to `.kiro/specs/grimoire-packs/tasks.md` (T31 onward), with matching requirement IDs (GRM-56 onward) and design notes. Do not renumber existing tasks.
- Part 3: the new `.kiro/specs/pack-import/` spec.
- Suggested order: T31 to T34 (format, checks, STM-03, SBX-09) → T35 to T37 (three packs) → skill, backlog, digest and guard → pack-import I01 to I08 → seven new packs (these can run in parallel with pack-import) → I09 when the judge runner lands.

## Acceptance criteria

1. `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings` and `cargo test --workspace` pass, and the test count did not drop.
2. `cargo run -q -p counterspell-grimoire -- validate --all packs --deny-warnings` exits 0 for all catalogue packs.
3. Each starter statement says "integer" where negatives are allowed, names the exact call, has a worked example matching sample 1, and states its edge cases. A beginner can understand it without reading TOML.
4. No authored hint assumes a specific bug. `abs-diagonal` is gone, and `both-diagonals` is verified by hand trace and model runner.
5. Every new diagnostic code has a negative fixture that fails with exactly that code. Hidden-case canaries never appear in validator output.
6. Each starter pack has at least 8 curated hidden cases, the reference agrees with all of them (model runner now, sandbox later), and none duplicates a sample.
7. `pg_` tests prove: drafts are never imported; re-import is a no-op; the version rules hold; unpublish never deletes; the server role cannot read hidden test cases, references or known ghosts; the API bodies contain no canaries.
8. The authoring skill, backlog, digest script and publish guard exist and are tested. At least the 3 starter packs have review digests. Every pack is still `draft`.
9. `docs/kiro-log/starter-pack-v2.md` lists each change, the decisions Kiro made, the deferred items (estimated time, objectives, prerequisites, editorial, `reference_complexity`/`[bench]`), and the tasks blocked on the judge.

## Deliverables

Updated starter packs with `hidden.toml`; validator changes and tests; the `pack-import` spec and its implementation; the authoring skill, backlog, digest script and publish guard; review digests; the changelog. Local commits only until each spec is finished and green. Never `main`.
