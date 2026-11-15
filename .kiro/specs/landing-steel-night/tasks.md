# Tasks: landing-steel-night

All tasks are for `web/` only. No task changes a Rust crate, a server route or a file outside `web/`, `THIRD-PARTY-NOTICES.md` and `docs/kiro-log/`. No task in this document is executed by this spec-generation run. Each task is sized for 30 minutes or less. `[P]` means it can run in parallel with other tasks in the same wave. `[S]` means it is sequential or has a required dependency.

Conventions for every task:

- Follow `.kiro/steering/frontend.md`: Svelte 5 runes, semantic tokens only (no raw hex in components), GSAP only in the landing scenes, plain text, no colour-only status.
- Run `cd web && npm run check` after each task.
- Do not use `unwrap()` or `expect()`-style shortcuts in any Rust file. (No Rust is expected here. If a task unexpectedly touches Rust, stop and ask.)
- Before creating a file under `web/src/lib/`, check whether `ide-workspace` has already created it. If so, extend it.

## Wave 0: assets and fonts

- [x] **T01 [P] (25 min) Author the five SVG scene assets**
  - Files: `web/static/landing/moon.svg`, `graveyard-far.svg`, `graveyard-near.svg`, `smoke.svg`, `bats.svg`.
  - Hand-author flat vector silhouettes in Steel Night colours (bone and steel for the moon, moss-muted and bg tones for the graveyard). Each file has its own `viewBox`, no `<script>`, no `foreignObject`, no external `href`, no embedded raster, and total size of the five files at most 100 KB.
  - Depends on: none.
  - Verify: `grep -L "<script\|foreignObject\|href=\"http" web/static/landing/*.svg` lists all five; `du -cb web/static/landing/*.svg` total is 100 KB or less; open each file in a browser to check it renders.
  - Requirements: R5, R20 (no serif), R29. Preserve: original work only, no copied art.

- [x] **T02 [P] (25 min) Self-host Geist and JetBrains Mono**
  - Files: `web/static/fonts/` (Latin woff2 files), `web/src/fonts.css`, `THIRD-PARTY-NOTICES.md` (append OFL notices), `web/package.json` (only if the packages are used as build inputs).
  - Copy the Latin subset woff2 for Geist 400 and 600 (from npm `geist` 1.9.2) and JetBrains Mono 400 (from `@fontsource/jetbrains-mono` 5.3.0). Declare `@font-face` with `font-display: swap`. Preload Geist 600 in `app.html`.
  - If `ide-workspace` already created `fonts.css`, extend it instead.
  - Depends on: none.
  - Verify: `npm run build` succeeds; the built CSS references local font files only; `THIRD-PARTY-NOTICES.md` names both OFL packages.
  - Requirements: R20, R29. Preserve: no CDN font request, no serif face.

## Wave 1: pure logic and copy

- [x] **T03 [P] (25 min) Write the landing copy module**
  - Files: `web/src/lib/landing/copy.ts`.
  - Export the hero headline, supporting line, the three beats (Submit, Hunt, Understand) with their bodies, the illustrative example values (`nums = [0, 0], target = 0`, expected `[0, 1]`, got `[0, 0]`) and the captions "Illustration, not a real run.", and the CTA labels ("Start practising", "Log in").
  - Depends on: none.
  - Verify: a unit test asserts that no string in `copy.ts` matches `/smallest possible|proven minimum|optimal/i` (R9).
  - Requirements: R8, R9, R10. Preserve: the Hunt beat says "smallest failing input it can find within its time budget".

- [x] **T04 [S] (30 min) Implement the pure motion planner with property tests**
  - Files: `web/src/lib/landing/motion-plan.ts`, `web/src/lib/landing/motion-plan.test.ts`.
  - Implement `planLanding(facts)` exactly as the design's rules 1 to 3. Use no DOM, no GSAP and no `window`.
  - Tests (vitest, with `fast-check` only if already in `web/` from another spec, otherwise table-driven over widths 320, 639, 640, 1280 and every combination of the booleans): reduced motion always gives static mode and `loadGsap: false`; static mode shows every layer and sets `bats` and `reveals` false; width under 640 with motion allowed gives `far: false` and `bats: false`; the headline and beats are never in the layer set.
  - Depends on: none.
  - Verify: `cd web && npm test -- landing` passes.
  - Requirements: R15, R16, R26. Preserve: pure module, no GSAP import.

## Wave 2: components

- [x] **T05 [S] (30 min) Build the hero**
  - Files: `web/src/lib/landing/Hero.svelte`.
  - Eyebrow (JetBrains Mono, candle, uppercase), `h1` from `copy.ts`, supporting line in `text-muted`, and the scene container with five `<img alt="" aria-hidden="true">` layers using explicit `width` and `height`. Responsive `h1` sizes (2.5rem on phones, 3.5rem on desktop).
  - Depends on: T01, T02, T03.
  - Verify: `npm run check`; the rendered HTML contains the `h1` text and five `img` tags with `alt=""`.
  - Requirements: R1, R3, R5, R6, R7, R25. Preserve: no raster image, semantic tokens only.

