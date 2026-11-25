# The Order of Servings

## Task
Every mark above zero joins the back of the cauldron line; every `0` serves the mark waiting at the front (serving an empty line does nothing). Call `serve_order(ops)` and return the served marks in the order they left the line.

## Input
A list of marks. Each is `0` or a positive number at most `99`.

## Output
Return the list of served marks, earliest serving first.

## Examples
- `serve_order([4, 3, 0, 2, 0])` returns `[4, 3]`. The `4` is served first, then the `3`; the `2` still waits.
- `serve_order([3, 0, 0])` returns `[3]`. The second `0` finds the line empty.
- `serve_order([])` returns `[]`. Nothing was ever served.

## Constraints
- The sequence has at most 40 entries.
- Each mark is `0` or a positive number at most `99`.
