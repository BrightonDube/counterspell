# The Wide Ferry

## Task
A ferry's limit is the thinnest arc it rides. Call `ferry_load(edges, weights, n)` and return the largest load that can cross from `0` to `n-1` — the best a route can promise on its weakest arc.

## Input
`edges` lists the arcs as `[a, b]` pairs between nodes `0..n-1`; `weights[k]` gives the capacity of `edges[k]`; `n` is the node count. Arcs run both ways.

## Output
The greatest capacity the ferry can keep on some route, or `-1` when no route reaches.

## Examples
- `ferry_load([[0, 1], [1, 2]], [5, 7], 3)` returns `5` — the thin first arc caps it.
- `ferry_load([[0, 2]], [4], 3)` returns `4`.
- `ferry_load([[1, 2]], [9], 3)` returns `-1` — node `0` is stranded.

## Constraints
- `0 <= len(edges) <= 24` rows of two endpoints each; `0 <= len(weights) <= 24`; `0 <= n <= 8`
