# Landing title-screen loop (`web/static/art/landing/`)

Approved by Brighton Dube, 15 Nov 2026 19:20 SAST, as the landing **title-screen
hero** background (variant B).

## Files

| File | Use |
|---|---|
| `hero-moon-living-poster.jpg` | First paint and reduced-motion still (1920×1080). |
| `hero-moon-living-1920.{webm,mp4}` | Desktop loop, loaded after first paint. |
| `hero-moon-living-960.{webm,mp4}` | Narrow viewports (900 px CSS width or less). |

The earlier `moon-graveyard-loop-*` and `moon-graveyard-poster-*` files stay in
this folder and must not be deleted. They are no longer the title-screen source.

## Motion and how to use it

The loop is the approved steel-blue moon/graveyard painting, with the painted
bats in flight (including the large bat across the moon) and the green fog
drifting around the laptop. It is seamless.

- Put the poster in the prerendered HTML (`fetchpriority="high"`, explicit
  width and height). Mount the `<video>` only after first paint, in motion
  mode, WebM then MP4, `preload="none"`, `muted`, `loop`, `playsinline`.
- No video under `prefers-reduced-motion: reduce`, `Save-Data`, or a hidden tab.
- The painted pumpkin's glow pulse is a **CSS overlay** (see the mockup at
  `.kiro/specs/landing-steel-night/reference/`), not a second pumpkin image.
  Do not add a middle familiar on the title screen.
- On narrow screens, `object-position` must keep the painted pumpkin partly
  visible on the left. Compare with `reference/v3-b-mobile.png`.

A local WebP of the poster may be derived and committed next to the JPEG if it
helps the Lighthouse mobile median >= 90 target. Do not fetch media from a CDN.

## Provenance and licence

Hand-built on 15 Nov 2026 from Brighton's approved realistic moon/graveyard
painting (project-owned, no trademarks, Steel Night grade). Bats and laptop
fog are animated in the encode; the product adds the CSS pumpkin glow. Licence:
same as `web/static/art/README.md` (project-owned, Brighton Dube).
