# The Light Isles

## Task
Heavy arcs are shut — only arcs priced `t` or less carry the signal. Call `light_isles(edges, weights, n, t)` and return how many isles remain when heavy arcs are ignored.

## Input
`edges` lists the arcs as `[a, b]` pairs between nodes `0..n-1`; `weights[k]` prices `edges[k]`; `n` is the node count; `t` is the cap. Lone nodes are isles of one.

## Output
The number of isles counting only arcs priced `t` or less.

## Examples
- `light_isles([[0, 1], [1, 2], [2, 3]], [5, 15, 25], 4, 10)` returns `3` — only the cheap first arc joins.
- `light_isles([[0, 1], [1, 2]], [5, 15], 3, 15)` returns `1`.
- `light_isles([], [], 3, 10)` returns `3`.

## Constraints
- `0 <= len(edges) <= 24` rows of two endpoints each; `0 <= len(weights) <= 24`; `0 <= n <= 8`; `0 <= t <= 100000`
