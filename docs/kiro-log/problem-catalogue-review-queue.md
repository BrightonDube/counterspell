# problem-catalogue review queue (SUPERVISED tasks)

Brighton reviews each diff before the task is treated as approved. Each entry lists the task, the commit and what to check.

| Task | Commit | What to check |
|---|---|---|
| C01 | (see git log "problem-catalogue C01") | Table columns and CHECKs match design.md "Data model". `hidden_tests` has `source` and `ordinal`. `problem_versions` has no `generator_seed`. |
| C02 | (see git log "problem-catalogue C02") | Trigger rules match design.md: version update/delete refused except the one-time sandbox stamp; child insert refused after the stamp; publish gate checks 2–5 examples, four hints, a concept tag. The "unknown track" case is refused by the foreign key, not by the publish gate. |
| C03 | (see git log "problem-catalogue C03") | Submissions now need a real problem row (`submissions_problem_fk`, NOT VALID). `pg_model_interleavings` in `crates/judge/tests/model.rs` gained a setup-only `two-sum` catalogue row, marked `test-change: CAT-F1`. No assertion changed. Check that the setup change is acceptable. |
| C04 | (see git log "problem-catalogue C04") | Grant matrix: server reads public catalogue tables only; judge reads run inputs and hidden tables only; neither role writes, and neither reads `catalogue_events`. The dev cluster has its own login role named `counterspell`; the test ignores it on purpose. |
| C05 | (see git log "problem-catalogue C05") | Test kit in `crates/judge/src/catalogue/testkit.rs` repeats the helper SQL in `crates/judge/tests/catalogue_support/mod.rs`. Merge the two in a later cleanup. Canary strings are fixed constants. |
| C06 | (see git log "problem-catalogue C06") | Manifest hash: check the prefix rule (`public/`, `private/`), that `approval.toml` is excluded, and that `public_map` never holds a private path. The expected digest in `catalogue_manifest.rs` was computed with `sha256sum`. |
| C08 | (see git log "problem-catalogue C08") | Discovery filter: `_` names are never imported, drafts are skipped with a reason, and an explicit `_` slug is refused. |
| C08a | (see git log "problem-catalogue C08a and C11 to C12") | Private-tree preflight: the caller must resolve symlinks before it sets `inside_public`. Codes are pinned by tests. |
| C11 | (see git log "problem-catalogue C08a and C11 to C12") | Import planner: a same-version, different-hash import is refused with the changed files listed. Check that the planner looks only at the latest version. |
| C11a | (see git log "problem-catalogue C08a and C11 to C12") | Approval and retired-value checks. The canonical-JSON key-order property holds trivially because `serde_json::Map` is sorted by default; the explicit sort in code is still what makes the hash stable. Check the retired-value message shows only 12 characters. |
| C13 | (see git log "problem-catalogue C13") | Report model. The canary test checks the report's shape, not a planted secret: `PackReport` has no field that would carry hidden data. The stronger check belongs in C14, where real import output is rendered. |

## C03 repair (found after the C03 commit)
C03's `submissions_problem_fk` broke about 75 existing database tests, because they inserted `two-sum` submissions with no catalogue problem row. The repair seeds the `two-sum` problem in the shared helpers (`crates/judge/src/testkit.rs`, `crates/server/src/testkit.rs`) and in the few tests that insert directly. Each edit is marked `test-change: CAT-F1`, and no assertion changed. Result: 257 ignored database tests and 1,009 unit tests pass. Brighton should check that the seeding is acceptable.

## C14 (static import transaction)

| Task | Commit | What Brighton should check |
|---|---|---|
| C14 | f23934f (problem-catalogue C14) | `write_static` in `crates/judge/src/catalogue/write.rs`. It locks the problem row, re-plans under the lock, and writes one transaction. Check: (1) the `problems` display copy is refreshed on every write, including the Unchanged path, which is then rolled back; (2) `write_static` takes the operator's own connection pool, so the operator role must be the importer role the approval names; (3) the concept set is replaced with DELETE then INSERT, which is allowed because `problem_concepts` has no trigger. The `VersionNotBumped` list only covers public files, because `manifest` holds public entries only (CAT-PT9). A private-only change still refuses, but its path is not listed. |

