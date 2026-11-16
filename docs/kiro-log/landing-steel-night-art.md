# landing-steel-night: art audit and first-load check (T18)

Run: 15 Nov 2026, implement pass (non-interactive). Scope: `web/` only.

## Method

- Read `.kiro/skills/impeccable/SKILL.md` and `reference/craft-floor.md`, then audited the landing page against the craft floor and the design's accessibility and performance rules.
- The Impeccable launcher was not run and no engine was downloaded, as Brighton's standing rule (15 Nov 2026) requires. Only the vendored markdown was used.
- No browser was available (no Chrome on this machine), so this audit is static: code, built HTML, built CSS and byte counts. The visual critique (rendered look, hover states, art behind text at real sizes) still needs a browser pass. It is listed under "Not checked" below.

## Measured first-load bytes (build of this commit)

Source files are the approved art in `web/static/art/`. Sizes are in bytes.

| Item | Raw | Gzip | When it loads |
|---|---|---|---|
| Poster, phone (`moon-graveyard-poster-960.webp`) | 34,150 | n/a | First paint (LCP image, `fetchpriority="high"`) |
| Poster, desktop (`moon-graveyard-poster-1920.webp`) | 77,924 | n/a | First paint, at widths above 900 px |
| Poster JPEG fallback (`moon-graveyard-poster-1920.jpg`) | 191,983 | n/a | Only in browsers without WebP |
| Fog overlay (`fx/fog-layer-cold.webp`) | 168,622 | n/a | After first paint (deferred, see fix 1) |
| Embers overlay (`fx/embers.webp`) | 2,668 | n/a | After first paint |
| Smoke overlay (`fx/smoke-plume-ash.webp`), motion only | 32,756 | n/a | After first paint, motion plan only |
| Bats overlay (`fx/bat-swarm.webp`), motion only | 7,174 | n/a | After first paint, motion plan only |
| Beat art (`objects/crypt-key`, `crystal-ball`, `brain-jar`) | 26,852 / 29,358 / 46,988 | n/a | Lazy (below the fold) |
| CTA candle (`objects/candle-flame.webp`) | 19,954 | n/a | Lazy |
| Loop video, phone (`landing/…-loop-960.webm` / `.mp4`) | 514,390 / 672,633 | n/a | Motion mode only, after first paint, `preload="none"` |
| Loop video, desktop (`…-loop-1920.webm` / `.mp4`) | 2,251,651 / 2,614,709 | n/a | Motion mode only, after first paint, `preload="none"` |
| Entry and route JavaScript (all chunks in `build/index.html`) | about 117 KB | about 42.6 KB | First load |
| Geist variable font | 69.5 KB | n/a | First load (text font) |

The JavaScript total covers every chunk the built `index.html` preloads, including the SvelteKit runtime. GSAP is not in that list, because it loads only in motion mode after first paint. The design budget is 35 KB gzip for the landing route JavaScript without GSAP. The measured 42.6 KB is over that budget by about 7.6 KB. The bulk is the framework runtime (`Czz0qr7B`, 19.4 KB gzip, and `D-zQm5MZ`, 14.9 KB gzip), not landing code. Not fixed here. The Lighthouse run (T13) decides whether it matters.

## Video timing check

- The static HTML has no `<video>` element (checked in the build output).
- The video mounts only inside an `afterPaint` callback, after first paint.
- It is requested with `preload="none"`.
- `chooseHeroVideo` returns no source under reduced motion, under Save-Data, and while the tab is hidden. The unit tests in `hero-media.test.ts` cover those cases.
- The video pauses when it leaves the viewport or the tab is hidden.

## Findings and fixes

1. **Important (fixed): the fog overlay is 168 KB and loaded with the first render.** It was the largest image on the page and competed with the poster for bandwidth. Fix: all overlays (fog, embers, smoke, bats) are added only after first paint, through `overlaysReady` in `Hero.svelte`. The poster stays in the static HTML, and the overlays are decorative, so the final frame still shows. Test: `Hero.test.ts` "defers every overlay until after first paint". I tried a lower-quality re-encode of the fog (quality 40 gives 151,686 bytes, about 10% smaller). It was not worth changing the approved art, so the art is unchanged.
2. **Important (fixed): browser selection colour was the default.** The craft floor asks for browser surfaces to carry the palette. Fix: `::selection` uses `--accent-moss-deep` and `--text` in `app.css`. This is a CSS-only change with no test, as styling config.
3. **Conflict, kept per spec: the eyebrow above the headline.** `craft-floor.md` bans a kicker or eyebrow above a heading. The landing design (`design.md`, Hero) requires the "Counterspell" eyebrow in JetBrains Mono. Brighton's rule says the spec wins on conflict, so the eyebrow stays. Brighton can drop it if the review prefers the craft-floor default.
4. **Conflict, flagged to Brighton: R25 says no raster image may be the largest element.** The 15 Nov amendment makes the poster the LCP image. The amendment is later and approved, so the poster is kept. R25 is superseded for the poster and is recorded in `docs/kiro-log/landing-steel-night-blockers.md`. The headline is still in the static HTML.
5. **Minor: JavaScript over the 35 KB gzip route budget (about 42.6 KB measured).** See the table. Not fixed here; for T13.
6. **Minor: Geist is one 69.5 KB variable file.** Already noted in the blockers file. T13 decides.

## Checked and found acceptable (static)

- Body measure: the supporting line and beat copy use `max-w-xl` (36 rem), about 65–75 characters per line at the body size.
- Contrast of text pairs against the `app.css` tokens: unchanged from the T14 static check (lowest pair ash on bg, 5.64:1). The hero scrim fades the art to the page background at the headline's position, so the headline sits on `--bg` (14.96:1).
- No section numbers, no gradient text, no glass or blur effects, no coloured `border-left`, and no emoji icons.
- Icons and art are decorative (`alt=""`, `aria-hidden="true"`) with explicit sizes, so no layout shift.
- Focus: the CTA keeps the candle focus ring from `app.css`.
- Motion: one moon push-in, one smoke rise, one bat flight and one reveal per section, each played once. No loop.

## Not checked (needs a browser; deferred E2E)

- Rendered look at the sizes the page ships at (phone, tablet, desktop), including how the art sits behind the headline.
- Hover, focus and keyboard behaviour on real devices.
- Reduced-motion check in devtools: no GSAP request and final frames visible.
- Network log: no third-party request and the video not requested before first paint.
- Lighthouse mobile median (T13) and the keyboard and focus path (T14).
