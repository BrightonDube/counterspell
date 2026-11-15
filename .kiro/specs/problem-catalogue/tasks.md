# Tasks: problem-catalogue

Status: approved for implementation by Brighton on 15 Nov 2026 10:26 (`brighton-decisions`, and `docs/kiro-log/problem-catalogue-blockers.md`). Each task is sized for 30 minutes or less. Leave C00a unchecked until Brighton creates the private repo.

Markers:
- `[P]` can run in parallel with other tasks in its wave once dependencies are done.
- `[S]` sequential, or has a required dependency.
- `[SUPERVISED]` touches database privileges, hidden data, credentials, the private tree, or an importer trust decision. Brighton reviews the diff before the task is ticked.
- `[HUMAN]` a step only Brighton or the operator can do (host, VPS, approval).
- `[BLOCKED on …]` waits for the named spec or task. Record the block in `docs/kiro-log/problem-catalogue-blockers.md` and continue with work that does not depend on it.
- `[GATE]` a checkpoint. Work after it does not start until it is recorded.

Conventions for every task (same as `run-lifecycle`):
- Production code has no `unwrap()`, `expect()` or `unsafe`. Tests may `expect` with a message.
- Database tests are named `pg_…`, marked `#[ignore = "needs DATABASE_URL"]`, and use `#[sqlx::test(migrations = "../../migrations")]`. Export `DATABASE_URL` from a file outside the repo. Never write the URL or password into a file.
- Red, green, refactor. Write the failing test first and see it fail for the expected reason.
- Expected values in tests are hand-derived literals. Canary values for hidden data are fixed strings such as `CANARY-HIDDEN-7f3a`.
- Every Rust task ends with a Rust lesson of 3–5 plain-English lines.
- Gate before finishing each Rust task: `cargo fmt --all -- --check`, `cargo clippy -p <package> --all-targets -- -D warnings`, and the relevant tests.
- No task creates a file outside `crates/`, `migrations/`, `packs/catalogue.toml`, `docs/` or `docs/kiro-log/` as named in the task.
- Private tree rule (amendment 38): no task reads the real private repo. Tests use synthetic fixtures under `crates/judge/tests/fixtures/private/`, which hold canary strings only. Only C38 and C40 touch the real checkout, and both are HUMAN.

## Prerequisites and status of other specs

| Need | Spec and task | State at spec time |
|---|---|---|
| submissions, jobs, run_events tables and roles | run-lifecycle T05–T07 | done |
| auth roles and submissions user FK | auth T07, T08 | T08 open |
| judge `ProblemTests` seam | judge-sandbox T24 | open (`[ ]`) |
| `PackRunner` implementation | judge-sandbox T20–T22, grimoire T17 and T20 | open or blocked |
| `CandidateStream`, `Schema`, `shrink` public | ghost-engine | done |
| grimoire `validate_static`, `ProblemRecord` | grimoire-packs T12, T14 | done |
| language registry read accessor | language-drivers | not yet specified (X10) |
| `ghosts` table DDL owner | none (D16) | unresolved |
| private repo and its read-only credential | Brighton (C00a) | not created; agents must not create them |

---

## Wave 0: approval

- [x] **C00 [GATE] [HUMAN] (15 min) Approve this spec and its open decisions**
  - Decisions needing approval: D2 (second binary in the judge package), D4 (operator uses the database owner login), D5 (difficulty enum), open questions 1, 4 and 5, D17 to D20 (amendment 38) and open questions 18 to 24.
  - Depends on: none.
  - Verify: `docs/kiro-log/problem-catalogue-blockers.md` records each approval or change. Until then, no wave below starts.

- [ ] **C00a [GATE] [HUMAN] [SUPERVISED] (20 min) Create the private repo and its read-only credential**
  - Brighton creates `BrightonDube/counterspell-packs-private`, adds a read-only deploy key scoped to that repository (or a fine-grained read-only token), and stores the secret outside the public working tree with owner-only permissions. Brighton also creates an empty `retired-values.toml` at the private root. No agent creates the repo, a key or a token, or reads the secret.
  - Depends on: C00.
  - Verify: `docs/kiro-log/problem-catalogue-blockers.md` records the repo name and the secret's location as a path only, never its contents. Brighton confirms with a read-only `git ls-remote` that the key works.

---

## Wave 1: schema, triggers and privileges

- [ ] **C01 [S] [SUPERVISED] (30 min) Migration: core catalogue tables**
  - Files: `migrations/20261116100000_catalogue_core.sql` (tables and CHECKs only; triggers in C02). `crates/judge/tests/pg_catalogue_schema.rs`.
  - Create `tracks`, `concepts`, `problems`, `problem_concepts`, `problem_versions`, `problem_examples`, `problem_starters`, `problem_hints`, `hidden_tests`, `hidden_generation`, `hidden_sources`, `reference_solutions`, `known_ghosts`, `catalogue_events` with the columns and CHECKs in `design.md` "Data model". Use `CREATE TABLE IF NOT EXISTS`. `hidden_tests` carries `source` and `ordinal`. `problem_versions` has no `generator_seed` column.
  - Requirements: CAT-S1, CAT-S5, CAT-S7, CAT-S6 (columns), CAT-PT9 (no seed or private hash in `problem_versions`).
  - Depends on: C00.
  - Verify: `cargo test -p counterspell-judge -- --ignored pg_catalogue_schema` (tests first: each table exists; each slug CHECK rejects `Bad_Slug`; `difficulty = 'intro'` is rejected; `content_sha256` of 31 bytes is rejected; `version = 0` is rejected; `problem_versions` has no `generator_seed` column).
  - Rust lesson: the test runner uses `sqlx::test`, which creates a fresh database per test. A failing CHECK comes back as a `Result::Err`, which the test asserts on.

