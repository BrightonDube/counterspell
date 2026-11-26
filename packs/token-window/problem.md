# The Token Window

## Task
A token stream scrolls past a fixed `w`-rune window; the ward watches for the densest sighting of one mark. Call `token_window(tokens, w, mark)` and return the highest count of `mark` inside any window of `w` consecutive tokens — the whole stream when it is shorter than `w`.

## Input
`tokens` is the stream, `w` the window width, `mark` the rune to count.

## Output
The maximum occurrences of `mark` in any contiguous window of size `min(w, len(tokens))`.

## Examples
- `token_window([1, 2, 1, 1, 2], 3, 1)` returns `2` — every window holds two marks.
- `token_window([7, 7, 7], 2, 7)` returns `2`.
- `token_window([1, 2, 3], 0, 1)` returns `0` — a closed window sees nothing.

## Constraints
- `0 <= len(tokens) <= 24`, `0 <= tokens[i] <= 9`, `0 <= w <= 30`, `0 <= mark <= 9`
