# The Idle Stretch

## Task
Duty seals hold marks of the watch `[0, t]`; everything else stands idle. Call `idle_stretch(spans, t)` and return the longest run of consecutive idle marks.

## Input
`spans` lists the `[a, b]` duty seals; `t` is the watch's last mark.

## Output
The longest idle run — `0` when every mark is held, `t + 1` when nothing is held.

## Examples
- `idle_stretch([[0, 1], [3, 4]], 5)` returns `1` — marks `2` and `5` idle alone.
- `idle_stretch([], 5)` returns `6` — the whole watch idles.
- `idle_stretch([[1, 3]], 5)` returns `2` — marks `4, 5` idle together.

## Constraints
- `0 <= len(spans) <= 20` rows of `[a, b]` with `0 <= a <= b <= 100`; `0 <= t <= 100`
- Seals reaching past `t` hold only the marks inside the watch