- [ ] **C02 [S] [SUPERVISED] (30 min) Triggers: immutability, insert gate, publish gate, append-only events**
  - Files: `migrations/20261116100000_catalogue_core.sql` (add the trigger functions and triggers). `crates/judge/tests/pg_catalogue_immutability.rs`.
  - Implement `design.md` "Immutability trigger" and "Publish gate": `problem_versions` raises on delete and on any update except the one-time sandbox fill that needs at least one `hidden_tests` row; child tables raise on update and delete and accept inserts only while `sandbox_validated_at IS NULL`; `problems` publish gate (sandbox validated, 4 hints, 2–5 examples, taxonomy present); `catalogue_events` raises on update and delete.
  - Requirements: CAT-S2, CAT-S3, CAT-S4, CAT-S6, CAT-P5 (append-only part).
  - Depends on: C01.
  - Verify: `pg_catalogue_immutability` (update of a title raises; delete of a version raises; sandbox fill without hidden tests raises; sandbox fill with hidden tests passes); `pg_catalogue_publish_gate` (P7: unvalidated, three hints, six examples, and an unknown track each refuse).
  - Rust lesson: tests check a database rule by sending the bad statement and asserting the error text, so the rule is proven by the database rather than by Rust code.

- [ ] **C03 [S] [SUPERVISED] (25 min) Migration: foreign keys and submissions column**
  - Files: `migrations/20261116100100_catalogue_foreign_keys.sql`, `crates/judge/tests/pg_catalogue_fks.rs`.
  - Add `submissions.problem_version` (nullable). Add `submissions_problem_fk` and `submissions_problem_version_fk` as `NOT VALID`, `ON DELETE RESTRICT`. Add the composite `problems(slug, published_version)` FK. Add the guarded FKs for `problem_baselines.problem_slug` and `concept_cards.concept_id` only when those tables exist (`to_regclass`).
  - Requirements: CAT-F1, CAT-F2, CAT-F3, CAT-S8, CAT-S9.
  - Depends on: C02, run-lifecycle T05.
  - Verify: `pg_catalogue_fks` (a submission for an unknown slug is rejected after the migration; an existing row with an unknown slug survives because of `NOT VALID`; `confdeltype = 'r'` for every new FK). Run the migration twice to prove it is idempotent.
  - Rust lesson: `NOT VALID` is a Postgres choice that tells the database to check new rows only. Explain in the reply why this avoids a long table scan at deploy time.

- [ ] **C04 [S] [SUPERVISED] (30 min) Migration: privilege matrix**
  - Files: `migrations/20261116100200_catalogue_privileges.sql`, `crates/judge/tests/pg_catalogue_privileges.rs`.
  - Use the same `DO` pattern as run-lifecycle to make sure the two existing roles exist (no new role). `REVOKE ALL` from `PUBLIC`. Grant exactly the matrix in `design.md`: server `SELECT` on public tables only; judge `SELECT` on `problems`, `problem_versions`, `problem_examples`, `problem_starters`, `hidden_tests`, `hidden_generation`, `hidden_sources`, `reference_solutions`, `known_ghosts`; no write for either role; no privilege on `catalogue_events` for either role.
  - Requirements: CAT-R1 to CAT-R6.
  - Depends on: C01, C03.
  - Verify: `pg_catalogue_privileges` loops over every table and privilege with `has_table_privilege` for `PUBLIC`, `counterspell_server`, `counterspell_judge`, and asserts the exact matrix (P8 database part). The test also asserts that `pg_roles` contains no new role name.
  - Rust lesson: `has_table_privilege` answers "could this role do X" from the catalogue, so the test checks the rule itself rather than a sample query.

- [ ] **C05 [S] [SUPERVISED] (30 min) Test kit: published fixture with canary hidden data**
  - Files: `crates/judge/src/catalogue/testkit.rs` (behind the `testkit` feature), `crates/judge/tests/pg_catalogue_testkit.rs`.
  - Provide `insert_published_fixture(pool, slug, version)` that inserts a track, concepts, a problem, a version, four hints, two examples, a starter per language, reference, known ghosts, hidden tests and the sandbox fields, through the same triggers as production. Hidden rows carry canary strings. Also a `insert_static_fixture` that skips hidden data.
  - Requirements: CAT-X2, CAT-L7 (X2 and X13 seam).
  - Depends on: C02, C04.
  - Verify: `pg_catalogue_testkit` checks the fixture passes every trigger and that the canary strings are present only in the judge-role view (the test runs as the judge role).
  - Rust lesson: the fixture is a function that takes `&PgPool` and returns `Result`, so a failed insert stops the test with a readable message rather than a panic deep inside a helper.

---

## Wave 2: pure model (no database; property-tested)

- [ ] **C06 [P] [SUPERVISED] (30 min) Canonical manifest and content hash**
  - Files: `crates/judge/src/catalogue/manifest.rs`, `crates/judge/tests/catalogue_props.rs` (P3).
  - Input: `(tree, relative_path, bytes)` triples, where `tree` is `public` or `private`. Each path is prefixed with its tree (`public/…` or `private/…`), and `approval.toml` is excluded before hashing. Sort by the prefixed path, hash each file with SHA-256, build the canonical bytes `path NUL hex LF`, hash those. Return the entries and the 32-byte hash. Return the public entries separately, so `problem_versions.manifest` never holds a private path (CAT-PT6, CAT-PT9).
  - Requirements: CAT-I6, CAT-PT6, CAT-PT9.
  - Depends on: none.
  - Verify: unit tests with hand-computed hashes for a two-file manifest; proptest: any permutation of input order gives the same hash; a one-byte change or a renamed file changes it. Moving a file from `public` to `private` changes it. Changing only `approval.toml` does not.
  - Rust lesson: `Vec<(String, Vec<u8>)>` owns its data, and `sort_by` borrows each pair to compare it. Explain why the hash function takes `&[u8]` (borrowed bytes) instead of taking ownership.