## C15 (dry run)

| Task | Commit | What Brighton should check |
|---|---|---|
| C15 | (see git log "problem-catalogue C15") | `dry_run_static` in `write.rs` runs in a read-only transaction and returns the plan, a public-file diff and the generator-version change. Check: the diff lists public files only, as with C14. The approval state, the retired-value count and the private commit id are not in this function yet; they belong to the CLI task C19, which must print them. |

## C20, C24, C25 (supervised, attempt 3)

| Task | Commit | What Brighton should check |
|---|---|---|
| C20 | 9d232b5 | `sync_taxonomy` in `taxonomy.rs` upserts with `ON CONFLICT DO UPDATE` and never deletes. Check the rename and keep-on-removal behaviour in `pg_catalogue_taxonomy`. |
| C24 | ffa2596 | `problem_store::load` is a free function, not the `ProblemTests` trait, because judge-sandbox T24 is not in the code yet. Check that `Run` never loads hidden rows and that a missing version gives `None`. C26 must wire the trait later. |
| C25 | 824fe60 | `reference_for` and `known_ghosts_for` read through the judge role. The server role is denied, as tested in `pg_catalogue_judge_ghosts`. Check that the testkit change (version-dependent content hash, idempotent concept link) only changes fixture bytes. |

## C28 (attempt 5)

| Task | Commit | What Brighton should check |
|---|---|---|
| C28 | 455e8db | `PgProblemLookup` in `crates/server/src/catalogue/lookup.rs`. Check: (1) `AppState::new` now uses the database lookup, so every server test that builds state reads `problems`; the `pg_` suite passed; (2) a database error is logged and answered as "not found" (fail closed), not as a 500; (3) the publication check is in the SQL `WHERE`. |

## C31 (attempt 6)

| Task | Commit | What Brighton should check |
|---|---|---|
| C31 | (see git log "problem-catalogue C31") | `read::published_detail` and `GET /api/problems/{slug}` in `crates/server/src/catalogue/`. Check: (1) the detail fields come from the pinned `published_version` only; (2) unknown and unpublished slugs return one identical 404 body; (3) the judge test-kit signature and constraints were changed to the full stored shape (`params`, `returns`), with no assertion weakened; (4) the no-leak test plants no canary in a synthetic private fixture yet, because `crates/judge/tests/fixtures/private/` does not exist. Canaries come from the judge test kit's hidden tests, reference and known ghosts. |

## C29 and C35 (attempt 7)

| Task | Commit | What Brighton should check |
|---|---|---|
| C29 and C35 | 925ac4a | `enqueue::insert_run_and_job` is one `INSERT … SELECT … WHERE publication_state = 'published' RETURNING id`. A missing or unpublished problem returns `EnqueueOutcome::NotPublished`, which `create.rs` maps to 404 `unknown_problem`. That is the same body the lookup gives, not the `not_found` code the spec names. Decide which code the API should use. Check also that the server tests now publish `two-sum` through the judge fixture (`support::publish_two_sum`, which is idempotent). No assertion changed. Server suite: 435 passed with `--include-ignored`, 0 failed. Judge `pg_` suite: 110 passed. |

## C23 (attempt 7)

| Task | Commit | What Brighton should check |
|---|---|---|
| C23 | ebb3738 | `crates/judge/tests/catalogue_purity.rs`. The scanner and its synthetic cases were written in the same change as the scanner, so the red step was not seen separately. The check is a text match, so it cannot see a value passed in through a variable. Its `PackRunner` rule is a no-op until C16 adds `sandbox_fill.rs`. C23 lists C16 as a dependency, and that dependency is only partly met. |

## C26 (attempt 1, unblocked by judge-sandbox T24)

