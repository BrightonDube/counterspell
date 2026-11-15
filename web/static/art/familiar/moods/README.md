# Efficiency Familiar — mood art

Approved by Brighton Dube, 15 Nov 2026 15:21 SAST.

Five realistic mood variants, one painted pumpkin per mood, with the same
framing so they swap in place without jumping:

1. `familiar-1-furious` — wrong, crashed or timed out
2. `familiar-2-angry` — passes, two complexity classes worse than the reference
3. `familiar-3-uneasy` — one class worse
4. `familiar-4-pleased` — same class, just slower (or an unmeasured pass)
5. `familiar-5-laughing` — matches the reference

Each mood ships as a 1024 px transparent `.png`, a `.webp` still (preferred in
the app), and a self-contained animated `.svg` (embedded WebP + CSS;
`prefers-reduced-motion: reduce` freezes it).

The IDE mascot uses the five WebP stills and cross-fades between them on a
mood change. Animated SVGs are not an idle loop in the IDE (calm-IDE rule).
Mood still comes only from `decide`, stays calm while the student is typing,
changes at most once per 2 s, and shows the static frame under reduced motion
or mute.

The earlier `jack-o-lantern-orange` and `jack-o-lantern-poison` files in the
parent folder are kept for other surfaces and must not be deleted. They are
not the mascot body.

Provenance and licence: see `web/static/art/README.md`.
