# Requirements: problem-catalogue

Status: **spec only.** No task in `tasks.md` runs until Brighton approves this spec (`.kiro/steering/brighton-decisions.md`, 14 Nov 2026). Open questions with the assumed choice are in `design.md`.

## Introduction

The catalogue closes the gap between reviewed Grimoire packs in git (`packs/<slug>/`, format owned by `grimoire-packs`) and the running server. It owns:

- tracks ("Crypts"), concepts and the problem-to-concept mapping;
- problems and their immutable `problem_versions`;
- public examples, starter code and authored hint rungs;
- server-only hidden tests, reference solutions and known-ghost data;
- publication state (publish and unpublish);
- the operator importer and CLI;
- the public read API for problems and languages;
- the Postgres lookups behind the `ProblemLookup` seam (run-lifecycle) and the stored ghost lookup (ide-workspace).

Sources: `docs/system-design.md` §2 (components), §4 (data model), §5 (run lifecycle, point 5: the Python reference is authoritative for expected outputs), §6 (languages), §9 (API), §10 (deployment); `docs/requirements.md` PRB-1 to PRB-3, PCK-1 to PCK-3, ADM-1, E14 / VAL-1 to VAL-5, walkthrough W3; `.kiro/specs/grimoire-packs/`; `.kiro/specs/run-lifecycle/`; `.kiro/specs/judge-sandbox/`; `.kiro/specs/ide-workspace/`; `.kiro/specs/hints/`; `.kiro/specs/dashboard-fsrs/`; `.kiro/specs/auth/`; `.kiro/specs/efficiency-familiar/`; `.kiro/specs/language-drivers/`; `.kiro/steering/*.md`.

### Out of scope

- A web admin UI, and any new Postgres role. Publication is an operator CLI action.
- Judging, comparison and verdicts (`judge-sandbox`). Ghost hunting and shrinking (`ghost-engine`). Hint reveal and AI hints (`hints`). Dashboard aggregates and FSRS (`dashboard-fsrs`). Bench fitting (`efficiency-familiar`).
- Authoring the remaining seven launch problems. Starter pack approval (grimoire-packs T28).
- Payments, community packs, and copied third-party problem text.
- The VPS deployment, backups and the operator login setup (`deploy-ops`, not yet approved).

## Glossary

- **Track**: a "Crypt", the top-level group of problems (for example `arrays`).
- **Concept**: a skill tag such as `nested-loops`. A problem has 1 to 5 concepts.
- **Problem**: the stable catalogue entry, keyed by `slug`.
- **Problem version**: an immutable snapshot of one pack's content at one `pack_version`. Runs and ghosts point at a version.
- **Publication state**: `unpublished` or `published`. Only published problems are served by the public API.
- **Public data**: anything a signed-in or anonymous browser may receive: statement, public examples, starter code, signature, constraints, limits, concept and track names, hint count.
- **Hidden data**: hidden tests (inputs and expected outputs), the reference solution, known-ghost inputs, expected values and explanations. Hidden data is readable only by the judge.
- **Importer**: the operator command that reads `packs/<slug>/` and writes catalogue rows.
- **Operator**: Brighton or the person running the CLI on a host with database owner access.
- **Sandbox validation**: running the reference, hidden inputs and known ghosts through the judge's sandbox `PackRunner` (grimoire-packs SBX-01 to SBX-03).

## Requirements

### A. Schema

