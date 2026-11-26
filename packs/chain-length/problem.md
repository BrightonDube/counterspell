# The Chain Length

## Task
A row of lantern links is chained: `nxt[i]` gives the index of the link after `i`, or `-1` at the tail. Call `chain_length(nxt)` and return how many links the walk from index `0` visits.

## Input
`marks` holds each link's mark and `nxt` holds each link's next index. `-1` means the chain ends. The walk always starts at index `0`; cells the walk never reaches do not belong to the chain. The chain always ends — it never loops back.

## Output
The number of links the walk visits.

## Examples
- `chain_length([1, 2, -1])` returns `3`.
- `chain_length([])` returns `0`.
- `chain_length([1, -1, 0])` returns `2` — cell 2 is never reached, so it does not count.

## Constraints
- `0 <= len(nxt) <= 40`
- every `nxt[i]` is `-1` or an index into the list
- the walk from `0` always reaches `-1`
