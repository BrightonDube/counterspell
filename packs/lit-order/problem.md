# The Lit Order

## Task
A row of lantern links is chained: `marks[i]` holds each link's mark and `nxt[i]` gives the next index, or `-1` at the tail. Call `lit_order(marks, nxt)` and return the marks in chain order.

## Input
`marks` holds each link's mark and `nxt` holds each link's next index. `-1` means the chain ends. Both lists share one index space. The walk always starts at index `0`; cells the walk never reaches do not belong to the chain. The chain always ends — it never loops back.

## Output
The marks in the order the walk visits them.

## Examples
- `lit_order([5, 7, 9], [1, 2, -1])` returns `[5, 7, 9]`.
- `lit_order([], [])` returns `[]`.
- `lit_order([9, 4, 2], [2, -1, 1])` returns `[9, 2, 4]` — the walk hops 0 to 2 to 1.

## Constraints
- `0 <= len(marks) == len(nxt) <= 40`
- marks are integers, `0 <= mark <= 99`
- every `nxt[i]` is `-1` or an index into the lists
- the walk from `0` always reaches `-1`
