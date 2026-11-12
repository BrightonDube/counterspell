# Tasks: grimoire-packs

All tasks are implementation tasks for the design above. No task in this document is run by this spec-generation step. Each task is sized for 30 minutes or less. `[P]` means it can run in parallel with other tasks in the same wave once its dependencies are done. `[S]` means it is sequential or has a required dependency. `[BLOCKED]` means it waits for another spec.

**Rules for every task:**
- Production code has no `unwrap()`, `expect()` or `unsafe`. Tests may use `expect` with a message.
- Every Rust change ends with a **Rust lesson** of 3–5 plain-English lines in the task's reply, explaining the ownership, borrowing, `Result`/`?`, or trait rule it uses (`.kiro/steering/rust-teaching.md`).
- Do not edit `.kiro/specs/ghost-engine/` or `packs/_test/`.
- Diagnostic codes come from `codes.rs` only. Do not invent a code in another module.

## Wave 0: foundations

- [x] **T01 [S] (25 min) Write the pack format reference for reviewers**
  - Files: `packs/README.md`.
  - Document the layout, the four required files, the optional ghost files, the allowed set, the `_` prefix rule, and the rule that hidden data is never committed by the authoring process. Quote the field tables from `design.md`. Mark the format "provisional until grimoire-packs is reviewed".
  - Depends on: none.
  - Verify: `packs/README.md` lists every LAY, MET, STM, SMP, GEN, HNT, KNG and REF rule in plain English and names `docs/system-design.md` §4, §6, §8, §9.
  - Preserve: `packs/` stays the reviewed-file home; no code in `packs/`.

- [x] **T02 [S] (30 min) Add dependencies, module skeleton, codes and report types**
  - Files: `crates/grimoire/Cargo.toml`, `crates/grimoire/src/{lib.rs,main.rs,codes.rs,report.rs}`, and empty module files listed in `design.md` under "Crate layout".
  - Add `counterspell-engine` (path), `serde` (derive), `serde_json`, `toml`. Keep `GRIMOIRE_VERSION`. In `codes.rs`, define one `pub const` per code in the design table, with severity. In `report.rs`, define `Severity`, `Diagnostic`, `Report` and a JSON-serialisable form.
  - Depends on: none (engine public surface is needed later; the path dependency compiles now).
  - Verify: `cargo check -p counterspell-grimoire`; `cargo metadata --no-deps` still lists six packages; `THIRD-PARTY-NOTICES.md` gets a `toml` entry (MIT OR Apache-2.0).
  - Preserve: no new package, no clap, no sqlx.
  - Rust lesson: explain `enum` for `Severity` and why a `const` per code is safer than typing strings.

## Wave 1: parsing and file safety

- [x] **T03 [S] (30 min) Pack discovery, allow-list, sizes, symlinks and UTF-8**
  - Files: `crates/grimoire/src/pack.rs`, `crates/grimoire/tests/layout.rs`.
  - Implement GRM-1 to GRM-5 with LAY-01 to LAY-05. Walk without following symlinks. Check sizes before reading. Check UTF-8 and control characters on bytes. Check paths stay inside the root.
  - Depends on: T02.
  - Verify: `cargo test -p counterspell-grimoire --test layout` (temp-dir cases for symlink, escape, oversize, NUL byte, non-UTF-8 name, dot file, and a `.pyc` file).
  - Preserve: the module only reads; `purity.rs` will enforce it later.
  - Rust lesson: explain `Result<T, E>` and `?` for I/O errors, and why a size check happens before the read.

- [x] **T04 [S] (30 min) meta.toml model with unknown keys denied**
  - Files: `crates/grimoire/src/meta.rs`.
  - Deserialise into plain structs with `#[serde(deny_unknown_fields)]`. Map TOML errors to MET-01 with line and column. Check required fields and ranges (MET-02), `origin`/`licence`/third-party fields (MET-03), and the slug equals the directory (LAY-06).
  - Depends on: T02, T03.
  - Verify: unit tests in `meta.rs` for each code, including an unknown key and a missing `licence`.
  - Preserve: `input_schema` is held as `counterspell_engine::Schema` (serde tag names come from the engine).
  - Rust lesson: explain why a struct field's type is a contract, and what `deny_unknown_fields` protects against.

