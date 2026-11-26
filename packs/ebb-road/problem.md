# The Ebb Road

## Task
Some arcs pay the traveller back — tolls can be negative. Each arc `[a, b]` runs one way, from `a` to `b`. Call `ebb_toll(edges, weights, n)` and return the cheapest one-way toll from `0` to `n-1`.

## Input
`edges` lists the one-way arcs as `[a, b]` pairs between nodes `0..n-1`; `weights[k]` prices `edges[k]` and may be negative; `n` is the node count.

## Output
The cheapest toll, `-1` when no road reaches, or `-1000000000000000` when a negative ring can cheapen the walk forever.

## Examples
- `ebb_toll([[0, 1], [1, 2]], [5, -7], 3)` returns `-2`.
- `ebb_toll([[1, 2]], [3], 3)` returns `-1` — no road leaves `0`.
- `ebb_toll([[0, 1], [1, 1], [1, 2]], [5, -4, 3], 3)` returns `-1000000000000000` — the loop pays forever.

## Constraints
- `0 <= len(edges) <= 24` rows of two endpoints each; `0 <= len(weights) <= 24`; `0 <= n <= 8`
