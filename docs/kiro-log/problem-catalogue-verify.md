# problem-catalogue verification

## C36 — full database and API pass (16 Nov 2026)

Throwaway Postgres on `localhost:5432` (`counterspell` container), `DATABASE_URL`
set, every `pg_*` catalogue suite run with `--ignored`.

Command: `cargo test -p counterspell-judge --test "pg_*" -- --ignored` plus the
server catalogue suites below.

### Judge — 65/65 passed

| Suite | Tests | Result |
|---|---|---|
| pg_catalogue_dry_run | 2 | pass |
| pg_catalogue_fks | 4 | pass |
| pg_catalogue_immutability | 7 | pass |
| pg_catalogue_import | 4 | pass |
| pg_catalogue_judge_ghosts | 2 | pass |
| pg_catalogue_judge_reads | 5 | pass |
| pg_catalogue_privileges | 3 | pass |
| pg_catalogue_publish | 5 | pass |
| pg_catalogue_publish_gate | 6 | pass |
| pg_catalogue_sandbox_import | 4 | pass |
| pg_catalogue_schema | 6 | pass |
| pg_catalogue_seed | 4 | pass |
| pg_catalogue_taxonomy | 2 | pass |
| pg_catalogue_testkit | 3 | pass |
| pg_catalogue_validate_sandbox | 4 | pass |
| pg_problem_tests | 3 | pass |
| catalogue_cli (pg_cli_dry_run_writes_nothing) | 1 | pass |

### Server — 29/29 passed

| Suite | Tests | Result |
|---|---|---|
| catalogue_api | 12 | pass |
| catalogue_no_leak | 2 | pass |
| pg_catalogue_api | 7 | pass |
| pg_catalogue_lookup | 5 | pass |
| pg_catalogue_pin | 3 | pass |

### Failure found and fixed during this pass

`catalogue_no_leak` failed 2/2 with `GET /api/problems/two-sum` returning 500.
Cause: `problem_versions.signature` stores the driver-protocol descriptor
(`parameters`/`returns` type descriptors) since C19, but `read.rs` decoded the
earlier meta shape (`params`/`returns` strings). Fixed in `0788e1e`: the detail
read now decodes the wire shape and maps descriptors back to launch type names.
Re-run: all suites green.


## C37 — repo-wide gates (16 Nov 2026)

| Gate | Result |
|---|---|
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo test --workspace` | **1469 passed, 0 failed** across 195 test binaries |
| `cargo deny check licenses` | ok (one non-blocking note: `MIT-0` allowance not encountered — an unused allowance, not a rejected licence) |

Environment note: `target/` on the Windows-mounted repo path had grown to
~205 GiB and filled C:, failing the linker with `No space left on device`.
`cargo clean` reclaimed it; the workspace now builds under
`CARGO_TARGET_DIR=/home/brighton/target-counterspell` (the WSL ext4 disk).
