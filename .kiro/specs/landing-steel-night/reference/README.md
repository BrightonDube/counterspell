# Title-screen mockup (reference only)

Approved by Brighton Dube on 15 Nov 2026: variant B of the full-viewport
landing title screen.

These files are a **visual reference** for Kiro. They are not production code
and must not be copied into `web/` as a page.

| File | What to match |
|---|---|
| `v3-b.png` | Desktop composition: full-bleed living background, Creepster wordmark, brighter tagline, glowing ENTER at the bottom centre, pumpkin glow, no middle familiar. |
| `v3-b-mobile.png` | Mobile crop: pumpkin still partly visible, wordmark scaled without clipping. |
| `index.html` | CSS typography, ember `text-shadow`, pumpkin-glow pulse, ENTER pulse, reduced-motion rules. Re-implement in the Svelte landing hero; do not serve this file. |

The production loop and poster live in `web/static/art/landing/hero-moon-living-*`.
The production font is `web/static/fonts/Creepster-Regular.woff2`.
