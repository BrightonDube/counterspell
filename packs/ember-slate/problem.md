# The Ember Slate

## Task
A slate of ember marks must climb strictly — each later mark must out-glow the last taken. Call `slate_len(marks)` and return the longest strictly climbing run you can pick, keeping row order.

## Input
`marks` lists the glows in row order.

## Output
The longest climbing pick; an empty slate climbs `0`.

## Examples
- `slate_len([])` returns `0`.
- `slate_len([3, 2, 1])` returns `1` — nothing climbs.
- `slate_len([1, 3, 2, 4, 3, 5])` returns `4` — `1, 2, 4, 5` or `1, 2, 3, 5`... the pick `1, 2, 4, 5` climbs four.

## Constraints
- `0 <= len(marks) <= 30`; each glow is `0..1000`
