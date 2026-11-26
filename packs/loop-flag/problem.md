# The Loop Flag

## Task
A chain may be knotted: `nxt[i]` gives the next index, or `-1` at the tail, but a knot sends the walk back to a cell it already visited. Call `loop_flag(nxt)` and return `1` when the walk from index `0` loops forever, else `0`.

## Input
`nxt` holds each link's next index. `-1` means the chain ends. The walk starts at index `0`; a loop reachable only through cells the walk never visits does not count.

## Output
`1` when the walk loops forever, `0` when it reaches `-1`.

## Examples
- `loop_flag([1, 2, 0])` returns `1` — the walk cycles 0,1,2,0.
- `loop_flag([1, 2, -1])` returns `0`.
- `loop_flag([])` returns `0`.

## Constraints
- `0 <= len(nxt) <= 40`
- every `nxt[i]` is `-1` or an index into the list
