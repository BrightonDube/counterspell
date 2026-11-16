# problem-catalogue: done, pending tick

Each entry: task id, commit(s), tests that prove it, the exact verification commands and their result. A task is done only after Kiro re-verifies and ticks it.

## C19 CLI binary and exit codes

- Commits: `9cfd94d` (CLI + `catalogue::load` + `packs/catalogue.toml`), `dfa25d9` (review fixes).
- Proving tests: `src/bin/counterspell-catalogue.rs` `#[cfg(test)]` (11 arg-parse / exit-map / missing-`--private-root` tests), `tests/catalogue_load.rs` (11 tests over tempdir packs: both-tree read, manifest prefixes + approval exclusion, private-first `approval_text`, `private_sources` hashes, `assemble_import`/`assemble_sandbox` happy paths and refusals), `tests/catalogue_cli.rs` (8 real-binary exit-code tests via `CARGO_BIN_EXE`, incl. missing `--private-root` => 2 with no `CATALOGUE_DATABASE_URL`).
- Pipeline: join -> draft gate -> `validate_static` -> taxonomy -> manifest -> approval -> assemble -> plan/write; `--private-root` preflight (C08a) before any database connection; 0/1/2 exit mapping; sandbox paths exit 2 `SBX-07` until a `PackRunner` is wired (`pack_runner()` returns `None`; `validate_one` already takes `&mut dyn PackRunner`).
- Verification run 16 Nov 2026 against the real scaffold (packs published by T28 `fc73f3c`):
  - `target/debug/counterspell-catalogue import --dry-run --packs packs --private-root /home/brighton/code/counterspell-packs-private` on a fresh migrated DB: three `refused:approval_missing` lines, exit 1, and `tracks`/`concepts`/`catalogue_events`/`problem_versions` all at **0 rows** — dry-run writes nothing.
  - Same command with `--static-only`: taxonomy synced (3 tracks, 8 concepts), all three packs refused `approval_missing`, zero `catalogue_events`/`problem_versions` — approval gates before any pack write (CAT-PT7).
  - Exit 2 confirmed live for: missing `--private-root`, missing root dir (`private_tree_missing`), root inside the public tree (`private_root_inside_public`), bare `import` and `validate-sandbox` (`SBX-07`), unknown verb.
  - Synthetic private root produced `private_part_missing` and `orphan_private_pack`; `--slug _demo` gave `underscore_pack`; `--json` emits proper report lines (slug, code, counts, hash prefix only — no private paths or values).
- Gates: `cargo fmt --all -- --check` ok; `cargo clippy -p counterspell-judge --all-targets` clean; 11 bin + 8 cli + 11 load + 66 catalogue + 5 purity tests green; `pg_catalogue_validate_sandbox` 4/4 unchanged.
- Note: `publish`/`unpublish`/`list`/`seed-dev`/`seed-demo` parse but report "implemented under C21/C22" — that is their tasks' scope.

## C19 review follow-ups (since-1754)

- Commits: `821b982` (wire-shape signatures in stored imports), `8c57df7` (panic removal + pg CLI dry-run proof).
- `import --dry-run` no longer calls `sync_taxonomy`: taxonomy sync moved behind the non-dry-run branch; `pg_cli_dry_run_writes_nothing` (`tests/catalogue_cli.rs`) runs the real binary against a migrated DB and asserts tracks/concepts/problems/problem_versions/catalogue_events all stay at 0 rows. Passed 16 Nov 2026.
- Production-path panics removed: `runtime`/`pack_runner` return `ExitCode`/`Option` instead of `.expect`/`unimplemented!()` (missing runner -> `SBX-07`, exit 2); both `unreachable!()` in `sandbox_fill.rs` replaced by `returned_shaped`, which extracts the `Returned` value it validated — a mismatch refuses `ReferenceFailed` instead of panicking.
- `catalogue_load.rs` direct tests added (`tests/catalogue_load.rs`, 11): public+private tree read, manifest prefixes and approval exclusion, private-first `approval_text`, `private_sources` sha256, `assemble_import`/`assemble_sandbox` happy paths, and `private_part_missing`/`orphan_private_pack`/`underscore_pack`/`draft_pack`/`approval_missing` refusals.
- Real-binary exit-code tests: missing `--private-root`, missing root dir, root inside public tree, and bare sandbox verbs all exit 2 (`tests/catalogue_cli.rs`).

## C26 review follow-up (since-1754)

- Commit: `caf986d`.
- `tests/pg_problem_tests.rs` (3 pg tests, all passing): `Run` returns the two public examples with `is_sample` and no hidden rows plus the joined reference; `Submit` returns 2 samples + 5 hidden; versions 0 (the worker`'s unpinned marker) and 2 return `None`, which the executor reports as `problem_unavailable`. Asserts the stored signature is the driver-protocol descriptor.
- `testkit` fixtures now insert wire-shape signatures (`protocol_version`/`parameters`/`returns` descriptors) matching what C19 imports actually store; dependent pg suites re-verified (testkit 4/4, judge_reads 5/5, import 4/4, validate_sandbox 3/3, sandbox_import 4/4).
