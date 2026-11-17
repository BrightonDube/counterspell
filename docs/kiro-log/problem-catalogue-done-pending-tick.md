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


## C21 publish and unpublish

- Commit: `ff75429`.
- `catalogue::publish::{publish, unpublish}`: `publish` resolves `None` to the latest sandbox-validated version; UPDATE + gate trigger + `catalogue_events` row commit in one transaction, so a refusal leaves no partial state; trigger refusals surface as `GateRefused` carrying the check text verbatim. `unpublish` keeps `published_version`, stamps `unpublished_at`, and is idempotent (no row change, no second event).
- Proving tests: `tests/pg_catalogue_publish.rs` (5 pg tests, all passing): default resolves to the latest validated version, explicit pin agrees with the raw-trigger path in `pg_catalogue_publish_gate`, the failed check text is reported verbatim (`sandbox-validated`), `no_validated_version`/`not_in_catalogue` refusals, unpublish keeps `published_version` + version rows + writes exactly one event, second unpublish is `already_unpublished`.
- Live CLI smoke 16 Nov 2026 on `counterspell_cat2`: `publish two-sum` -> `two-sum created version=1` exit 0; `unpublish` -> `two-sum created` exit 0; second `unpublish` -> `refused:already_unpublished` exit 1.

## C22 list, seed-dev and seed-demo

- Commit: `0306cf2`.
- `catalogue::seed::list` reads `problems` (slug, state, pinned version); `publish_validated` is the shared publish step (`publish` with a `None` pin). `run_seed` is one function with a `demo` flag so the two paths cannot drift: eligibility (real pack dir + `published` status + private part) is decided before any DB connect or runner demand; drafts/missing parts are skipped, not refused; `seed-demo` additionally publishes each imported problem.
- Proving tests: `tests/pg_catalogue_seed.rs` (4 pg tests, all passing): `seed-dev --static-only` and `seed-demo` over the shipped draft packs both print `no eligible packs` and exit 0 with zero problem rows written; `--private-root` is required (exit 2); `list` prints `slug state version` lines including the post-unpublish `unpublished <v>` history shape.
- Live CLI smoke 16 Nov 2026: `list` -> `two-sum unpublished 1` exit 0; `seed-dev --static-only` -> `no eligible packs` exit 0. The `seed-demo` publish step is covered by `pg_catalogue_publish` (it needs the sandbox runner, so it stays out of CI per spec).


## C32 GET /api/languages

- Commit: `a8c72ba`.
- `catalogue::languages::get_languages` clones `state.languages.entries()` into `LanguageSummary { id, display_name, max_source_bytes }` inside `{"languages": [...]}` — the response never shares the `Arc<LanguageCatalog>`; disabled stretch languages (java, cpp) are never in the catalogue so they cannot appear. `seams::Language` gained `display_name`; `LanguageCatalog` gained `entries()`.
- Proving tests: `tests/catalogue_api.rs` — `launch_catalogue_lists_exactly_python_and_javascript` (ids, plus java/cpp absent), `language_keys_match_the_allow_list_exactly`, and `get_languages_returns_the_enabled_languages` over the real router (200, exact key set per entry). 12/12 tests pass; `cargo clippy -p counterspell-server --all-targets` clean.


## C19b refused public-only import and approval-gate tests

- Commit: `8af8074`. Fixtures under `crates/judge/tests/fixtures/c19b/` (published `add-one` pack + `lonesome` with no private part; canary strings only; `approval.toml` written per-test with the recomputed manifest hash).
- `tests/pg_catalogue_private_tree_required.rs` (2 pg tests, passing): bare `import` exits 2 with every catalogue table at zero rows; a private root missing `lonesome` refuses it (`private_part_missing`) while `add-one` is created end-to-end — zero rows for the refused slug.
- `tests/pg_catalogue_approval_gate.rs` (3 pg tests, passing): missing approval -> `approval_missing`; wrong `pack_version` and hash mismatch -> `approval_mismatch`, each through the real binary with zero writes. The in-process test drives the production order (`check_approval` before `validate_sandbox`) with a counting `PackRunner`: 0 calls across all refusals, then `Validated` with calls > 0 once the gate opens.
- Verified live: `cargo test -p counterspell-judge --test pg_catalogue_private_tree_required --test pg_catalogue_approval_gate -- --ignored` -> 5/5 pass.

## C34 operator runbook

- Commit: `30cc2df`. `docs/catalogue-operations.md` names every command (import, validate-sandbox, publish, unpublish, list, seed-dev, seed-demo), both env-var names only, the draft gate, the `pack_version` bump rule, dry-run semantics, the private checkout (CAT-PT14), approval.toml, retired-values.toml and rotation. Linked from `packs/README.md`.

## C36 full database and API pass

