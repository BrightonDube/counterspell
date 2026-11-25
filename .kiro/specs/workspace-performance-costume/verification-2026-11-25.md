# Workspace performance and costume — verification (local, 2026-11-25)

Profile (fixed): 390x844, CDP Fast-3G shaping (150 ms RTT, 1.6 Mbps / 750 kbps),
4x CPU throttle — the Lighthouse mobile numbers. Harness: `web/e2e/workspace-perf.spec.js`.

## Before / after (`baseline-before.json` vs `baseline-latest.json`)

| Metric | Before | After |
|---|---|---|
| `workspace-shell-interactive` | 3080 ms | 2877 ms |
| `problem-detail-loaded` | 3034 ms | 2832 ms |
| `monaco-ready` | 18016 ms | 17258 ms |
| Requests (to networkidle) | 46 | 42 |
| Transfer (to networkidle) | 3 284 112 B | 3 184 683 B |
| Script bytes | 2 997 774 B | 2 904 675 B |
| CLS | 0.0003 | 0.0003 |

The API requests now START at ~2029 ms instead of ~2800 ms: the root layout's
`load()` warms `/api/problems/{slug}` + `/api/languages` while the page module
is still downloading, instead of the component fetching after mount. The shell
mark is bounded by route-chunk evaluation, not the fetch — and lands under the
3000 ms contract.

## Stage ordering (asserted green in the readiness test)

- No `/art/familiar/` media request starts before `monaco-ready`.
- No `/api/me/familiar` request starts before `monaco-ready` — and signed-out
  visits never send it (Familiar prefs/speech are session-gated).
- Script bytes before the shell mark: 180 174 B (< 350 kB ceiling).
- Script bytes before `monaco-ready`: ~2.9 MB (< 3.0 MB ceiling; Monaco core
  alone is ~2.64 MB — language packs and contribs load after the mark).

## WP06/WP07 evidence

- `workspace-motion.spec.js` (e2e, green): ember `animation-duration` collapses
  under `reducedMotion: 'reduce'`; Ctrl+Enter runs, Ctrl+Shift+Enter submits;
  language picker, hide/show problem, Run and Submit all answer on the
  testserver.
- `AnimatedArt.test.ts` (component, green): reduced-motion mounts only the
  still `img` — no `video` element; full motion mounts the loop; lazy/async
  hints stay on the fallback.
- `workspace-css.test.ts` (green): `ember-drift` animates transform/opacity
  only; `.workspace-ember` is pointer-events:none; the global reduced-motion
  collapse covers it.

## WP09 — production source-map policy

**Decision: ship no source maps.** Nothing consumes them (no telemetry sink),
public maps expose readable source for zero benefit. `vite.config.js` sets
`build.sourcemap: false` explicitly; `web/scripts/verify-artifact.mjs` runs as
the last build step and fails if any `*.map` file or `sourceMappingURL`
reference reaches `web/build`. If telemetry arrives later, switch to `hidden`
maps uploaded in CI — never deployed.

Local verification: `node scripts/verify-artifact.mjs` → clean; `find web/build
-name "*.map"` → 0 files. Production re-verification (no `.map` reachable at
counterspel.online) lands in the post-deploy check for this PR.

## Production evidence (post-deploy, `d2aa1086`)

Deploy workflow succeeded on the merge commit. Verified against
`https://counterspel.online`:

- `GET /api/health` → `{"status":"ok"}`.
- Edge headers present: `Strict-Transport-Security: max-age=31536000;
  includeSubDomains`, `X-Content-Type-Options: nosniff`.
- `GET /api/problems` → 200 with the live catalogue; `GET
  /api/problems/count-pairs` → 200 problem detail (the `two-sum` slug used
  by the e2e fixture is test-only and correctly 404s in production).
- `GET /problems/count-pairs` serves the SPA shell with the fresh
  `_app/immutable` chunk graph — the warm-fetch layout path is live.
- Source-map policy: `.map` URLs under `_app/immutable/` 404 and served
  chunks contain no `sourceMappingURL` reference — WP09 verified in
  production.

Review round on PR #120: 3 Devin findings (familiar session gating on
direct visits, partial-preference clobbering, Monaco required-contribution
staging) fixed in `cb423cae` with regression tests; 3 CodeRabbit minors
(request-level run assertions, `waitForFunction` options slot, stale
`monaco-ready` mark scoping) fixed in `d5edf5dd`. Final head: 34 CI checks
green, review marks resolved, squash-merged `d2aa1086`.
