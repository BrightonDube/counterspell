# The Wood Span

## Task
The keeper must string the whole web with the cheapest set of arcs. Call `span_toll(edges, weights, n)` and return the lowest total toll that joins every node — or `-1` when the web cannot be joined at all.

## Input
`edges` lists the arcs as `[a, b]` pairs between nodes `0..n-1`; `weights[k]` prices `edges[k]`; `n` is the node count. Arcs run both ways; doubled arcs each carry their own price.

## Output
The minimum joining toll, or `-1` when some node can never be reached.

## Examples
- `span_toll([[0, 1], [1, 2]], [5, 7], 3)` returns `12`.
- `span_toll([[0, 1], [1, 2], [0, 2]], [5, 7, 1], 3)` returns `6` — the ring's cheapest two arcs.
- `span_toll([[0, 1], [2, 3]], [5, 7], 4)` returns `-1` — two isles can never join.

## Constraints
- `0 <= len(edges) <= 24` rows of two endpoints each; `0 <= len(weights) <= 24`; `0 <= n <= 8`
