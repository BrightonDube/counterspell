# The Corner Turns

## Task
A route walks mark to mark in order. At each middle mark it either turns left, turns right, or keeps straight. Call `corner_turns(points)` and return how many middle marks turn **left** — counter-clockwise.

## Input
`points` lists `[x, y]` pairs in walk order, coordinates from `-50` to `50`.

## Output
The count of interior marks where the path turns left; `0` for fewer than three marks.

## Examples
- `corner_turns([[0, 0], [1, 0], [1, 1]])` returns `1` — one left turn.
- `corner_turns([[0, 0], [1, 0], [1, -1]])` returns `0` — a right turn.
- `corner_turns([[0, 0], [1, 0], [2, 0]])` returns `0` — straight through.

## Constraints
- `0 <= len(points) <= 30` rows of `[x, y]` with `-50 <= x, y <= 50`
