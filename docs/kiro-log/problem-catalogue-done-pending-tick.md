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