- Recorded in `docs/kiro-log/problem-catalogue-verify.md`: judge 65/65, server 29/29. Found and fixed a real regression — `read.rs` decoded the old meta-shape signature while C19 stores the driver-protocol descriptor; `catalogue_no_leak` 500s reproduced it, `0788e1e` decodes the wire shape.

## C37 repo-wide gates

- Recorded in `docs/kiro-log/problem-catalogue-verify.md`: fmt clean, `clippy --workspace --all-targets -D warnings` clean, `cargo test --workspace` 1469 passed / 0 failed, `cargo deny check licenses` ok (one unused-allowance note, not a rejection).

## catalogue-cli crate split (15 Nov decision)

- Commit: `0330f58`. `counterspell-catalogue` binary, `catalogue/load.rs`,
  `catalogue/sandbox_fill.rs` and `sandbox/pack_runner.rs` moved from
  `crates/judge` into new `crates/catalogue-cli`; `crates/judge` no longer
  depends on `counterspell-grimoire`. DB-side catalogue modules (approval,
  manifest, plan, write, taxonomy, publish, seed, private_tree, report,
  starter, statement, discover, retired, testkit) stay in the judge —
  they never needed grimoire.
- Grimoire two-root API: `pack::load_pair`, `validate_static_pair`,
  `validate_sandbox_pair`, `record::build_pair` — the private tree contributes
  `reference/`, `incorrect/` and `known_ghosts.toml` only; duplicates resolve
  private-first deterministically.
- MCP `load_pack`/`validate_pack` are pair-aware with a sibling-checkout or
  `COUNTERSPELL_PACKS_PRIVATE_ROOT` fallback (`paths::private_pack_dir`).
- New `crates/catalogue-cli/tests/catalogue_purity.rs` guards the moved
  importer source (no process/network/git/credential tokens); the judge
  counterpart still pins `PackRunner` out of `src/catalogue`.
- Gates: `cargo fmt --check` clean; `clippy -p catalogue-cli -p grimoire -p
  mcp -p judge --all-targets -D warnings` clean; judge suite green;
  catalogue-cli 49 non-gated tests green; grimoire suite green.
- Real daemon (SANDBOX_DOCKER_SOCKET=/run/user/1000/docker.sock):
  sbx_pack_runner 6/6 incl. starter packs through `validate_sandbox_pair`
  (114.5 s); sbx_pack_props 1/1.

## Verified (spec-driver `verify all`, 17 Nov 2026)

Clean detached worktree at `335968e`, throwaway dev Postgres, `PGPASSWORD` derived from the dev env (the catalogue-cli test helpers build URLs without a password).

| Task | Result |
|---|---|
| C19 | ticked. Bin unit tests, `catalogue_load`, `catalogue_cli` (incl. `pg_cli_dry_run_writes_nothing`) pass. |
| C21 | ticked. `pg_catalogue_publish_gate_*` (6) and `pg_unpublish_keeps_history_and_is_idempotent` pass. |
| C22 | ticked. `pg_catalogue_seed` 4/4 pass. |
| C32 | ticked. `launch_catalogue_lists_exactly_python_and_javascript`, `get_languages_returns_the_enabled_languages` pass. |
| C34 | ticked. `docs/catalogue-operations.md` names all 7 commands and contains no URL or secret. |
| C19b | NOT ticked. `pg_approval_gate_runs_the_sandbox_only_after_approval` fails: it calls the public-only `validate_sandbox`, so after the pack-tree split the stored `add-one` pack reports `LAY-01`/`REF-01` (`reference/solution.py` missing). The test (or its call) must use `validate_sandbox_pair`. The other 4 C19b tests pass. |
| C36 | NOT ticked. Depends on C19b; one pg test is red. |
| C37 | NOT ticked. Gates are not all green at `335968e`: `cargo fmt --check` failed (fixed by the uncommitted rustfmt change in `crates/grimoire/src/lib.rs`) and 12 `counterspell-mcp` tests fail. |

## Pack-split fallout fixes (17 Nov)

- `3477caa`: mcp `program_from` resolves `incorrect/` through the private pack
  root (12 NotFound failures fixed); `validate_pack` test mirrors the pair;
  pg approval-gate test uses `record::build_pair`; `catalogue_url` helpers
  keep the password via `PgConnectOptions::to_url_lossy`.
- `c865ea1`: grimoire-packs T20 verify line amended to
  `counterspell-catalogue validate-sandbox --private-root`; reason logged in
  problem-catalogue-blockers.md.
- Dev env: tests fall back to the sibling `counterspell-packs-private`
  checkout automatically; `~/code/counterspell-packs-private` is the empty
  scaffold — leave `COUNTERSPELL_PACKS_PRIVATE_ROOT` unset or point it at
  `Personal Projects/counterspell-packs-private`.
- TMPDIR=/home/brighton/tmp and CARGO_TARGET_DIR=/home/brighton/cargo-target
  keep builds off the 3.9 GB /tmp tmpfs.
