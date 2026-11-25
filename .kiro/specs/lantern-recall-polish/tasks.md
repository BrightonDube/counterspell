# Lantern Recall Polish Tasks

- [x] **LR01 Capture current gameplay, accessibility and performance baselines** — `baseline-2026-11-25.md`
- [x] **LR02 Add failing reveal-state behavioural tests** — `page.test.js` reveal describes, `memory-css.test.ts`, `memory.test.js` TILE_ART contract
- [x] **LR03 Add deterministic lazy image assignments** — `TILE_ART` in `memory.js`: 9 stable `/art/objects/*.webp` paths, verified to exist on disk
- [x] **LR04 Implement pattern and press reveal transitions** — opacity crossfade (160 ms), playback-lit and press reveal share `.tile-reveal`
- [x] **LR05 Preserve correct, wrong and reduced-motion feedback** — verdict colour washes through the art 180 ms after the reveal; transitions collapse globally and in scoped media
- [x] **LR06 Verify keyboard play, announcements and score persistence** — roving-focus/Enter-press regression test green; live-region and save-flow tests unchanged and green
- [x] **LR07 Deploy and verify Lantern Recall on mobile and desktop** — merged `31559a2b`, deployed; live SSR page renders all 9 decorative lazy `tile-art` images for desktop and mobile UAs, all art assets 200
