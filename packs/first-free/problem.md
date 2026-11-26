# The First Free Mark

## Task
Wax seals cover stretches of the lintel. Call `first_free(spans)` and return the smallest mark `0` or higher that no seal covers — the first bare mark.

## Input
`spans` lists the `[a, b]` seals, `0 <= a <= b <= 100`.

## Output
The first uncovered mark — `0` when mark `0` is bare, `101` when every mark is sealed.

## Examples
- `first_free([])` returns `0` — a bare lintel starts free.
- `first_free([[0, 1], [2, 3]])` returns `4`.
- `first_free([[1, 2]])` returns `0` — mark `0` is bare.

## Constraints
- `0 <= len(spans) <= 20` rows of `[a, b]` with `0 <= a <= b <= 100`
