# The Ember Isles

## Task
The web splits into isles — clusters of nodes joined by arcs, plus lone nodes. Call `isle_count(edges, n)` and return the number of isles.

## Input
`edges` lists the arcs as `[a, b]` pairs between nodes `0..n-1`; `n` is the node count. Arcs may repeat and may loop back to the same node.

## Output
The number of connected isles; each lone node is its own isle.

## Examples
- `isle_count([[0, 1], [2, 3]], 4)` returns `2`.
- `isle_count([], 3)` returns `3` — three lone nodes.
- `isle_count([[0, 1], [1, 2]], 3)` returns `1`.

## Constraints
- `0 <= len(edges) <= 20` rows of two endpoints each; `0 <= n <= 8`
