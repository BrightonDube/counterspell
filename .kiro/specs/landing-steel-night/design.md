# Low-level design: landing-steel-night

## Design approach

The landing page is a static, prerendered SvelteKit route inside the existing `web/` app. It replaces the skeleton hello page at `/` and keeps the same build, routing and smoke marker. The look comes from `docs/palette.md` tokens and original SVG silhouettes. GSAP ScrollTrigger adds the cinematic reveals, but only after the page has painted and only when the visitor has not asked for reduced motion.

The design favours three things, in this order: the page reads without JavaScript, the Lighthouse mobile score stays at 90 or above, and motion is a layer on top of content that is already visible.

No part of this spec touches the Rust crates or the judge. The page is served by `counterspell-server` through the existing static and SPA fallback routes (repo-skeleton R7 and R8).

## Scope and non-goals

**Included:** the `/` route and its components, four original SVG scene assets, the motion planner, the GSAP scene module (dynamically imported), the three-beat copy, the sign-up call to action, self-hosted Geist and JetBrains Mono, and performance and accessibility checks.

**Not included:** sign-up or log-in forms (`auth`), the IDE, the dashboard, Monaco, the Familiar, a light theme, and any real example run.

## File layout

```text
web/
├── static/landing/                    # original SVG scene assets (served as-is)
│   ├── moon.svg                       # moon disc with a soft halo
│   ├── graveyard-far.svg              # distant headstones and a dead tree line
│   ├── graveyard-near.svg             # foreground headstones and grass
│   ├── smoke.svg                      # rising wisps (opacity and transform only)
│   └── bats.svg                       # one flock silhouette
├── src/
│   ├── app.css                        # (existing) tokens; landing adds only layout utilities
│   ├── fonts.css                      # @font-face for Geist and JetBrains Mono (shared with ide-workspace)
│   ├── routes/
│   │   ├── +layout.js                 # (existing) prerender = true
│   │   ├── +layout.svelte             # (existing) imports app.css and fonts.css
│   │   └── +page.svelte               # landing composition; keeps smoke marker
│   └── lib/landing/
│       ├── copy.ts                    # all visible strings (one place to review)
│       ├── motion-plan.ts             # planLanding(...)           (PURE)
│       ├── motion-plan.test.ts        # property tests
│       ├── scenes.ts                  # GSAP ScrollTrigger timelines (dynamic import)
│       ├── Hero.svelte                # headline, moon, graveyard layers, CTA
│       ├── GhostBeats.svelte          # Submit / Hunt / Understand
│       ├── GhostExample.svelte        # labelled illustration of expected vs got
│       └── SignUpCta.svelte           # primary + secondary links
```

`web/static/landing/` and `web/static/familiar/` are separate folders with separate assets. The Familiar's art is owned by `efficiency-familiar`, and nothing here reads or writes it.

## Routes and links

| Element | Target | Note |
|---|---|---|
| Primary CTA "Start practising" | `/login` | The auth spec has no separate sign-up route. Sign-up happens on `/login` through GitHub, Google or email. |
| Secondary link "Log in" | `/login` | Same page. |

No route other than `/` is added. The page links to `/login` and does not depend on it existing at build time. If `/login` is missing, the static build still succeeds, and the SPA fallback serves the landing page for that path, so the check in the verification section catches it.

## Component design

### `+page.svelte`

Composes `Hero`, `GhostBeats` and `SignUpCta` in that order inside `<main data-smoke-marker="counterspell-svelte-static" lang="en">`. It holds no logic. Its `onMount` calls `planLanding(...)` and, if the plan asks for motion, dynamically imports `scenes.ts`.

### `Hero.svelte`

