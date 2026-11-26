# The Merge Glow

## Task
Two lantern chains each hold marks sorted low to high along the walk. Call `merge_glow(amarks, anxt, bmarks, bnxt)` and return one list with every mark from both chains in ascending order.

## Input
Each pair (`marks`, `nxt`) describes one chain as in the single-chain packs: `nxt[i]` is the next index or `-1`. Each chain's own walk order is ascending. Cells a walk never reaches do not belong to that chain.

## Output
All marks from both chains, ascending.

## Examples
- `merge_glow([1, 3, 5], [1, 2, -1], [2, 4], [1, -1])` returns `[1, 2, 3, 4, 5]`.
- `merge_glow([], [], [7], [-1])` returns `[7]`.
- `merge_glow([2, 4], [1, -1], [1, 3, 5], [1, 2, -1])` returns `[1, 2, 3, 4, 5]`.

## Constraints
- `0 <= len(amarks) == len(anxt) <= 40` and likewise for the b-lists
- marks are integers, `0 <= mark <= 99`
- each `nxt[i]` is `-1` or an index into its own list
- both walks always reach `-1`
