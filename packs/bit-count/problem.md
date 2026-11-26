# The Bit Count

## Task
Each set bit in a rune is a lit spark. Call `bit_count(n)` and return how many bits of `n` burn — the number of `1`s in its binary form.

## Input
`n` is a whole number from `0` to `2^53 - 1`.

## Output
The popcount of `n`.

## Examples
- `bit_count(13)` returns `3` — `1101` holds three sparks.
- `bit_count(0)` returns `0`.
- `bit_count(255)` returns `8`.

## Constraints
- `0 <= n <= 1152921504606846976` (`2^60`)
