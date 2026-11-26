# The Sentry Gap

## Task
A wall of `cells` posts must hold `sentries` identical sentries with at least `gap` empty posts between any two. Call `sentry_ways(cells, sentries, gap)` and return the number of layouts.

## Input
`cells` is the wall length, `sentries` the posted count and `gap` the required empties between sentries.

## Output
The number of layouts honouring the gap.

## Examples
- `sentry_ways(5, 2, 1)` returns `6` — pairs like 0+2, 0+3, 0+4, 1+3, 1+4, 2+4.
- `sentry_ways(4, 2, 2)` returns `1` — posts 0 and 3.
- `sentry_ways(3, 0, 1)` returns `1`.

## Constraints
- `0 <= cells <= 20`, `0 <= sentries <= 20`, `0 <= gap <= 10`
