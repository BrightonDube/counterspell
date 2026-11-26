# The Vector Clock

## Task
Two scribes keep tally-vectors of what they have seen — one tally per tower. A vector `a` happened before `b` when every tally in `a` is no larger and at least one is smaller. Call `clock_cmp(a, b)` and return `0` when the vectors match, `-1` when `a` happened before `b`, `1` when `b` happened before `a`, and `2` when neither order holds (the rites ran concurrently).

## Input
`a` and `b` are tally lists of equal length (`0` to `6`), each tally `0` to `60`.

## Output
`0` equal, `-1` a before b, `1` b before a, `2` concurrent.

## Examples
- `clock_cmp([1, 0], [2, 0])` returns `-1` — `a` is strictly behind.
- `clock_cmp([2, 0], [0, 2])` returns `2` — each scribe saw something the other missed.
- `clock_cmp([1, 1], [1, 1])` returns `0`.

## Constraints
- `0 <= len(a) = len(b) <= 6`, tallies `0` to `60`
