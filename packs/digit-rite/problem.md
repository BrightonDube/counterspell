# The Digit Rite

## Task
A candle burns only when one of its rune's digits divides the rune evenly — the digit `0` never burns anything. Call `digit_rite(n)` and return how many digits of `n` divide `n`.

## Input
`n` is a whole number from `0` to `10^9`.

## Output
The count of digits `d` of `n` with `d != 0` and `n % d == 0`.

## Examples
- `digit_rite(128)` returns `3` — `1`, `2` and `8` all divide `128`.
- `digit_rite(101)` returns `1` — the `0` never burns, only `1` divides.
- `digit_rite(7)` returns `1`.

## Constraints
- `0 <= n <= 1000000000`
