# The Rune Depth

## Task
Each arc `[a, b]` is an order: `a` must stand before `b`. Call `rune_depth(edges, n)` and return the length of the longest ordered chain the rules force — the longest run where each node must precede the next.

## Input
`edges` lists the orders as directed pairs `[a, b]` over nodes `0..n-1`; `n` is the node count. Repeated orders rule once; lone nodes count as chains of one.

## Output
The number of nodes in the longest forced chain, or `0` when the orders cycle or a self-order exists.

## Examples
- `rune_depth([[0, 1], [1, 2]], 3)` returns `3`.
- `rune_depth([[0, 1], [0, 2]], 3)` returns `2` — either branch ends the chain.
- `rune_depth([[0, 1], [1, 0]], 2)` returns `0` — the pair chase each other.

## Constraints
- `0 <= len(edges) <= 20` rows of two endpoints each; `0 <= n <= 8`
