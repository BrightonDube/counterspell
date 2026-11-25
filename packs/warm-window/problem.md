# The Warm Window

## Task
A keeper logs warm readings one by one. A hearth can sit under any `width` consecutive readings, and its glow is the sum of those readings. Call `warmest_window(readings, width)` and return the largest glow any placement can reach, or `0` when fewer than `width` readings exist.

## Input
A list of readings and a window width. Each reading is at most `99`, and the width is at least `1`.

## Output
Return the largest sum found in any run of `width` consecutive readings, or `0` when no complete window fits.

## Examples
- `warmest_window([3, 1, 4, 1, 5], 2)` returns `6`. The last pair, `1 + 5`, glows strongest.
- `warmest_window([2, 2, 2], 4)` returns `0`. Three readings cannot fill a window of four.
- `warmest_window([7], 1)` returns `7`. A single reading is one window.

## Constraints
- The log holds at most 40 readings.
- Each reading is at most `99`.
- The width is between `1` and `40`.
