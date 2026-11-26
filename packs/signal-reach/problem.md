# The Signal Reach

## Task
A signal enters the web at node `0` and travels every arc both ways. Call `reach_count(edges, n)` and return how many nodes the signal can stand on, counting node `0` itself.

## Input
`edges` lists the arcs as `[a, b]` pairs between nodes `0..n-1`; `n` is the node count. Arcs may repeat and may loop back to the same node.

## Output
The count of nodes reachable from node `0`, node `0` included.

## Examples
- `reach_count([[0, 1], [1, 2]], 3)` returns `3`.
- `reach_count([[1, 2]], 4)` returns `1` — only node `0`.
- `reach_count([], 3)` returns `1` — the signal never leaves.

## Constraints
- `0 <= len(edges) <= 20` rows of two endpoints each; `0 <= n <= 8`
