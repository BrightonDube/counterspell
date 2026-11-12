# Tasks: landing-steel-night

All tasks are for `web/` only. No task changes a Rust crate, a server route or a file outside `web/`, `THIRD-PARTY-NOTICES.md` and `docs/kiro-log/`. No task in this document is executed by this spec-generation run. Each task is sized for 30 minutes or less. `[P]` means it can run in parallel with other tasks in the same wave. `[S]` means it is sequential or has a required dependency.

Conventions for every task:

- Follow `.kiro/steering/frontend.md`: Svelte 5 runes, semantic tokens only (no raw hex in components), GSAP only in the landing scenes, plain text, no colour-only status.
- Run `cd web && npm run check` after each task.
- Do not use `unwrap()` or `expect()`-style shortcuts in any Rust file. (No Rust is expected here. If a task unexpectedly touches Rust, stop and ask.)
- Before creating a file under `web/src/lib/`, check whether `ide-workspace` has already created it. If so, extend it.

## Wave 0: assets and fonts

- [ ] **T01 [P] (25 min) Author the five SVG scene assets**
  - Files: `web/static/landing/moon.svg`, `graveyard-far.svg`, `graveyard-near.svg`, `smoke.svg`, `bats.svg`.
  - Hand-author flat vector silhouettes in Steel Night colours (bone and steel for the moon, moss-muted and bg tones for the graveyard). Each file has its own `viewBox`, no `<script>`, no `foreignObject`, no external `href`, no embedded raster, and total size of the five files at most 100 KB.
  - Depends on: none.
  - Verify: `grep -L "<script\|foreignObject\|href=\"http" web/static/landing/*.svg` lists all five; `du -cb web/static/landing/*.svg` total is 100 KB or less; open each file in a browser to check it renders.
  - Requirements: R5, R20 (no serif), R29. Preserve: original work only, no copied art.

- [ ] **T02 [P] (25 min) Self-host Geist and JetBrains Mono**
  - Files: `web/static/fonts/` (Latin woff2 files), `web/src/fonts.css`, `THIRD-PARTY-NOTICES.md` (append OFL notices), `web/package.json` (only if the packages are used as build inputs).
  - Copy the Latin subset woff2 for Geist 400 and 600 (from npm `geist` 1.9.2) and JetBrains Mono 400 (from `@fontsource/jetbrains-mono` 5.3.0). Declare `@font-face` with `font-display: swap`. Preload Geist 600 in `app.html`.
  - If `ide-workspace` already created `fonts.css`, extend it instead.
  - Depends on: none.
  - Verify: `npm run build` succeeds; the built CSS references local font files only; `THIRD-PARTY-NOTICES.md` names both OFL packages.
  - Requirements: R20, R29. Preserve: no CDN font request, no serif face.

## Wave 1: pure logic and copy

- [ ] **T03 [P] (25 min) Write the landing copy module**
  - Files: `web/src/lib/landing/copy.ts`.
  - Export the hero headline, supporting line, the three beats (Submit, Hunt, Understand) with their bodies, the illustrative example values (`nums = [0, 0], target = 0`, expected `[0, 1]`, got `[0, 0]`) and the captions "Illustration, not a real run.", and the CTA labels ("Start practising", "Log in").
  - Depends on: none.
  - Verify: a unit test asserts that no string in `copy.ts` matches `/smallest possible|proven minimum|optimal/i` (R9).
  - Requirements: R8, R9, R10. Preserve: the Hunt beat says "smallest failing input it can find within its time budget".

- [ ] **T04 [S] (30 min) Implement the pure motion planner with property tests**
  - Files: `web/src/lib/landing/motion-plan.ts`, `web/src/lib/landing/motion-plan.test.ts`.
  - Implement `planLanding(facts)` exactly as the design's rules 1 to 3. Use no DOM, no GSAP and no `window`.
  - Tests (vitest, with `fast-check` only if already in `web/` from another spec, otherwise table-driven over widths 320, 639, 640, 1280 and every combination of the booleans): reduced motion always gives static mode and `loadGsap: false`; static mode shows every layer and sets `bats` and `reveals` false; width under 640 with motion allowed gives `far: false` and `bats: false`; the headline and beats are never in the layer set.
  - Depends on: none.
  - Verify: `cd web && npm test -- landing` passes.
  - Requirements: R15, R16, R26. Preserve: pure module, no GSAP import.

