# The Signal Span

## Task
A signal leaves node `0` and hops arc to arc. Call `span_max(edges, n)` and return the longest hop count the signal needs to reach any node it can touch.

## Input
`edges` lists the arcs as `[a, b]` pairs between nodes `0..n-1`; `n` is the node count. Arcs run both ways. Nodes out of reach do not count.

## Output
The greatest shortest-hop distance from node `0`, or `0` when nothing beyond it is reached.

## Examples
- `span_max([[0, 1], [1, 2]], 3)` returns `2`.
- `span_max([[0, 1], [0, 2]], 3)` returns `1`.
- `span_max([], 3)` returns `0`.

## Constraints
- `0 <= len(edges) <= 20` rows of two endpoints each; `0 <= n <= 8`
