# The Split Rite

## Task
When links fall, the ward splits into isles — and only an isle holding more than half of all towers can still bind rites. `down` lists which link positions in `links` have fallen. Call `split_rite(n, links, down)` and return how many isles still hold a strict majority.

## Input
`n` towers (`0` to `15`), `links` undirected `[a, b]` rows, and `down` — a list of link indexes into `links` that have failed.

## Output
The count of connected isles whose tower count exceeds `n / 2`.

## Examples
- `split_rite(6, [[0, 1], [1, 2], [2, 0], [3, 4], [4, 5], [5, 3], [2, 3]], [6])` returns `0` — two isles of three, and three is not more than half of six.
- `split_rite(7, [[0, 1], [1, 2], [2, 3], [3, 0], [4, 5], [5, 6], [6, 4], [3, 4]], [7])` returns `1` — the four-tower isle holds a majority.
- `split_rite(5, [[0, 1], [1, 2], [2, 3], [3, 4]], [])` returns `1`.

## Constraints
- `0 <= n <= 15`, `0 <= len(links) <= 24`, `0 <= len(down) <= 24` indexes into `links`
- `links` rows whose endpoints lie outside `0..n-1` name no tower and are skipped.
