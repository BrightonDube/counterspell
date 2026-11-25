# CP12 verification evidence — 2026-11-25

All evidence produced against the real Axum testserver (`counterspell-testserver`)
serving the production static build, driven by Windows Chrome through Playwright's
`E2E_EXECUTABLE`. Testserver ran in WSL with `DATABASE_URL` via `WSLENV`,
`RUN_RATE_IP=10000`, `CS_TEST_PORT=5198`, `STATIC_DIR=web/build`.

## Unit / static gates

- `npx vitest run` — 681 tests green, 0 unhandled errors (new coverage: session
  generation guard, login sign-in invalidation, memory/puzzle anonymous-save
  gates, app.html shell metadata).
- `npx svelte-check --threshold error` — 0 errors, 0 warnings.
- `npm run lint` — clean.
- `cargo test -p counterspell-server --test http` — 28/28 (includes the
  extensionless prerendered-route regression test).
- `cargo check -p counterspell-testserver` — clean.
- `npm run build` — production build green; `break/bot.html`,
  `break/puzzle.html`, `break/memory.html` now prerender with title/canonical/OG.

## AQ08 browser verification — `web/e2e/a11y.spec.js`

13/13 green (54s):

- axe-core: 0 verified violations on landing, login, problems, break room,
  chess bot, daily puzzle, dashboard (signed-out and signed-in via Bearer).
- Direct-HTTP metadata: `/`, `/login`, `/problems` all serve prerendered HTML
  carrying title, description, canonical, `og:*` and `twitter:*` tags.
- Chessboard: exactly one accessible image name (`role="img" aria-label` on the
  container; injected svg `aria-hidden`); back link ≥ 24px.
- Keyboard: Tab from the nav reaches the first break-room game.
- Signed-out `/break`: no 401s, no `/api/me`, no `/api/breakroom/scores`; the
  only console noise is the documented empty-catalogue `not_imported` 503.
- Session answer fetched once across all consumers.
- Chess bot board CLS < 0.1.

## Lighthouse 12 (headless Chrome, local testserver)

| Route | Perf | A11y | SEO | Best-practices | CLS |
|-------|------|------|-----|----------------|-----|
| `/` | 0.77 | 1.00 | 1.00 | 1.00 | 0 |
| `/login` | 0.96 | 1.00 | 1.00 | 0.96 | 0.017 |
| `/problems` | 0.85 | 1.00 | 1.00 | 1.00 | 0.085 |
| `/break` | 0.95 | 1.00 | 1.00 | 0.96 | 0.001 |

- Accessibility and SEO are 1.00 on every audited route; CLS stays under the
  0.1 "good" threshold everywhere.
- The two 0.96 best-practices deductions are `errors-in-console`:
  `/api/breakroom/puzzle/daily` 503 (`not_imported` — the documented
  empty-catalogue contract, absent once puzzles are imported) and, before the
  testserver shim was added, `/api/auth/config` 404. The shim now answers the
  production-shaped config, so `/login` is clean on the next run.
- Performance scores reflect the unoptimized local testserver plus Lighthouse
  throttling; they are recorded as the baseline, not waived — production's
  edge/caching is measured separately after deploy.

## Review

- Code review (MAJOR_FIXES → resolved): stale `signed_out` cache after in-page
  sign-in fixed by `session.clear()` before navigation; generation counter
  guards `clear()` against in-flight loads; memory/puzzle scores gated on the
  resolved session; `app.html` default title/description; `no-store` on the
  session probe shim; worker construction moved after the detach guard.
- Declined findings, with reasoning: `/api/` early-out in
  `resolve_prerendered_html` (the api router owns `/api/*` before the static
  fallback — unreachable) and percent-decoding the probe path (encoded paths
  still resolve via ServeDir's own decode; canonical URLs are unencoded).
- Demo-user capability reduction in the testserver shim: declined — fixture
  users are never `is_demo`, and no test exercises demo restrictions.
