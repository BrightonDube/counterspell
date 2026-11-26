# The Loop Entry

## Task
A chain may be knotted: `nxt[i]` gives the next index, or `-1` at the tail, but a knot sends the walk back to a cell it already visited. Call `loop_entry(nxt)` and return the index where the loop begins — the first index the walk enters that it will visit again — or `-1` when the chain ends cleanly.

## Input
`nxt` holds each link's next index; `-1` ends the chain. The walk starts at index `0`; a loop reachable only through cells the walk never visits does not count.

## Output
The loop's entry index, or `-1` when the chain reaches `-1`.

## Examples
- `loop_entry([1, 2, 0])` returns `0` — the loop is 0,1,2 back to 0.
- `loop_entry([1, 2, 3, 1])` returns `1` — a tail link leads into a loop that begins at index 1.
- `loop_entry([1, 2, -1])` returns `-1`.

## Constraints
- `0 <= len(nxt) <= 40`
- every `nxt[i]` is `-1` or an index into the list