1. **CAT-S1** THE SYSTEM SHALL create the catalogue tables `tracks`, `concepts`, `problems`, `problem_concepts`, `problem_versions`, `problem_examples`, `problem_starters`, `problem_hints`, `hidden_tests`, `reference_solutions`, `known_ghosts` and `catalogue_events` in SQLx migrations whose timestamps sort after every existing migration.
2. **CAT-S2** THE SYSTEM SHALL keep `problem_versions` immutable. No row SHALL be updated or deleted, except that the sandbox-validation columns may change once from NULL to a value (CAT-I14).
3. **CAT-S3** THE SYSTEM SHALL accept inserts into `hidden_tests`, `reference_solutions` and `known_ghosts` only for a version whose `sandbox_validated_at` is NULL, and SHALL reject every update and delete of those rows.
4. **CAT-S4** THE SYSTEM SHALL enforce in the database that a problem can be `published` only when its published version is sandbox-validated, has exactly four hint rungs, and has 2 to 5 public examples. The CLI checks these first, but the database is the final gate.
5. **CAT-S5** THE SYSTEM SHALL store a 32-byte SHA-256 `content_sha256` per version, unique for `(slug, version)`, and a `version` of at least 1 that equals the pack's `pack_version`.
6. **CAT-S6** THE SYSTEM SHALL keep `catalogue_events` append-only. Each row records a slug, an action, a version (when relevant), an operator label and a time. It SHALL NOT store pack content or hidden data.
7. **CAT-S7** THE SYSTEM SHALL constrain identifiers to kebab-case slugs (`^[a-z][a-z0-9-]{2,39}$` for problems, `^[a-z][a-z0-9-]{1,39}$` for tracks and concepts), and difficulty to `easy`, `medium` or `hard`.
8. **CAT-S8** THE SYSTEM SHALL add `submissions.problem_version` (nullable, so existing rows remain valid) and foreign keys so that a submission references its problem and, when the version is set, its version.
9. **CAT-S9** THE SYSTEM SHALL never delete a problem, a version or a submission through the catalogue. Foreign keys use `ON DELETE RESTRICT`.

### B. Foreign keys owned by this spec

10. **CAT-F1** THE SYSTEM SHALL add `submissions(problem_slug)` → `problems(slug)` and `submissions(problem_slug, problem_version)` → `problem_versions(slug, version)`.
11. **CAT-F2** WHERE the `hints`, `concept_cards`, `problem_baselines` or `game_scores` tables already exist when the catalogue migration runs, THE SYSTEM SHALL add their `problem_slug` or `concept_id` foreign keys in the same migration. Where a table does not yet exist, the owning spec's migration SHALL reference `problems(slug)` or `concepts(id)` directly, and CAT-F3 applies.
12. **CAT-F3** THE SYSTEM SHALL add new foreign keys `NOT VALID`, so existing rows are not re-checked during migration, while new and updated rows are always checked.

### C. Roles and privileges

13. **CAT-R1** THE SYSTEM SHALL reuse the existing `NOLOGIN` roles `counterspell_server` and `counterspell_judge`. It SHALL NOT create a role.
14. **CAT-R2** THE SYSTEM SHALL revoke all privileges on catalogue tables from `PUBLIC`.
15. **CAT-R3** THE server role SHALL have `SELECT` only on `tracks`, `concepts`, `problems`, `problem_concepts`, `problem_versions`, `problem_examples`, `problem_starters` and `problem_hints`. It SHALL have no privilege on `hidden_tests`, `reference_solutions` or `known_ghosts`, and no write privilege on any catalogue table.
16. **CAT-R4** THE judge role SHALL have `SELECT` on `problems`, `problem_versions`, `problem_examples`, `problem_starters`, `hidden_tests`, `reference_solutions` and `known_ghosts`. It SHALL have no write privilege on any catalogue table and no privilege on `catalogue_events`.
17. **CAT-R5** THE importer and the operator CLI SHALL connect with the operator database URL (`CATALOGUE_DATABASE_URL`). No server or judge process SHALL read that variable.
18. **CAT-R6** THE SYSTEM SHALL test the privilege matrix with `has_table_privilege` for both roles and for `PUBLIC`, on every catalogue table.

### D. Taxonomy

19. **CAT-T1** THE SYSTEM SHALL read tracks and concepts from `packs/catalogue.toml`, which lists each slug and display name. The file is outside every pack directory.
20. **CAT-T2** THE SYSTEM SHALL refuse a pack whose `track` or any `concepts` entry is not listed in `packs/catalogue.toml`.
21. **CAT-T3** Taxonomy sync SHALL upsert names and sort order and SHALL never delete a track or concept that a version refers to.

### E. Importer

