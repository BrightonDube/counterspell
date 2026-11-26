# The Lone Nodes

## Task
Some nodes carry no arc at all — the web never touches them. Call `lone_count(edges, n)` and return how many nodes no arc names.

## Input
`edges` lists the arcs as `[a, b]` pairs between nodes `0..n-1`; `n` is the node count. A node named by a self-loop is not lone.

## Output
The count of nodes that no arc touches.

## Examples
- `lone_count([[0, 1]], 3)` returns `1` — node `2`.
- `lone_count([], 2)` returns `2`.
- `lone_count([[0, 0]], 2)` returns `1` — the loop names node `0`.

## Constraints
- `0 <= len(edges) <= 20` rows of two endpoints each; `0 <= n <= 8`
