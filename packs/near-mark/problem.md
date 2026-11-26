# The Near Mark

## Task
The keeper cares only about arcs that touch node `0`. Call `near_mark(edges, weights, n)` and return the cheapest toll among them.

## Input
`edges` lists the arcs as `[a, b]` pairs between nodes `0..n-1`; `weights[k]` prices `edges[k]`; `n` is the node count. Arcs run both ways, so either end touching `0` counts. A self-loop on `0` counts.

## Output
The smallest toll among arcs touching node `0`, or `-1` when none touch it.

## Examples
- `near_mark([[0, 1], [1, 2]], [5, 7], 3)` returns `5`.
- `near_mark([[1, 2]], [7], 3)` returns `-1`.
- `near_mark([[0, 1], [0, 1]], [9, 3], 2)` returns `3` — the doubled arc's cheap copy wins.

## Constraints
- `0 <= len(edges) <= 24` rows of two endpoints each; `0 <= len(weights) <= 24`; `0 <= n <= 8`
