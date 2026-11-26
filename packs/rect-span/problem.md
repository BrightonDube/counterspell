# The Rect Span

## Task
Two wards are drawn as corner pairs `[x1, y1, x2, y2]` — corners may arrive in either order. Call `rect_span(a, b)` and return the overlapping area the wards share; touching edges share nothing.

## Input
`a` and `b` are four-integer lists `[x1, y1, x2, y2]`, coordinates from `-50` to `50`.

## Output
The shared area as a non-negative integer; `0` when the wards only touch or miss.

## Examples
- `rect_span([0, 0, 2, 2], [1, 1, 3, 3])` returns `1`.
- `rect_span([0, 0, 1, 1], [1, 1, 2, 2])` returns `0` — they touch at a corner.
- `rect_span([0, 0, 4, 4], [1, 1, 2, 2])` returns `1`.

## Constraints
- `len(a) == len(b) == 4`, each coordinate `-50..50`
