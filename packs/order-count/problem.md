# The Order Count

## Task
Each arc `[a, b]` is an order: `a` must stand before `b`. Call `order_count(edges, n)` and return the number of line-ups of the `n` nodes that honour every order — `0` when the orders chase their own tail.

## Input
`edges` lists the orders as directed pairs `[a, b]` over nodes `0..n-1`; `n` is the node count. Repeated orders rule once. A self-order rules nothing out by itself — it simply can never be met.

## Output
The number of valid line-ups, or `0` when the orders cycle.

## Examples
- `order_count([[0, 1], [0, 2]], 3)` returns `2` — `1` and `2` may stand either way.
- `order_count([[0, 1], [1, 0]], 2)` returns `0` — the pair chase each other.
- `order_count([], 3)` returns `6` — any line-up works.

## Constraints
- `0 <= len(edges) <= 20` rows of two endpoints each; `0 <= n <= 8`
