# The Bit Isolate

## Task
Every rune's lowest lit bit is its anchor — the value of that single spark. Call `bit_isolate(n)` and return the lowest set bit of `n`; an unlit rune anchors nothing.

## Input
`n` is a whole number from `0` to `2^53 - 1`.

## Output
`n & (-n)` — the lowest set bit's value, or `0` when `n` is `0`.

## Examples
- `bit_isolate(12)` returns `4` — `1100` anchors at bit two.
- `bit_isolate(7)` returns `1`.
- `bit_isolate(0)` returns `0`.

## Constraints
- `0 <= n <= 1152921504606846976` (`2^60`)