- [x] **T06 [S] (30 min) Build the three ghost beats and the labelled example**
  - Files: `web/src/lib/landing/GhostBeats.svelte`, `web/src/lib/landing/GhostExample.svelte`.
  - Three `<section>` elements with `h2` headings Submit, Hunt, Understand in order, each with an inline SVG icon (`aria-hidden`). The example shows input, "expected" value in the success token and "got" value in the danger token, each with a text label, and the caption "Illustration, not a real run."
  - Depends on: T03.
  - Verify: component test checks the heading order and that the example has text labels and the caption.
  - Requirements: R8, R10, R22, R27, R28. Preserve: no colour-only status, no computed or server-supplied values.

- [x] **T07 [S] (25 min) Build the sign-up call to action**
  - Files: `web/src/lib/landing/SignUpCta.svelte`.
  - Primary `<a href="/login">` styled with `bg-accent-candle`, dark ink text, hover `accent-candle-glow`, focus ring `focus` 2px with 2px offset. Secondary text link "Log in" to `/login` in `accent-moss`.
  - Depends on: T03.
  - Verify: component test checks the `href` values and the presence of the focus-visible class; manual keyboard check.
  - Requirements: R17, R18, R19. Preserve: no payment text, no paid tier.

- [x] **T08 [S] (30 min) Compose the landing route and keep the smoke marker**
  - Files: `web/src/routes/+page.svelte`.
  - Replace the hello page body with `<main data-smoke-marker="counterspell-svelte-static" lang="en">` containing `Hero`, `GhostBeats`, `SignUpCta`. Keep `<svelte:head>` with the title and description. Call `planLanding` in `onMount`; in static mode do nothing more.
  - Depends on: T04, T05, T06, T07.
  - Verify: `npm run build`, then check that `web/build/index.html` contains the `h1`, the three beat headings, the CTA link and `data-smoke-marker="counterspell-svelte-static"`, all without running JavaScript.
  - Requirements: R1, R2, R3, R4, R27. Preserve: static prerender, no dev server.

## Wave 3: motion

- [x] **T09 [S] (30 min) Implement the GSAP scene module**
  - Files: `web/src/lib/landing/scenes.ts`.
  - Dynamic import of `gsap` and `gsap/ScrollTrigger` after first paint (idle callback with a 1.5 s timeout). Implement the reveals (`immediateRender: false`, `once: true`), the moon fade and scale, the parallax on `far` and `near` (`yPercent`), the smoke rise (opacity and transform), and the one-shot bat flight. Export `startScenes(plan)` and `dispose()`. Transform and opacity only.
  - Depends on: T04, T08.
  - Verify: `npm run build`; in the browser, GSAP is requested only when reduced motion is off; with reduced motion on, the network log has no `gsap` request.
  - Requirements: R11, R12, R13, R14, R15. Preserve: GSAP stays on the landing page only; no infinite loops.

- [x] **T10 [S] (25 min) Wire the planner into the page and dispose on leave**
  - Files: `web/src/routes/+page.svelte`, `web/src/lib/landing/scenes.ts` (only the dispose hook).
  - In `onMount`, call `planLanding({ reducedMotion, viewportWidth, scriptsRunning: true })`. If `loadGsap`, call `startScenes(plan)`. Return the `dispose` function from `onMount` so tweens and triggers are killed on route leave.
  - Depends on: T08, T09.
  - Verify: navigate to `/` and away, and check in devtools that no ScrollTrigger instances remain.
  - Requirements: R14, R16. Preserve: no ambient animation after leave.

- [x] **T11 [P] (20 min) Add reduced-motion and phone CSS fallbacks**
  - Files: `web/src/app.css` (landing utility classes only), the landing components' class lists.
  - `@media (prefers-reduced-motion: reduce)` sets `transition-duration: 0ms` on the CTA and any landing transition. Phone (under 640 px) hides the far layer and bats through the planner, not CSS, so the static layout is correct without JavaScript.
  - Depends on: T05, T07.
  - Verify: with reduced motion on in devtools, the CTA hover has no transition and all layers show their final frames.
  - Requirements: R15, R26. Preserve: no new colour values; tokens only.

## Wave 3b: real art (amendment 15 Nov 2026)

These follow the design amendment "real art". T13 and T14 run after T18, on the page with the real art.

