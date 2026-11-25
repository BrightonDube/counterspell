# The Gale Peaks

## Task
A storm watch records wind readings one per hour. Every `width` consecutive hours form a gust window. Call `gale_peaks(readings, width)` and return, in order, the highest reading of each gust window — one peak per window.

## Input
A list of readings and a window width. Each reading is at most `99`, and the width is at least `1`.

## Output
Return a list with the peak of every window of `width` consecutive hours, in order. Return an empty list when fewer than `width` readings exist.

## Examples
- `gale_peaks([2, 1, 3, 4], 2)` returns `[2, 3, 4]`. The windows are `[2, 1]`, `[1, 3]` and `[3, 4]`.
- `gale_peaks([1, 2], 3)` returns `[]`. Two readings cannot fill a window of three.
- `gale_peaks([5], 1)` returns `[5]`. One reading is one window of one.

## Constraints
- The log holds at most 40 readings.
- Each reading is at most `99`.
- The width is between `1` and `40`.