22. **CAT-I1** THE SYSTEM SHALL provide `counterspell-catalogue import` with the options `--packs <root>` (default `packs`), `--slug <slug>`, `--dry-run`, `--static-only` and `--sandbox` (the default for production runs). The binary is built from `counterspell-judge` (design decision D2).
23. **CAT-I2** THE SYSTEM SHALL discover packs only as directories directly under the packs root that contain `meta.toml` and whose names do not start with `_`. The `_test` fixtures and any `_` pack (including the dogfood pack) SHALL NOT be imported, and an explicit `--slug` for an `_` pack SHALL be refused.
24. **CAT-I3** THE SYSTEM SHALL refuse a pack whose `meta.toml` `status` is not `published` (a draft pack), write nothing for it, and report `draft_pack`.
25. **CAT-I4** THE SYSTEM SHALL require `counterspell-grimoire` static validation to report zero errors. Warnings are listed in the report and do not block import unless `--deny-warnings` is given.
26. **CAT-I5** THE SYSTEM SHALL build the problem record through the grimoire `ProblemRecord` API. The catalogue SHALL NOT parse pack TOML itself beyond the grimoire model and the manifest.
27. **CAT-I6** THE SYSTEM SHALL compute a canonical manifest: each allowed file's relative path and SHA-256, sorted by path. `content_sha256` is the SHA-256 of the manifest bytes. A change to any file, or to the set of files, changes the hash.
28. **CAT-I7** IF the incoming `content_sha256` equals the latest stored version's hash, THEN THE SYSTEM SHALL write nothing and report `unchanged`. Re-running an unchanged import is a no-op.
29. **CAT-I8** IF the incoming hash differs from the latest version and `meta.pack_version` is greater than the latest stored version, THEN THE SYSTEM SHALL insert exactly one new version (`version = pack_version`).
30. **CAT-I9** IF the incoming hash differs and `meta.pack_version` is not greater than the latest stored version, THEN THE SYSTEM SHALL refuse with `version_not_bumped`, list the changed files by name, and write nothing.
31. **CAT-I10** THE SYSTEM SHALL never update or delete an existing version row, its examples, starters, hints, hidden tests, reference, known ghosts or any history row.
32. **CAT-I11** THE SYSTEM SHALL write each pack in one transaction. A failure in any step rolls back that pack entirely. Other packs in the same run are not affected, and the report states the outcome per pack.
33. **CAT-I12** THE SYSTEM SHALL serialise imports of the same slug with a row lock on `problems` inside the transaction, so two concurrent runs cannot both insert the same version.
34. **CAT-I13** WHERE `--dry-run` is given, THE SYSTEM SHALL write nothing (no row in any table, including `catalogue_events`). It SHALL print, per pack, the planned action (`create`, `unchanged`, `refuse` with a code), the static report, and the manifest diff (added, removed and changed files against the latest version). WITH `--sandbox` it SHALL also run the sandbox checks and print their results, still writing nothing.
35. **CAT-I14** WHERE sandbox mode runs, THE SYSTEM SHALL run the reference on every public example (SBX-01), generate the hidden inputs from the pack's seed and compute their expected values from the reference (SBX-02), and reproduce every known ghost on its program (SBX-03), all through the judge's sandbox `PackRunner`. Any error refuses the pack with no write. Hidden expected values are computed only in this step (design decision D3).
36. **CAT-I15** WHERE `--static-only` is given, THE SYSTEM SHALL write the version with `sandbox_validated_at` and `sandbox_image_digest` NULL and no hidden data. Such a version cannot be published (CAT-S4).
37. **CAT-I16** THE SYSTEM SHALL provide `counterspell-catalogue validate-sandbox <slug>`, which completes a static-only latest version once. It writes hidden data, sets `sandbox_validated_at` and `sandbox_image_digest` in one transaction, and refuses when the version is already validated.
38. **CAT-I17** THE SYSTEM SHALL generate starter code for `python` and `javascript` from the signature, using one fixed template per language. Reviewers see the generated text in the dry run.
39. **CAT-I18** THE SYSTEM SHALL extract `constraints` as the bullet items under the `## Constraints` heading of `problem.md`, in order.
40. **CAT-I19** THE SYSTEM SHALL record the engine's `generator_version` and `engine_version` with each version. A dry run warns when the current engine's generator version differs from the stored one. The importer SHALL NOT regenerate hidden data for an existing version.
41. **CAT-I20** THE SYSTEM SHALL keep logs and reports free of hidden inputs, hidden expected values, the reference source, known-ghost inputs and expected values, and explanation text. Logs may contain the slug, version number, counts, the first 12 hex characters of the hash and the outcome code.
42. **CAT-I21** THE SYSTEM SHALL exit with 0 when every selected pack is `created`, `unchanged` or dry-run clean; 1 when any pack is refused or failed; and 2 for usage errors, a missing sandbox runner (`SBX-07`) or infrastructure failure (`SBX-06`).

