# Counterspell art (`web/static/art/`)

Official art for Counterspell, approved by Brighton Dube (15 Nov 2026). It replaces the flat-vector
placeholders in `web/static/landing/` and `web/static/familiar/`, which stay as fallbacks.

## Provenance and licence

- AI-generated for this project with Grok Bot's image generator (photoreal horror key art, prompts
  with no text, logos or third-party characters), plus Brighton Dube's own moon/graveyard painting
  for the landing loop.
- Backgrounds removed with `rembg` (open source); motion is hand-written SVG/CSS and ffmpeg encodes.
- Original designs, no trademarks. Licence: project-owned (Brighton Dube), shipped with the project.
- Colours are graded to the Steel Night palette: steel-blue night, moss/poison green, candle amber,
  blood-red accents only.

## Folders

| Folder            | Contents                                                                                                                                                                                                                 | Notes                                                                                                                                         |
| ----------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------- |
| `landing/`        | Title-screen loop `hero-moon-living-{1920,960}.{webm,mp4}` and `hero-moon-living-poster.jpg` (approved 15 Nov 2026 19:20). Earlier `moon-graveyard-loop-*` and `moon-graveyard-poster-*` remain and must not be deleted. | Title-screen hero: poster first, living video after first paint, 960 on narrow screens. See `landing/README.md`.                              |
| `familiar/`       | `jack-o-lantern-orange` and `jack-o-lantern-poison`: `.svg` (self-contained animated, embedded WebP), `.png`/`.webp` 512 px transparent, `.webm` (VP9 alpha loop)                                                        | Kept for other surfaces (landing CTA accent, dashboard empty/streak/loading, 404/error, celebration). Not the IDE mascot body. Do not delete. |
| `familiar/moods/` | `familiar-1-furious` … `familiar-5-laughing`: 1024 px transparent `.png`/`.webp`, self-contained animated `.svg`                                                                                                         | Approved 15 Nov 2026 15:21 SAST. The IDE mascot: one image per mood, same framing, opacity cross-fade.                                        |
| `creatures/`      | wraiths (green, pale, lunge), skulls (green/blood eyes), raven, black cat, spider drop, bat, grave hand, reaper tarot                                                                                                    | `.svg` animated, `.png`/`.webp` stills. Empty, error and failure states.                                                                      |
| `objects/`        | cauldron, candles, lanterns, potion, poison vial, grimoires, coffin, tombstone, eyeball, crystal ball, crypt key, chain, hourglass, cursed CRT, bug (beetle/glitch), skeleton keyboard, brain jar                        | 512 px icon-style props for dashboard cards, empty states, IDE accents.                                                                       |
| `fx/`             | fog layers (ash/cold/poison), smoke plumes, lightning, crow flock, bat swarm, embers, poison spores                                                                                                                      | Transparent overlays; `fog-layer-cold`, `embers`, `poison-spores` also as `.webm`.                                                            |
| `heroes/`         | `hero-cursed-crypt-desk.svg`, `hero-graveyard-poison.svg` (16:9 animated scenes)                                                                                                                                         | Optional section/dashboard banners.                                                                                                           |

## Usage notes

- Animated `.svg` files work as a plain `<img>`; they honour `prefers-reduced-motion: reduce`.
- `.webm` alpha works in Chrome/Firefox/Edge; Safari ignores alpha, so use the `.webp`/`.svg` there.
- Prefer `.webp` over `.png` for size; always set `width`/`height`, `loading="lazy"` below the fold,
  and `decoding="async"`.
- Known limits: sources are 512-1280 px, so the 1920 loop is upscaled and soft on big monitors;
  some cut-out edges are imperfect (grave hand, wraith smoke).

## Derived small variants (19 Nov 2026)

Resized copies of approved art, made with Lanczos resampling and alpha kept, so phones do not download 512 px files for 64 and 96 px slots. The originals stay in place and remain the `src` fallback. No new art was generated.

- `objects/crypt-key-128.webp`, `objects/crystal-ball-128.webp`, `objects/brain-jar-128.webp` (from the 512 px files; landing beat icons)
- `objects/candle-flame-192.webp`, `familiar/jack-o-lantern-orange-192.webp` (landing call to action)