- [x] **T05 [S] (30 min) Signature, schema mapping and limits**
  - Files: `crates/grimoire/src/signature.rs`, `crates/grimoire/src/limits.rs`.
  - Implement MET-04 (launch types), MET-05 (tuple arity and kind mapping), MET-06 (call `Schema::check_definition`), MET-07 (comparator rules), MET-08 (launch caps 100–2000 ms, 64–256 MB), and MET-09 (`reference_complexity` set and `[bench]` pairing, GRM-16 and GRM-17). Export the return-shape table.
  - Depends on: T04.
  - Verify: unit tests with one bad input per code, plus a test that `unordered` is refused for `int`.
  - Preserve: caps are constants with a test hook for `languages/languages.toml` (open question 10).
  - Rust lesson: explain `match` on an enum as the way to make every mapping case explicit.

## Wave 2: content checks (parallel)

- [x] **T06 [P] (25 min) Statement checks**
  - Files: `crates/grimoire/src/statement.rs`.
  - STM-01 (title line equals `meta.title`, five headings in order) and STM-02 (8000 characters, no raw HTML tags).
  - Depends on: T03, T04.
  - Verify: unit tests for a wrong title, a missing heading, a heading out of order, and `<script>`.
  - Rust lesson: explain string slices and why `chars().count()` is used for length, not `len()`.

- [x] **T07 [P] (30 min) Sample checks**
  - Files: `crates/grimoire/src/samples.rs`.
  - SMP-01 (2–5), SMP-02 (input accepted by schema, expected matches returns shape), SMP-03 (no duplicate inputs, compared as canonical JSON text).
  - Depends on: T05.
  - Verify: unit tests for each code, including a `1.0` float in place of an int.
  - Rust lesson: explain `serde_json::Value` and why a float is not an integer in this check.

- [x] **T08 [P] (30 min) Generator check through the engine**
  - Files: `crates/grimoire/src/generator.rs`.
  - GEN-01 (seed 0 to 2^63−1, hidden count 5–50) and GEN-02 (draw 200 candidates from `CandidateStream`, check `Schema::contains` and uniqueness).
  - Depends on: T05. Blocked until the ghost-engine public API (`CandidateStream`, `GeneratorConfig`) is merged.
  - Verify: unit test with a schema that yields many candidates; a test that the same seed gives the same first 20 candidates.
  - Rust lesson: explain an `Iterator` and why `take(200)` stops the work early.

- [x] **T09 [P] (25 min) Hint checks**
  - Files: `crates/grimoire/src/hints.rs`.
  - HNT-01 (exactly four entries, rungs 1 to 4 in order) and HNT-02 (1–600 characters, fences of at most 3 lines).
  - Depends on: T04.
  - Verify: unit tests for three hints, a rung out of order, and a 4-line fence.
  - Rust lesson: explain `Vec` and why the rung check uses an index loop rather than a clever iterator.

- [x] **T10 [P] (30 min) Known ghost checks (static)**
  - Files: `crates/grimoire/src/ghosts.rs`.
  - KNG-01 (at most 10, unique kebab `id`), KNG-02 (input accepted by schema, `program` exists under `incorrect/`, defines the function by text), KNG-03 (kind rules), KNG-04 (400 bytes, no forbidden phrase), KNG-05 (concepts subset), KNG-06 (`dogfood` only in `_` packs).
  - Depends on: T05.
  - Verify: unit tests per code. Confirm the forbidden list matches the engine's constant, or that a copy equals it.
  - Rust lesson: explain `Option<T>` for `got` and `exception_class`, and why `kind` is an enum, not a string.

