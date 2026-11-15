# Auth definition of done: results (15 Nov 2026 run)

Record for auth T37. Task T37 stays unchecked because its manual checklist needs the VPS and real services (see the last section).

## Automated gates (this session, dev Postgres 17 from the runner environment)

| Gate | Command | Result |
|---|---|---|
| Format | `cargo fmt --all -- --check` | pass |
| Clippy | `cargo clippy --workspace --all-targets --all-features -- -D warnings` | pass |
| Tests | `PROPTEST_CASES=512 cargo test --workspace --no-fail-fast -- --include-ignored --skip sbx_` | 1331 passed, 0 failed |
| Licences | `cargo deny check licenses` | ok (one unmatched-allowance warning, pre-existing) |
| Compose | `docker compose config -q` | pass |
| Web check | `cd web && npm run check` | 0 errors, 0 warnings |
| Web tests | `cd web && npm run test` | 60 files, 411 tests passed |
| Web build | `cd web && npm run build` | pass |

Skipped on purpose: `sbx_*` tests. They need `SANDBOX_DOCKER_SOCKET` (rootless sandbox daemon, judge-sandbox T37). Running them without the socket fails with `NotPresent`, which is the judge crate's test, not an auth test.

## Property coverage (design.md P1 to P22)

Each property has tests in `crates/server/tests/`. The file names below are the auth tests that carry the property. Full list: `grep -rn "P[0-9]" crates/server/tests | grep auth`.

- P1, P2, P3, P4 (tokens): `pg_auth_tokens.rs`
- P5, P8 (decisions and link): `pg_auth_oauth_model.rs`, `pg_auth_link_provider.rs`, unit tests in `auth::decision` and `auth::capabilities`
- P6, P7 (OAuth sign-in and flow): `pg_auth_oauth_model.rs`, `pg_auth_flow.rs`
- P9 to P13 (sessions): `pg_auth_session_props.rs`, `pg_auth_session.rs`
- P14 (magic link): `auth_magic.rs`
- P15 (email parser): unit tests in `auth::email`
- P16 (return_to): unit tests in `auth::return_to`
- P17, P18 (demo): `pg_auth_demo_matrix.rs`, `pg_auth_demo_db.rs`, `pg_auth_demo_reset.rs`
- P19 (rate limits): `auth_limits.rs`, `valkey_auth` tests
- P20 (privileges and cascade): `pg_auth_privileges.rs`
- P21 (log canaries): `auth_logs.rs` (T35, this run)
- P22 (CSRF guard): `auth_csrf.rs`

## Manual checklist (deferred, not run)

Needs a running stack, real provider apps and Resend. Not run in this session. Results go here when someone runs them.

- GitHub sign-in, Google sign-in
- magic link through Resend (SPF and DKIM pass)
- link GitHub to a magic-link account, link Google
- add an email to an OAuth-only account
- sign in with a Google account whose email matches an existing account and confirm `account_exists`
- logout-all from a second browser
- the demo account (restrictions visible)
- the next-day demo reset

## Open items that block T37

- T33 (Caddy log filter, open question 16) and T34 (SSE heartbeat check) are still unchecked. See `auth-blockers.md`.
- The manual checklist above.