| Task | Commit | What Brighton should check |
|---|---|---|
| C26 | 54b1d2b | `ProblemTests::load(slug, version, mode)` in `crates/judge/src/sandbox/problem.rs`. Check: (1) `ClaimedRun.problem_version` is `Option<u32>` and the worker maps a missing pin to `0`, which no loader can satisfy — an unpinned run reports `problem_unavailable` rather than picking a version; (2) `StaticProblemTests`/`DirProblemTests` return `None` on a version mismatch so the doubles behave like a pinned store; (3) `PgProblemTests` in `problem_store.rs` maps the version row + examples + hidden + `reference_for` into `ProblemSpec`, resolves `bench.size_param` (a name in the stored JSON) to a signature index, and turns malformed catalogue data into `ProblemError::Store` — infrastructure, never a verdict; (4) `PinnedProblem` grew the spec columns (`signature`, `input_schema`, `comparator`, `limits`, `reference_complexity`, `bench`) so one SELECT serves both the C24 reader and the trait impl. Test counts — before: 28 on the touched binaries (executor 8, model_bench 6, model_sandbox_executor 14); after: same 28 green plus 3 new `problem_store` unit tests, model_logs 1, familiar_measure 10; daemon regression sbx_functional 7/7 and sbx_bench 3/3. |

## C16 (attempt 1)

| Task | Commit | What Brighton should check |
|---|---|---|
| C16 | 34e3374 | `crates/judge/src/catalogue/sandbox_fill.rs` + `tests/catalogue_sandbox_fill.rs` (scripted model runner, labelled not sandbox evidence). Check: (1) check order is SBX-01 samples -> curated -> SBX-02 generated -> SBX-03 ghosts -> retired, so a disagreement refuses before ghosts run; (2) a curated case is verified against the authored value AND must produce a well-shaped reference return; the stored expected is the reference`s own answer, not the authored claim; (3) `fill_generated` takes `hidden_count*8` max candidates so a small domain cannot loop, and reports `hidden_count_shortfall` instead of guessing; (4) `RunnerError` propagates as `Err` (infrastructure / exit 2) while every pack fault is `Ok(Refused(..))`; (5) the retired check hashes canonical `(input, expected)` pairs and the refusal carries only the hash prefix via `RetiredRefusal`; (6) `canonical_json` in `retired.rs` went `pub` for cross-source dedup, no other retired internals exposed. 10 fill tests + 5 purity tests + 15 pure + 5 plan all green; clippy clean; workspace check clean. Rust lesson: `&mut dyn PackRunner` is a trait object, so the same `sandbox_fill` runs against the scripted model in tests and the judge`s real sandbox runner in production without either side naming the other`s type — the importer stays independent of the sandbox backend, and the purity test can pin that `PackRunner` appears in this file only. |

## C17 (attempt 1)

| Task | Commit | What Brighton should check |
|---|---|---|
| C17 | aba3b7d | `write_sandbox` + `SandboxImport`/`SandboxOutcome` in `crates/judge/src/catalogue/write.rs`, `tests/pg_catalogue_sandbox_import.rs`. Check: (1) tx order is problems-row lock -> version-state read -> hidden_tests -> hidden_generation -> hidden_sources -> stamp -> sandbox_validate event; the version guard makes the stamp impossible without hidden tests; (2) `hidden_tests` ordinals continue after the stored sample count so one ordinal space covers examples+hidden (test asserts 3,4,5 behind two samples); (3) a stamped version returns `Unchanged` before touching a row, a missing version returns `Missing` — neither is an error; (4) the event detail is counts only (`hidden_curated`, `hidden_generated`, `hidden_sources`); (5) the new module imports `CaseSource`/`SandboxFill` types only — the purity test still pins `PackRunner` to `sandbox_fill.rs`. pg tests: 4/4 new + 4/4 pg_catalogue_import + 2/2 pg_catalogue_dry_run + full catalogue suite green; clippy clean. pg tests ran against a throwaway postgres container on :5432. Rust lesson: the whole fill is one transaction, so either the version has hidden data and the sandbox stamp, or it has neither — a half state (stamp without hidden rows, or rows without a stamp) would either deadlock the publish gate (stamp set, rows frozen missing) or leak unvalidated hidden data into a version a later fill could never re-stamp. |

## C18 (attempt 1)

| Task | Commit | What Brighton should check |
|---|---|---|
| C18 | 1b70b6f | `sandbox_gate` in `write.rs` + `SandboxPack`/`ValidateOutcome`/`validate_sandbox` in `sandbox_fill.rs`, `tests/pg_catalogue_validate_sandbox.rs`. Check: (1) the orchestration lives in `sandbox_fill.rs` so the purity test still pins `PackRunner` to one file — `write.rs` only gained the pure-DB gate; (2) the gate locks the problem row, reads latest-version state, compares the recomputed hash to stored `content_sha256`, and rolls back — the sandbox run happens *outside* a transaction so no code executes under a row lock; `write_sandbox` re-checks the stamp under its own lock, so a mid-run stamper surfaces as `AlreadyValidated`, not a double write; (3) all four refusal shapes are values: `AlreadyValidated`, `Missing`, `PrivateTreeChanged`, `Refused(FillRefusal)` — only `ValidateError::{Write,Runner}` are errors (exit 2); `RunnerError` has no `Error` impl so the variant carries it without `#[from]`; (4) the pg test proves a static-only version becomes publishable after validation — the publish UPDATE succeeds against the real gate trigger; (5) `generator_seed` is stored bit-preserved (`from_ne_bytes`), not a magnitude cast. pg tests: 4/4 new + 4/4 sandbox_import + 4/4 import + catalogue suite green; clippy clean. Rust lesson: the refusal is a value in the report, not a panic — `validate_sandbox` returns `Ok(Refused(..))` so the operator sees the reason in the same output shape as every other command, while `Err` is reserved for infrastructure where there is nothing to report but a failure. |

