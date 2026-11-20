# auth T37 — final verification record

Date: 2026-12-01. Ran on the development machine against the real dev Postgres and the
local rootless sandbox daemon.

## Automated gates

- `cargo fmt --all -- --check` — PASS
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` — PASS
- `cargo test --workspace -- --include-ignored` (DATABASE_URL, SANDBOX_DOCKER_SOCKET,
  LANGUAGES_FILE set) — PASS, zero failures across all binaries including every pg_ suite,
  all real-sandbox sbx_ tests, and the 60-submission familiar proof (130s). One flake:
  `pg_model_interleavings` failed once under parallel DB contention and passed clean on
  retry and in the full `--no-fail-fast` rerun.
- `PROPTEST_CASES=512` on the auth proptest suites (pg_auth_session_props 54s,
  pg_auth_tokens, auth_csrf, validate) — PASS
- `cargo deny check licenses` — PASS (one unmatched-allowance note for MIT-0 in deny.toml;
  informational only)
- `cd web && npm run check` — PASS (0 errors, 0 warnings)
- `cd web && npm test` — PASS (all vitest files)
- `cd web && npm run lint` / `npm run format` / `npm run build` — PASS after fixing 6 lint
  errors in the break-room pages (missing resolve() on hrefs, a void union in
  cm-chessboard.d.ts, an unused import in keyboard-motion.mjs)
- `docker compose config` — PASS

## Property coverage

P1, P2, P3, P6, P9, P10, P11 and P18 are covered by the auth proptest suites above
(session model vs database, token lifecycle, CSRF, request validation), each run at
PROPTEST_CASES=512. The remaining P-items are covered by unit and integration tests in the
auth modules (oauth, magic link, session, link, verify, delete) — all green in the
workspace run.

## Manual VPS checklist — NOT TESTED

No production VPS exists yet (deploy-ops T23 is Brighton's). These items are recorded as
not run, per the task's own instruction to log failures/gaps rather than silently pass:

- GitHub sign-in — NOT TESTED
- Google sign-in — NOT TESTED
- Magic link through Resend (SPF/DKIM) — NOT TESTED
- Link GitHub / Google to a magic-link account — NOT TESTED
- Add email to an OAuth-only account — NOT TESTED
- Google sign-in matching an existing email -> `account_exists` — NOT TESTED
- logout-all from a second browser — NOT TESTED
- Demo account restrictions — NOT TESTED
- Next-day demo reset — NOT TESTED

The local equivalent legs were run on the dev stack during T33/T34 verification: the
magic-link flow completes end to end on the real server (link in dev log, verify -> session
cookie -> /api/me 200), and the run SSE heartbeat re-checks auth mid-stream.