- Eyebrow `Counterspell` in JetBrains Mono, candle colour, uppercase, tracking 0.2em.
- `h1`: "Learn why your code breaks." Geist 600, tracking -0.01em, size 2.5rem on phones and 3.5rem on desktop (the hero-only scale step from `docs/palette.md`). This is the largest contentful paint element, and it is plain text.
- Supporting line (`text-muted`): "Your solution fails. Counterspell hunts down the input that breaks it, shrinks it to the smallest failing input it can find, and explains the mistake."
- Scene: a `<div class="scene">` with absolutely positioned SVG layers in this order (back to front): `moon.svg`, `graveyard-far.svg`, `smoke.svg`, `graveyard-near.svg`, `bats.svg`. Each layer is an `<img>` with `alt=""`, `aria-hidden="true"`, explicit `width` and `height`, and `decoding="async"`. The `<img>` form keeps the SVGs out of the DOM tree and out of the accessibility tree.

### `GhostBeats.svelte`

Three `<section>` elements, each with one `h2` in this order:

| Beat | Heading | Body (plain copy in `copy.ts`) |
|---|---|---|
| 1 | Submit | "Your code runs in a sealed sandbox on our server. A trusted judge holds the tests and the answers, so your program only prints what it computes." |
| 2 | Hunt | "When the code is wrong, the ghost engine searches for an input that breaks it, then shrinks that input. You get the smallest failing input it can find within its time budget." |
| 3 | Understand | "You see the failing input, what you expected, what you got and the mistake behind it, so you learn the concept and not just the fix." |

Each beat holds an icon (inline SVG, `aria-hidden`), so beats are not identified by colour. The Hunt beat includes `GhostExample.svelte`.

### `GhostExample.svelte`

A static illustration of the Bug Autopsy shape, always captioned "Illustration, not a real run.":

- Input: `nums = [0, 0], target = 0` in JetBrains Mono.
- Expected: `[0, 1]` shown with the text label "expected" and the success token colour.
- Got: `[0, 0]` shown with the text label "got" and the danger token colour.

Colour is never the only signal: each value has a text label, and the caption states that this is an illustration. The values are fixed strings in `copy.ts`. Nothing is computed, and nothing comes from the server.

### `SignUpCta.svelte`

One `<a class="cta">` in candle (`bg-accent-candle`, text `bg-bg` dark ink, which is 8.63 contrast). Hover uses `accent-candle-glow`. A focus ring uses `focus`, 2px, 2px offset. A second text link, "Log in", uses `accent-moss`.

## Motion design

### Motion plan (pure)

`motion-plan.ts` is a pure function with no DOM, GSAP or window access. The page passes in the facts and gets a plan.

```ts
export interface MotionFacts {
  reducedMotion: boolean;   // matchMedia('(prefers-reduced-motion: reduce)')
  viewportWidth: number;    // CSS px
  scriptsRunning: boolean;  // true once onMount has run
}

export interface LandingPlan {
  mode: 'motion' | 'static';
  loadGsap: boolean;        // true only in 'motion' mode
  layers: { moon: boolean; far: boolean; near: boolean; smoke: boolean };
  bats: boolean;
  reveals: boolean;         // beats and CTA reveal on scroll
}

export function planLanding(facts: MotionFacts): LandingPlan;
```

Rules:

1. `reducedMotion` or not `scriptsRunning` gives `mode: 'static'`, `loadGsap: false`, `bats: false`, `reveals: false`, `layers.far: true`, `layers.near: true`, `layers.moon: true`, `layers.smoke: false`. Every visible layer is shown in its final position.
2. `viewportWidth < 640` (and motion allowed) sets `layers.far: false` and `bats: false`, so the far graveyard and the bats are static images, and `far` stays hidden rather than parallaxed.
3. Otherwise `mode: 'motion'`, `loadGsap: true`, `bats: true`, `reveals: true`, and all layers are on.

The planner never returns a state where content is hidden without a way to show it. A static plan shows everything in its final position.

### Scene timelines (`scenes.ts`)

