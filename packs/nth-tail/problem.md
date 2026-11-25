# The Nth From The Tail

## Task
A lantern chain holds marks. Call `nth_tail(marks, nxt, n)` and return the mark `n` places before the tail — `0` names the tail itself — or `-1` when the chain is too short or empty.

## Input
`marks` holds each link's mark and `nxt` holds each link's next index; `-1` ends the chain. The walk starts at index `0`; unreached cells are stray. The chain never loops. The extra param `n` is the distance from the tail.

## Output
The mark `n` places before the tail, or `-1` when the chain has fewer than `n + 1` links.

## Examples
- `nth_tail([5, 7, 9], [1, 2, -1], 0)` returns `9`.
- `nth_tail([5, 7, 9], [1, 2, -1], 2)` returns `5`.
- `nth_tail([5, 7, 9], [1, 2, -1], 3)` returns `-1`.

## Constraints
- `0 <= len(marks) == len(nxt) <= 40`
- marks are integers, `0 <= mark <= 99`
- every `nxt[i]` is `-1` or an index into the lists
- `0 <= n <= 40`
- the walk from `0` always reaches `-1`
