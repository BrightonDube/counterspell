# Themed Chess Experience Design

Create one shared chess presentation module for board sizing, palette, piece sprite and accessible labelling. Keep `chess.js`, the bot worker and puzzle state as the game-logic sources of truth.

The board uses a CSS `clamp` bounded by available content width and viewport height. Route surfaces may widen within their existing layout without reordering the page.

The piece set is one original SVG sprite sheet. Vector filters are kept minimal and tested for browser cost. The skull uses an approved small existing creature asset or a licence-compliant original derivative, explicit dimensions and CSS transform or opacity keyframes with long pauses.

Tests cover responsive geometry, accessible naming, reduced motion, one legal bot exchange, one puzzle move sequence and saved-score behaviour.