- [x] **T11 [P] (20 min) Reference presence and text check**
  - Files: `crates/grimoire/src/reference.rs`.
  - REF-01: exactly one `reference/solution.py`, and a line starting with `def <function>(` (text match, no execution).
  - Depends on: T03, T05.
  - Verify: unit tests for a missing reference, a misspelled function, and an indented `def`.
  - Rust lesson: explain why this check reads text and does not execute the file, and why that matters for the security model.

## Wave 3: pipeline and CLI

- [x] **T12 [S] (30 min) Static pipeline and report assembly**
  - Files: `crates/grimoire/src/lib.rs`, `crates/grimoire/src/report.rs`.
  - Implement `validate_static(root) -> Report`. Run stages in the order in `design.md`. Skip dependent stages when MET-01 fires, and record the `checked` and `skipped` lists in the report.
  - Depends on: T03 to T11.
  - Verify: a unit test that a pack with a broken `meta.toml` reports MET-01 and lists the content stages as skipped; a clean starter-like pack reports zero errors.
  - Rust lesson: explain `&Path` (borrowed) versus `PathBuf` (owned), and why `validate_static` only borrows the root.

- [x] **T13 [S] (30 min) CLI: validate, --json, --all, --deny-warnings, exit codes**
  - Files: `crates/grimoire/src/main.rs`.
  - Parse arguments with std only. Print text (one line per diagnostic) or JSON. Implement the exit codes 0, 1, 2 from GRM-37, GRM-38 and GRM-39. `--sandbox` returns exit 2 with SBX-07 until a runner exists.
  - Depends on: T12.
  - Verify: `cargo run -q -p counterspell-grimoire -- validate packs/_test/pair-count` exits 0 or 1 with diagnostics, never panics; `--all packs` skips `_test`; `--sandbox` exits 2.
  - Rust lesson: explain `std::process::ExitCode` and why `main` returns it instead of calling `exit` inside helper code.

- [x] **T14 [S] (25 min) ProblemRecord handoff**
  - Files: `crates/grimoire/src/record.rs`.
  - Build `ProblemRecord` only from a pack with zero errors (GRM-54). No database types.
  - Depends on: T12.
  - Verify: unit test that building a record from a valid pack gives the expected slug, version and concepts; `cargo tree -p counterspell-grimoire` shows no `sqlx`.
  - Rust lesson: explain why a function that returns `Result<ProblemRecord, Report>` makes the "valid only" rule part of the type.

## Wave 4: tests that prove the rules

- [ ] **T15 [P] (30 min) One mutation per diagnostic code**
  - Files: `crates/grimoire/tests/mutations.rs`.
  - For each code in `codes.rs` (except sandbox codes), copy a starter pack into a temp directory, apply one text change, and assert the report contains that code and no unrelated error. This is GRM-51. The starter packs must exist first (T21 to T23), so this task runs after Wave 6 starts, or uses `_test` packs as the base until then.
  - Depends on: T12, T21 to T23.
  - Verify: `cargo test -p counterspell-grimoire --test mutations`.
  - Rust lesson: explain why tests can `expect` (test code is allowed to panic with a message) while production code cannot.

- [ ] **T16 [P] (25 min) Purity guard and wording guard**
  - Files: `crates/grimoire/tests/purity.rs`, plus a wording test in `report.rs` or `ghosts.rs`.
  - Scan `src/` for `std::process`, `Command::`, `std::net`, `TcpStream`, `File::create`, `fs::write`, `OpenOptions`, outside `pack.rs` (read only). Scan every diagnostic template for the engine's forbidden phrases (GRM-31 and GHO-6).
  - Depends on: T12.
  - Verify: `cargo test -p counterspell-grimoire --test purity` and the wording unit test pass; a deliberate `Command::new` in a scratch copy fails the scan.
  - Rust lesson: explain `#[cfg(test)]` and why a test that reads the source is a cheap architectural guard.

## Wave 5: sandbox boundary (no live execution in this spec)