- [ ] **T15 [S] (30 min) Hero background: moon/graveyard poster and video loop**
  - Files: `web/src/lib/landing/Hero.svelte`, a small pure helper (for example `hero-media.ts`) and its test, landing CSS; assets in `web/static/art/landing/`.
  - Poster as a `<picture>` (960 and 1920 WebP, JPEG fallback) in the static HTML with width, height and `fetchpriority="high"`. In motion mode only, after first paint, mount the `<video>` (WebM then MP4, 960 at <= 900 px wide, else 1920, `preload="none"`, muted, loop, playsinline). No video for reduced motion, Save-Data or a hidden tab; pause when off-screen.
  - Verify: vitest for the pure source choice (reduced motion gives no video, narrow gives 960, Save-Data gives none); `cd web && npm run check && npm run build`; no third-party request.
  - Requirements: R5, R23, R24, R25. Preserve: headline, beats and CTA stay in the static HTML; no server or Rust change.
- [ ] **T16 [S] (30 min) Recompose the motion layers over the real art**
  - Files: `web/src/lib/landing/scenes.ts`, `motion-plan.ts`, their tests, landing CSS.
  - The flat moon and graveyard SVGs become the fallback only (used if the poster fails). Use `art/fx/` overlays (fog, embers, bat swarm) for subtle parallax driven by the existing planner. Add a token-based scrim so all text pairs keep their contrast.
  - Verify: planner property tests and `scenes.test.ts` green; reduced motion shows the poster still with no GSAP request.
  - Requirements: R5, R21, R24. Preserve: GSAP only on the landing page, loaded after first paint.
- [ ] **T17 [P] (25 min) Art accents for the three beats and the CTA**
  - Files: `GhostBeats.svelte`, `SignUpCta.svelte`, their tests, CSS.
  - Add tasteful decorative art from `art/objects/` and `art/creatures/` (WebP, `loading="lazy"`, `decoding="async"`, explicit size, `alt=""`) following Impeccable layout and polish. No layout shift, no colour-only meaning.
  - Verify: tests green; `npm run check && npm run build`.
  - Requirements: R21, R22, R29.
- [ ] **T18 [S] (20 min) Impeccable audit and first-load art check**
  - Files: fixes in `web/` only, `docs/kiro-log/landing-steel-night-art.md` (new).
  - Run the Impeccable audit and critique playbooks on the landing page and fix Critical and Important findings. Record the first-load bytes (poster, fonts, JS) and confirm the video is not requested before first paint and never under reduced motion.
  - Verify: the log file lists the findings, the fixes and the byte counts.
  - Requirements: R21, R23.

- [ ] **T19 [optional] [P] (20 min) Orange jack-o'-lantern accent near the CTA**
  - Files: `SignUpCta.svelte` or the CTA section, tests, CSS.
  - Place `art/familiar/jack-o-lantern-orange` as a decorative accent near the sign-up CTA (WebP still preferred, `loading="lazy"`, `decoding="async"`, explicit size, `alt=""`). The animated SVG may be used only in motion mode; reduced motion and Save-Data show the still. Do not delete or relocate the file. Keep the Steel Night palette and do not regress the Lighthouse mobile median target of 90.
  - Verify: `npm run check && npm run build`; no layout shift; CTA text and link remain in the static HTML.
  - Requirements: R21, R22, R23, R29.

- [ ] **T20 [optional] [P] (25 min) 404 and error page with the orange pumpkin**
  - Files: `web/src/routes/+error.svelte` (or equivalent), tests, CSS.
  - A branded not-found / error page using `art/familiar/jack-o-lantern-orange` (static WebP, explicit size, `alt=""`), a plain-language message, and a link home. Impeccable layout; tokens only; reduced motion uses the still; no ambient loop. Do not invent a mood or a verdict on this page.
  - Verify: `npm run check && npm run build`; visiting an unknown path shows the page and the home link works.
  - Requirements: R21, R22.

## Wave 4: verification

- [x] **T12 [S] (30 min) Run the no-JavaScript and content checks**
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
- **Wave 3b (amendment 15 Nov 2026):** T15, then T16; T17 can run in parallel; T18 last. Optional T19 and T20 may follow and must not block T13 or T14.
- **Wave 4:** T12, then T18, then T13, then T14, in order.

## Completion checklist

- [ ] `/` is prerendered from `web/` and served by `counterspell-server`; the smoke marker is present.
- [ ] Headline, three beats and CTA are in the static HTML.
- [ ] GSAP is loaded only in motion mode, after first paint, and only on the landing page.
- [ ] Reduced motion shows the static final frames, with no GSAP request.
- [ ] Scene SVGs are original, script-free and within the size budget.
- [ ] Geist and JetBrains Mono are self-hosted; their OFL notices are in `THIRD-PARTY-NOTICES.md`.
- [ ] Lighthouse mobile performance median of three runs is at least 90, recorded in `docs/kiro-log/landing-steel-night-lighthouse.md`.
- [ ] The Hunt copy never claims a smallest possible or proven minimum input.
- [ ] No Rust, server or route change was made by this spec (except the approved 200.html fallback).
- [ ] The hero uses the approved moon/graveyard art: poster first, video only in motion mode after first paint, never under reduced motion or Save-Data.