- Loaded with `await import('gsap')` and `import('gsap/ScrollTrigger')` after the first paint (on `requestIdleCallback`, with a 1.5 s timeout fallback to `setTimeout`).
- `gsap.registerPlugin(ScrollTrigger)` is called once.
- Reveals use `gsap.from(target, { opacity: 0, y: 16, duration: 0.8, ease: 'power2.out', immediateRender: false, scrollTrigger: { trigger, start: 'top 85%', once: true } })`. `immediateRender: false` means the hidden start state only applies when the trigger fires, so static HTML is never hidden.
- Moon: a 1.6 s `opacity` and `scale` (1.04 to 1) ease `sine.inOut` on load.
- Parallax: `far` moves `yPercent` 0 to -8 and `near` moves 0 to -16 across the hero, scrubbed to scroll. `transform` only.
- Smoke: opacity 0 to 0.35 and a 12 px rise, once, on load.
- Bats: one path flight across the hero, 2.4 s, `power2.out`, once. No loop.
- The module exposes `dispose()`, which kills all tweens and triggers. It is called on route leave.

Only transform and opacity animate. No layout-affecting property (width, top, margin) animates.

### Reduced motion

If `prefers-reduced-motion: reduce` is set, `planLanding` returns the static mode, `scenes.ts` is never imported, and the page shows the final frames. The CSS also sets `transition-duration: 0ms` for the CTA hover, so nothing moves. The text is identical in both modes (R16).

## Performance budget

The page must score at least 90 in Lighthouse mobile performance (requirement R23). The budget below is what the design allows. Each number is an estimate [E] until measured.

| Item | Budget (gzip or raw as noted) | Why |
|---|---|---|
| Landing route JavaScript, excluding GSAP | ≤ 35 KB gzip | Svelte runtime plus the page and planner |
| GSAP core and ScrollTrigger | Loaded after first paint, ≤ 30 KB gzip [E] | Not on the critical path |
| Scene SVGs, total | ≤ 100 KB raw | Each file is hand-simplified; no embedded rasters |
| Geist, Latin subset, weights 400 and 600 | ≤ 60 KB woff2 each | The headline uses 600 and is preloaded |
| JetBrains Mono, Latin subset, weight 400 | ≤ 25 KB woff2 | Only the eyebrow label uses it on this page |
| Third-party requests | 0 | R29 |

Other rules:

- Each `<img>` for a scene layer has explicit dimensions, so the layout does not shift (CLS).
- The LCP element is the `h1`, not an image. The `h1` uses `font-display: swap`, and the 600 weight is preloaded.
- The static build is compressed by Caddy (compression is part of the Caddy setup in `docs/system-design.md` §3 and `repo-skeleton`).
- `prefers-reduced-motion` and the phone rule (under 640 px) also reduce GPU work.

## Fonts

`web/src/fonts.css` declares `@font-face` for Geist and JetBrains Mono from files in `web/static/fonts/`. The files come from two OFL-1.1 packages:

- Geist: npm `geist` (1.9.2), Latin subset, weights 400 and 600.
- JetBrains Mono: npm `@fontsource/jetbrains-mono` (5.3.0), Latin subset, weight 400.

Both are copied into `web/static/fonts/` at build time or committed once, and their OFL notices are added to `THIRD-PARTY-NOTICES.md`. No font is requested from a CDN (R29). No serif face is declared.

`ide-workspace` (its T05) also loads these fonts. If that spec lands first, this spec reuses its `fonts.css` and does not create a second copy.

## Copy

Copy lives in `copy.ts` so it is easy to review. Two rules apply to all of it:

- "minimized counterexample", "smallest failing input found", never "smallest possible", "proven minimum" or "optimal" (R9).
- No claim about users, speed, or results that has not been measured.

## Accessibility

- One `h1`, then one `h2` per beat, in order (R27).
- `lang="en"` on `<main>` and the document (set in `app.html`).
- Decorative SVGs use `alt=""` and `aria-hidden`. The Hunt example is text, not an image.
- The CTA is a real link. Focus is visible (candle ring). Keyboard order is headline, CTA, beats, log-in link.
- Colour is never the only signal: the example has text labels, and the beats have icons and headings.
- Contrast is taken from `docs/palette.md`: bone on bg 14.96, fog 8.01, ash 5.64, candle on bg 8.63. Ash is used only for captions.

## Testing