### F. Publication (ADM-1)

43. **CAT-P1** THE SYSTEM SHALL provide `counterspell-catalogue publish <slug> [--version <n>]`. It sets the problem to `published` and points `published_version` at the requested version, or at the latest sandbox-validated version when no version is given.
44. **CAT-P2** IF the chosen version fails CAT-S4, THEN THE SYSTEM SHALL refuse with a message that names the failed check and change nothing.
45. **CAT-P3** THE SYSTEM SHALL provide `counterspell-catalogue unpublish <slug>`. It sets the state to `unpublished`, keeps `published_version` and every version, and is idempotent.
46. **CAT-P4** WHILE a problem is unpublished, THE public API SHALL NOT list or return it. Its submissions stay readable by their owner through `GET /api/runs/{id}` and `GET /api/submissions`.
47. **CAT-P5** THE SYSTEM SHALL write a `catalogue_events` row for every import, sandbox validation, publish and unpublish.
48. **CAT-P6** THE SYSTEM SHALL provide no web admin UI and no HTTP route for publication.

### G. Public read API

49. **CAT-A1** THE SYSTEM SHALL serve `GET /api/problems` with optional `track`, `concept` and `difficulty` filters. An unknown filter value SHALL return `400 invalid_filter`.
50. **CAT-A2** WHEN a request carries a valid session, THE SYSTEM SHALL add to each item a `status` of `solved` (an accepted, completed submission exists), `attempted` (a submission exists, none accepted) or `unattempted`. The status SHALL come only from the caller's own submissions (PRB-2). Anonymous responses SHALL set `status` to `null`.
51. **CAT-A3** THE SYSTEM SHALL return each item with `slug`, `title`, `track` (`slug`, `name`), `difficulty`, `concepts` (`slug`, `name`) and `status`, sorted by track order, then slug. Results are capped at 200 items with no cursor (design decision D15).
52. **CAT-A4** THE SYSTEM SHALL serve `GET /api/problems/{slug}` with `slug`, `title`, `track`, `concepts`, `difficulty`, `statement_markdown`, `constraints` (string array), `examples` (`input`, `expected`, public samples only), `signature` (`function_name`, `parameters` with `name` and `type`, `return_type`), `starters` (a map from language to code), `enabled_languages`, `limits` (`time_ms`, `mem_mb`), and `hint_count` (always 4 for a published problem).
53. **CAT-A5** THE detail response SHALL NOT contain hint bodies. Hints are revealed only through the hints API, one rung at a time.
54. **CAT-A6** IF the slug is unknown or unpublished, THEN THE SYSTEM SHALL return `404 not_found` with the same body for both cases, so the response does not reveal that an unpublished problem exists.
55. **CAT-A7** THE SYSTEM SHALL serve `GET /api/languages` from the language catalogue seam. It lists enabled languages with `id`, `display_name` and `max_source_bytes`, and never lists a disabled stretch language (design decision D8).
56. **CAT-A8** EVERY response in this section SHALL be built from explicit response structs. Tests SHALL compare the set of JSON keys with an allow-list (deny by default). Unknown fields are never added by serialising database rows.
57. **CAT-A9** THE SYSTEM SHALL read problem data only through the server role (CAT-R3). The server SHALL contain no SQL that names `hidden_tests`, `reference_solutions` or `known_ghosts`.

### H. Seams and run integration

