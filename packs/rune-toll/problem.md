# The Rune Toll

## Task
Every arc carries a toll. `weights[k]` prices the arc `edges[k]`. Call `toll_sum(edges, weights, n)` and return the total toll of the web.

## Input
`edges` lists the arcs as `[a, b]` pairs between nodes `0..n-1`; `weights` prices each arc by position; `n` is the node count. Extra entries in either list are ignored.

## Output
The sum of every priced arc's toll.

## Examples
- `toll_sum([[0, 1], [1, 2]], [5, 7], 3)` returns `12`.
- `toll_sum([], [], 3)` returns `0`.
- `toll_sum([[0, 1]], [5, 9, 9], 2)` returns `5` — only the priced arc counts.

## Constraints
- `0 <= len(edges) <= 24` rows of two endpoints each; `0 <= len(weights) <= 24`; `0 <= n <= 8`
