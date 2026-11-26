# The Stale Tally

## Task
The queue ledger records a version for each seat; queued events carry the version they saw. An event is stale when its version no longer matches the seat's current one. Call `stale_count(seen, queued)` and return the number of stale events.

## Input
`seen` holds current seat versions; `queued` holds the version each event saw, in the same order. The two lists share one length.

## Output
The count of seats whose queued version differs from the current version.

## Examples
- `stale_count([1, 2, 3], [1, 5, 3])` returns `1` — seat 1 moved on.
- `stale_count([1, 1, 1], [1, 1, 1])` returns `0`.
- `stale_count([], [])` returns `0`.

## Constraints
- `0 <= len(seen) = len(queued) <= 40`, `0 <= seen[i], queued[i] <= 99`
