# The Lantern Watch

## Task
Duty seals must each carry a lantern mark inside them. Call `watch_points(spans)` and return the fewest lantern marks that light every seal — one mark lights every seal holding it.

## Input
`spans` lists the `[a, b]` duty seals, `0 <= a <= b <= 100`.

## Output
The fewest marks lighting every seal; `0` for no seals.

## Examples
- `watch_points([[0, 2], [1, 3]])` returns `1` — mark `1` or `2` lights both.
- `watch_points([[0, 1], [2, 3]])` returns `2`.
- `watch_points([[0, 9], [1, 2], [3, 4], [5, 6], [7, 8]])` returns `4`.

## Constraints
- `0 <= len(spans) <= 20` rows of `[a, b]` with `0 <= a <= b <= 100`
