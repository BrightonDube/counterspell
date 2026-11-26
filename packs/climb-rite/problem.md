# The Climb Rite

## Task
A rite climbs a stair of `steps` ledges. Each pace covers one ledge or two. Call `climb_ways(steps)` and return how many distinct pace orders finish the climb.

## Input
`steps` is the ledge count, `0..40`.

## Output
The number of pace orders; an empty stair counts as one finished climb.

## Examples
- `climb_ways(0)` returns `1`.
- `climb_ways(3)` returns `3` — `1+1+1`, `1+2` and `2+1`.
- `climb_ways(5)` returns `8`.

## Constraints
- `0 <= steps <= 40`
