# The Double Book

## Task
Wax seals mark marks each keeper must hold; two keepers holding the same mark collide. Call `room_count(spans)` and return the fewest keepers that cover every seal without collision — the deepest overlap.

## Input
`spans` lists the `[a, b]` holds, `0 <= a <= b <= 100`.

## Output
The most seals alive at one mark — the keepers needed.

## Examples
- `room_count([[0, 1], [1, 2]])` returns `2` — mark `1` is shared.
- `room_count([[0, 1], [2, 3]])` returns `1` — the holds never collide.
- `room_count([[0, 4], [1, 2], [2, 3], [3, 4]])` returns `3`.

## Constraints
- `0 <= len(spans) <= 20` rows of `[a, b]` with `0 <= a <= b <= 100`
