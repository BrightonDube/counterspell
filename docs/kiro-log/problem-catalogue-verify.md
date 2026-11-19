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
