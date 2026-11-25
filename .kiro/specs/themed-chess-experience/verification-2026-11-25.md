# Themed chess experience — post-deploy verification (2026-11-25)

Merge: `e9b552f9` (PR #124). Deploy run succeeded on `e9b552f9`.

## Live checks (https://counterspel.online)

- `GET /api/health` → `{"status":"ok"}`
- Edge headers: HSTS `max-age=31536000; includeSubDomains`,
  `X-Content-Type-Options: nosniff`, CSP with `connect-src 'self'`.
- Routes: `/break`, `/break/bot`, `/break/puzzle` → 200 text/html
  (8263 B / 6804 B / 7037 B).
- SSR markup on both chess routes: `class="break-board"` container with
  `aria-label="Chessboard"` — the single accessible name contract holds.
- Sprite assets (same-origin, required by `connect-src 'self'`):
  - `/break/pieces-steel-night.svg` → 200 `image/svg+xml`, 8628 B
  - `/break/markers.svg` → 200 `image/svg+xml`, 3341 B
- Deployed chunks match the pushed build (`app.D_sVLS57.js`,
  `4.-PTJJpq5.js`); shared chunk `CCkMlFNT.js` carries `steel-night`,
  `pieces-steel-night`, `markers.svg` and `marker-frame-primary`.

## Local evidence (pre-merge)

- 727 vitest tests green (5 chess suites: theme, pieces, chess-css,
  bot page, puzzle page).
- `svelte-check` 0 errors / 0 warnings; eslint clean; prettier clean.
- Build green; `artifact check: no source maps in build output` (WP09).
- Real-browser (Playwright headless chromium against `vite preview`):
  - desktop 1440×900: board mounts at 580 px, full coordinates above the
    fold, skull sentinel centred; e2→e4 drag paints `marker-frame-primary`
    on both squares, then the bot's h7→h6 reply reframes the exchange.
  - mobile 390×844: 358 px board fills the column, no overflow.
  - short 568×320: `min-inline-size: 280px` floor holds — board playable.
  - `/break/pieces-steel-night.svg` and `/break/markers.svg` XHR 200.
- axe on `/break/bot` and `/break/puzzle`: 13 e2e checks green
  (`e2e/a11y.spec.ts`).

## Review rounds on #124

- Devin: 4 findings — all fixed and marked Resolved
  (board floor; isolation move to `lib/break/`; same-origin sprites;
  real last-move frames via `marker-frame-primary`).
- CodeRabbit: 2 minors — fixed in `3e48d054`
  (position getter evaluated after lazy imports; post-mount history
  resync so a pre-mount Play-Black opening keeps its frame).

## Behaviour preservation

chess.js validation, bot worker + random fallback, puzzle `tryMove`,
session-gated `saveScore` (`chess_bot` / `chess_puzzle`) all unchanged —
the presentation layer only wraps `enableMoveInput` validation and adds
markers after accepted/applied moves.
