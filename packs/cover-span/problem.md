# The Cover Span

## Task
Wax seals must cover every mark of `[0, t]`. Call `span_cover(spans, t)` and return the fewest seals that seal the stretch — each next seal must reach `front` or earlier... A seal `[a, b]` may follow only when `a` is no later than the first bare mark.

## Input
`spans` lists the `[a, b]` seals; `t` is the last mark to cover.

## Output
The fewest seals covering `0..t`, or `-1` when coverage breaks.

## Examples
- `span_cover([[0, 2], [3, 5]], 5)` returns `2` — mark `2` to `3` is contiguous.
- `span_cover([[0, 9]], 9)` returns `1`.
- `span_cover([[0, 2], [4, 9]], 9)` returns `-1` — mark `3` stays bare.

## Constraints
- `0 <= len(spans) <= 20` rows of `[a, b]` with `0 <= a <= b <= 100`; `0 <= t <= 100`
