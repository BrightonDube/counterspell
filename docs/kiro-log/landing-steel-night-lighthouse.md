# Landing page: Lighthouse mobile (landing-steel-night T13)

## Result
**Median performance score 91** over three runs (91, 91, 90), measured on the title-screen hero. The target is at least 90 (R23).

| Run | Score | FCP | LCP | CLS | TBT |
|---|---|---|---|---|---|
| 1 | 91 | 1.2 s | 3.5 s | 0 | 0 ms |
| 2 | 91 | 1.2 s | 3.5 s | 0 | 0 ms |
| 3 | 90 | 1.2 s | 3.6 s | 0 | 0 ms |

- JavaScript transferred before GSAP: **121 KB** (235 KB in total once GSAP loads after first paint; the two GSAP chunks are 114 KB).
- Page weight in the run: 34 requests, 1.05 MB, of which the hero video is 0.5 MB and loads after first paint. Images 108 KB.
- Every request went to the one local origin. No third-party host appeared.

## How it was measured
- Date: 19 Nov 2026 (run time 20:09 UTC). Lighthouse **13.6.0** (`npx lighthouse@13.6.0`), performance category only, the default mobile configuration (412 px emulated phone, simulated slow-4G throttling). Lighthouse 13 has no `--preset=mobile`; the default config is the mobile one, and it is what PageSpeed Insights reports.
- Browser: HeadlessChrome 159 (Playwright's cached Chromium). It was launched by hand with `--remote-debugging-port` and Lighthouse attached with `--port`, because Lighthouse's own launcher treats this WSL host as Windows and fails on its temp path.
- Server: `counterspell-testserver`, which runs the production Axum router and serves the built `web/build` through the same static path as `counterspell-server`. It is on `127.0.0.1:5198`, not 8080 (the task text), and has a scripted judge that does not affect the landing page. Both processes were stopped afterwards.
- The first attempt used `--preset=perf`, which switches to DevTools throttling. It gave 72, 86, 87 and is not comparable; it is not used.

## What changed the number
| Step | Scores | Median |
|---|---|---|
| Before any change | 89, 88, 88 | 88 |
| Hero poster `<link rel="preload" as="image">` with the same `srcset` | 88, 88, 88 | 88 (no effect) |
| Small `srcset` candidates for the five decorative images | 91, 91, 90 | **91** |

- The LCP element is the hero poster (`fetchpriority=high`, discoverable, 38 KB on phones). Its real-browser phases total about 67 ms. The 3.5 s is the simulated slow link sharing bandwidth with everything fetched before it.
- Lighthouse's image-delivery insight showed about 152 KiB wasted: the three beat icons (64 px) and the two call-to-action pictures (96 px) were 512 px files of 20 to 47 KB each. New variants, made by resizing the approved art (Lanczos, alpha kept; the originals are untouched and are still the `src` fallback): `objects/{crypt-key,crystal-ball,brain-jar}-128.webp` (4 to 6 KB) and `objects/candle-flame-192.webp`, `familiar/jack-o-lantern-orange-192.webp` (6 and 11 KB). Roughly 140 KB less is fetched on a phone.
- The poster preload gave no measurable gain in this setup. It is kept because it is correct practice and costs one line; remove it if the code review prefers less.
- The task's cut list (far layer, bats, smoke, parallax) was not needed. Those are post-paint layers and do not affect LCP. Nothing was cut.

## Tests added
- `page.landing.test.js`: the head preloads the poster with the picture's own `srcset`, `sizes`, type and high priority.
- `GhostBeats.test.ts`, `SignUpCta.test.ts`: each image offers the small candidate and keeps the 512 px file as fallback, with `sizes` matching the displayed size.
- `art-variants.test.ts`: the five variant files exist at 128 or 192 px square, are under 12 KB, and the five originals are still 512 px.
- Web gate: svelte-check 0 errors and 0 warnings, vitest 542 passed (4 skipped), prettier and eslint clean on the touched folders.

## Other findings (not changed)
- Lighthouse reports `cache-insight` (static files have no long-lived cache header) and 48 KiB of unused CSS. Neither moves the score today. A far-future `Cache-Control` for `/_app/immutable/*` would help repeat visits; it is a `crates/server` change and was not made here.
- T14 is still open: the keyboard walk-through, contrast pairs and the Compose and Caddy smoke test are separate checks.

## 2026-11-20 — feat/signin-redesign re-run

Three mobile runs, Lighthouse 13.6.0, production build served by counterspell-server (chromium headless-shell):

| Run | Perf | A11y | BP | SEO |
|---|---|---|---|---|
| 1 | 89 | 100 | 96 | 91 |
| 2 | 91 | 100 | 96 | 91 |
| 3 | 91 | 100 | 96 | 91 |

Median perf **91** — above the 90 gate; no fallback cuts needed.
