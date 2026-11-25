# The Varied Fragment

## Task
A frieze is carved with the runes `a`, `b`, `c` and `d`. A fragment is varied when it holds at least one `a`, one `b` and one `c`. Call `shortest_varied(inscription)` and return the length of the shortest varied fragment, or `0` when no fragment holds all three.

## Input
A string of `a`, `b`, `c` and `d` runes. It may be empty.

## Output
Return the length of the shortest contiguous fragment containing all of `a`, `b` and `c`, or `0` when none exists.

## Examples
- `shortest_varied("abc")` returns `3`. The whole inscription is the only varied fragment.
- `shortest_varied("aabbc")` returns `4`. The `abbc` fragment is the shortest that covers all three runes.
- `shortest_varied("ddd")` returns `0`. No `a`, `b` or `c` ever appears.

## Constraints
- The inscription has at most 40 runes.
- Only the runes `a`, `b`, `c` and `d` appear.