## C19 (attempt 1)

| Task | Commit | What Brighton should check |
|---|---|---|
| C19 | 9cfd94d | `src/bin/counterspell-catalogue.rs` + `src/catalogue/load.rs` + `packs/catalogue.toml`. Check: (1) `pack_gate` runs `private_tree::preflight` before `connect()` — a missing flag, missing dir, unreadable root or a root inside the public tree all exit 2 without opening the pool (verified live: `private_tree_missing`, `private_root_inside_public`); (2) the per-pack order is join -> draft gate -> validate_static -> taxonomy -> manifest -> approval -> assemble -> plan/write, so a draft never reaches record parsing and a failed approval never reaches the write (CAT-PT7); (3) sandbox paths (bare `import`, `--sandbox`, `validate-sandbox`) exit 2 with SBX-07 — no PackRunner is wired, and `validate_one` sits compiled behind `runner_available()` ready for the runner task; (4) `load.rs` is the only fs-walking catalogue module and never names `PackRunner` — purity suite still pins it to `sandbox_fill.rs`; private bytes only feed manifest hashes and `private_sources`; (5) `packs/catalogue.toml` (D13) is new scaffold holding the three tracks + eight concepts the draft packs declare; it was missing and every import would otherwise have failed as infrastructure; (6) report lines carry slug, code, counts and a hash *prefix* only — no hidden values, seeds or private paths (CAT-PT9). Verify run against the real scaffold: `import --dry-run` printed three `refused:draft_pack` lines, exit 1; a synthetic root produced `private_part_missing` + `orphan_private_pack`; `--slug _demo` gave `underscore_pack`; `--json` emits proper JSON lines; usage/verb errors exit 2. Unit: 11 bin tests (parse, exit map, missing --private-root) + 66 catalogue tests + 5 purity + workspace clippy all green; pg validate_sandbox suite 4/4 unchanged. Rust lesson: `ExitCode` is opaque (no PartialEq), so `exit_code_for` returns a `u8` the tests can assert and `ExitCode::from` wraps it once at the boundary — the test checks the mapping, not the process. |