58. **CAT-L1** THE SYSTEM SHALL change the `ProblemLookup` trait to an async trait (matching `UserResolver`) returning `ProblemRef { slug, version }`. The in-memory test lookup and `EmptyCatalogue` SHALL be updated in the same change, and the existing test assertions SHALL be unchanged.
59. **CAT-L2** THE production `AppState` SHALL use `PgProblemLookup`, which returns only published problems with their published version.
60. **CAT-L3** THE enqueue transaction SHALL write `submissions.problem_version` in the same `INSERT … SELECT` that checks publication, so an unpublish between the check and the insert cannot queue a run for an unpublished problem.
61. **CAT-L4** THE judge SHALL implement the judge's `ProblemTests` seam with `PgProblemTests`. It loads the version pinned on the submission. It returns samples for Run, all tests for Submit, the reference for ghost hunts, and known-ghost data for matching autopsies. It returns `None` when the version does not exist.
62. **CAT-L5** A queued or running submission SHALL finish against its pinned version even after that problem is unpublished or a newer version is published (design decision D10).
63. **CAT-L6** THE SYSTEM SHALL provide a `GhostLookup` seam returning the stored ghost input for a submission the caller owns, for the custom ghost run in `ide-workspace` T02. This requirement is blocked until the owner of the `ghosts` table is confirmed (design open question 7).
64. **CAT-L7** Existing tests (run-lifecycle, judge-sandbox, auth) SHALL keep passing. Where a test inserts a submission for an unknown slug, it SHALL first insert a test problem through the catalogue test kit (cross-spec change X2 in design.md).

### I. Dev and demo seeding

65. **CAT-D1** THE SYSTEM SHALL provide `counterspell-catalogue seed-dev --static-only`, which imports every eligible starter pack without sandbox validation into a local database. Seeded problems are not published.
66. **CAT-D2** THE SYSTEM SHALL provide `counterspell-catalogue seed-demo`, which runs sandbox import for every eligible pack, validates the sandbox results, and publishes the latest version. It requires the sandbox runner and is documented as an operator step on the host with the rootless sandbox daemon.
67. **CAT-D3** BOTH seed commands SHALL be idempotent (CAT-I7) and SHALL skip draft packs with `draft_pack`. Until Brighton approves the starter packs (grimoire-packs T28), the seed commands import nothing and say so.
68. **CAT-D4** THE SYSTEM SHALL document both commands and their environment variable names in a later doc task (task C34 in `tasks.md`), without putting any password or URL in a file.

### J. Confidentiality

69. **CAT-X1** Hidden tests, reference solutions, known-ghost inputs, expected values and explanations SHALL NOT appear in any HTTP response, log line, `run_events` payload, `catalogue_events` row, CLI report or error message.
70. **CAT-X2** Tests SHALL plant canary values in hidden data and assert that no response, log or event contains any of them.
71. **CAT-X3** The only test data a learner may see is the public examples (CAT-A4).
72. **CAT-X4** Errors from the importer and API SHALL use stable codes and safe messages. They SHALL NOT echo SQL text, file contents or program output.

### K. Quality

73. **CAT-Q1** THE SYSTEM SHALL include property tests (proptest) for manifest canonicalisation, idempotence, versioning and the import planner (design properties P1 to P4).
74. **CAT-Q2** Database tests SHALL be named `pg_…`, marked `#[ignore = "needs DATABASE_URL"]`, and use `#[sqlx::test(migrations = "../../migrations")]`.
75. **CAT-Q3** Production code SHALL NOT use `unwrap()`, `expect()` or `unsafe`. The judge crate keeps `#![warn(missing_docs)]`, and Clippy pedantic runs with `-D warnings`.
76. **CAT-Q4** The importer's static path SHALL NOT execute pack code or spawn a process. A purity test SHALL scan the catalogue module for process, network and direct execution APIs. Only the sandbox path reaches the `PackRunner` trait.
77. **CAT-Q5** Every Rust task SHALL end with a short Rust lesson (`.kiro/steering/rust-teaching.md`).

## Acceptance summary

The spec is complete when a reviewer can: import the starter packs in dry-run mode and read the planned actions; see that a second import is a no-op; see that a changed pack needs a bumped `pack_version`; publish only a sandbox-validated version; confirm that the server role cannot select hidden tables; and find no hidden value in any response, log or event in the test run.