- [ ] **C07 [P] (25 min) Taxonomy parser**
  - Files: `crates/judge/src/catalogue/taxonomy.rs`.
  - Parse `packs/catalogue.toml` with `serde` and `deny_unknown_fields`: `[[tracks]] slug, name` and `[[concepts]] slug, name`. Reject duplicate slugs and slugs that break the kebab rule.
  - Requirements: CAT-T1, CAT-T2 (parse part).
  - Depends on: none.
  - Verify: unit tests for a valid file, a duplicate slug, an unknown key, and a bad slug.
  - Rust lesson: `deny_unknown_fields` turns a typo in a reviewed file into an error at parse time instead of a silently ignored field.

- [ ] **C08 [P] [SUPERVISED] (25 min) Discovery and draft and underscore filter**
  - Files: `crates/judge/src/catalogue/discover.rs`.
  - A pure function takes directory entries `(name, is_dir, has_meta)` and the `status` read from each `meta.toml`, and returns the importable slugs. Names starting with `_` are excluded. A status other than `published` is `DraftPack`. Explicit `--slug` for an `_` name is refused.
  - Requirements: CAT-I2, CAT-I3.
  - Depends on: none.
  - Verify: unit tests for `_test`, `_dogfood`, a draft pack, a published pack, a file named `README.md`, and an explicit `--slug _x`.
  - Rust lesson: the filter returns an enum per entry (`Import`, `Skip(reason)`), so every skipped entry carries the reason the report needs.

- [ ] **C08a [P] [SUPERVISED] (30 min) Private-tree join and preflight (pure over listings)**
  - Files: `crates/judge/src/catalogue/private_tree.rs`, `crates/judge/tests/catalogue_private_preflight.rs`. No database and no network.
  - `preflight(root: RootState) -> Result<(), PreflightError>`, where `RootState` is `Missing`, `Unreadable` or `Readable { inside_public: bool }`. The caller resolves symlinks before it sets `inside_public`. `join(public_slugs, private_slugs) -> Vec<JoinOutcome>` returns `Paired(slug)`, `PrivatePartMissing(slug)` or `OrphanPrivatePack(slug)`. The stable codes are `private_tree_missing`, `private_tree_unreadable`, `private_root_inside_public`, `private_part_missing` and `orphan_private_pack`.
  - Requirements: CAT-PT1, CAT-PT2, CAT-PT3, CAT-X4.
  - Depends on: C08.
  - Verify: unit tests for `Missing`, `Unreadable` and `inside_public` (each exit 2), a public slug with no private part, a private directory with no public pack, and a fully paired list (passes). A test asserts each code string exactly.
  - Rust lesson: a function that takes a plain enum and returns `Result` can be tested without a filesystem. Explain why the caller, not this function, touches the disk.

- [ ] **C09 [P] (30 min) Starter code generator**
  - Files: `crates/judge/src/catalogue/starter.rs`.
  - Build `starter(lang, signature) -> String` for `python` and `javascript`. One template per language: a function header from `function_name` and parameter names, and a body that raises or throws "not implemented". Keep the output under 8 KB.
  - Requirements: CAT-I17.
  - Depends on: none.
  - Verify: golden tests with hand-written expected strings for `count_pairs(values, target)` in both languages; a test that a parameter name with a space is refused.
  - Rust lesson: `format!` borrows its arguments and returns a new `String`. Note why the generator takes `&str` names instead of owned strings.

