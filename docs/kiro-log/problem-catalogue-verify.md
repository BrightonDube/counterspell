# problem-catalogue verification (kiro/drive-queue run)

## C36: full database and API test pass (CAT-Q2)
Run: `cargo test --workspace --no-fail-fast -- --ignored pg_ --test-threads=1` against `cs-dev-pg`.
Result: 373 passed, 3 failed. All three failures are `VersionMismatch(20261118100000)` from the shared dev database, which holds an older draft of the `custom_ghost_runs` migration (stored checksum 8b69d9dd..., file cf324f2f...). They are not code faults:
- `pg_model_interleavings` (judge), `pg_session_model_matches_database` and `pg_consume_agrees_with_the_reference_rule` (server) migrate the shared database directly.
- Each passed on a throwaway database created and dropped for the check (1, 1 and 8 tests).
The shared dev database was not modified. To clear: recreate the `cs-dev-pg` database.

Fixed in this run: `pg_catalogue_privileges_match_matrix_for_each_role` failed because the matrix predated the additive judge `SELECT` grant on `problem_concepts` (migration 20261119090000, needed for the Bug Autopsy concept tags, GHO-6). Test matrix and `design.md` row updated; the test now passes (3/3).

## C19b
`pg_catalogue_approval_gate` 3/3 and `pg_catalogue_private_tree_required` 2/2 pass.

## C37: repo-wide gates (CAT-Q3)
`cargo fmt --all -- --check` ok; `cargo clippy --workspace --all-targets -- -D warnings` ok; `cargo test --workspace` 1508 passed, 0 failed; `cargo deny check licenses` ok.

## C38 — sandbox import of approved starter packs (2026-11-20)

Host: WSL2, rootless docker `unix:///run/user/1000/docker.sock`, image digest recorded per pack in `pack_images`.

- `counterspell-catalogue import --packs packs --private-root /home/brighton/code/counterspell-packs-private --sandbox` → all three approved packs report `unchanged` (count-pairs v4 `1b7eade72efa`, diagonal-total v5 `0a9701bb1057`, longest-run-length v6 `b04d6385903e`). Re-import = `unchanged`, the idempotent leg.
- `GET /api/problems` → all three slugs present (plus seeded add-two, two-sum); no ghost/hidden/seed/reference keys in list or detail responses.
- `approval.toml` regenerated (combined content hash over public+private trees per `build_manifest`) and committed to counterspell-packs-private@45ce0f0 together with the restored `_demo/mcp-dogfood` private parts (reference, never-negative incorrect, known_ghosts) — the earlier copies were uncommitted and lost.

## C40 — rotation gate: remains HUMAN

Hidden values (reference/incorrect/known_ghosts) DO appear in public git history (grimoire-packs T21–T23 commits). `retired-values.toml` still has no `[[retired]]` entries; rotation requires Brighton to decide which pre-split seeds/ghost inputs retire before new values are generated.
