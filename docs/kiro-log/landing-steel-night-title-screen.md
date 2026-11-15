# landing-steel-night: title-screen audit (T26)

Run: 15 Nov 2026, Kiro implementation run (attempt 1). Scope: `web/` only. Audit method: code-level review against the Impeccable audit playbook (`.kiro/skills/impeccable/reference/audit.md`), plus the measurable checks below. No browser was used; per the test rules, browser E2E is out of scope, so the visual and Lighthouse checks are listed as open items.

## Findings and fixes

| # | Dimension | Finding | Status |
|---|---|---|---|
| 1 | Performance | The poster was a 286,156-byte JPEG on the first-load path. | Fixed. Added derived WebP posters (`hero-moon-living-poster-1920.webp`, 88,220 bytes; `-960.webp`, 38,570 bytes), with the JPEG kept as the `img` fallback. The poster is a `picture` with a WebP `source` (960 and 1920 widths). |
| 2 | Performance | The living-loop video must not be requested before first paint or under reduced motion. | Confirmed. The static HTML has no `<video>`. `Hero.svelte` mounts it only inside `afterPaint`, and `chooseHeroVideo` returns null for reduced motion, Save-Data and a hidden tab. Unit tests cover these in `hero-media.test.ts`. |
| 3 | Performance | GSAP must load only in motion mode, after first paint. | Confirmed. The GSAP core (about 70 KB) is in its own chunk, which `index.html` does not preload. The route chunk (about 12 KB) holds only the scene module's code. |
| 4 | Accessibility | Heading outline: one `h1` (wordmark), then `h2` (headline), then `h3` (beats). | Confirmed by tests and the built HTML. |
| 5 | Accessibility | ENTER link: accessible name "Enter, go to sign in", a visible focus ring from the `--focus` token, and the Enter-key shortcut only when no other control has focus. | Confirmed by unit tests. |
| 6 | Accessibility | Reduced motion: the ENTER pulse and the pumpkin glow pulse are disabled in CSS. The scene module is not loaded. | Confirmed in CSS. Not run in a browser. |
| 7 | Theming | No raw hex or `rgb()` literals in `web/src/lib/landing/*.svelte` or `+page.svelte`. The glow colour is the single `--hero-ember` token in `app.css`. | Fixed during T24 (`#ffcc6a` changed to `--accent-candle-glow`). |
| 8 | Visual | The painted bat flock and fog are already in the living loop. The planner's bat-swarm and fog overlays may duplicate them and add visual noise. | Open. Kept for now because the existing overlay tests cover them. Brighton should review on the page and decide whether to remove them (see the review queue). |
| 9 | Contrast | Wordmark and tagline use `--color-text` on a dark `--bg` pool and glow. ENTER label uses `--accent-candle-glow`. Both sit above the 4.5:1 body-text target by palette values (`docs/palette.md`). | Checked against the palette values. Not measured on screen. |
| 10 | Copy | No hackathon mention, and no "smallest possible", "proven minimum" or "optimal" in `copy.ts` (R9 test). | Confirmed. |

## First-load bytes (production build, raw, not gzip)

| Asset | Bytes | Loaded when |
|---|---|---|
| Poster, WebP 1920 (desktop) | 88,220 | First paint (`fetchpriority="high"`) |
| Poster, WebP 960 (phones) | 38,570 | First paint on phones |
| Poster, JPEG fallback | 286,156 | Only if WebP is unsupported (not preloaded) |
| Creepster woff2 | 28,120 | Preloaded from the landing route |
| Geist variable woff2 | 69,488 | First paint (body text) |
| Entry JS and CSS named in `index.html` | 187,519 | First paint |
| GSAP core chunk (about 70 KB) | not preloaded | Motion mode only, after first paint |
| Living-loop video | not requested | Motion mode only, after first paint |

## Checks run in this session

- `npm test`: 419 tests passed in 60 files.
- `npm run check`: 0 errors, 0 warnings.
- `npm run build`: passed. The built `index.html` contains the wordmark, the tagline, "Learn why your code breaks.", the ENTER link to `/login` (`aria-label` "Enter, go to sign in"), "Start practising" and the smoke marker, all without JavaScript.

## Open items (not verified here)

- Lighthouse mobile median (T13) and the keyboard, contrast and Compose smoke checks (T14) need a browser or a running stack. They are left open for Brighton (see the blockers file).
- The bat-swarm and fog overlays (finding 8) need a visual review on the page.
- The poster has no WebP at 960 for very large phones, which is acceptable at the 960 breakpoint, but the living loop MP4/WebM is not re-encoded (the approved assets are used as delivered).