## Wave 2: components

- [ ] **T05 [S] (30 min) Build the hero**
  - Files: `web/src/lib/landing/Hero.svelte`.
  - Eyebrow (JetBrains Mono, candle, uppercase), `h1` from `copy.ts`, supporting line in `text-muted`, and the scene container with five `<img alt="" aria-hidden="true">` layers using explicit `width` and `height`. Responsive `h1` sizes (2.5rem on phones, 3.5rem on desktop).
  - Depends on: T01, T02, T03.
  - Verify: `npm run check`; the rendered HTML contains the `h1` text and five `img` tags with `alt=""`.
  - Requirements: R1, R3, R5, R6, R7, R25. Preserve: no raster image, semantic tokens only.

- [ ] **T06 [S] (30 min) Build the three ghost beats and the labelled example**
  - Files: `web/src/lib/landing/GhostBeats.svelte`, `web/src/lib/landing/GhostExample.svelte`.
  - Three `<section>` elements with `h2` headings Submit, Hunt, Understand in order, each with an inline SVG icon (`aria-hidden`). The example shows input, "expected" value in the success token and "got" value in the danger token, each with a text label, and the caption "Illustration, not a real run."
  - Depends on: T03.
  - Verify: component test checks the heading order and that the example has text labels and the caption.
  - Requirements: R8, R10, R22, R27, R28. Preserve: no colour-only status, no computed or server-supplied values.

- [ ] **T07 [S] (25 min) Build the sign-up call to action**
  - Files: `web/src/lib/landing/SignUpCta.svelte`.
  - Primary `<a href="/login">` styled with `bg-accent-candle`, dark ink text, hover `accent-candle-glow`, focus ring `focus` 2px with 2px offset. Secondary text link "Log in" to `/login` in `accent-moss`.
  - Depends on: T03.
  - Verify: component test checks the `href` values and the presence of the focus-visible class; manual keyboard check.
  - Requirements: R17, R18, R19. Preserve: no payment text, no paid tier.

- [ ] **T08 [S] (30 min) Compose the landing route and keep the smoke marker**
  - Files: `web/src/routes/+page.svelte`.
  - Replace the hello page body with `<main data-smoke-marker="counterspell-svelte-static" lang="en">` containing `Hero`, `GhostBeats`, `SignUpCta`. Keep `<svelte:head>` with the title and description. Call `planLanding` in `onMount`; in static mode do nothing more.
  - Depends on: T04, T05, T06, T07.
  - Verify: `npm run build`, then check that `web/build/index.html` contains the `h1`, the three beat headings, the CTA link and `data-smoke-marker="counterspell-svelte-static"`, all without running JavaScript.
  - Requirements: R1, R2, R3, R4, R27. Preserve: static prerender, no dev server.

## Wave 3: motion

- [ ] **T09 [S] (30 min) Implement the GSAP scene module**
  - Files: `web/src/lib/landing/scenes.ts`.
  - Dynamic import of `gsap` and `gsap/ScrollTrigger` after first paint (idle callback with a 1.5 s timeout). Implement the reveals (`immediateRender: false`, `once: true`), the moon fade and scale, the parallax on `far` and `near` (`yPercent`), the smoke rise (opacity and transform), and the one-shot bat flight. Export `startScenes(plan)` and `dispose()`. Transform and opacity only.
  - Depends on: T04, T08.
  - Verify: `npm run build`; in the browser, GSAP is requested only when reduced motion is off; with reduced motion on, the network log has no `gsap` request.
  - Requirements: R11, R12, R13, R14, R15. Preserve: GSAP stays on the landing page only; no infinite loops.

- [ ] **T10 [S] (25 min) Wire the planner into the page and dispose on leave**
  - Files: `web/src/routes/+page.svelte`, `web/src/lib/landing/scenes.ts` (only the dispose hook).
  - In `onMount`, call `planLanding({ reducedMotion, viewportWidth, scriptsRunning: true })`. If `loadGsap`, call `startScenes(plan)`. Return the `dispose` function from `onMount` so tweens and triggers are killed on route leave.
  - Depends on: T08, T09.
  - Verify: navigate to `/` and away, and check in devtools that no ScrollTrigger instances remain.
  - Requirements: R14, R16. Preserve: no ambient animation after leave.