- **Unit and property tests (vitest):**
  - `planLanding` returns `static` whenever reduced motion is set, for any width and any `scriptsRunning` value, and `loadGsap` is false in every static case.
  - When the mode is `static`, every layer is visible and `bats` and `reveals` are false.
  - Width under 640 with motion allowed gives `far: false` and `bats: false`.
  - Motion mode never sets `layers` to a state that hides the headline or the beats (the headline and beats are not in the layer set).
  - `copy.ts` contains no string matching `/smallest possible|proven minimum|optimal/i`.
- **Component tests:** the `h1` text, the three beat headings in order, the CTA `href` equal to `/login`, and the smoke marker present on `<main>`.
- **Static build checks:** `npm run build` produces the landing HTML with the h1 and beats inside the static file (not added by JavaScript).
- **Lighthouse:** `npx lighthouse` (Apache-2.0) with the mobile preset, three runs, median performance, against the production build served by the Rust server on localhost. This is a manual script step, recorded in `docs/kiro-log/landing-steel-night-lighthouse.md`.
- **Reduced motion:** a manual check in the browser with reduced motion on: no GSAP request appears in the network log, and all layers are at their final positions.

## Verification

1. `cd web && npm run check && npm test -- landing && npm run build` passes.
2. The built `web/build/index.html` contains the `h1`, the three beat headings and the CTA link, with no JavaScript needed to see them.
3. In the Compose stack, `http://localhost/` returns the page through Caddy and `counterspell-server`, and the smoke marker is present.
4. Lighthouse mobile performance median of three runs is at least 90. If it is below 90, the page is not finished, and the first thing to cut is the far layer and the bats (not the content).
5. With reduced motion on, the GSAP network request is absent and every element is visible.
6. No request leaves the origin (R29).

## Security and privacy

- Static content only. No user data is read, stored or sent.
- No third-party script or font.
- No inline script is added to `app.html`, so the strict CSP from the server is not weakened.
- The SVGs are local files, and no SVG contains script, external references or `foreignObject`. Each one is checked for those before it is committed.

## Open questions

1. **Art source.** `docs/system-design.md` and `efficiency-familiar` mention realistic painted art in `assets/pack-realistic/`, but that folder is not in the repository and no moon or graveyard art exists. **Assumed choice:** hand-authored flat vector silhouettes in `web/static/landing/`, original work under the project's MIT licence. Replace them later if realistic art arrives.
2. **Sign-up route.** The auth spec has no sign-up route; sign-up happens on `/login`. **Assumed choice:** the primary CTA "Start practising" links to `/login`. If Brighton wants a `/signup` alias, it is a one-line redirect, decided in the `auth` spec.
3. **Hero copy and the three beats.** The wording is my draft. **Assumed choice:** use it as written in `copy.ts` and change it only with Brighton's approval.
4. **Wordmark.** The palette lists Pirata One as optional. **Assumed choice:** the eyebrow uses JetBrains Mono text, and no Pirata One font is shipped.
5. **Lighthouse conditions.** Lighthouse can be run on a laptop, a VPS or a throttled profile. **Assumed choice:** the default mobile preset from the Lighthouse CLI, median of three runs, against localhost, so the score reflects the page and not the network. The VPS is measured again on freeze day.
6. **Looping ambient motion.** `docs/palette.md` says cinematic, but loops cost battery and conflict with reduced motion. **Assumed choice:** every landing animation plays once; bats fly once and smoke rises once.
7. **Dark only.** **Assumed choice:** the page ignores the operating system light setting and always renders the dark tokens, as `docs/palette.md` says.
8. **GSAP licence.** GSAP is under Webflow's Standard No Charge licence, which `docs/licence-rules.md` accepts as an exception. **Assumed choice:** use it as an npm dependency only, keep its notice in `THIRD-PARTY-NOTICES.md`, and exclude it from the licence checker as `.kiro` hook `licence-check-on-deps` already does.
9. **Font files.** The palette names the packages but not the exact Latin subset files. **Assumed choice:** the Latin subset woff2 from the named packages, verified against the OFL text in `THIRD-PARTY-NOTICES.md` before commit.
