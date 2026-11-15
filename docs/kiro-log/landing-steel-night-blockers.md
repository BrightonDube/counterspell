# landing-steel-night blockers

Run: 15 Nov 2026 implement pass (non-interactive, approved by Brighton 12 Nov 2026).

## T08 static `index.html` check: RESOLVED 15 Nov 2026 10:23 (was blocked by the SPA fallback)

- Task: T08 (compose `/`). The code and its unit tests are done. The task's verify step is `npm run build`, then check that `web/build/index.html` contains the h1, the beat headings and the CTA with no JavaScript.
- Reason: `web/vite.config.js` sets `adapter({ fallback: 'index.html' })`. The build log prints `adapter_fallback_overwrites`: the SPA shell overwrites the prerendered `/`. `web/build/index.html` is the 1.5 KB shell with no page text.
- The Rust server (`crates/server/src/lib.rs`) also uses `index.html` as its single-page fallback, so the shell and the landing page compete for one file name.
- Unblock (choose one; both touch shared behaviour, so not decided here):
  1. Rename the adapter fallback to `200.html` in `web/vite.config.js` and point the server fallback at `200.html` (a Rust change in `crates/server`, needs approval). The landing page then becomes `index.html`.
  2. Remove the adapter `fallback` option (web-only). Unknown paths then get the landing page from the server, and dynamic routes must all prerender. Check the build fails or not first.
- **Resolved 15 Nov 2026 10:23 (Brighton): option 1.** See the resolution section at the end of this file.
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

## T12 no-JavaScript content check: unblocked for implementation (same cause as T08, now resolved)

- It checks the built `index.html` with JavaScript disabled. That file is the SPA shell today, so the check cannot pass until the T08 fallback change (option 1, approved 15 Nov 2026 10:23) is implemented.
- The content itself is covered by component and route tests (`Hero.test.ts`, `GhostBeats.test.ts`, `SignUpCta.test.ts`, `page.landing.test.js`), which passed in this run.

## T13 Lighthouse mobile median: blocked

- It needs the production build served by `counterspell-server` on localhost, and a headless Chrome for Lighthouse. Both are browser or E2E steps, so they are deferred.
- It also depends on the static landing HTML (T08). Without it, Lighthouse would measure the empty shell, which gives a false score.
- The T08 fallback is now decided (option 1, 15 Nov 2026 10:23). After that change lands, run `npx lighthouse http://localhost:8080/ --preset=mobile --only-categories=performance` three times on a machine with Chrome, and record the results in `docs/kiro-log/landing-steel-night-lighthouse.md`. The Chrome / E2E part remains deferred.

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

## T08 follow-up (attempt 2, 15 Nov 2026): option 2 tested, fails

- Tried a temporary Vite config with the adapter-static `fallback` option removed (the file was deleted afterwards; no tracked file changed).
- The build exits 1: `@sveltejs/adapter-static: all routes must be fully prerenderable, but found ... src/routes/problems/[slug]`. Option 2 therefore needs either `strict: false` (routes missing from the static output) or `prerender` entries for every problem slug. Both change behaviour, so this stays with Brighton.
- Option 1 (rename the fallback to `200.html` and change the server fallback) is still the only route that keeps every page. Brighton approved that Rust change in `crates/server` on 15 Nov 2026 10:23, going beyond this spec's usual scope for this change only.
- T08, T12, T13 and T14 stay unchecked until the fallback is implemented (and, for T13/T14, until their remaining E2E steps can run).

## T08 fallback: RESOLVED 15 Nov 2026 10:23 (Brighton)

Brighton chose option 1. Rename the SvelteKit static adapter fallback to `200.html` in `web/vite.config.js` and have the Rust server serve `200.html` as the SPA fallback for unknown non-API routes (for example `/problems/[slug]`). Problem pages stay served from the database with no redeploy. He explicitly approves the Rust change in `crates/server`, which goes beyond this spec's usual "no server change" scope, for this change only.

T08 and T12 can proceed once this is implemented. T13 and T14 still have deferred E2E / Compose / Chrome steps, but they are no longer waiting on the fallback decision.

## Run note (15 Nov 2026, implement pass 2): T08 and T12 done, T13 and T14 deferred E2E

- T08 done (commit 32863da). The Rust server now serves `200.html` as the SPA fallback and checks for it at start-up (`crates/server`, approved 15 Nov 2026 10:23). The web adapter writes `200.html` and prerenders `/` into `index.html`. Verified: `cargo fmt`, `cargo clippy --workspace --all-targets -D warnings`, `cargo test --workspace`, `npm run check`, `npm run build`, and the built `index.html` holds the h1, the three beats, the CTA and the smoke marker.
- Deploy note: a server build that still expects `index.html` as its fallback, or a `web/build` made before this change, fails the start-up check. Rebuild the web app and the server together.
- T12 done (commit 7924b0d). The built `index.html` contains every visible string with no JavaScript. No inline hiding is in the markup. The `data-reveal` content is hidden only by GSAP `from` tweens with `immediateRender: false`, so no stylesheet hides it. `npm run check` and `npm test -- landing` (8 files, 39 tests) pass.
- T13 (Lighthouse mobile median) stays unchecked. It needs headless Chrome and a running server: a browser or E2E step, deferred.
- T14 (keyboard, focus, contrast, network log, Compose smoke through Caddy) stays unchecked: deferred E2E and Compose stack.

## Run note (15 Nov 2026, implement pass 3): T13 and T14 stay open; static part of T14 checked