- [ ] **C10 [P] (25 min) Constraints extraction**
  - Files: `crates/judge/src/catalogue/statement.rs` (or in `starter.rs`'s sibling).
  - Return the bullet items under `## Constraints` in `problem.md`, in order, trimmed. Stop at the next `##` heading. Return an empty list when the section is missing (the grimoire validator already requires it).
  - Requirements: CAT-I18.
  - Depends on: none.
  - Verify: unit tests with two bullets, no bullets, and bullets followed by a `## Examples` heading.
  - Rust lesson: `lines()` yields `&str` slices that borrow from the statement, so the extraction allocates only for the strings it keeps.

- [ ] **C11 [S] [SUPERVISED] (30 min) Import planner**
  - Files: `crates/judge/src/catalogue/plan.rs`.
  - Pure function `plan(latest: Option<&StoredVersion>, incoming: &Incoming) -> Plan` with `Create`, `Unchanged`, `Refuse(VersionNotBumped { changed })`, `Refuse(VersionRegressed)`. `StoredVersion` holds version and hash. `Incoming` holds `pack_version`, hash and the file list.
  - Requirements: CAT-I7, CAT-I8, CAT-I9.
  - Depends on: C06.
  - Verify: table-driven unit tests: no latest → `Create`; same hash → `Unchanged`; bigger version with new hash → `Create`; same version with new hash → `Refuse(VersionNotBumped)` listing the changed files; lower version → `Refuse(VersionRegressed)`.
  - Rust lesson: the planner returns an enum instead of a bool, so the caller cannot forget a refusal. Explain why `match` must list every variant.

- [ ] **C11a [P] [SUPERVISED] (30 min) Approval record and retired-value check (pure)**
  - Files: `crates/judge/src/catalogue/approval.rs`, `crates/judge/src/catalogue/retired.rs`, `crates/judge/tests/catalogue_props.rs` (approval and retired properties).
  - `approval.toml` parser with `deny_unknown_fields`: `slug`, `pack_version`, `content_sha256` (64 lower-case hex), `public_commit` (40 hex, recorded but not verified), `approved_by`, `approved_at`. `check_approval(approval, slug, pack_version, combined_hash) -> Result<(), ApprovalRefusal>` returns `ApprovalMissing` or `ApprovalMismatch`.
  - `retired-values.toml` parser: `[[retired]]` with `slug` and `sha256`, and `[[rotated]]` with `slug`, `pack_version` and `rotated_on`. `canonical_hash(input, expected)` is the SHA-256 of canonical JSON, with object keys sorted explicitly and no insignificant whitespace. `check_retired(file, slug, pack_version, case_hashes)` returns a match with a 12-character prefix only, or the rotation refusal of CAT-PT13.
  - Requirements: CAT-PT7, CAT-PT8, CAT-PT13.
  - Depends on: C06.
  - Verify: hand-derived literal hashes. The expected digest for one fixed canonical string is written as a literal, computed once with `sha256sum` on that exact string, not by the code under test. Proptest: reordering JSON object keys leaves `canonical_hash` unchanged. Refusals for a wrong slug, a wrong `pack_version`, a retired match, and a retired slug with no `[[rotated]]` record. An error's text shows at most the 12-character prefix.
  - Rust lesson: `serde_json::Value` objects do not promise a key order. Explain why the canonical form sorts the keys itself before hashing.

- [ ] **C12 [S] (30 min) Property tests for idempotence and versioning**
  - Files: `crates/judge/tests/catalogue_props.rs` (P1, P2, P4-pure).
  - Model store: a `Vec<StoredVersion>`. Apply random sequences of incoming packs (random content, random bumps). P1: planning the same pack twice gives `Create` then `Unchanged`, and the model has one entry. P2: after any accepted sequence, the model length equals the number of distinct accepted hashes, and earlier entries are unchanged (compare clones).
  - Requirements: CAT-Q1, CAT-I7, CAT-I8, CAT-I10.
  - Depends on: C06, C11.
  - Verify: `cargo test -p counterspell-judge --test catalogue_props` (proptest with a fixed case count for CI, and a seed file committed under `proptest-regressions/`).
  - Rust lesson: proptest generates many inputs and shrinks a failing one to a small example. Explain why the model store is a plain `Vec`: it is the simplest possible reference for the property.

---

## Wave 3: importer writes and CLI

- [ ] **C13 [S] [SUPERVISED] (30 min) Report model and log hygiene**
  - Files: `crates/judge/src/catalogue/report.rs`, `crates/judge/tests/catalogue_logs.rs`.
  - Typed `Outcome` per pack (`Created`, `Unchanged`, `Refused(code)`, `Failed(code)`). Text and JSON renderers. Fields: slug, version, codes, counts, the first 12 hex characters of the hash. A test plants canary values in a fixture and asserts that no rendered line contains them.
  - Requirements: CAT-I20, CAT-X1, CAT-X4, CAT-X2.
  - Depends on: C11.
  - Verify: `cargo test -p counterspell-judge --test catalogue_logs`.
  - Rust lesson: the renderer takes the report by reference, so printing cannot move data out of it. Explain `&Report` in one sentence.

- [ ] **C14 [S] [SUPERVISED] (30 min) Static import, one transaction per pack**
  - Files: `crates/judge/src/catalogue/write.rs`, `crates/judge/tests/pg_catalogue_import.rs`.
  - For a valid static pack: lock the `problems` row (`FOR UPDATE`), re-plan, insert the version, examples, starters, hints, reference, known ghosts and the concept set, insert one `catalogue_events` row, commit. Any error rolls back.
  - Requirements: CAT-I10, CAT-I11, CAT-I12, CAT-I15, CAT-P5 (import event).
  - Depends on: C02, C04, C05, C08, C09, C10, C11, C13.
  - Verify: `pg_catalogue_import_idempotent` (P1 against the database: second run is `Unchanged` and row counts are equal); `pg_catalogue_versions_history` (P2: bump creates one version, old row bytes equal); `pg_catalogue_refusals_write_nothing` (P4); `pg_catalogue_rollback_on_failure` (P6: inject a CHECK violation after the first insert).
  - Rust lesson: `sqlx::Transaction` rolls back when it is dropped without `commit`. Explain why an early `?` in the middle of the function is safe.

- [ ] **C15 [S] [SUPERVISED] (25 min) Dry run and diff output**
  - Files: `crates/judge/src/catalogue/write.rs` (dry-run branch), `crates/judge/src/catalogue/report.rs`, `crates/judge/tests/pg_catalogue_dry_run.rs`.
  - `--dry-run` uses a read-only transaction or none, and never writes. Print the planned action, the static report, and the manifest diff (added, removed, changed files by name). Also print the approval state (`ok` or the refusal code), the private commit identifier if recorded, and a count of retired-value matches (no values). Warn when the engine's generator version differs from the stored one (CAT-I19).
  - Requirements: CAT-I13, CAT-I19, CAT-P5 (none on dry run).
  - Depends on: C14.
  - Verify: `pg_catalogue_dry_run_no_writes` (P5: row counts and `catalogue_events` unchanged); unit tests for the diff output.
  - Rust lesson: a read-only transaction is a database-level promise. Explain why it is stronger than an `if dry_run` check around each insert.

- [ ] **C16 [S] [SUPERVISED] (30 min) Sandbox checks through `PackRunner`**
  - Files: `crates/judge/src/catalogue/sandbox_fill.rs`, `crates/judge/tests/catalogue_sandbox_fill.rs`.
  - Use `counterspell_grimoire::PackRunner` (trait from grimoire T17). Implement SBX-01 (examples), SBX-02 (hidden inputs from `CandidateStream`, expected from the reference), SBX-03 (known ghosts reproduce). Return `SandboxFill { hidden: Vec<(input, expected)>, digest }` or a refusal. Hidden values stay in memory.
  - Requirements: CAT-I14, CAT-I21 (SBX codes), CAT-X1, CAT-PT8, CAT-PT9.
  - Depends on: judge-sandbox T20–T22 (for the real runner, not for these tests), grimoire T17.
  - Verify: unit tests with the grimoire model runner (labelled in the file header as not sandbox evidence, the same convention as grimoire T18). Cases: reference matches, reference disagrees (refused), reference times out (refused), ghost not reproduced (refused), `RunnerError` (exit 2, no write), and a retired hidden value (refused, hash prefix only).
  - Rust lesson: `&mut dyn PackRunner` lets the tests pass a fake and the production code pass the judge's real runner, with the same function. Explain why a trait object keeps the importer independent of the sandbox type.

- [ ] **C17 [S] [SUPERVISED] (30 min) Sandbox import writes hidden data in the same transaction**
  - Files: `crates/judge/src/catalogue/write.rs` (sandbox branch), `crates/judge/tests/pg_catalogue_sandbox_import.rs`.
  - Insert `hidden_tests` (while `sandbox_validated_at` is NULL), then `hidden_generation` (seed and generator version) and `hidden_sources` (private file hashes), then set `sandbox_validated_at` and `sandbox_image_digest` in the same transaction. Log only the counts.
  - Requirements: CAT-I14, CAT-S3, CAT-S2 (one-time fill).
  - Depends on: C14, C16, C02.
  - Verify: `pg_catalogue_sandbox_import_writes_hidden` (hidden row count equals `hidden_count`; the sandbox fields are set; a second sandbox import is `Unchanged`); a fake runner that fails SBX-02 leaves zero rows.
  - Rust lesson: the whole fill is one transaction, so either the version has hidden data and the sandbox stamp, or it has neither. Explain why half a state would break the publish gate.

- [ ] **C18 [S] [SUPERVISED] (25 min) `validate-sandbox <slug>`**
  - Files: `crates/judge/src/catalogue/write.rs`, `crates/judge/tests/pg_catalogue_validate_sandbox.rs`.
  - Runs C16 and C17 for the latest version only, and only when its sandbox columns are NULL. Before running, it recomputes the combined hash and refuses `private_tree_changed` on a mismatch with the stored `content_sha256` (CAT-PT10). Refuses when already validated. Writes a `sandbox_validate` event.
  - Requirements: CAT-I16, CAT-P5, CAT-PT10.
  - Depends on: C17.
  - Verify: `pg_catalogue_validate_sandbox` (static-only version becomes publishable after validation; a second call refuses with no write).
  - Rust lesson: the refusal is a value in the report, not a panic, so the operator sees the reason in the same output format as every other command.

- [ ] **C19 [S] (30 min) CLI binary and exit codes**
  - Files: `crates/judge/src/bin/counterspell-catalogue.rs`, `crates/judge/src/catalogue/mod.rs`, `crates/judge/Cargo.toml` (`[[bin]]` entry and the `counterspell-grimoire` dependency, X9).
  - Parse arguments with `std::env::args` only (no clap, as in grimoire). `--private-root <path>` is required for every command that reads packs. Run the preflight (C08a) before any database connection, so a missing or unreadable root exits 2 without connecting. Map outcomes to exit codes: 0 all created or unchanged; 1 any refusal or failure; 2 usage, missing runner (`SBX-07`) or infrastructure. Read `CATALOGUE_DATABASE_URL` only here.
  - Requirements: CAT-I1, CAT-I21, CAT-R5, CAT-PT2, CAT-PT4.
  - Depends on: C14, C15, C18.
  - Verify: unit tests for argument parsing, exit mapping and a missing `--private-root` (exit 2, no connection attempted); `cargo run -q -p counterspell-judge --bin counterspell-catalogue -- import --dry-run --packs packs --private-root <path>` exits 0 or 1 with a report and never panics.
  - Rust lesson: `std::process::ExitCode` lets `main` return the code after all output has been printed. Explain why helper functions return an outcome and `main` alone calls the exit.

- [ ] **C19b [S] [SUPERVISED] (30 min) Refused public-only import and approval-gate tests**
  - Files: `crates/judge/tests/pg_catalogue_private_tree_required.rs`, `crates/judge/tests/pg_catalogue_approval_gate.rs`, and synthetic fixtures under `crates/judge/tests/fixtures/private/` (canary strings only).
  - `pg_catalogue_private_tree_required`: a run with no `--private-root` exits 2 and leaves every catalogue table and `catalogue_events` at zero rows. A run whose private root lacks one slug's part refuses that slug, imports the others, and writes nothing for the refused slug.
  - `pg_catalogue_approval_gate`: a missing approval, a wrong `pack_version` and a hash mismatch each refuse before the sandbox, with zero writes. A fake `PackRunner` counts its calls, and the test asserts zero calls.
  - Requirements: CAT-PT2, CAT-PT3, CAT-PT7, CAT-PT15, CAT-I11.
  - Depends on: C14, C19, C11a.
  - Verify: both tests pass with `--ignored`. The fake runner's call count is zero in every refusal case.
  - Rust lesson: counting calls on a fake object proves that a code path did not run. Explain why that is stronger than checking only the printed output.

- [ ] **C20 [S] [SUPERVISED] (25 min) Taxonomy sync**
  - Files: `crates/judge/src/catalogue/taxonomy.rs` (database part), `crates/judge/tests/pg_catalogue_taxonomy.rs`.
  - Upsert `tracks` and `concepts` from `packs/catalogue.toml` by slug, updating names and sort order. Never delete. Run once at the start of each import command.
  - Requirements: CAT-T3, CAT-T2 (database check).
  - Depends on: C07, C01.
  - Verify: `pg_catalogue_taxonomy` (a renamed concept updates its name; a removed entry stays in the table; a pack referring to an unknown concept is refused).
  - Rust lesson: `INSERT … ON CONFLICT DO UPDATE` is one statement, so two syncs running at once cannot both insert the same slug.

- [ ] **C21 [S] [SUPERVISED] (30 min) Publish and unpublish**
  - Files: `crates/judge/src/catalogue/publish.rs`, `crates/judge/tests/pg_catalogue_publish.rs`.
  - `publish <slug> [--version n]` sets `publication_state`, `published_version`, `published_at`, and writes an event. Default version: latest sandbox-validated. Report the failed check text when the trigger refuses. `unpublish <slug>` sets the state and `unpublished_at`, keeps `published_version`, and is idempotent.
  - Requirements: CAT-P1 to CAT-P5.
  - Depends on: C02, C18.
  - Verify: `pg_catalogue_publish_gate` (CLI and trigger agree); `pg_catalogue_unpublish_keeps_history` (versions and submissions unchanged; a second unpublish is a no-op with an event only on the first change).
  - Rust lesson: `Option<u32>` for "use the default version" makes the default a visible decision at the call site.

- [ ] **C22 [S] (25 min) List, seed-dev and seed-demo commands**
  - Files: `crates/judge/src/catalogue/seed.rs`, `crates/judge/src/bin/counterspell-catalogue.rs`, `crates/judge/tests/pg_catalogue_seed.rs`.
  - `list` prints slugs, state and version. `seed-dev --static-only` and `seed-demo` import eligible packs (C19), skip drafts, and print `no eligible packs` when there are none. Both require `--private-root` (CAT-PT2). `seed-demo` publishes the latest validated version of each imported problem.
  - Requirements: CAT-D1, CAT-D2, CAT-D3.
  - Depends on: C19, C21.
  - Verify: `pg_catalogue_seed` (with the three starter packs in draft, both seeds import nothing and exit 0). `seed-demo` is not run in CI (it needs the sandbox).
  - Rust lesson: `seed-demo` and `seed-dev` share one function with a flag, so the two paths cannot drift apart.

- [ ] **C23 [P] [SUPERVISED] (20 min) Purity guard for the importer**
  - Files: `crates/judge/tests/catalogue_purity.rs`.
  - Scan `src/catalogue/` for `std::process`, `Command::`, `std::net`, `TcpStream`. Only `sandbox_fill.rs` may name `PackRunner`. The static path must not reference it. Also fail on any `git` invocation and on credential names (`TOKEN`, `DEPLOY_KEY`, `SSH_AUTH_SOCK`, `GIT_SSH`), per CAT-PT4.
  - Requirements: CAT-Q4, CAT-PT4.
  - Depends on: C16.
  - Verify: `cargo test -p counterspell-judge --test catalogue_purity`; a scratch copy with `Command::new` fails the scan.
  - Rust lesson: a test that reads source text is a cheap guard for an architecture rule. Explain why it does not replace the type system.

---

## Wave 4: judge runtime (hidden data, judge role only)

- [ ] **C24 [S] [SUPERVISED] (30 min) `PgProblemTests` (runtime read path)**
  - Files: `crates/judge/src/problem_store.rs`, `crates/judge/tests/pg_catalogue_judge_reads.rs`.
  - `load(slug, version, mode)` reads the pinned version. Examples for `Run`; examples plus `hidden_tests` for `Submit`; `reference_solutions`; `known_ghosts` for autopsy matching. Returns `Ok(None)` for a missing version. Uses only the judge role's connection pool.
  - Requirements: CAT-L4, CAT-R4, CAT-X1, P9 (test side).
  - Depends on: C04, C05, judge-sandbox T24 (X1 signature change, see C26).
  - Verify: `pg_catalogue_judge_reads` (connects as the judge role: reads succeed; a `Run` load never includes hidden rows; an unpublished problem still loads its pinned version (P9); a missing version returns `None`); a write attempt as the judge role fails.
  - Rust lesson: the function returns `Result<Option<T>, E>`. The outer `Result` is infrastructure, and the inner `Option` is "not found". Explain why the judge treats them differently.

- [ ] **C25 [S] [SUPERVISED] (25 min) Reference and known-ghost loaders for the judge**
  - Files: `crates/judge/src/problem_store.rs`.
  - Functions `reference_for(slug, version)` and `known_ghosts_for(slug, version)` for ghost-engine and autopsy matching. Known-ghost data is loaded only by the judge; the autopsy copies the explanation into the ghost row at run time.
  - Requirements: CAT-L4, CAT-X1.
  - Depends on: C24, ghost-engine (autopsy matching).
  - Verify: `pg_catalogue_judge_ghosts` (rows load for the pinned version only; the server role cannot run the same SQL: `permission denied`).
  - Rust lesson: the server's denied query is a test, so a future change that gives the server access fails a test instead of shipping.

- [ ] **C26 [BLOCKED on judge-sandbox T24] [S] (20 min) Version parameter in `ProblemTests`**
  - Files: change agreed with judge-sandbox owner (X1). This spec's file: `crates/judge/src/problem_store.rs` (impl only).
  - `ProblemTests::load(slug, version, mode)`. Update `EmptyProblemTests`, `StaticProblemTests` and `DirProblemTests` in judge-sandbox testkit in the same change. Assertions unchanged.
  - Requirements: CAT-L4, CAT-L7.
  - Depends on: judge-sandbox T24.
  - Verify: judge-sandbox and this spec's tests compile and pass with the new signature. Record the test count before and after.
  - Rust lesson: adding a parameter to a trait method breaks every implementation, so the compiler lists each one for you. Explain why that is a useful kind of breakage.

---

## Wave 5: server read path and seams

- [ ] **C27 [S] (30 min) `ProblemLookup` async and the version field (seam replacement)**
  - Files: `crates/server/src/seams.rs`, `crates/server/src/lib.rs` (`AppState::with_problems`), test kit implementations in `crates/server/src/testkit.rs`.
  - Change to `#[async_trait]` returning `Option<ProblemRef>`, with `ProblemRef { slug, version }`. Update the in-memory lookup and `EmptyCatalogue`. Existing run tests keep their assertions. Compile errors from struct literals get the `version` field; no assertion changes.
  - Requirements: CAT-L1, CAT-L7. Cross-spec: X4 (run-lifecycle T21).
  - Depends on: C00, run-lifecycle T21.
  - Verify: `cargo test -p counterspell-server` passes, and the number of tests is the same as before the change (record both numbers in the reply).
  - Rust lesson: `#[async_trait]` boxes the future so a trait object can hold an async method. Explain why the repo already uses it for `UserResolver`.

- [ ] **C28 [S] [SUPERVISED] (30 min) `PgProblemLookup` and production wiring**
  - Files: `crates/server/src/catalogue/mod.rs`, `crates/server/src/catalogue/lookup.rs`, `crates/server/src/lib.rs`, `crates/server/tests/pg_catalogue_lookup.rs`.
  - Query `problems` for published rows and return the `published_version`. Production `AppState` uses it. `EmptyCatalogue` stays for the database-free constructor.
  - Requirements: CAT-L2, CAT-A9.
  - Depends on: C27, C04.
  - Verify: `pg_catalogue_lookup` (published returns the version; unpublished returns `None`; the server role cannot select from `hidden_tests`, checked with `has_table_privilege`).
  - Rust lesson: a `WHERE publication_state = 'published'` clause is part of the query, not a check after it, so a missed check cannot return an unpublished row.

- [ ] **C29 [S] [SUPERVISED] (30 min) Enqueue pins the version in one statement**
  - Files: `crates/server/src/runs/` (enqueue function from run-lifecycle T24), `crates/server/tests/pg_catalogue_pin.rs`.
  - Replace the separate publication check with the `INSERT … SELECT … WHERE publication_state = 'published'` shape from `design.md`. No row returned means `404 not_found` and nothing queued.
  - Requirements: CAT-L3, CAT-L5, P9. Cross-spec: X3 (run-lifecycle T24).
  - Depends on: C28, run-lifecycle T24.
  - Verify: `pg_catalogue_pin` (a run queued before an unpublish still has its version; an unpublished problem returns `404` and creates no submission row; the idempotency key still returns the same run).
  - Rust lesson: one statement either inserts or inserts nothing, so the publication check and the insert cannot be separated by another transaction.

- [ ] **C30 [S] (30 min) `GET /api/problems` list**
  - Files: `crates/server/src/catalogue/{read.rs,dto.rs,handlers.rs}`, `crates/server/tests/catalogue_api.rs` (no database), `crates/server/tests/pg_catalogue_api.rs`.
  - Parse `track`, `concept`, `difficulty`. Unknown values give `400 invalid_filter`. Use the list query in `design.md`. `status` from the caller's session (via the existing `UserResolver`); anonymous gives `null`. Cap at 200.
  - Requirements: CAT-A1, CAT-A2, CAT-A3, CAT-P4, PRB-1, PRB-2.
  - Depends on: C28, auth T11 (resolver), C04.
  - Verify: `pg_catalogue_api` (filters; solved, attempted and unattempted from the caller's own submissions only; another user's solved submission does not change the status; unpublished problems are absent). The unit test checks the invalid filter error code.
  - Rust lesson: the handler builds the response from `dto` structs, so a column that is not in the struct cannot reach the JSON, even if the query returns it.

- [ ] **C31 [S] [SUPERVISED] (30 min) `GET /api/problems/{slug}` detail and the no-leak tests**
  - Files: `crates/server/src/catalogue/{read.rs,dto.rs,handlers.rs}`, `crates/server/tests/pg_catalogue_api.rs`, `crates/server/tests/catalogue_no_leak.rs`.
  - Build the detail response from the explicit struct in `design.md`. Examples are public samples only. No hint bodies. Unknown and unpublished slugs give the same `404 not_found` body.
  - Requirements: CAT-A4, CAT-A5, CAT-A6, CAT-A8, CAT-A9, CAT-X1 to CAT-X3, PRB-3.
  - Depends on: C30, C17 (fixtures with canaries), C05.
  - Verify: `pg_catalogue_api` (detail fields and key set equal the allow-list exactly); `catalogue_no_leak` (the canary strings from the fixture, including one planted in the synthetic private fixture, are absent from every list and detail response body, and from every error body); the 404 bodies for unknown and unpublished are byte-identical.
  - Rust lesson: the allow-list test compares sets of keys, not individual values, so a new field fails the test until someone decides it is public.

- [ ] **C32 [P] (25 min) `GET /api/languages` from the language seam**
  - Files: `crates/server/src/catalogue/languages.rs`, `crates/server/tests/catalogue_api.rs`.
  - Return enabled languages with `id`, `display_name` and `max_source_bytes` from the language catalogue accessor (X10). Disabled stretch languages are not listed.
  - Requirements: CAT-A7, CAT-A8.
  - Depends on: language-drivers accessor (X10), run-lifecycle T21 (`LanguageCatalog`).
  - Verify: unit test with the launch catalogue lists exactly `python` and `javascript`; a disabled entry is absent; the key allow-list passes.
  - Rust lesson: `Vec<Language>` is cloned into the response, so the request cannot change the server's catalogue. Explain why a shared reference would be a different design.

- [ ] **C33 [BLOCKED on D16 (ghosts table owner)] [S] [SUPERVISED] (30 min) `GhostLookup` for the custom ghost run**
  - Files: `crates/server/src/catalogue/ghosts.rs`, `crates/server/tests/pg_catalogue_ghosts.rs`.
  - `stored_for(user, submission_id)` returns the stored ghost input only when the submission belongs to the caller. Blocked until the `ghosts` table exists and its owner is decided (open question 1).
  - Requirements: CAT-L6. ide-workspace T02 depends on this.
  - Depends on: the `ghosts` DDL (owner open), C28.
  - Verify: a `pg_` test that another user's submission returns `None`; a submission with no ghost returns `None`.
  - Rust lesson: the ownership check is in the SQL `WHERE`, not in Rust after the query, so a forgotten check cannot leak a row.

---

## Wave 6: integration, documentation and gates

- [ ] **C34 [S] [SUPERVISED] (25 min) Operator runbook and seeding documentation**
  - Files: `docs/catalogue-operations.md` (new), a link added to `packs/README.md`.
  - Explain the commands, the environment variable names (never values), the draft gate, the bump rule, what dry run shows, and how to publish and unpublish. State that seeding imports nothing until the starter packs are approved. Also explain the private checkout (CAT-PT14: a read-only deploy key or token kept outside the repo, given to git by the operator, and deleted after import), the approval file, the retired-values file and rotation (C40).
  - Requirements: CAT-D4, CAT-PT14.
  - Depends on: C22, C21.
  - Verify: the document names every command in `design.md` and no secret or URL; a reviewer can run a dry run from the document alone.
  - Rust lesson: not applicable (documentation only).

- [ ] **C35 [S] [SUPERVISED] (30 min) Test-helper changes in other specs (X2, X13)**
  - Files: run-lifecycle and judge-sandbox test helpers that insert submissions (name them in the reply), using `catalogue::testkit` (C05).
  - Insert a fixture problem before each submission the tests create. Keep every assertion unchanged.
  - Requirements: CAT-L7.
  - Depends on: C05, C29.
  - Verify: `cargo test -p counterspell-server -- --ignored pg_` and `cargo test -p counterspell-judge -- --ignored pg_`; the test counts before and after are recorded and equal.
  - Rust lesson: a test helper is ordinary code, so a shared fixture function removes the same setup from each test without hiding it.

- [ ] **C36 [S] (30 min) Full database and API test pass**
  - Files: none new. Fix only catalogue files.
  - Run every `pg_catalogue_*` test against a throwaway Postgres 17 with `DATABASE_URL` set (the existing `cs-dev-pg` container, not a new one). Record the results in `docs/kiro-log/problem-catalogue-verify.md`.
  - Requirements: CAT-Q2.
  - Depends on: C01 to C35.
  - Verify: every test in the list passes; any failure is recorded with the test name in the blockers file, never removed.
  - Rust lesson: `--ignored` runs only the database tests, so the normal `cargo test` stays fast and needs no database.

- [ ] **C37 [S] (25 min) Repo-wide gates**
  - Files: none new.
  - Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets -- -D warnings`, `cargo test --workspace`, and `cargo deny check licenses`. If `cargo deny` rejects a licence, stop and ask Brighton before editing `deny.toml`.
  - Requirements: CAT-Q3.
  - Depends on: C36.
  - Verify: all four commands exit 0; the test count is recorded.
  - Rust lesson: `-D warnings` turns Clippy warnings into errors so they cannot be ignored in review.

- [ ] **C38 [HUMAN] [SUPERVISED] (30 min) Sandbox import of the approved starter packs on the host**
  - Run `counterspell-catalogue import --sandbox --private-root <checkout>` and then `publish` for each approved pack, on the host with the rootless sandbox daemon. The operator makes the checkout (CAT-PT14) and deletes it afterwards. Record the image digest and the report in `docs/kiro-log/problem-catalogue-verify.md`.
  - Requirements: CAT-D2, CAT-I14, CAT-P1.
  - Depends on: C37, judge-sandbox T37 (host verification), grimoire-packs T28 (Brighton's approval), C00a, C40.
  - Verify: the three approved problems appear in `GET /api/problems`; a second import reports `unchanged`; hidden values are absent from the response.
  - Rust lesson: not applicable (operator step).

- [ ] **C39 [HUMAN] (10 min) Approval gate before publication**
  - Brighton approves each starter pack (grimoire-packs T28) before C38 runs. He writes `packs/<slug>/approval.toml` in the private repo with the combined content hash (CAT-PT7). The importer refuses without it.
  - Depends on: grimoire-packs T28.
  - Verify: `docs/kiro-log/problem-catalogue-blockers.md` records the approval.

- [ ] **C40 [GATE] [HUMAN] [SUPERVISED] (30 min) Rotate hidden values that are in public git history**
  - Before any new hidden file is written, Brighton records each retired hidden value's SHA-256 under `[[retired]]` in the private `retired-values.toml`. The rotated pack work (new curated cases, new generator seed, `pack_version` bump) then goes through the normal stages. Each rotated slug gets a `[[rotated]]` record. Brighton approves the rotated packs (grimoire-packs T28, pack-authoring PIP-35).
  - Brighton decides separately whether public git history is rewritten (design open question 21). This spec never force-pushes or rewrites history.
  - Depends on: C00a, grimoire-packs T28.
  - Verify: each starter slug with a `[[retired]]` entry also has a `[[rotated]]` entry, so the CAT-PT13 check passes (C11a tests show the refusal without it). Recorded in `docs/kiro-log/problem-catalogue-blockers.md`. Agents do not write the `[[rotated]]` record.

---

## Traceability

| Requirement group | Tasks |
|---|---|
| CAT-S1 to CAT-S9 (schema) | C01, C02, C03, C05 |
| CAT-R1 to CAT-R6 (roles) | C04, C24, C25, C28 |
| CAT-T1 to CAT-T3 (taxonomy) | C07, C20 |
| CAT-I1 to CAT-I21 (importer) | C06, C08, C09, C10, C11, C12, C13, C14, C15, C16, C17, C18, C19 |
| CAT-P1 to CAT-P6 (publication) | C02, C21, C22 |
| CAT-A1 to CAT-A9 (read API) | C30, C31, C32 |
| CAT-L1 to CAT-L7 (seams) | C24, C26, C27, C28, C29, C33, C35 |
| CAT-D1 to CAT-D4 (seeding) | C22, C34, C38 |
| CAT-X1 to CAT-X4 (confidentiality) | C13, C17, C24, C31 |
| CAT-Q1 to CAT-Q5 (quality) | C12, C23, C36, C37 |
| CAT-PT1 to CAT-PT15 (private tree, approval, rotation) | C00a, C06, C08a, C11a, C15, C16, C17, C18, C19, C19b, C22, C23, C34, C38, C40 |

Properties P1 to P10 are proven at the tasks named in `design.md` "Correctness properties".

## Blocked and deferred items (for the blockers file)

| Item | Waits for | Continue with |
|---|---|---|
| C26 (version parameter) | judge-sandbox T24 | C24 uses a local adapter until then |
| C33 (ghost lookup) | `ghosts` DDL owner (D16) | everything else |
| C38 (VPS import) | judge-sandbox T37, grimoire-packs T28 | all tests; C38 is operator work |
| C16 real runner | judge-sandbox T20–T22, grimoire T17 | model-runner tests |
| C00a, C38, C40 (private repo, credential, rotation) | Brighton (HUMAN) | every task except the real private checkout; tests use synthetic fixtures |
