# Lantern Recall Polish Design

Each tile has a front face and an image-backed reveal face. Existing sequence and press state drives a shared semantic `revealed` presentation, while correct and wrong feedback remains a separate state.

A small curated set of existing Counterspell assets is assigned deterministically to the nine tile indices. The browser may preload only low-cost metadata; image bytes load lazily after the game shell is usable. Explicit dimensions prevent layout shift.

The normal transition uses 3D CSS perspective sparingly or an opacity crossfade if measurements show the flip is costly. Reduced motion uses no rotation.

Tests exercise sequence playback, presses, wrong and correct states, keyboard control, score persistence and reduced motion through visible behaviour rather than CSS implementation details.
