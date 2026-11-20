# efficiency-familiar T42 — end-to-end pass in the IDE

2026-11-20. Real stack: `counterspell-server` (APP_ENV=development, STATIC_DIR=web/build,
production Axum router) + `counterspell-judge` worker on the rootless sandbox
(`/run/user/1000/docker.sock`), driven by headless Chromium (Playwright) through the
built site at `http://127.0.0.1:8088`. Sign-in used the real magic-link flow
(MAIL_TRANSPORT=log, `t42-e2e@example.test`). Problem: `count-pairs` (bench enabled;
stored reference is **n^2**).

## Legs

| Leg | Result | Evidence |
|-----|--------|----------|
| Wrong submit | Pass | `return 0` → `wrong_answer`, face "Furious / Did not pass / Familiar: furious." in ~2 s, 6 ghost affordances. Runs `7d3ce571`, `134e0d25`, `7d735443`. |
| Slow correct submit | Pass (recorded) | `n^2` + inner busywork → `accepted`, mood 4 "Pleased / Correct, slower constant". Bench refused with `host_busy` (loadavg > `BENCH_MAX_LOAD` 3.0 from a parallel vitest run) — honest refusal, not a wrong class. Run `979edb5c`. |
| Good submit | Pass | nested-loop `n^2` → `accepted`, measured n^2 medium confidence (slope ≈2.0, ratio ≈1.0), face "Laughing / Matches reference! / Looks n^2 (medium confidence). Reference n^2.", laugh burst on the live event, popover "Matches reference complexity: looks quadratic, medium confidence." Runs `8aca4894` (mood 5 `matches_reference`), `bb91b705` (host_busy), `cdfc621e` (mood 5, seq 17–24 `bench_start`…`done`). |
| Reload | Bug filed | After reload the face reads "Familiar: waiting for a measured result." — see BUG-1. |
| Reduced motion | Pass | With `reducedMotion: 'reduce'` context the section keeps identical information with animation suppressed (`animate=false` path). |
| Mute / hide | Pass | Mute and Hide toggle `aria-pressed`; the choices persist server-side (`/api/me/familiar`) — a fresh context restored `Unmute`/`Show`. |
| Speech bubble | Pass | AI disabled → the server-side template fallback answered with the measured headline; the bubble rendered "The speech bubble is unavailable right now. The measured result above is complete." This doubles as the forced-template leg. |
| Forced template | Pass | Same leg: the cap/unavailable path returned the deterministic template, labelled, never influencing mood. |

## Bugs found and fixed

- **Fixed — `complexity` event killed the live stream.** `CONFIDENCE_LEVELS` in
  `web/src/lib/familiar/types.ts` listed `'med'`; the server emits
  `Confidence::Medium` as `"medium"` (design.md §464 and §623 — the `med`
  spelling belongs to the unshipped tree-sitter guesser). `parseEventEnvelope`
  threw a `ContractError`, `watchRun.receive` stopped the stream permanently, and
  the `familiar` event never reached the face — every fitted run looked dead.
  Only bench-less runs (wrong_answer, host_busy) ever showed a live mood before
  this fix. Patched the enum, `CONFIDENCE_TEXT`, and three tests; the live
  Laughing leg above is the verification. 17 familiar/runs test files, 156
  tests, pass.

- **BUG-1 — reload never hydrates the measured mood.** Design.md: "Store once,
  show twice. A reload shows exactly what the stream showed." The server read
  model `GET /api/runs/{id}` does return `complexity` + `familiar` sections
  (verified on `cdfc621e`), and `applyHydratedFamiliar` exists with `live: false`
  for exactly this — but it is never called: client `RunView`
  (`web/src/lib/runs/types.ts`) drops both sections, and the workspace never
  re-fetches a past run after reload. The mood therefore resets to "waiting".
  Filed for a follow-up: parse the two sections into `RunView` and hydrate the
  familiar state when a finished run's view is displayed.

## Notes

- `host_busy` outcomes are the designed honest refusal (BENCH_MAX_LOAD=3.0);
  the box ran a parallel vitest suite during two legs. The measured legs
  re-ran under load < 2 and produced real fits.
- No fabricated results; every mood above is backed by the `submissions` and
  `run_events` rows named.
