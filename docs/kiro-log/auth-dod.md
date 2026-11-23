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

## Manual VPS checklist — 2026-11-23 update

The original note predated production deployment and email-auth removal. The applicable
live checks were later observed on `https://counterspel.online` from Brighton's signed-in
browser:

- GitHub sign-in/account identity — PASS
- Google sign-in/account identity — PASS
- Account settings after OAuth sign-in — PASS
- Magic link through Resend (SPF/DKIM) — SUPERSEDED: email auth was removed in favour of
  OAuth plus the demo-password path
- Link GitHub / Google to a magic-link account — SUPERSEDED: magic-link accounts no longer
  exist in production
- Add email to an OAuth-only account — SUPERSEDED: email collection is no longer a sign-in
  requirement
- Google sign-in matching an existing email -> `account_exists` — covered by the auth model
  tests; not separately repeated in the browser
- logout-all from a second browser — NOT TESTED
- Demo account restrictions — NOT TESTED in the live browser
- Next-day demo reset — NOT TESTED against the live clock

The local equivalent legs were run on the dev stack during T33/T34 verification: the
magic-link flow completed end to end before email auth was retired (link in dev log,
verify -> session cookie -> /api/me 200), and the run SSE heartbeat re-checks auth
mid-stream.
