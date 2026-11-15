# efficiency-familiar: mascot art log (T49 to T51, 15 Nov 2026 run)

## Assets (T49)
- Approved mood paintings (Brighton, 15 Nov 2026 15:21): `web/static/art/familiar/moods/familiar-{1..5}-*.webp`, each 1024 x 1024 (checked from the PNG headers).
- WebP stills total 800,224 bytes (five files, 149,574 to 167,018 bytes each). They load from the SVG `<image>` layers, so they are not counted in the SVG budget.
- `web/static/familiar/jack-o-lantern.svg`: 1.5 KB, within the 120 KB budget. It declares `fam-body`, the glow stops, `fam-mood-1` to `fam-mood-5` and `fam-fx`.
- `web/static/familiar/jack-o-lantern.placeholder.svg` is kept as a labelled fallback. It satisfies the same contract.
- `web/static/art/familiar/jack-o-lantern-orange` and `-poison` are untouched.
- No art was generated or drawn over.

## Code changes
- `mascot/contract.ts`: the contract is five mood images plus body, glow stops and effect layer. Carved eye, mouth and crack layers are no longer required.
- `mascot/mood-style.ts`: each mood names one image layer (`layer`).
- `mascot/Mascot.svelte`: shows one image at a time. On a live change the new image cross-fades in over 300 ms. Reduced motion or mute swaps at once. Loads the real SVG first and falls back to the placeholder.
- `mascot/timelines.ts`: the crack-burst no longer draws crack paths.
- `Familiar.svelte` (the editor-corner dock): now renders the mascot for a measured mood. Before a measured mood it shows only the text status.
- `familiar-state.ts`: adds `headline` (the result sentence without the mood prefix), so the mascot does not repeat the live text.

## Deviations to review (Brighton)
1. The dock passes `reason="measured"` to the mascot, because the state does not keep the reason code. The attribute is cosmetic only.
2. The dock passes a fixed `seed` (default 1), because the state does not keep the run id. Burst particles therefore look the same on every run.
3. The crack burst plays only for a live failed run at mood 1, and the laugh burst only for a live mood 5. The design's effect field is not yet in the measured payload.

## Impeccable findings and fixes (T51)
- Applied the craft and polish principles from `.kiro/skills/impeccable`: one centred figure, a token-based surface (`bg-surface`, `border-border`), 44 px touch targets on the mute and hide buttons, and no ambient loop in the IDE.
- Removed the duplicate mood label. The mascot now carries the mood name, badge and headline.
- Visual review in a browser is deferred. It is an E2E check under the standing rule, so Brighton should look at the dock on the real stack.