- [ ] **T11 [P] (20 min) Add reduced-motion and phone CSS fallbacks**
  - Files: `web/src/app.css` (landing utility classes only), the landing components' class lists.
  - `@media (prefers-reduced-motion: reduce)` sets `transition-duration: 0ms` on the CTA and any landing transition. Phone (under 640 px) hides the far layer and bats through the planner, not CSS, so the static layout is correct without JavaScript.
  - Depends on: T05, T07.
  - Verify: with reduced motion on in devtools, the CTA hover has no transition and all layers show their final frames.
  - Requirements: R15, R26. Preserve: no new colour values; tokens only.

## Wave 4: verification

- [ ] **T12 [S] (30 min) Run the no-JavaScript and content checks**
  - Files: `web/src/lib/landing/landing.test.ts` (component tests from T05 to T08 if not already written).
  - Confirm the static HTML contains all visible text and the CTA with JavaScript disabled, using the built `index.html`. Confirm no content is inside a GSAP-only hidden state.
  - Depends on: T08, T10.
  - Verify: `cd web && npm run check && npm test -- landing && npm run build`.
  - Requirements: R3, R4, R16, R27. Preserve: no test fixtures presented as real results.

- [ ] **T13 [S] (30 min) Measure Lighthouse mobile and record the result**
  - Files: `docs/kiro-log/landing-steel-night-lighthouse.md` (new), and fixes in `web/` only if the score is below 90.
  - Serve the production build through `counterspell-server` on localhost. Run `npx lighthouse http://localhost:8080/ --preset=mobile --only-categories=performance` three times and record each score, the median, LCP, CLS, TBT and the JavaScript transferred before GSAP.
  - If the median is below 90, cut in this order: the far layer, the bats, the smoke, then the parallax. Do not cut the headline, beats or CTA.
  - Depends on: T10, T11.
  - Verify: median performance is at least 90 and the log records the numbers, the Lighthouse version and the run date.
  - Requirements: R23, R24, R25. Preserve: the page is judged as finished only with the measured number.

- [ ] **T14 [S] (25 min) Check accessibility, privacy and the full verification list**
  - Files: fixes in `web/` only.
  - Keyboard path (headline, CTA, beats, log-in link) with visible candle focus. Contrast of every text pair against `docs/palette.md`. No third-party request in the network log. No inline script added to `app.html`. Run the Compose smoke test: `http://localhost/` through Caddy shows the marker.
  - Depends on: T12, T13.
  - Verify: each item in the design's Verification list is checked and recorded in the Lighthouse log file.
  - Requirements: R21, R22, R27, R28, R29, and the smoke marker (R2). Preserve: no change to the server, no new route.

## Dependency summary and parallel waves

- **Wave 0:** T01 and T02 can run in parallel with each other.
- **Wave 1:** T03 and T04 can run in parallel.
- **Wave 2:** T05, T06 and T07 can run in parallel after Wave 0 and T03. T08 follows T04, T05, T06 and T07.
- **Wave 3:** T09 follows T08. T10 follows T09. T11 can run in parallel with T09 and T10.
- **Wave 4:** T12, then T13, then T14, in order.

## Completion checklist

- [ ] `/` is prerendered from `web/` and served by `counterspell-server`; the smoke marker is present.
- [ ] Headline, three beats and CTA are in the static HTML.
- [ ] GSAP is loaded only in motion mode, after first paint, and only on the landing page.
- [ ] Reduced motion shows the static final frames, with no GSAP request.
- [ ] Scene SVGs are original, script-free and within the size budget.
- [ ] Geist and JetBrains Mono are self-hosted; their OFL notices are in `THIRD-PARTY-NOTICES.md`.
- [ ] Lighthouse mobile performance median of three runs is at least 90, recorded in `docs/kiro-log/landing-steel-night-lighthouse.md`.
- [ ] The Hunt copy never claims a smallest possible or proven minimum input.
- [ ] No Rust, server or route change was made by this spec.
