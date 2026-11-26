# The Chunk Toll

## Task
A token stream must be cut into chunks of at most `w` runes; each chunk pays `base` plus the spread between its widest and narrowest rune. Call `chunk_toll(tokens, w, base)` and return the smallest total toll that covers the whole stream.

## Input
`tokens` is the stream, `w` the widest chunk allowed (`0` behaves as `1`), `base` the flat cut fee.

## Output
The minimum sum of `base + (max(chunk) - min(chunk))` over all covers of the stream.

## Examples
- `chunk_toll([1, 5, 2, 9], 2, 3)` returns `12` — `[1] [5, 2] [9]` pays `3 + 6 + 3`.
- `chunk_toll([4, 4, 4], 3, 5)` returns `5` — one even chunk has no spread.
- `chunk_toll([], 2, 3)` returns `0`.

## Constraints
- `0 <= len(tokens) <= 24`, `0 <= tokens[i] <= 60`, `0 <= w <= 30`, `0 <= base <= 50`
