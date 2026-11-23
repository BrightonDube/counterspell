# Lantern Recall Polish Requirements

## Goal

Give Lantern Recall a themed image-backed reveal while preserving its memory-game clarity and speed.

## Requirements

- Unrevealed tiles keep their current quiet visual state and numbering.
- Sequence playback reveals a themed image beneath the active tile.
- Player presses reveal the same tile image before correct or wrong feedback.
- Reveal uses a lightweight flip, fade or transparency transition.
- Tile images load outside the critical board interaction path and use stable cached URLs.
- Sequence order, playback timing, level progression, score saving and signed-out behaviour remain unchanged.
- Roving focus, arrow-key navigation, button labels and live announcements remain correct.
- Reduced motion uses an immediate or short opacity state rather than spatial flipping.
- Images are decorative to assistive technology; tile identity remains textual.