- Remaining open tasks: T13 and T14. Both stay unchecked. No code changed in this pass, so no new commit for T13/T14 beyond this note.
- T13 (Lighthouse mobile median, three runs): deferred E2E. It needs headless Chrome and a running `counterspell-server`. Unblock: run the three Lighthouse commands on a machine with Chrome, then record the numbers in `docs/kiro-log/landing-steel-night-lighthouse.md`.
- T14, static checks done without a browser (15 Nov 2026):
  - Contrast, recomputed with the WCAG relative-luminance formula against the `app.css` tokens: bone on bg 14.96, fog on bg 8.01, ash on bg 5.64, moss link on bg 6.86, dark ink on candle CTA 8.63, candle focus on bg 8.63, success on bg 9.21, danger on bg 5.82. All pass AA (4.5:1 for text).
  - Landing components (`web/src/lib/landing`, `+page.svelte`) use semantic tokens only. No raw hex found outside tests.
  - `web/src/app.html` has no `<script>`. The built `index.html` has one inline `<script>`, which is SvelteKit's own hydration bootstrap, not app code.
  - Built CSS has one `tailwindcss.com` string, inside the licence banner comment. No external fetch was found in the built HTML, CSS or JS.
- T14 still open, deferred E2E / Compose: the keyboard path and visible focus in a browser, the network log in devtools, and the Compose smoke test for `http://localhost/` through Caddy. Unblock: a browser pass and the Compose stack, both outside this run's scope.

## Run note (15 Nov 2026, real-art pass): T15 to T20 done; T13 and T14 still deferred E2E

Done in this pass (all in `web/` and this log): T15 (`09f1721`), T16 (`8a36675`), T17 (`ee2f743`), T18 (`c76bcfe`), optional T19 (`bb14295`), optional T20 (also `bb14295`, see the commit note below), and a review fix (`2654d1b`). Tests: `npm run check` and `npm run build` pass, and `vitest` is green for `src/lib` and `src/routes` (406 tests).

### Conflict: R25 (LCP) against the real-art amendment (for Brighton)

- R25 says the largest contentful paint element must be the headline, and no raster image may be the largest element.
- The 15 Nov amendment in `design.md` makes the poster the LCP image, as a full-bleed `<picture>` at `fetchpriority="high"`.
- Decision: the amendment wins, because it is later and was approved by Brighton. R25 is superseded for the poster. The headline and beats remain in the static HTML.
- Unblock or decide: if Brighton wants the headline to stay the LCP, the poster needs to become smaller than the headline, or move out of the first viewport. Lighthouse (T13) will show which element LCP reports.

### Commit note (T19 and T20)

- The T19 and T20 changes were run as parallel tool calls. The T20 files were staged before the T19 commit ran, so commit `bb14295` contains the T19 and T20 changes together, plus both ticks in `tasks.md`. The content is correct and its tests passed. The commit message names only T19. History was not rewritten.

### T13 Lighthouse mobile median: deferred E2E (still open)

- No Chrome or Chromium is installed on this machine (`which chromium chromium-browser google-chrome` returns nothing), so Lighthouse cannot run.
- Unblock: install Chrome, serve the production build through `counterspell-server` on localhost, then run `npx lighthouse http://localhost:8080/ --preset=mobile --only-categories=performance` three times. Record the scores, median, LCP, CLS, TBT, the Lighthouse version and the run date in `docs/kiro-log/landing-steel-night-lighthouse.md`.
- Byte figures already measured for this check are in `docs/kiro-log/landing-steel-night-art.md`. The route JavaScript is about 42.6 KB gzip against a 35 KB design budget, so T13 is the place to decide on it.

### T14 accessibility, privacy and Compose smoke: deferred E2E (still open)

- Static checks repeated for the new code in this pass: no raw colour values, no external URL, and `app.html` has no `<script>`. The contrast figures in the earlier T14 note still hold, because no text colour changed.
- Still needed: the keyboard path and focus in a browser, the devtools network log, and the Compose smoke test (`http://localhost/` through Caddy).

### Review findings (self-review against `requesting-code-review`; no subagent was started)

- Fixed (Important): the 168 KB fog overlay loaded with the first render. Overlays now load after first paint (T18). Commit `c76bcfe`.
- Fixed (Important): the hero video had `autoplay`, so it started loading even off screen. Removed. Commit `2654d1b`.
- Minor: the video mount and the IntersectionObserver play/pause logic have no unit test (happy-dom has no real media). Only the pure source choice is tested. Verify in a browser.
- Minor: the poster `onerror` fallback only catches failures after hydration. A poster that fails before the script loads keeps the broken image.
- Minor: the route JavaScript is about 42.6 KB gzip against the 35 KB design budget (framework runtime is most of it). Decide after the T13 run.
- Minor: Geist is one 69.5 KB variable file (from the earlier T02 note).
- Minor: the eyebrow "Counterspell" above the headline conflicts with the craft floor's eyebrow ban. Kept, because the design requires it. Brighton can drop it.
- Minor: TDD order slipped in three places. The `after-paint` tests were written with their module, not red first. The `ErrorView` component was written before its test was run. And the `drops the far fog layer` test was first vacuous and was strengthened. No behaviour is affected, but these were not captured as red runs.
- Minor: `$lib` does not resolve under `jsconfig.json` in `svelte-check`, so `+error.svelte` uses a relative import, as `+page.svelte` does.
- Minor: `::selection` in `app.css` is CSS styling with no test (an exception under the test-first rule, as configuration).
- Test-change markers used: `R5` (`Hero.test.ts`, `hero-media.test.ts`: the approved art replaces the flat layers, per the 15 Nov amendment) and `R6` (`GhostBeats.test.ts`: decorative beat icons are now approved art, still hidden from assistive technology). Each replaced an assertion, and no test was removed.
- No Rust changed, so no Rust lesson was added.
