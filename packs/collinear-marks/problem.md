# The Collinear Marks

## Task
A fence-line runs through the first two marks on the map. Call `collinear_marks(points)` and return how many marks sit on that line — when the first two marks coincide (or fewer than two exist), no line stands and the answer is `-1`.

## Input
`points` lists `[x, y]` pairs, coordinates from `-50` to `50`.

## Output
The count of points on the line through `points[0]` and `points[1]`, or `-1` when that line is undefined.

## Examples
- `collinear_marks([[0, 0], [1, 1], [2, 2], [0, 1]])` returns `3`.
- `collinear_marks([[0, 0], [0, 0], [1, 1]])` returns `-1` — coincident anchors.
- `collinear_marks([[1, 1]])` returns `-1`.

## Constraints
- `0 <= len(points) <= 30` rows of `[x, y]` with `-50 <= x, y <= 50`
