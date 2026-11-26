# The Sigil Bridges

## Task
A bridge is an arc whose removal splits its isle in two. Call `bridge_count(edges, n)` and return how many of the arcs are bridges.

## Input
`edges` lists the arcs as `[a, b]` pairs between nodes `0..n-1`; `n` is the node count. Two copies of one arc keep each other — neither is a bridge — and a self-loop never is.

## Output
The count of bridges across the whole web.

## Examples
- `bridge_count([[0, 1], [1, 2]], 3)` returns `2`.
- `bridge_count([[0, 1], [1, 2], [0, 2]], 3)` returns `0` — the triangle holds itself.
- `bridge_count([[0, 1], [0, 1]], 2)` returns `0` — the doubled arc survives alone.

## Constraints
- `0 <= len(edges) <= 20` rows of two endpoints each; `0 <= n <= 8`
