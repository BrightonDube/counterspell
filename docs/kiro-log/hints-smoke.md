# Hints smoke test (H-verify, manual checklist)

Status: **deferred: E2E.** Brighton's rule for this run is no browser or localhost end-to-end testing. Nothing below was run. Run it by hand on a local stack before the freeze.

Automated checks that did pass on 16 Nov 2026 (this session):

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo test --workspace` | 1,307 passed, 0 failed |
| `cargo test -p counterspell-server -- --ignored hints` and the hints `pg_` and `valkey_` integration tests | all pass (Postgres 17 and Valkey 8 test containers) |
| `cargo deny check licenses` | ok |
| `cd web && npm run test` | 64 files pass |
| `cd web && npm run check` | 0 errors, 0 warnings |
| `cd web && npm run build` | builds |
| `docker compose config` | hint variables pass through as empty; Valkey has `--appendonly yes` |

## Manual checklist (to run by hand)

1. Sign in with the demo account.
2. Open a failed submission and reveal rung 1. Confirm the authored text appears.
3. Request one AI hint on rung 2. Confirm the response is labelled "AI hint" and the notice is visible next to the button.
4. Confirm the quota count went up by one, and that a repeat request for the same rung does not.
5. Run `docker compose logs server`. Confirm it contains no key, no prompt text and no learner code.

Record the result here, with the date and the commit hash, after the run.

## Rerun on attempt 3 (16 Nov 2026, HEAD c0f0612)

Automated checks rerun in this session, all passing:

| Check | Result |
|---|---|
| `cargo fmt --all -- --check` | pass |
| `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo test --workspace` (dev env loaded) | all suites pass, 0 failed |
| `cargo test -p counterspell-server -- --ignored hints` | 5 `pg_` tests pass |
| `cargo test -p counterspell-server -- --ignored valkey_` | 10 hints `valkey_` tests pass, plus the auth `valkey_` test |
| `cargo test -p counterspell-server --test hints_integration -- --ignored` | 2 pass |
| `cargo deny check licenses` | ok |
| `cd web && npm run check` | 0 errors, 0 warnings |
| `cd web && npm run test` | 64 files, 461 tests pass |
| `cd web && npm run build` | builds |
| `docker compose config` | exits 0 |

The manual checklist above is still not run. It is browser work and stays deferred: E2E.
