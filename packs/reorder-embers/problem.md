# The Reorder Embers

## Task
A lantern chain holds marks in walk order. Call `reorder_embers(marks, nxt)` and return the marks re-woven: first half of the walk kept in order, second half reversed, then interleaved one from each — head, tail, second, second-last, and so on.

## Input
`marks` holds each link's mark and `nxt` holds each link's next index; `-1` ends the chain. The walk starts at index `0`; unreached cells are stray. The chain never loops.

## Output
The interleaved marks in walk-length order.

## Examples
- `reorder_embers([1, 2, 3, 4], [1, 2, 3, -1])` returns `[1, 4, 2, 3]`.
- `reorder_embers([5], [-1])` returns `[5]`.
- `reorder_embers([], [])` returns `[]`.

## Constraints
- `0 <= len(marks) == len(nxt) <= 40`
- marks are integers, `0 <= mark <= 99`
- every `nxt[i]` is `-1` or an index into the lists
- the walk from `0` always reaches `-1`
