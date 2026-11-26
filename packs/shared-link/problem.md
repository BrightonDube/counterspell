# The Shared Link

## Task
Two chains share one vault of cells: `nxt[i]` gives the next index for both. Call `shared_link(nxt, ha, hb)` and return the first index reachable from head `hb` that is also reachable from head `ha`, or `-1` when the two walks never meet.

## Input
`nxt` holds each cell's next index; `-1` ends a walk. `ha` and `hb` are the two heads. Neither walk loops back on itself.

## Output
The first shared index the b-walk meets, or `-1`.

## Examples
- `shared_link([1, 3, 4, -1, 3], 0, 2)` returns `3` — the a-walk visits 0,1,3 and the b-walk visits 2,4,3.
- `shared_link([1, -1, 3, -1], 0, 2)` returns `-1` — the walks visit 0,1 and 2,3.
- `shared_link([-1], 0, 0)` returns `0` — both walks start and end at index 0.

## Constraints
- `1 <= len(nxt) <= 40`
- every `nxt[i]` is `-1` or an index into the list
- `0 <= ha, hb < len(nxt)`
- neither walk loops back on itself
