# landing-steel-night blockers

Run: 15 Nov 2026 implement pass (non-interactive, approved by Brighton 12 Nov 2026).

## T08 static `index.html` check: blocked by the SPA fallback (needs Brighton)

- Task: T08 (compose `/`). The code and its unit tests are done. The task's verify step is `npm run build`, then check that `web/build/index.html` contains the h1, the beat headings and the CTA with no JavaScript.
- Reason: `web/vite.config.js` sets `adapter({ fallback: 'index.html' })`. The build log prints `adapter_fallback_overwrites`: the SPA shell overwrites the prerendered `/`. `web/build/index.html` is the 1.5 KB shell with no page text.
- The Rust server (`crates/server/src/lib.rs`) also uses `index.html` as its single-page fallback, so the shell and the landing page compete for one file name.
- Unblock (choose one; both touch shared behaviour, so not decided here):
  1. Rename the adapter fallback to `200.html` in `web/vite.config.js` and point the server fallback at `200.html` (a Rust change in `crates/server`, needs approval). The landing page then becomes `index.html`.
  2. Remove the adapter `fallback` option (web-only). Unknown paths then get the landing page from the server, and dynamic routes must all prerender. Check the build fails or not first.
- Downstream: T12 (static content check) and T14 (smoke marker through Caddy) depend on this. T09 to T11 do not.

## T02 font wiring: deviation from the plan (noted, not blocking)

- The fonts were self-hosted by `ide-workspace` T05 (`web/src/app.css`), not by this spec. Geist is the full variable woff2 (`Geist-Variable.woff2`), not a Latin subset. JetBrains Mono uses the fontsource subsets for weights 400 to 700.
- The build CSS references local font files only (checked). OFL notices for both packages are already in `THIRD-PARTY-NOTICES.md`.
- Not done: the `<link rel="preload">` for Geist 600 in `app.html`. Vite hashes the font file name in the build, so a static preload would point to a missing file. Lighthouse (T13) decides whether this costs LCP.
- Decision: keep the existing wiring. No new copies in `web/static/fonts/`.

## T01 render check: deferred

- Browser rendering of the five SVGs is an E2E check and is deferred (unit and integration scope only). Parsing, script/foreignObject/href checks and the size budget passed.

## Design conflict: smoke layer in static mode (noted)

- `design.md` sets `layers.smoke: false` in static mode, but `tasks.md` T04 says static mode "shows every layer". The implementation follows `design.md` (the smoke wisps are a motion effect, so the static page hides them). The planner tests assert the design rule.

## T12 no-JavaScript content check: blocked (same cause as T08)

- It checks the built `index.html` with JavaScript disabled. That file is the SPA shell today, so the check cannot pass until the T08 fallback issue is decided.
- The content itself is covered by component and route tests (`Hero.test.ts`, `GhostBeats.test.ts`, `SignUpCta.test.ts`, `page.landing.test.js`), which passed in this run.

## T13 Lighthouse mobile median: blocked

- It needs the production build served by `counterspell-server` on localhost, and a headless Chrome for Lighthouse. Both are browser or E2E steps, so they are deferred.
- It also depends on the static landing HTML (T08). Without it, Lighthouse would measure the empty shell, which gives a false score.
- Unblock: decide the T08 fallback, then run `npx lighthouse http://localhost:8080/ --preset=mobile --only-categories=performance` three times on a machine with Chrome, and record the results in `docs/kiro-log/landing-steel-night-lighthouse.md`.

## T14 accessibility, privacy and Compose smoke: blocked / deferred E2E

- Keyboard path, visible focus and contrast checks need a browser. Deferred E2E.
- The Compose smoke test (`http://localhost/` through Caddy) needs the Compose stack, which is out of scope for this web-only run. Also depends on T08.
- What is covered now: the CTA focus-visible classes (`SignUpCta.test.ts`), the `lang="en"` and single `h1` (`page.landing.test.js`), and no third-party URL in the built CSS (checked in T02). Still to check by hand: the network log and the contrast pairs against `docs/palette.md`.

## Process note

- T09 tests were written before `scenes.ts` existed, but I did not capture a red run before writing the module. The 7 tests were green on the first full run.
- T10 red was captured: the page did not call `startScenes` before the change.

## Wave review findings (self-review against the requesting-code-review checklist)

- Minor: Geist is one variable woff2 of 69.5 KB (`Geist-Variable.woff2`). It covers weights 400 and 600 and is above the design's 60 KB per-weight guideline. Lighthouse (T13) decides whether it matters. A static Latin subset could cut it.
- Minor: `motion-plan` keeps `layers.smoke` false in static mode (see the design conflict above).
- Minor: the landing `.cta` transition rule duplicates the global reduced-motion rule in `app.css`. Harmless; kept because the spec names the CTA rule.
- Minor: the hero scene wrapper and each scene `img` both carry `aria-hidden="true"`. Redundant but correct.
- Fixed in this wave: an unhandled rejection when the GSAP chunk fails to load (page now keeps the static content; test added).
