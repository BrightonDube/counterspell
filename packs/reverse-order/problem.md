# The Reversed Order

## Task
A row of lantern links is chained: `marks[i]` holds each link's mark and `nxt[i]` gives the next index, or `-1` at the tail. Call `reverse_order(marks, nxt)` and return the marks in reverse chain order — tail first.

## Input
`marks` holds each link's mark and `nxt` holds each link's next index; `-1` ends the chain. The walk starts at index `0`; unreached cells are stray. The chain never loops.

## Output
The marks visited from tail back to head.

## Examples
- `reverse_order([5, 7, 9], [1, 2, -1])` returns `[9, 7, 5]`.
- `reverse_order([], [])` returns `[]`.
- `reverse_order([9, 4, 2], [2, -1, 1])` returns `[4, 2, 9]` — the walk goes 0,2,1, so the tail is index 1.

## Constraints
- `0 <= len(marks) == len(nxt) <= 40`
- marks are integers, `0 <= mark <= 99`
- every `nxt[i]` is `-1` or an index into the lists
- the walk from `0` always reaches `-1`
