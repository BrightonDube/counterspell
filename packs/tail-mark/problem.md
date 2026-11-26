# The Tail Mark

## Task
A row of lantern links is chained: `marks[i]` holds each link's mark and `nxt[i]` gives the next index, or `-1` at the tail. Call `tail_mark(marks, nxt)` and return the tail link's mark, or `-1` when the chain is empty.

## Input
`marks` holds each link's mark and `nxt` holds each link's next index. `-1` means the chain ends. Both lists share one index space. The walk always starts at index `0`; cells the walk never reaches do not belong to the chain. The chain always ends — it never loops back.

## Output
The mark carried by the chain's tail, or `-1` for an empty chain.

## Examples
- `tail_mark([5, 7, 9], [1, 2, -1])` returns `9`.
- `tail_mark([], [])` returns `-1`.
- `tail_mark([5, 9, 7], [1, -1, 0])` returns `9` — the walk ends at index 1, and cell 2 is stray.

## Constraints
- `0 <= len(marks) == len(nxt) <= 40`
- marks are integers, `0 <= mark <= 99`
- every `nxt[i]` is `-1` or an index into the lists
- the walk from `0` always reaches `-1`
