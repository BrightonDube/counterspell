# The Frame Rite

## Task
The frame sees the whole stream — on a fault it keeps the pages needed soonest and drops the one whose next use is farthest away (never again is farthest of all). Call `frame_rite(pages, f)` and return the fault count.

## Input
`pages` lists arrivals (`0` to `30`); `f` is the frame size (`1` to `6`). Ties between equally-far pages evict the one sitting earliest in the frame.

## Output
The optimal fault count — the fewest any strategy can reach.

## Examples
- `frame_rite([0, 1, 2, 0, 3, 1, 4], 2)` returns `6`.
- `frame_rite([0, 1, 0, 1], 2)` returns `2`.
- `frame_rite([], 3)` returns `0`.

## Constraints
- `0 <= len(pages) <= 24` with `0 <= page <= 30`, `1 <= f <= 6`
