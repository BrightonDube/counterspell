# The Twin Walls

## Task
The web must split into two camps where no arc joins members of the same camp. Call `twin_walls(edges, n)` and return `1` when such a split exists, `0` when it cannot be done.

## Input
`edges` lists the arcs as `[a, b]` pairs between nodes `0..n-1`; `n` is the node count. Arcs may repeat and may loop back to the same node. A self-loop can never be split.

## Output
`1` when the web splits two ways, `0` otherwise. An empty or lone web always splits.

## Examples
- `twin_walls([[0, 1], [1, 2], [2, 0]], 3)` returns `0` — the triangle cannot split.
- `twin_walls([[0, 1], [1, 2]], 3)` returns `1`.
- `twin_walls([], 3)` returns `1`.

## Constraints
- `0 <= len(edges) <= 20` rows of two endpoints each; `0 <= n <= 8`