- [ ] **T17 [S] (30 min) PackRunner trait and ProgramOracle adapter**
  - Files: `crates/grimoire/src/runner.rs`.
  - Define `PackRunner`, `Target`, `RunnerError` (infrastructure only), and `ProgramOracle { runner, target }` implementing the engine's `Oracle`. No implementation of the trait in this crate except in tests.
  - Depends on: T02. Blocked until the ghost-engine `Oracle` and `ProgramResult` are public.
  - Verify: `cargo check -p counterspell-grimoire`; a doc test shows the trait shape.
  - Rust lesson: explain a trait as a promise a type makes, and why `&mut dyn PackRunner` lets the judge pass in its own type.

- [ ] **T18 [S] (30 min) Sandbox checks SBX-01 to SBX-04 with a model runner**
  - Files: `crates/grimoire/src/sandbox.rs`, `crates/grimoire/tests/sandbox_fake.rs`.
  - SBX-01 (reference on samples), SBX-02 (reference on hidden inputs, in memory), SBX-03 (known ghosts reproduce), SBX-04 (minimality warning through the engine's `shrink`). Use a model runner: plain Rust functions that mirror each starter reference and incorrect program. The file header says this proves logic, not the sandbox.
  - Depends on: T17, T12.
  - Verify: `cargo test -p counterspell-grimoire --test sandbox_fake`.
  - Rust lesson: explain why hidden expected values stay in a local variable and are never returned in the report (ownership makes the value's lifetime visible).

- [ ] **T19 [S] (20 min) Infrastructure exit and bench placeholder**
  - Files: `crates/grimoire/src/sandbox.rs`, `crates/grimoire/src/main.rs`.
  - SBX-06 (runner infrastructure error maps to exit 2, not a pack error). SBX-05 and SBX-08 (bench: emit the "not checked" warning until `PackRunner::bench` is implemented).
  - Depends on: T18.
  - Verify: a fake runner that returns `RunnerError` gives exit 2 with SBX-06; a pack with `[bench]` gives SBX-08.
  - Rust lesson: explain why infrastructure errors are a separate type from program results, so they cannot be mixed up.

- [ ] **T20 [BLOCKED] Judge implements PackRunner and wires `--sandbox`**
  - Files (in the judge spec, not here): `crates/judge/` implements `PackRunner` through the rootless sandbox. This spec only records the requirement.
  - Depends on: `judge-sandbox` spec, and a decision on open question 7.
  - Verify (later): `counterspell-grimoire validate packs/count-pairs --sandbox` runs the reference through the real sandbox, and the vectors in `sandbox_fake.rs` match the real Python output.

## Wave 6: starter packs

- [ ] **T21 [P] (30 min) Starter pack `count-pairs`**
  - Files: `packs/count-pairs/{problem.md,meta.toml,hints.toml,known_ghosts.toml,reference/solution.py,incorrect/same-index.py,incorrect/empty-min.py}`.
  - Write the statement from scratch (GRM-49). Use the signature and schema in `design.md`. Use `status = "draft"`, `seed = 1729`, `hidden_count = 12`. Two hand-checked samples (or more, up to 5). Ghosts `same-index` (`[[0], 0]`, got 1) and `empty-min` (`[[], 0]`, ValueError), as in `design.md`.
  - Depends on: T12, T13 (to run the validator).
  - Verify: `cargo run -q -p counterspell-grimoire -- validate packs/count-pairs` exits 0 with no errors. Warnings are listed and explained.
  - Rust lesson: not applicable to these data files. Note in the reply that the Python reference is reviewed as text, not run.

- [ ] **T22 [P] (30 min) Starter pack `longest-run-length`**
  - Files: `packs/longest-run-length/...` (same file set).
  - Signature `longest_run(text: str) -> int`, alphabet `abc`, length 0–40. Ghosts `forgot-last-run` (`["a"]`, got 0) and `empty-first-char` (`[""]`, IndexError). Incorrect programs as in `design.md`.
  - Depends on: T12, T13.
  - Verify: as T21 for this slug.

- [ ] **T23 [P] (30 min) Starter pack `diagonal-total`**
  - Files: `packs/diagonal-total/...`.
  - Signature `diagonal_total(grid: list[list[int]]) -> int`, matrix rows 0–6, columns 0–6, values −9 to 9. Ghosts `abs-diagonal` (`[[[-1]]]`, got 1) and `assumes-square` (`[[[]]]`, IndexError).
  - Depends on: T12, T13.
  - Verify: as T21 for this slug.

- [ ] **T24 [S] (30 min) Starter packs test and zero-error gate**
  - Files: `crates/grimoire/tests/starter_packs.rs`.
  - Validate every `packs/<slug>` (not `_`) and assert zero errors (GRM-52). Fix any real defect in a starter pack, not the test.
  - Depends on: T21 to T23.
  - Verify: `cargo test -p counterspell-grimoire --test starter_packs`.
  - Rust lesson: explain iterating a `read_dir` result and why each entry can fail, so it is a `Result`.

- [ ] **T25 [S] (30 min) Model vectors for starter programs**
  - Files: `crates/grimoire/tests/sandbox_fake.rs` (extend), `crates/grimoire/tests/support/` if needed.
  - Add plain Rust model functions and the hand-checked vectors for the three references and six incorrect programs, and assert SBX-03 and SBX-04 expectations with the model runner. Record in the file header that T20 must rerun these vectors through the real sandbox.
  - Depends on: T18, T21 to T23.
  - Verify: `cargo test -p counterspell-grimoire --test sandbox_fake`.
  - Rust lesson: explain `fn` pointers versus closures when the model functions are stored in a table.

## Wave 7: gates and handover

- [ ] **T26 [S] (25 min) Repo-wide gates**
  - Files: none new. Fix only grimoire files.
  - Run `cargo fmt --all -- --check`, `cargo clippy --workspace --all-targets --all-features -- -D warnings`, `cargo test -p counterspell-grimoire`, `cargo deny check licenses`.
  - Depends on: T01 to T25.
  - Verify: all four commands exit 0. If `cargo deny` rejects a licence that `toml` pulls in, stop and ask Brighton before editing `deny.toml`.
  - Rust lesson: explain what `-D warnings` does: it turns Clippy warnings into errors so they cannot be ignored.

- [ ] **T27 [S] (15 min) Handover note for the server and judge specs**
  - Files: `crates/grimoire/README.md` (short) or a section in `packs/README.md`.
  - State the public entry points (`validate_static`, `validate_sandbox`, `ProblemRecord`, `PackRunner`), the codes, the blocked items (T08, T17, T20), and open questions 1 and 7 as decisions needed before import.
  - Depends on: T26.
  - Verify: the note names every blocked task and its owning spec.

- [ ] **T28 [BLOCKED on Brighton] Review and publish starter packs**
  - Human step, not agent work. Brighton reads each statement, hint, ghost explanation and reference, then changes `status` to `published` (GRM-47). Agent work stops at `draft`.
  - Depends on: T24, T25, and Brighton's review.
  - Verify: each pack's `status` is `published` only after Brighton's sign-off recorded in the commit message.

- [ ] **T29 [BLOCKED on ghost-engine] Convert `packs/_test/` to the new format**
  - Files: `packs/_test/*` (only after ghost-engine ships and its tests are updated).
  - Depends on: `ghost-engine` spec complete, and a decision on open question 11.
  - Verify: `validate --all packs --include-underscore` passes for the converted fixtures, and ghost-engine tests still pass.

- [ ] **T30 [BLOCKED on CI spec] Add pack validation to CI**
  - Files: `.github/workflows/ci.yml` (in the CI spec).
  - Add a step that runs `cargo run -q -p counterspell-grimoire -- validate --all packs --deny-warnings`.
  - Depends on: T26, and the CI owner.
  - Verify: the CI run passes on a clean checkout.
