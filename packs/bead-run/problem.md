# The Bead Run

## Task
A strand is threaded with `a` and `b` beads. A run is a stretch where the same bead repeats with no break. Call `longest_bead(strand)` and return the length of the longest run.

## Input
A string of `a` and `b` runes. It may be empty.

## Output
Return the length of the longest unbroken run of one bead kind, or `0` when the strand is empty.

## Examples
- `longest_bead("aabbbab")` returns `3`. The `bbb` stretch is the longest.
- `longest_bead("")` returns `0`. An empty strand has no run.
- `longest_bead("ab")` returns `1`. Both runs are one bead long.

## Constraints
- The strand has at most 40 beads.
- Only the beads `a` and `b` appear.
